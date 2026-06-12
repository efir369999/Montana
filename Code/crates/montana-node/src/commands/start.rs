use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use mt_account::{apply_proposal, ProposalSettle};
use mt_codec::CanonicalEncode;
use mt_consensus::{proposal_hash, ProposalHeader};
use mt_crypto::{sign, Hash32, Signature, SIGNATURE_SIZE};
use mt_entry::{
    apply_candidate_expiry, apply_noderegistrations_batch, apply_selection_event,
    candidate_vdf_init, is_selection_window, nodereg_hash, validate_noderegistration,
    NodeRegistration,
};
use mt_genesis::genesis_params;
use mt_lottery::{
    bundle_hash, compute_endpoint, quorum, reveal_hash, validate_bundle, validate_reveal,
    weighted_ticket_node, BundledConfirmation, VdfReveal,
};
use mt_merkle::{empty_internal, SparseMerkleTree, TREE_DEPTH};
use mt_net::{MsgType, ProtocolMessage};
use mt_state::compute_state_root;
use mt_store::FsStore;
use mt_timechain::{cemented_bundle_aggregate, next_d, vdf_step};

use crate::clock::{load_current_window, save_current_window};
use crate::identity::{default_data_dir, load_identity, NodeError};
use crate::node_lifecycle::{load_or_init_lifecycle, save_lifecycle, NodeLifecycle, NodePhase};
use crate::state::LocalState;
use crate::timechain_state::{load_or_init_timechain, save_timechain, TimeChainState};

static STOP: AtomicBool = AtomicBool::new(false);

extern "C" fn shutdown_handler(_: libc::c_int) {
    STOP.store(true, Ordering::SeqCst);
}

// M7 fast-sync trigger threshold (network-layer implementation guidance per
// Network spec — not consensus-critical, may vary between implementations).
// Replay costs ~6 min / 1000 windows on 1 vCPU (mt-sync lib doc); beyond this
// lag snapshot delivery is bandwidth-bound and cheaper than apply_proposal loop.
const FAST_SYNC_LAG_THRESHOLD: u64 = 1000;

// Operational override of the fast-sync lag threshold via env. The threshold is
// network-layer (not consensus); operators tune it for deployment/observation.
// Empty, unparsable, or zero values fall back to the production default.
fn resolve_fast_sync_lag_threshold(override_val: Option<String>) -> u64 {
    override_val
        .and_then(|v| v.trim().parse::<u64>().ok())
        .filter(|&t| t > 0)
        .unwrap_or(FAST_SYNC_LAG_THRESHOLD)
}

pub struct StartArgs {
    pub data_dir: Option<PathBuf>,
    pub max_windows: Option<u64>,
    pub d_test_override: Option<u64>,
    /// Multiaddr для libp2p listen (например `/ip4/0.0.0.0/tcp/8444`).
    /// При наличии — узел стартует cross-machine peering thread.
    pub listen_multiaddr: Option<String>,
    /// Путь к genesis-manifest.json с peer list для bootstrap connectivity.
    /// При наличии `--listen` обязателен.
    pub genesis_manifest: Option<PathBuf>,
}

pub fn run(args: StartArgs) -> Result<(), NodeError> {
    let data_dir = args.data_dir.unwrap_or_else(default_data_dir);
    let identity = load_identity(&data_dir)?;
    let params = genesis_params();

    // Cross-machine M8: spawn network thread с собственным tokio runtime.
    // Network событийный loop отделён от consensus loop (VDF compute) —
    // separate OS thread предотвращает блокировку async задач CPU-heavy
    // операциями подсчёта VDF.
    let mut network_handle: Option<NetworkHandle> = None;
    if let (Some(listen_str), Some(manifest_path)) =
        (&args.listen_multiaddr, &args.genesis_manifest)
    {
        network_handle = Some(spawn_network_thread(&identity, listen_str, manifest_path)?);
    }

    // Parse the genesis manifest once at startup (cheap; JSON) so that
    // test-cohort `force_active` peers can be pre-seeded into NodeTable /
    // AccountTable on first run. Production manifest has no such peers, so
    // extras is empty and bootstrap behaves identically.
    let genesis_manifest_for_bootstrap: Option<mt_genesis::GenesisManifest> = if let Some(path) =
        args.genesis_manifest.as_ref()
    {
        let text = std::fs::read_to_string(path)
            .map_err(|e| NodeError::InvalidArguments(format!("genesis-manifest {path:?}: {e}")))?;
        Some(
            mt_genesis::GenesisManifest::parse(&text)
                .map_err(|e| NodeError::InvalidArguments(format!("parse manifest: {e}")))?,
        )
    } else {
        None
    };
    let extra_actives: Vec<&mt_genesis::GenesisPeer> = genesis_manifest_for_bootstrap
        .as_ref()
        .map(|m| m.extra_actives())
        .unwrap_or_default();
    let mut state = LocalState::load_or_bootstrap(&data_dir, &identity, params, &extra_actives)?;
    // Persist the seeded genesis state immediately so a separate `status`
    // invocation (which loads with empty extra_actives) reads the real
    // bootstrapped tables instead of re-bootstrapping an incomplete view.
    if !data_dir.join("accounts.bin").exists() {
        state.save(&data_dir)?;
    }
    let mut current = load_current_window(&data_dir)?;
    let mut timechain = load_or_init_timechain(&data_dir)?;
    let mut lifecycle = load_or_init_lifecycle(&data_dir, &identity, params)?;
    let mut effective_d = args.d_test_override.unwrap_or(timechain.current_d);

    if let Some(d) = args.d_test_override {
        eprintln!(
            "ВНИМАНИЕ: --d-test-override={d} активен. Это test-only режим. \
             В production D = params.d0 = {} итераций (Genesis Decree).",
            params.d0
        );
        timechain.current_d = d;
    }

    install_shutdown_handlers();

    let stop_at = args.max_windows.map(|n| current.saturating_add(n));

    let my_account = identity.account_id();
    let my_node = identity.node_id();
    let initial_balance = state
        .accounts
        .get(&my_account)
        .map(|a| a.balance)
        .unwrap_or(0);

    let is_genesis = NodeLifecycle::is_bootstrap_node(&identity, params);
    // Genesis cohort member = the compiled bootstrap node OR a force_active peer
    // pre-seeded into the genesis NodeTable (state.rs seeds both as Active
    // operators). Both start Active; every other node joins via the candidate
    // VDF + admission path.
    let is_genesis_member = is_genesis || state.nodes.contains(&my_node);

    // Phase Bootstrap = первая загрузка. Genesis-член (bootstrap или force_active,
    // уже активный в NodeTable) → сразу Active. Прочие узлы → CandidateVdf
    // с target_chain_length = τ₂ и w_start = текущее окно + 1.
    if lifecycle.phase == NodePhase::Bootstrap {
        if is_genesis_member {
            lifecycle.phase = NodePhase::Active;
        } else {
            lifecycle.phase = NodePhase::CandidateVdf;
            lifecycle.target_chain_length = params.tau2_windows;
            lifecycle.w_start = current.saturating_add(1);
            lifecycle.candidate_progress = 0;
            // candidate_endpoint начинается с T_r текущего timechain — это
            // canonical seed для chain старта; на каждом окне ticks через
            // vdf_step_chunked в Active phase code path ниже.
            lifecycle.candidate_endpoint = timechain.t_r;
        }
        save_lifecycle(&data_dir, &lifecycle)?;
    }

    println!("=== montana-node start — узел Montana ===");
    println!();
    println!("data-dir         : {}", data_dir.display());
    println!("account_id       : {}", hex16(&my_account));
    println!("node_id          : {}", hex16(&my_node));
    println!("phase            : {:?}", lifecycle.phase);
    println!("current_window   : {current}");
    println!("D                : {} итераций SHA-256 / окно", effective_d);
    println!("T_r              : {}", hex16(&timechain.t_r));
    println!(
        "balance start    : {} nɈ ({} Ɉ)",
        initial_balance,
        initial_balance / 1_000_000_000
    );
    if let Some(stop) = stop_at {
        println!(
            "stop_at          : окно {stop} (через {} окон)",
            stop - current
        );
    } else {
        println!("stop_at          : Ctrl-C");
    }
    if lifecycle.phase == NodePhase::CandidateVdf {
        let remaining = lifecycle
            .target_chain_length
            .saturating_sub(lifecycle.candidate_progress);
        println!(
            "candidate VDF    : прогресс {}/{}, осталось {} окон до регистрации",
            lifecycle.candidate_progress, lifecycle.target_chain_length, remaining
        );
    }
    println!();
    println!("--- VDF тикает ---");
    println!();

    let session_start = Instant::now();
    let mut session_emitted: u128 = 0;
    let mut session_windows: u64 = 0;
    let mut prev_proposal_hash: Hash32 = [0u8; 32];

    let store = FsStore::open(&data_dir)
        .map_err(|e| NodeError::InvalidArguments(format!("FsStore::open: {e:?}")))?;

    // DEV-012 T_r history: per-window T_r snapshot for BC endpoint validation
    // when BCs arrive after current has advanced.
    let mut t_r_history: BTreeMap<u64, Hash32> = BTreeMap::new();
    // DEV-022: bootstrap_node_id used both in main drain (Proposal verify) and
    // active arm (Lookback proposer selection); hoist to outer scope.
    let bootstrap_node_id = mt_state::derive_node_id(&params.bootstrap_node_pubkey);
    // DEV-023: track per-proposer last cemented window so each node can decide
    // when an elected proposer has gone silent (≥ K_FALLBACK_WINDOWS windows
    // without producing cement). Bootstrap is the canonical fallback.
    let mut last_proposer_cement: BTreeMap<mt_state::NodeId, u64> = BTreeMap::new();
    #[allow(dead_code)]
    const K_FALLBACK_WINDOWS: u64 = 3;
    // DEV-022 Lookback Leadership: track winner_id per cemented window so any
    // Active node can compute proposer_W = winner_{W-2} for its own window decisions.
    // Genesis bootstrap rule: proposer_0 и proposer_1 = bootstrap-узел.
    let mut winner_history: BTreeMap<u64, mt_state::NodeId> = BTreeMap::new();
    // DEV-020: per-window reveal pool, keyed by (window_index → (node_id → VdfReveal)).
    // Все Active узлы публикуют собственный Reveal каждое окно через MsgType::VdfReveal.
    // Proposer на cement-time собирает cemented Reveal-ы (те, чей reveal_hash вошёл в
    // 67% chain_length BC) и вычисляет winner = argmin(weighted_ticket_node) per spec.
    let mut reveal_pool: BTreeMap<u64, BTreeMap<mt_state::NodeId, VdfReveal>> = BTreeMap::new();

    // DEV-012 multi-confirmer: per-window accumulator of BCs from Active peers.
    // Keyed by window then node_id so duplicates from same node deduplicate.
    let mut bc_accumulator: BTreeMap<u64, BTreeMap<mt_state::NodeId, BundledConfirmation>> =
        BTreeMap::new();

    // M7 fast-sync: held across loop iterations while a snapshot is in flight.
    let fast_sync_lag_threshold =
        resolve_fast_sync_lag_threshold(std::env::var("MONTANA_FASTSYNC_LAG_THRESHOLD").ok());
    println!("fast-sync lag    : порог {fast_sync_lag_threshold} окон");
    let mut fast_sync: Option<mt_sync::FastSyncClient> = None;
    let mut fast_sync_deadline: Option<Instant> = None;
    // M7 fast-sync: recent cemented bootstrap state_roots (window -> root),
    // the trusted set a reconstructed snapshot root must match.
    let mut recent_roots: BTreeMap<u64, Hash32> = BTreeMap::new();

    // Пооконные истории консенсуса (ограничены HISTORY_BOUND окнами).
    let mut lottery_history: BTreeMap<u64, Vec<mt_lottery::Candidate>> = BTreeMap::new();
    let mut bc_set_history: BTreeMap<u64, Vec<mt_state::NodeId>> = BTreeMap::new();
    let mut own_bc_cache: BTreeMap<u64, BundledConfirmation> = BTreeMap::new();
    let mut pending_msgs: Vec<ProtocolMessage> = Vec::new();
    let mut last_cement_at = Instant::now();
    // Восстановление историй из архива зацементированных конвертов после
    // рестарта: набор подтвердивших, победители, значения часов.
    for w in current.saturating_sub(8)..=current {
        if let Ok(Some(env)) = store.load_proposal_envelope(w) {
            if let Some(h) = parse_header(&env) {
                t_r_history.insert(w, h.timechain_value);
                if w >= 1 {
                    winner_history.insert(w - 1, h.winner_id);
                }
                if let Some((bundles_prev, _evidence)) = parse_envelope_bundles(&env) {
                    if w >= 1 {
                        let mut ids: Vec<mt_state::NodeId> =
                            bundles_prev.iter().map(|b| b.node_id).collect();
                        ids.sort();
                        bc_set_history.insert(w - 1, ids);
                    }
                }
                prev_proposal_hash = proposal_hash(&h);
            }
        }
    }

    loop {
        // Эффективное D пере-считывается каждый виток (адаптация на границе τ₂
        // происходит в едином переходе состояния settle_and_bookkeep).
        effective_d = args.d_test_override.unwrap_or(timechain.current_d);
        macro_rules! net_ctx {
            () => {
                NetCtx {
                    state: &mut state,
                    current: &mut current,
                    timechain: &mut timechain,
                    recent_roots: &mut recent_roots,
                    t_r_history: &mut t_r_history,
                    reveal_pool: &mut reveal_pool,
                    bc_accumulator: &mut bc_accumulator,
                    winner_history: &mut winner_history,
                    lottery_history: &mut lottery_history,
                    bc_set_history: &mut bc_set_history,
                    last_proposer_cement: &mut last_proposer_cement,
                    own_bc_cache: &mut own_bc_cache,
                    pending_msgs: &mut pending_msgs,
                    fast_sync: &mut fast_sync,
                    fast_sync_deadline: &mut fast_sync_deadline,
                    last_cement_at: &mut last_cement_at,
                    prev_proposal_hash: &mut prev_proposal_hash,
                    fast_sync_lag_threshold,
                    data_dir: &data_dir,
                    params,
                    store: &store,
                    my_node,
                    bootstrap_node_id,
                }
            };
        }
        if let Some(ref mut handle) = network_handle {
            let mut ctx = net_ctx!();
            drain_network(&mut ctx, handle)?;
        }

        if STOP.load(Ordering::SeqCst) {
            println!();
            println!("[shutdown] получен SIGINT/SIGTERM, сохраняю состояние...");
            break;
        }
        if let Some(stop) = stop_at {
            if current >= stop {
                println!();
                println!("[stop_at] достигнуто целевое окно {stop}");
                break;
            }
        }

        // Окно, которое сеть закрывает следующим.
        let next_window = current + 1;

        if timechain.last_window < next_window {
            // Спецификация «Непрерывность VDF»: цепочка следующего окна
            // вычисляется непрерывно; финализация и приём билетов идут
            // параллельно — вычитка сети между порциями витка.
            let tick_seed = timechain.t_r;
            let next_t_r = vdf_step_chunked(
                &tick_seed,
                effective_d,
                "TimeChain VDF",
                next_window,
                || {
                    if let Some(ref mut handle) = network_handle {
                        let mut ctx = net_ctx!();
                        if let Err(e) = drain_network(&mut ctx, handle) {
                            eprintln!("[drain] посреди витка: {e:?}");
                        }
                    }
                },
            );
            // Сверка с сетевым значением, если окно уже видели в конверте
            // (или его успел применить drain посреди нашего витка).
            if let Some(known) = t_r_history.get(&next_window) {
                if *known != next_t_r {
                    return Err(NodeError::InvalidArguments(format!(
                        "расхождение цепочки времени в окне {next_window}: сеть ≠ локально"
                    )));
                }
            }
            if timechain.last_window < next_window {
                timechain.t_r = next_t_r;
                timechain.last_window = next_window;
            }
            t_r_history.insert(next_window, next_t_r);
            bound_map(&mut t_r_history);
        } else {
            // Виток этого окна уже посчитан; ждём цементирования из сети.
            std::thread::sleep(Duration::from_millis(50));
        }

        // Каждый Active узел публикует артефакты окна от канонических часов:
        // билет (VDF_Reveal) окна next_window + подтверждение
        // (BundledConfirmation) с хэшами билетов предыдущего окна. Билет окна
        // next_window цементируется уликами окна next_window+1, поэтому
        // публикация уместна и тогда, когда часы продвинул конверт из сети.
        if lifecycle.phase == NodePhase::Active
            && state.nodes.get(&my_node).is_some()
            && !own_bc_cache.contains_key(&next_window)
        {
            if let Some(t_r_w) = t_r_history.get(&next_window).copied() {
                publish_window_artifacts(
                    &state,
                    &mut reveal_pool,
                    &mut bc_accumulator,
                    &mut own_bc_cache,
                    &bc_set_history,
                    &identity,
                    my_node,
                    next_window,
                    &t_r_w,
                    network_handle.as_ref().map(|h| &h.broadcast_tx),
                )?;
            }
        }

        if current >= next_window {
            save_progress(&data_dir, &state, &timechain, &lifecycle, current)?;
            continue;
        }

        match lifecycle.phase {
            NodePhase::Bootstrap => unreachable!("Bootstrap → CandidateVdf transition выше"),
            NodePhase::CandidateVdf => {
                lifecycle.candidate_endpoint = vdf_step_chunked(
                    &lifecycle.candidate_endpoint,
                    effective_d,
                    "Candidate VDF",
                    next_window,
                    || {},
                );
                lifecycle.candidate_progress = lifecycle
                    .candidate_progress
                    .checked_add(1)
                    .expect("candidate_progress overflow at u64::MAX");

                if lifecycle.candidate_progress >= lifecycle.target_chain_length {
                    let cba_w_start_minus_2 =
                        cba_from(&bc_set_history, lifecycle.w_start.saturating_sub(2));
                    let proof_endpoint =
                        candidate_vdf_init(&timechain.t_r, &cba_w_start_minus_2, &my_node);

                    let mut nr = NodeRegistration {
                        suite_id: identity.suite_id as u16,
                        node_pubkey: *identity.node_pk.as_bytes(),
                        operator_account_id: my_account,
                        proof_endpoint,
                        w_start: lifecycle.w_start,
                        vdf_chain_length: lifecycle.candidate_progress,
                        signature: Signature::from_array([0u8; SIGNATURE_SIZE]),
                    };
                    let mut scope = Vec::new();
                    nr.encode_signed_scope(&mut scope);
                    nr.signature = sign(&identity.node_sk, &scope).map_err(NodeError::Crypto)?;

                    validate_noderegistration(
                        &nr,
                        &state.nodes,
                        &state.candidates,
                        &state.accounts,
                    )
                    .map_err(|e| {
                        NodeError::InvalidArguments(format!("validate_noderegistration: {e:?}"))
                    })?;

                    let pending_baseline = state.candidates.len() as u64;
                    let active_nodes = state.nodes.len() as u64;
                    let cba_w_p_minus_2 = cba_from(&bc_set_history, next_window.saturating_sub(2));
                    let outcome = apply_noderegistrations_batch(
                        &mut state.candidates,
                        &[nr.clone()],
                        &timechain.t_r,
                        &cba_w_p_minus_2,
                        pending_baseline,
                        active_nodes,
                        next_window,
                        params,
                    );
                    if outcome.applied.len() != 1 {
                        return Err(NodeError::InvalidArguments(format!(
                            "apply_noderegistrations_batch: applied={}, rejected={}, expected applied=1",
                            outcome.applied.len(),
                            outcome.rejected.len()
                        )));
                    }
                    lifecycle.nodereg_hash = nodereg_hash(&nr);
                    lifecycle.registration_window = next_window;
                    lifecycle.phase = NodePhase::Registered;
                    println!(
                        "[register W={next_window}] nodereg_hash={} | vdf_chain_length={}",
                        hex16(&lifecycle.nodereg_hash),
                        lifecycle.candidate_progress
                    );
                }
            },
            NodePhase::Registered => {},
            NodePhase::Active => 'active_arm: {
                // Законный ведущий окна next_window: победитель окна
                // next_window-2; при молчании — каскад запасных по
                // возрастанию взвешенного билета (спецификация Lookback).
                let silence = last_cement_at.elapsed().as_secs();
                let (acting, my_depth) = {
                    let ctx = net_ctx!();
                    expected_proposer(&ctx, next_window, silence)
                };
                if acting != my_node {
                    break 'active_arm;
                }
                let propose_w = next_window;
                if my_depth > 1 {
                    eprintln!(
                        "[lookback W={propose_w}] вступаю как запасной ведущий (глубина {my_depth})"
                    );
                }

                // Уведомление-кандидат: сигнал «окно propose_w собирается»
                // + каноническое значение часов для отстающих узлов.
                let t_r_w = *t_r_history.get(&propose_w).unwrap_or(&timechain.t_r);
                let notify = {
                    let mut h = ProposalHeader {
                        prev_proposal_hash,
                        window_index: propose_w,
                        protocol_version: 1,
                        control_root: empty_internal(TREE_DEPTH),
                        node_root: [0u8; 32],
                        candidate_root: state.candidates.root(),
                        account_root: [0u8; 32],
                        state_root: [0u8; 32],
                        timechain_value: t_r_w,
                        included_bundles_root: [0u8; 32],
                        included_reveals_root: [0u8; 32],
                        winner_endpoint: [0u8; 32],
                        winner_id: my_node,
                        proposer_node_id: my_node,
                        target: u128::MAX,
                        fallback_depth: my_depth,
                        signature: Signature::from_array([0u8; SIGNATURE_SIZE]),
                    };
                    let mut scope = Vec::new();
                    h.encode_signed_scope(&mut scope);
                    h.signature = sign(&identity.node_sk, &scope).map_err(NodeError::Crypto)?;
                    let mut bytes = Vec::with_capacity(3722);
                    h.encode(&mut bytes);
                    bytes
                };
                if let Some(ref handle) = network_handle {
                    let _ = handle.broadcast_tx.send(ProtocolMessage::new(
                        MsgType::Proposal,
                        propose_w,
                        notify.clone(),
                    ));
                }

                // Ожидание кворума подтверждений окна propose_w.
                // Спецификация «Свойство темпа сети»: быстрейший узел ждёт,
                // пока 67% активной длины цепочки подтвердят окно.
                let mut last_notify = Instant::now();
                loop {
                    if let Some(ref mut handle) = network_handle {
                        let mut ctx = net_ctx!();
                        drain_network(&mut ctx, handle)?;
                    }
                    if current >= propose_w || STOP.load(Ordering::SeqCst) {
                        break;
                    }
                    let active_cl = active_chain_length_at(&state.nodes, propose_w, params);
                    let need = quorum(active_cl);
                    let got: u64 = bc_accumulator
                        .get(&propose_w)
                        .map(|m| {
                            m.keys()
                                .filter_map(|id| state.nodes.get(id).map(|n| n.chain_length))
                                .sum()
                        })
                        .unwrap_or(0);
                    if got >= need {
                        break;
                    }
                    if last_notify.elapsed() > Duration::from_secs(10) {
                        if let Some(ref handle) = network_handle {
                            let _ = handle.broadcast_tx.send(ProtocolMessage::new(
                                MsgType::Proposal,
                                propose_w,
                                notify.clone(),
                            ));
                        }
                        last_notify = Instant::now();
                    }
                    std::thread::sleep(Duration::from_millis(20));
                }
                if current >= propose_w {
                    // Окно зацементировал другой законный ведущий.
                    break 'active_arm;
                }
                if STOP.load(Ordering::SeqCst) {
                    break 'active_arm;
                }

                // Цементация: улики = подтверждения окна propose_w,
                // included_bundles = подтверждения окна propose_w-1.
                let active_cl = active_chain_length_at(&state.nodes, propose_w, params);
                let need = quorum(active_cl);
                let evidence = bc_accumulator.get(&propose_w).cloned().unwrap_or_default();
                let mut cemented = weighted_cemented_hashes(&evidence, &state.nodes, need);
                cemented.sort();
                let prev_w = propose_w.saturating_sub(1);
                let candidates_prev =
                    candidates_from_pool(reveal_pool.get(&prev_w), &cemented, &state.nodes);
                let (winner_id, winner_endpoint) = mt_lottery::determine_winner(&candidates_prev)
                    .map(|win| {
                        let ep = reveal_pool
                            .get(&prev_w)
                            .and_then(|m| m.get(&win.id))
                            .map(|r| r.endpoint)
                            .unwrap_or([0u8; 32]);
                        (win.id, ep)
                    })
                    .unwrap_or((my_node, [0u8; 32]));
                let bundles_prev: Vec<BundledConfirmation> = if propose_w >= 1 {
                    bc_accumulator
                        .get(&prev_w)
                        .map(|m| m.values().cloned().collect())
                        .unwrap_or_default()
                } else {
                    Vec::new()
                };
                let confirmers: Vec<mt_state::NodeId> =
                    bundles_prev.iter().map(|b| b.node_id).collect();
                let included_bundles_root = meta_root(
                    &bundles_prev
                        .iter()
                        .map(|b| (b.node_id, bundle_hash(b)))
                        .collect::<Vec<_>>(),
                );
                let reveal_entries: Vec<(mt_state::NodeId, Hash32)> = reveal_pool
                    .get(&prev_w)
                    .map(|m| {
                        m.values()
                            .filter(|r| cemented.binary_search(&reveal_hash(r)).is_ok())
                            .map(|r| (r.node_id, reveal_hash(r)))
                            .collect()
                    })
                    .unwrap_or_default();
                let included_reveals_root = meta_root(&reveal_entries);

                eprintln!(
                    "[lottery W={prev_w}] кандидатов={} победитель={}",
                    candidates_prev.len(),
                    hex16(&winner_id)
                );

                // Единый переход состояния (тот же, что у ведомых).
                let post_root = {
                    let mut ctx = net_ctx!();
                    settle_and_bookkeep(
                        &mut ctx,
                        propose_w,
                        winner_id,
                        &confirmers,
                        candidates_prev.clone(),
                    )?
                };
                let recomputed = compute_state_root(
                    &state.nodes.root(),
                    &state.candidates.root(),
                    &state.accounts.root(),
                );
                if recomputed != post_root {
                    panic!(
                        "state_root self-verify failed: {:02x?}.. ≠ {:02x?}..",
                        &post_root[..4],
                        &recomputed[..4]
                    );
                }

                let mut header = ProposalHeader {
                    prev_proposal_hash,
                    window_index: propose_w,
                    protocol_version: 1,
                    control_root: empty_internal(TREE_DEPTH),
                    node_root: state.nodes.root(),
                    candidate_root: state.candidates.root(),
                    account_root: state.accounts.root(),
                    state_root: post_root,
                    timechain_value: t_r_w,
                    included_bundles_root,
                    included_reveals_root,
                    winner_endpoint,
                    winner_id,
                    proposer_node_id: my_node,
                    target: u128::MAX,
                    fallback_depth: my_depth,
                    signature: Signature::from_array([0u8; SIGNATURE_SIZE]),
                };
                let mut header_scope = Vec::new();
                header.encode_signed_scope(&mut header_scope);
                header.signature =
                    sign(&identity.node_sk, &header_scope).map_err(NodeError::Crypto)?;

                // Конверт: [header 3722][u16 n1][BC окна w-1][u16 n2][BC окна w].
                let mut envelope_payload: Vec<u8> =
                    Vec::with_capacity(3722 + 4 + 3500 * (bundles_prev.len() + evidence.len()));
                header.encode(&mut envelope_payload);
                envelope_payload.extend_from_slice(&(bundles_prev.len() as u16).to_le_bytes());
                for bc in &bundles_prev {
                    bc.encode(&mut envelope_payload);
                }
                envelope_payload.extend_from_slice(&(evidence.len() as u16).to_le_bytes());
                for bc in evidence.values() {
                    bc.encode(&mut envelope_payload);
                }
                if let Some(ref handle) = network_handle {
                    let envelope = ProtocolMessage::new(
                        MsgType::Proposal,
                        propose_w,
                        envelope_payload.clone(),
                    );
                    if let Err(e) = handle.broadcast_tx.send(envelope) {
                        eprintln!(
                            "[consensus] broadcast CEMENTED Proposal w={propose_w} failed: {e}"
                        );
                    } else {
                        eprintln!(
                            "[consensus] broadcast CEMENTED Proposal window={propose_w} → peers (bundles={}, evidence={})",
                            bundles_prev.len(),
                            evidence.len()
                        );
                    }
                }
                store
                    .archive_proposal_envelope(propose_w, &envelope_payload)
                    .map_err(|e| {
                        NodeError::InvalidArguments(format!("archive_proposal_envelope: {e:?}"))
                    })?;
                recent_roots.insert(propose_w, post_root);
                bound_map(&mut recent_roots);
                prev_proposal_hash = proposal_hash(&header);
                last_proposer_cement.insert(my_node, propose_w);
                last_cement_at = Instant::now();
                session_emitted = session_emitted.saturating_add(params.emission_moneta);
                session_windows += 1;
            },
        }

        save_progress(&data_dir, &state, &timechain, &lifecycle, current)?;
    }

    let elapsed = session_start.elapsed();
    println!();
    println!("--- сессия завершена ---");
    println!("phase            : {:?}", lifecycle.phase);
    println!(
        "candidate VDF    : {}/{}",
        lifecycle.candidate_progress, lifecycle.target_chain_length
    );
    println!("обработано окон  : {session_windows} (только в Active phase)");
    println!(
        "выплачено        : {} nɈ ({} Ɉ)",
        session_emitted,
        session_emitted / 1_000_000_000
    );
    println!("session wall     : {:.1}s", elapsed.as_secs_f64());
    let final_balance = state
        .accounts
        .get(&my_account)
        .map(|a| a.balance)
        .unwrap_or(0);
    println!(
        "balance final    : {} nɈ ({} Ɉ)",
        final_balance,
        final_balance / 1_000_000_000
    );

    Ok(())
}

fn save_progress(
    data_dir: &std::path::Path,
    state: &LocalState,
    timechain: &TimeChainState,
    lifecycle: &NodeLifecycle,
    window: u64,
) -> Result<(), NodeError> {
    state.save(data_dir)?;
    save_current_window(data_dir, window)?;
    save_timechain(data_dir, timechain)?;
    save_lifecycle(data_dir, lifecycle)?;
    Ok(())
}

fn install_shutdown_handlers() {
    STOP.store(false, Ordering::SeqCst);
    unsafe {
        // SAFETY: shutdown_handler — extern "C" функция с правильной signature
        // (void (*)(int)). libc::signal принимает usize-cast pointer и
        // регистрирует обработчик. Atomic store в глобальный flag —
        // единственный side-effect handler-а, signal-safe (POSIX async-signal-safe).
        // SIGINT — Ctrl-C из терминала. SIGTERM — launchctl unload / systemd stop /
        // kill PID. Оба роутятся на тот же handler для единообразного graceful shutdown.
        libc::signal(libc::SIGINT, shutdown_handler as *const () as usize);
        libc::signal(libc::SIGTERM, shutdown_handler as *const () as usize);
    }
}

// Декомпозиция vdf_step(prev, d) на chunks с прогрессом в stdout.
// Корректность byte-exact: SHA-256^d ассоциативно по composition,
// vdf_step(vdf_step(x, a), b) = vdf_step(x, a + b) для a + b = d.
//
// Boundaries вычисляются как (d × i) / N — точно делит D на N равных
// долей даже при D не кратном N (последняя chunk может быть на 1 итерацию
// больше из-за rounding, но проценты в выводе всегда точно 10, 20, …, 100).
const VDF_PROGRESS_CHUNKS: u64 = 10;

/// Тайм-аут молчания ведущего: после стольких секунд без цементирования
/// каскад запасных сдвигается на одну позицию (спецификация «Fallback cascade»).
const FALLBACK_TIMEOUT_SECS: u64 = 120;
/// Глубина хранения пооконных историй (пулы, победители, наборы подтвердивших).
const HISTORY_BOUND: usize = 64;

fn bound_map<K: Ord + Copy, V>(m: &mut BTreeMap<K, V>) {
    while m.len() > HISTORY_BOUND {
        let k = *m.keys().next().unwrap();
        m.remove(&k);
    }
}

/// Активная длина цепочки по спецификации «Confirmations»: учитываются только
/// узлы с подтверждением за последние 2τ₂ (active predicate); start_window
/// служит нижней границей для свежеактивированных узлов.
fn active_chain_length_at(
    nodes: &mt_state::NodeTable,
    w: u64,
    params: &mt_genesis::ProtocolParams,
) -> u64 {
    let horizon = 2 * params.tau2_windows;
    nodes
        .iter()
        .filter(|n| {
            let last_seen = n.last_confirmation_window.max(n.start_window);
            w.saturating_sub(last_seen) <= horizon
        })
        .map(|n| n.chain_length)
        .sum()
}

/// Весь консенсусный контекст узла одним пакетом ссылок — общий для
/// разборщика сообщений, ведущего и эпилога витка.
struct NetCtx<'a> {
    state: &'a mut LocalState,
    current: &'a mut u64,
    timechain: &'a mut TimeChainState,
    recent_roots: &'a mut BTreeMap<u64, Hash32>,
    t_r_history: &'a mut BTreeMap<u64, Hash32>,
    reveal_pool: &'a mut BTreeMap<u64, BTreeMap<mt_state::NodeId, VdfReveal>>,
    bc_accumulator: &'a mut BTreeMap<u64, BTreeMap<mt_state::NodeId, BundledConfirmation>>,
    winner_history: &'a mut BTreeMap<u64, mt_state::NodeId>,
    lottery_history: &'a mut BTreeMap<u64, Vec<mt_lottery::Candidate>>,
    bc_set_history: &'a mut BTreeMap<u64, Vec<mt_state::NodeId>>,
    last_proposer_cement: &'a mut BTreeMap<mt_state::NodeId, u64>,
    own_bc_cache: &'a mut BTreeMap<u64, BundledConfirmation>,
    pending_msgs: &'a mut Vec<ProtocolMessage>,
    fast_sync: &'a mut Option<mt_sync::FastSyncClient>,
    fast_sync_deadline: &'a mut Option<Instant>,
    last_cement_at: &'a mut Instant,
    prev_proposal_hash: &'a mut Hash32,
    fast_sync_lag_threshold: u64,
    data_dir: &'a std::path::Path,
    params: &'static mt_genesis::ProtocolParams,
    store: &'a FsStore,
    my_node: mt_state::NodeId,
    bootstrap_node_id: mt_state::NodeId,
}

/// Канонический агрегат подтверждений окна w: набор подтвердивших берётся из
/// included_bundles предложения, закрывшего окно w (bc_set_history).
fn cba_from(history: &BTreeMap<u64, Vec<mt_state::NodeId>>, w: u64) -> Hash32 {
    let set = history.get(&w).map(|v| v.as_slice()).unwrap_or(&[]);
    cemented_bundle_aggregate(w, set)
}

fn cba_for(ctx: &NetCtx, w: u64) -> Hash32 {
    cba_from(ctx.bc_set_history, w)
}

/// Законный ведущий окна w по спецификации Lookback Leadership:
/// первые два окна — узел-первопоселенец; далее победитель окна w-2,
/// при молчании — каскад запасных по возрастанию взвешенного билета окна w-2.
fn expected_proposer(ctx: &NetCtx, w: u64, silence_secs: u64) -> (mt_state::NodeId, u8) {
    if w < 2 {
        return (ctx.bootstrap_node_id, 1);
    }
    let depth_extra = (silence_secs / FALLBACK_TIMEOUT_SECS).min(254) as u8;
    if depth_extra == 0 {
        if let Some(p) = ctx.winner_history.get(&(w - 2)) {
            return (*p, 1);
        }
    }
    let sorted = ctx
        .lottery_history
        .get(&(w - 2))
        .cloned()
        .unwrap_or_default();
    let depth = 1u8.saturating_add(depth_extra);
    (
        mt_consensus::fallback_proposer(w, ctx.bootstrap_node_id, &sorted, depth),
        depth,
    )
}

/// Разбор 3722-байтного заголовка предложения из канонического layout.
fn parse_header(payload: &[u8]) -> Option<ProposalHeader> {
    if payload.len() < 3722 {
        return None;
    }
    let h32 = |a: usize| -> Hash32 {
        let mut b = [0u8; 32];
        b.copy_from_slice(&payload[a..a + 32]);
        b
    };
    let mut w8 = [0u8; 8];
    w8.copy_from_slice(&payload[32..40]);
    let mut v4 = [0u8; 4];
    v4.copy_from_slice(&payload[40..44]);
    let mut t16 = [0u8; 16];
    t16.copy_from_slice(&payload[396..412]);
    let mut sig = [0u8; SIGNATURE_SIZE];
    sig.copy_from_slice(&payload[413..3722]);
    Some(ProposalHeader {
        prev_proposal_hash: h32(0),
        window_index: u64::from_le_bytes(w8),
        protocol_version: u32::from_le_bytes(v4),
        control_root: h32(44),
        node_root: h32(76),
        candidate_root: h32(108),
        account_root: h32(140),
        state_root: h32(172),
        timechain_value: h32(204),
        included_bundles_root: h32(236),
        included_reveals_root: h32(268),
        winner_endpoint: h32(300),
        winner_id: h32(332),
        proposer_node_id: h32(364),
        target: u128::from_le_bytes(t16),
        fallback_depth: payload[412],
        signature: Signature::from_array(sig),
    })
}

/// Разбор хвоста зацементированного конверта:
/// [u16 n1][n1 × BC окна w-1][u16 n2][n2 × BC окна w].
fn parse_envelope_bundles(
    payload: &[u8],
) -> Option<(Vec<BundledConfirmation>, Vec<BundledConfirmation>)> {
    let mut off = 3722usize;
    let mut lists: Vec<Vec<BundledConfirmation>> = Vec::with_capacity(2);
    for _ in 0..2 {
        if payload.len() < off + 2 {
            return None;
        }
        let mut cbuf = [0u8; 2];
        cbuf.copy_from_slice(&payload[off..off + 2]);
        let n = u16::from_le_bytes(cbuf) as usize;
        off += 2;
        let mut list = Vec::with_capacity(n);
        for _ in 0..n {
            match BundledConfirmation::decode(&payload[off..]) {
                Ok((bc, used)) => {
                    list.push(bc);
                    off += used;
                },
                Err(_) => return None,
            }
        }
        lists.push(list);
    }
    let second = lists.pop().unwrap();
    let first = lists.pop().unwrap();
    Some((first, second))
}

/// Дерево Меркла поверх (node_id ‖ hash)-пар — формат included_bundles_root /
/// included_reveals_root из спецификации «Структура proposal-level Merkle roots».
fn meta_root(entries: &[(mt_state::NodeId, Hash32)]) -> Hash32 {
    if entries.is_empty() {
        return empty_internal(TREE_DEPTH);
    }
    let mut tree = SparseMerkleTree::new();
    for (id, h) in entries {
        let mut meta = Vec::with_capacity(64);
        meta.extend_from_slice(id);
        meta.extend_from_slice(h);
        tree.insert(*id, &meta);
    }
    tree.root()
}

/// Взвешенная цементация билетов: хэш билета зацементирован, когда суммарная
/// длина цепочки подтвердивших его узлов достигает кворума (67% активной длины).
fn weighted_cemented_hashes(
    evidence: &BTreeMap<mt_state::NodeId, BundledConfirmation>,
    nodes: &mt_state::NodeTable,
    need_quorum: u64,
) -> Vec<Hash32> {
    let mut weight: BTreeMap<Hash32, u64> = BTreeMap::new();
    for (nid, bc) in evidence {
        let cl = nodes.get(nid).map(|n| n.chain_length).unwrap_or(0);
        for rh in &bc.reveal_hashes {
            *weight.entry(*rh).or_insert(0) += cl;
        }
    }
    weight
        .into_iter()
        .filter(|(_, w)| *w >= need_quorum)
        .map(|(h, _)| h)
        .collect()
}

/// Отсортированный список кандидатов розыгрыша окна w из локального пула,
/// ограниченный данным набором зацементированных хэшей.
fn candidates_from_pool(
    pool: Option<&BTreeMap<mt_state::NodeId, VdfReveal>>,
    cemented: &[Hash32],
    nodes: &mt_state::NodeTable,
) -> Vec<mt_lottery::Candidate> {
    let cem: std::collections::BTreeSet<&Hash32> = cemented.iter().collect();
    let mut v: Vec<mt_lottery::Candidate> = pool
        .map(|m| {
            m.values()
                .filter(|r| cem.contains(&reveal_hash(r)))
                .filter_map(|r| {
                    nodes.get(&r.node_id).map(|n| {
                        let snapshot = n.chain_length_snapshot.max(1);
                        mt_lottery::Candidate {
                            ticket: weighted_ticket_node(&r.endpoint, n.chain_length, snapshot),
                            class: mt_lottery::WINNER_CLASS_NODE,
                            id: r.node_id,
                        }
                    })
                })
                .collect()
        })
        .unwrap_or_default();
    v.sort_by(|a, b| a.ticket.cmp(&b.ticket).then(a.id.cmp(&b.id)));
    v
}

/// Единый переход состояния при закрытии окна settled_w — общий для ведущего
/// и ведомых (детерминизм): apply_proposal + истечение кандидатов + событие
/// отбора + адаптация D на границе τ₂ + пооконные истории.
#[allow(clippy::too_many_arguments)]
fn settle_and_bookkeep(
    ctx: &mut NetCtx,
    settled_w: u64,
    winner_id: Hash32,
    confirmers: &[mt_state::NodeId],
    candidates_w_minus_1: Vec<mt_lottery::Candidate>,
) -> Result<Hash32, NodeError> {
    let settle = ProposalSettle {
        window_w: settled_w,
        winner_id,
        cemented_confirmers: confirmers.to_vec(),
    };
    let post_root = apply_proposal(
        &mut ctx.state.accounts,
        &mut ctx.state.nodes,
        &ctx.state.candidates,
        &settle,
        ctx.params,
    );
    let _ = apply_candidate_expiry(&mut ctx.state.candidates, settled_w);
    if is_selection_window(settled_w, ctx.params) {
        let active = ctx.state.nodes.len() as u64;
        let cba = cba_for(ctx, settled_w.saturating_sub(2));
        let t_r_s = ctx
            .t_r_history
            .get(&settled_w)
            .copied()
            .unwrap_or(ctx.timechain.t_r);
        let activated = apply_selection_event(
            &mut ctx.state.candidates,
            &mut ctx.state.nodes,
            &mut ctx.state.accounts,
            &t_r_s,
            &cba,
            active,
            settled_w,
            ctx.params,
        );
        if !activated.is_empty() {
            println!(
                "[selection W={settled_w}] активировано {} узл(ов)",
                activated.len()
            );
        }
    }
    if settled_w > 0 && settled_w % ctx.params.tau2_windows == 0 {
        let median_permille = 1000u32;
        let new_d = next_d(ctx.timechain.current_d, median_permille, ctx.params);
        if new_d != ctx.timechain.current_d {
            println!(
                "[next_d W={settled_w}] D: {} → {}",
                ctx.timechain.current_d, new_d
            );
            ctx.timechain.current_d = new_d;
        }
    }
    if settled_w >= 1 {
        ctx.bc_set_history
            .insert(settled_w - 1, confirmers.to_vec());
        ctx.winner_history.insert(settled_w - 1, winner_id);
        ctx.lottery_history
            .insert(settled_w - 1, candidates_w_minus_1);
    }
    bound_map(ctx.bc_set_history);
    bound_map(ctx.winner_history);
    bound_map(ctx.lottery_history);
    ctx.bc_accumulator.remove(&settled_w.saturating_sub(1));
    bound_map(ctx.bc_accumulator);
    *ctx.current = settled_w;
    save_current_window(ctx.data_dir, settled_w)?;
    ctx.store
        .save_meta_last_cemented(settled_w)
        .map_err(|e| NodeError::InvalidArguments(format!("save_meta_last_cemented: {e:?}")))?;
    Ok(post_root)
}

/// Обработка одного входящего протокольного сообщения. Вызывается из
/// drain_network в начале витка, между порциями последовательной SHA-256
/// цепочки и из цикла ожидания кворума ведущего — одна логика везде.
fn handle_protocol_message(
    ctx: &mut NetCtx,
    broadcast_tx: &tokio::sync::mpsc::UnboundedSender<mt_net::ProtocolMessage>,
    msg: ProtocolMessage,
) -> Result<(), NodeError> {
    match msg.msg_type {
        MsgType::Proposal => {
            let Some(header) = parse_header(&msg.payload) else {
                eprintln!(
                    "[consensus] Proposal envelope wrong size {} — skip",
                    msg.payload.len()
                );
                return Ok(());
            };
            let w = header.window_index;
            let is_cemented = msg.payload.len() > 3722;
            // Сверка канонических часов: одно окно — одно значение цепочки времени.
            if let Some(known) = ctx.t_r_history.get(&w) {
                if *known != header.timechain_value {
                    eprintln!(
                        "[consensus] РАСХОЖДЕНИЕ ЦЕПОЧКИ ВРЕМЕНИ w={w}: конверт ≠ локально — skip"
                    );
                    return Ok(());
                }
            }
            if !is_cemented {
                // Уведомление-кандидат: сигнал «окно w собирается». Если наш BC
                // этого окна уже опубликован — повторить (восстановление ведущего
                // после рестарта). Самим пересчитывать BC из чужого t_r нельзя:
                // подтверждение окна допустимо только от собственных часов.
                if header.proposer_node_id != ctx.my_node {
                    if let Some(bc) = ctx.own_bc_cache.get(&w) {
                        let mut bc_payload = Vec::new();
                        bc.encode(&mut bc_payload);
                        let _ = broadcast_tx.send(ProtocolMessage::new(
                            MsgType::BundledConfirmation,
                            w,
                            bc_payload,
                        ));
                    }
                }
                return Ok(());
            }
            // --- зацементированный конверт ---
            // DEV-018c: сброс зависшего клиента быстрой синхронизации по дедлайну.
            if let Some(deadline) = *ctx.fast_sync_deadline {
                if Instant::now() > deadline {
                    eprintln!("[m7] fast-sync deadline exceeded — drop client, retry");
                    *ctx.fast_sync = None;
                    *ctx.fast_sync_deadline = None;
                }
            }
            if ctx.fast_sync.is_some() {
                return Ok(());
            }
            if w.saturating_sub(*ctx.current) > ctx.fast_sync_lag_threshold {
                let mut fs_payload = Vec::new();
                mt_net::FastSyncRequest {
                    anchor_window: w,
                    resume_offset: 0,
                }
                .encode(&mut fs_payload);
                match broadcast_tx.send(ProtocolMessage::new(
                    MsgType::FastSyncRequest,
                    msg.request_id,
                    fs_payload,
                )) {
                    Ok(()) => {
                        eprintln!(
                            "[m7] {} windows behind (> {}) → fast-sync anchored at window {w}",
                            w.saturating_sub(*ctx.current),
                            ctx.fast_sync_lag_threshold
                        );
                        *ctx.fast_sync = Some(mt_sync::FastSyncClient::new());
                        *ctx.fast_sync_deadline =
                            Some(Instant::now() + std::time::Duration::from_secs(10));
                    },
                    Err(e) => eprintln!("[m7] FastSyncRequest broadcast failed: {e}"),
                }
                return Ok(());
            }
            if w != *ctx.current + 1 {
                if w > *ctx.current {
                    eprintln!(
                        "[consensus] cemented w={w} gap (current={}) — жду последовательного цементирования или быстрой синхронизации",
                        *ctx.current
                    );
                }
                return Ok(());
            }
            // Спецификация «Финальность proposal»: подпись ведущего проверяется
            // по таблице узлов (НЕ только первопоселенец), окно монотонно.
            if let Err(e) =
                mt_consensus::validate_header(&header, &ctx.state.nodes, *ctx.current, 1, 1)
            {
                eprintln!("[consensus] header w={w} отклонён: {e:?}");
                return Ok(());
            }
            // Законность ведущего (Lookback + каскад). Терпимость ±1 уровень
            // глубины на расхождение настенных часов между узлами.
            let silence = ctx.last_cement_at.elapsed().as_secs();
            let (exp_now, _) = expected_proposer(ctx, w, silence);
            let (exp_next, _) = expected_proposer(ctx, w, silence + FALLBACK_TIMEOUT_SECS);
            let proposer_ok = header.proposer_node_id == exp_now
                || header.proposer_node_id == exp_next
                || header.proposer_node_id == ctx.bootstrap_node_id;
            if !proposer_ok {
                eprintln!(
                    "[consensus] w={w}: ведущий {} не является законным (ожидался {}) — skip",
                    hex16(&header.proposer_node_id),
                    hex16(&exp_now)
                );
                return Ok(());
            }
            let Some((bundles_prev, evidence)) = parse_envelope_bundles(&msg.payload) else {
                eprintln!("[consensus] cemented w={w}: хвост конверта не разобран — skip");
                return Ok(());
            };
            let active_cl = active_chain_length_at(&ctx.state.nodes, w, ctx.params);
            let need_quorum = quorum(active_cl);
            // Проверка included_bundles (подтверждения окна w-1).
            let mut confirmers: Vec<mt_state::NodeId> = Vec::new();
            if w >= 1 {
                let t_r_prev = ctx.t_r_history.get(&(w - 1)).copied().or_else(|| {
                    // После рестарта истории может не быть: единогласный endpoint
                    // кворумного набора w-1 канонически задаёт T_r(w-1).
                    let mut it = bundles_prev.iter().map(|b| b.endpoint);
                    let first = it.next()?;
                    it.all(|e| e == first).then_some(first)
                });
                if let Some(t_r_prev) = t_r_prev {
                    ctx.t_r_history.entry(w - 1).or_insert(t_r_prev);
                    let mut sum = 0u64;
                    for bc in &bundles_prev {
                        if validate_bundle(bc, &ctx.state.nodes, &t_r_prev).is_ok() {
                            sum += ctx
                                .state
                                .nodes
                                .get(&bc.node_id)
                                .map(|n| n.chain_length)
                                .unwrap_or(0);
                            confirmers.push(bc.node_id);
                        }
                    }
                    if mt_consensus::validate_bundles_threshold(sum, active_cl).is_err() {
                        eprintln!(
                            "[consensus] w={w}: included_bundles {sum} < кворума {need_quorum} — skip"
                        );
                        return Ok(());
                    }
                } else if !bundles_prev.is_empty() {
                    eprintln!("[consensus] w={w}: разногласие endpoint в included_bundles — skip");
                    return Ok(());
                }
            }
            confirmers.sort();
            // Проверка цементации билетов окна w-1 уликами окна w (взвешенно).
            let mut ev_map: BTreeMap<mt_state::NodeId, BundledConfirmation> = BTreeMap::new();
            let mut ev_sum = 0u64;
            for bc in &evidence {
                if validate_bundle(bc, &ctx.state.nodes, &header.timechain_value).is_ok() {
                    ev_sum += ctx
                        .state
                        .nodes
                        .get(&bc.node_id)
                        .map(|n| n.chain_length)
                        .unwrap_or(0);
                    ev_map.insert(bc.node_id, bc.clone());
                }
            }
            if w >= 1 && ev_sum < need_quorum {
                eprintln!(
                    "[consensus] w={w}: улики цементации {ev_sum} < кворума {need_quorum} — skip"
                );
                return Ok(());
            }
            let mut cemented = weighted_cemented_hashes(&ev_map, &ctx.state.nodes, need_quorum);
            cemented.sort();
            // included_reveals_root обязан совпасть с нашим пересчётом.
            let reveal_entries: Vec<(mt_state::NodeId, Hash32)> = ctx
                .reveal_pool
                .get(&w.saturating_sub(1))
                .map(|m| {
                    m.values()
                        .filter(|r| cemented.binary_search(&reveal_hash(r)).is_ok())
                        .map(|r| (r.node_id, reveal_hash(r)))
                        .collect()
                })
                .unwrap_or_default();
            let candidates_prev = candidates_from_pool(
                ctx.reveal_pool.get(&w.saturating_sub(1)),
                &cemented,
                &ctx.state.nodes,
            );
            if reveal_entries.len() == cemented.len() {
                if meta_root(&reveal_entries) != header.included_reveals_root {
                    eprintln!("[consensus] w={w}: included_reveals_root mismatch — skip");
                    return Ok(());
                }
                if w >= 1 && !candidates_prev.is_empty() {
                    if let Err(e) = mt_consensus::validate_winner(&header, &candidates_prev) {
                        eprintln!("[consensus] w={w}: победитель не сходится ({e:?}) — skip");
                        return Ok(());
                    }
                }
            } else {
                eprintln!(
                    "[consensus] w={w}: пул билетов неполон ({}/{}) — победитель принят по кворуму",
                    reveal_entries.len(),
                    cemented.len()
                );
            }
            // Применение единым переходом состояния.
            let post_root =
                settle_and_bookkeep(ctx, w, header.winner_id, &confirmers, candidates_prev)?;
            if post_root != header.state_root {
                panic!(
                    "расхождение state_root в окне {w}: конверт {:02x?}.. ≠ локально {:02x?}..",
                    &header.state_root[..4],
                    &post_root[..4]
                );
            }
            ctx.recent_roots.insert(w, header.state_root);
            bound_map(ctx.recent_roots);
            ctx.t_r_history.insert(w, header.timechain_value);
            bound_map(ctx.t_r_history);
            if ctx.timechain.last_window < w {
                // Каноническое значение часов из кворумного конверта — догоняем.
                ctx.timechain.t_r = header.timechain_value;
                ctx.timechain.last_window = w;
            }
            ctx.store
                .archive_proposal_envelope(w, &msg.payload)
                .map_err(|e| {
                    NodeError::InvalidArguments(format!("archive_proposal_envelope: {e:?}"))
                })?;
            *ctx.prev_proposal_hash = proposal_hash(&header);
            ctx.last_proposer_cement.insert(header.proposer_node_id, w);
            *ctx.last_cement_at = Instant::now();
            eprintln!(
                "[consensus] applied cemented Proposal w={w} (confirmers={}, winner={})",
                confirmers.len(),
                hex16(&header.winner_id)
            );
        },
        MsgType::FastSyncRequest => match mt_net::FastSyncRequest::decode(&msg.payload) {
            Ok(req) => {
                let snap = mt_sync::Snapshot::from_tables(
                    *ctx.current,
                    &ctx.state.accounts,
                    &ctx.state.nodes,
                    &ctx.state.candidates,
                );
                let chunks = snap.to_wire_chunks(32);
                let total = chunks.len();
                for chunk in chunks {
                    let table_id_byte = match chunk.table_id {
                        mt_sync::FastSyncTableId::Account => mt_net::TableId::Account,
                        mt_sync::FastSyncTableId::Node => mt_net::TableId::Node,
                        mt_sync::FastSyncTableId::Candidate => mt_net::TableId::Candidate,
                        mt_sync::FastSyncTableId::Proposals => mt_net::TableId::Proposals,
                    };
                    let mut flat: Vec<u8> = Vec::new();
                    for r in &chunk.records {
                        flat.extend_from_slice(r);
                    }
                    let wire_chunk = mt_net::FastSyncResponseChunk {
                        chunk_index: chunk.chunk_index,
                        total_chunks: chunk.total_chunks,
                        table_id: table_id_byte,
                        record_count: chunk.records.len() as u32,
                        anchor_window: *ctx.current,
                        records: flat,
                    };
                    let mut payload = Vec::new();
                    wire_chunk.encode(&mut payload);
                    let envelope =
                        ProtocolMessage::new(MsgType::FastSyncResponse, msg.request_id, payload);
                    if broadcast_tx.send(envelope).is_err() {
                        break;
                    }
                }
                eprintln!(
                    "[m7] served FastSync snapshot: anchor_window={} req={} chunks={total}",
                    *ctx.current, req.anchor_window
                );
            },
            Err(e) => {
                eprintln!("[m7] FastSyncRequest decode failed: {e:?}");
            },
        },
        MsgType::FastSyncResponse => {
            if let Some(mut client) = ctx.fast_sync.take() {
                let chunk_anchor = mt_net::FastSyncResponseChunk::decode(&msg.payload)
                    .ok()
                    .map(|c| c.anchor_window)
                    .unwrap_or(0);
                if chunk_anchor <= *ctx.current {
                    eprintln!(
                        "[m7] discard FastSyncResponse anchor={chunk_anchor} <= current={} — drop client, retry on next cemented",
                        *ctx.current
                    );
                    drop(client);
                    *ctx.fast_sync_deadline = None;
                    return Ok(());
                }
                let parsed = mt_net::FastSyncResponseChunk::decode(&msg.payload)
                    .map_err(|e| format!("decode: {e:?}"))
                    .and_then(|wc| {
                        crate::commands::fastsync::wire_chunk_to_sync(wc)
                            .map_err(|e| format!("wire: {e:?}"))
                    });
                match parsed {
                    Ok(chunk) => match client.accept_chunk(chunk) {
                        Ok(mt_sync::AcceptOutcome::Complete) => {
                            match client.finalize(ctx.recent_roots) {
                                Ok((window, tables)) => {
                                    ctx.state.apply_fast_sync(tables, ctx.data_dir, window)?;
                                    *ctx.current = window;
                                    save_current_window(ctx.data_dir, window)?;
                                    *ctx.fast_sync_deadline = None;
                                    eprintln!(
                                    "[m7] fast-sync complete → state replaced, current_window={window}"
                                );
                                },
                                Err(e) => eprintln!(
                                    "[m7] fast-sync finalize rejected: {e:?} — retry on next lag"
                                ),
                            }
                        },
                        Ok(mt_sync::AcceptOutcome::Progress { received, total }) => {
                            eprintln!("[m7] fast-sync chunk {received}/{total}");
                            *ctx.fast_sync = Some(client);
                        },
                        Err(e) => eprintln!(
                            "[m7] fast-sync chunk rejected: {e:?} — discard, retry on next lag"
                        ),
                    },
                    Err(reason) => {
                        eprintln!("[m7] FastSyncResponse {reason}");
                        *ctx.fast_sync = Some(client);
                    },
                }
            }
        },
        MsgType::VdfReveal => {
            if let Ok((rec_reveal, _)) = VdfReveal::decode(&msg.payload) {
                let rw = rec_reveal.window_index;
                if !ctx.t_r_history.contains_key(&rw) && rw > ctx.timechain.last_window {
                    // Часы этого окна нам ещё неизвестны — отложить до своего витка.
                    if ctx.pending_msgs.len() < 512 {
                        ctx.pending_msgs.push(msg);
                    }
                    return Ok(());
                }
                let exp_t_r = ctx
                    .t_r_history
                    .get(&rw)
                    .copied()
                    .unwrap_or(ctx.timechain.t_r);
                let cba = cba_for(ctx, rw.saturating_sub(2));
                if validate_reveal(&rec_reveal, &ctx.state.nodes, &exp_t_r, &cba, rw).is_ok() {
                    let nid = rec_reveal.node_id;
                    ctx.reveal_pool
                        .entry(rw)
                        .or_default()
                        .insert(nid, rec_reveal);
                    bound_map(ctx.reveal_pool);
                    eprintln!("[lottery] принят билет от {} за окно {rw}", hex16(&nid));
                } else {
                    eprintln!(
                        "[lottery] билет {} w={rw} не прошёл проверку",
                        hex16(&rec_reveal.node_id)
                    );
                }
            }
        },
        MsgType::BundledConfirmation => match BundledConfirmation::decode(&msg.payload) {
            Ok((bc, _used)) => {
                let bw = bc.window_index;
                if !ctx.t_r_history.contains_key(&bw) && bw > ctx.timechain.last_window {
                    if ctx.pending_msgs.len() < 512 {
                        ctx.pending_msgs.push(msg);
                    }
                    return Ok(());
                }
                let expected_t_r = ctx
                    .t_r_history
                    .get(&bw)
                    .copied()
                    .unwrap_or(ctx.timechain.t_r);
                if validate_bundle(&bc, &ctx.state.nodes, &expected_t_r).is_ok() {
                    let node_id = bc.node_id;
                    ctx.bc_accumulator
                        .entry(bw)
                        .or_default()
                        .insert(node_id, bc);
                    eprintln!(
                        "[bc] принято подтверждение от {} за окно {bw}",
                        hex16(&node_id)
                    );
                } else {
                    eprintln!(
                        "[bc] подтверждение {} w={bw} не прошло проверку",
                        hex16(&bc.node_id)
                    );
                }
            },
            Err(e) => eprintln!("[bc] decode failed: {e:?}"),
        },
        _ => {},
    }
    Ok(())
}

/// Публикация артефактов окна w от собственных канонических часов:
/// билет розыгрыша (VDF_Reveal) окна w и подтверждение (BundledConfirmation)
/// окна w с хэшами билетов окна w-1 — спецификация «Confirmations»:
/// «Bundle содержит операции текущего окна W и VDF_Reveals предыдущего окна W-1».
#[allow(clippy::too_many_arguments)]
fn publish_window_artifacts(
    state: &LocalState,
    reveal_pool: &mut BTreeMap<u64, BTreeMap<mt_state::NodeId, VdfReveal>>,
    bc_accumulator: &mut BTreeMap<u64, BTreeMap<mt_state::NodeId, BundledConfirmation>>,
    own_bc_cache: &mut BTreeMap<u64, BundledConfirmation>,
    bc_set_history: &BTreeMap<u64, Vec<mt_state::NodeId>>,
    identity: &crate::identity::Identity,
    my_node: mt_state::NodeId,
    w: u64,
    t_r_w: &Hash32,
    broadcast_tx: Option<&tokio::sync::mpsc::UnboundedSender<mt_net::ProtocolMessage>>,
) -> Result<(), NodeError> {
    let cba = cba_from(bc_set_history, w.saturating_sub(2));
    let endpoint = compute_endpoint(t_r_w, &cba, &my_node, w);
    let mut reveal = VdfReveal {
        node_id: my_node,
        window_index: w,
        endpoint,
        signature: Signature::from_array([0u8; SIGNATURE_SIZE]),
    };
    let mut scope = Vec::new();
    reveal.encode_signed_scope(&mut scope);
    reveal.signature = sign(&identity.node_sk, &scope).map_err(NodeError::Crypto)?;
    validate_reveal(&reveal, &state.nodes, t_r_w, &cba, w)
        .map_err(|e| NodeError::InvalidArguments(format!("validate_reveal: {e:?}")))?;
    reveal_pool
        .entry(w)
        .or_default()
        .insert(my_node, reveal.clone());
    bound_map(reveal_pool);
    if let Some(tx) = broadcast_tx {
        let mut payload = Vec::new();
        reveal.encode(&mut payload);
        let _ = tx.send(ProtocolMessage::new(MsgType::VdfReveal, w, payload));
    }
    // Подтверждение окна w: хэши билетов предыдущего окна (двухоконный конвейер).
    let mut bc_reveal_hashes: Vec<Hash32> = if w >= 1 {
        reveal_pool
            .get(&(w - 1))
            .map(|m| m.values().map(reveal_hash).collect())
            .unwrap_or_default()
    } else {
        Vec::new()
    };
    bc_reveal_hashes.sort();
    let mut bc = BundledConfirmation {
        node_id: my_node,
        endpoint: *t_r_w,
        window_index: w,
        op_hashes: Vec::new(),
        reveal_hashes: bc_reveal_hashes,
        signature: Signature::from_array([0u8; SIGNATURE_SIZE]),
    };
    let mut bc_scope = Vec::new();
    bc.encode_signed_scope(&mut bc_scope);
    bc.signature = sign(&identity.node_sk, &bc_scope).map_err(NodeError::Crypto)?;
    validate_bundle(&bc, &state.nodes, t_r_w)
        .map_err(|e| NodeError::InvalidArguments(format!("validate_bundle: {e:?}")))?;
    bc_accumulator
        .entry(w)
        .or_default()
        .insert(my_node, bc.clone());
    own_bc_cache.insert(w, bc.clone());
    bound_map(own_bc_cache);
    if let Some(tx) = broadcast_tx {
        let mut payload = Vec::new();
        bc.encode(&mut payload);
        let _ = tx.send(ProtocolMessage::new(
            MsgType::BundledConfirmation,
            w,
            payload,
        ));
    }
    Ok(())
}

/// Вычитка входящей очереди: сперва отложенные сообщения, чьи окна уже
/// получили каноническое значение часов, затем свежие из канала.
fn drain_network(ctx: &mut NetCtx, handle: &mut NetworkHandle) -> Result<(), NodeError> {
    if !ctx.pending_msgs.is_empty() {
        let pend = std::mem::take(ctx.pending_msgs);
        let tx = handle.broadcast_tx.clone();
        for msg in pend {
            handle_protocol_message(ctx, &tx, msg)?;
        }
    }
    let tx = handle.broadcast_tx.clone();
    while let Ok(msg) = handle.incoming_rx.try_recv() {
        handle_protocol_message(ctx, &tx, msg)?;
    }
    Ok(())
}

fn vdf_step_chunked<F: FnMut()>(
    prev: &Hash32,
    d: u64,
    label: &str,
    window: u64,
    mut on_chunk: F,
) -> Hash32 {
    if d == 0 {
        return *prev;
    }
    let mut current = *prev;
    let chunk_start = Instant::now();
    let mut prev_boundary: u64 = 0;
    use std::io::Write;
    for i in 1..=VDF_PROGRESS_CHUNKS {
        // Boundary распределяет D ровно: (d × i) / N (overflow безопасен:
        // d ≤ 2^32 typical, × N=10 ≤ 2^36).
        let boundary = d.saturating_mul(i) / VDF_PROGRESS_CHUNKS;
        let this_chunk = boundary - prev_boundary;
        current = vdf_step(&current, this_chunk);
        prev_boundary = boundary;
        on_chunk();
        let percent = (i * 100) / VDF_PROGRESS_CHUNKS;
        let bar = progress_bar(boundary, d, 30);
        let elapsed = chunk_start.elapsed().as_secs_f64();
        let line = format!(
            "окно {window:>5} {label:<14} {} {:>3}% | {:>4}/{:>4} M | {:>5.1}s",
            bar,
            percent,
            boundary / 1_000_000,
            d / 1_000_000,
            elapsed
        );
        // `\r` в начале — carriage return; bar обновляется в одной logical
        // строке. Финал на 100% chunk завершается `\n`. Работает одинаково
        // в TTY (cursor возврат) и при просмотре через `tail -F` (терминал
        // рендерит `\r` как возврат курсора → animated bar).
        if i == VDF_PROGRESS_CHUNKS {
            println!("\r{line}");
        } else {
            print!("\r{line}");
            let _ = std::io::stdout().flush();
        }
    }
    current
}

fn progress_bar(done: u64, total: u64, width: usize) -> String {
    if total == 0 {
        return "[".to_string() + &"░".repeat(width) + "]";
    }
    let filled = ((done as u128) * (width as u128) / (total as u128)) as usize;
    let filled = filled.min(width);
    let empty = width - filled;
    let mut s = String::with_capacity(width + 2);
    s.push('[');
    for _ in 0..filled {
        s.push('▓');
    }
    for _ in 0..empty {
        s.push('░');
    }
    s.push(']');
    s
}

fn hex16(bytes: &[u8]) -> String {
    let take = bytes.len().min(8);
    let mut s = String::with_capacity(take * 2);
    for b in &bytes[..take] {
        s.push_str(&format!("{b:02x}"));
    }
    s
}

fn spawn_network_thread(
    identity: &crate::identity::Identity,
    listen_str: &str,
    manifest_path: &std::path::Path,
) -> Result<NetworkHandle, NodeError> {
    use std::str::FromStr;

    let manifest_text = std::fs::read_to_string(manifest_path).map_err(|e| {
        NodeError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("genesis-manifest {manifest_path:?}: {e}"),
        ))
    })?;
    let manifest = mt_genesis::GenesisManifest::parse(&manifest_text)
        .map_err(|e| NodeError::Network(format!("parse manifest: {e}")))?;

    let listen_addr = libp2p::Multiaddr::from_str(listen_str)
        .map_err(|e| NodeError::Network(format!("parse --listen {listen_str}: {e}")))?;

    let local_keypair = identity.libp2p_keypair();
    let mldsa_id_pk = identity.node_pk.clone();
    let mldsa_id_sk_bytes: [u8; mt_crypto::SECRET_KEY_SIZE] = *identity.node_sk.as_bytes();
    let mldsa_id_sk = mt_crypto::SecretKey::from_array(mldsa_id_sk_bytes);
    let local_peer_id = mt_net_transport::derive_peer_id(&identity.node_pk)
        .map_err(|e| NodeError::Network(format!("derive XX peer_id: {e}")))?;
    let manifest_clone = manifest.clone();

    let (broadcast_tx, broadcast_rx) =
        tokio::sync::mpsc::unbounded_channel::<mt_net::ProtocolMessage>();
    let (incoming_tx, incoming_rx) =
        tokio::sync::mpsc::unbounded_channel::<mt_net::ProtocolMessage>();

    eprintln!(
        "[main] spawning network thread, local_peer_id={local_peer_id},          listen={listen_addr}, peers={n}",
        n = manifest.peers.len()
    );

    std::thread::Builder::new()
        .name("montana-network".into())
        .spawn(move || {
            let runtime = match tokio::runtime::Builder::new_multi_thread()
                .worker_threads(2)
                .enable_all()
                .build()
            {
                Ok(rt) => rt,
                Err(e) => {
                    eprintln!("[network] failed to start tokio runtime: {e}");
                    return;
                },
            };
            if let Err(e) = runtime.block_on(crate::network::run_network_loop(
                local_keypair,
                local_peer_id,
                mldsa_id_pk,
                mldsa_id_sk,
                manifest_clone,
                listen_addr,
                broadcast_rx,
                incoming_tx,
            )) {
                eprintln!("[network] event loop exited with error: {e}");
            }
        })
        .map_err(|e| NodeError::Network(format!("spawn network thread: {e}")))?;

    Ok(NetworkHandle {
        broadcast_tx,
        incoming_rx,
    })
}

/// Handle к network thread. Через broadcast_tx consensus loop рассылает
/// envelope-ы всем подключённым peer-ам. Через incoming_rx consensus loop
/// принимает входящие envelope-ы (Proposal, BundledConfirmation, NodeRegistration,
/// ...) от peer-ов для apply.
pub struct NetworkHandle {
    pub broadcast_tx: tokio::sync::mpsc::UnboundedSender<mt_net::ProtocolMessage>,
    pub incoming_rx: tokio::sync::mpsc::UnboundedReceiver<mt_net::ProtocolMessage>,
}

#[cfg(test)]
mod tests {
    use super::resolve_fast_sync_lag_threshold as resolve;
    use super::FAST_SYNC_LAG_THRESHOLD as DEFAULT;

    #[test]
    fn lag_threshold_override_resolution() {
        assert_eq!(resolve(None), DEFAULT);
        assert_eq!(resolve(Some("5".to_string())), 5);
        assert_eq!(resolve(Some("  7 ".to_string())), 7);
        assert_eq!(resolve(Some("0".to_string())), DEFAULT);
        assert_eq!(resolve(Some("abc".to_string())), DEFAULT);
        assert_eq!(resolve(Some(String::new())), DEFAULT);
    }
}
