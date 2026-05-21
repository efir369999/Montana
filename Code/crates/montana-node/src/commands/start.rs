use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;

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
    bundle_hash, compute_endpoint, is_cemented, lottery_weight, quorum, reveal_hash,
    seniority_term, validate_bundle, validate_reveal, weighted_ticket_node, BundledConfirmation,
    VdfReveal,
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

    let mut state = LocalState::load_or_bootstrap(&data_dir, &identity, params)?;
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

    // Phase Bootstrap = первая загрузка. Для genesis узла (identity.node_pk
    // совпадает с params.bootstrap_node_pubkey либо placeholder pre-ceremony) —
    // переход прямо в Active. Для candidate узла — переход в CandidateVdf
    // с target_chain_length = τ₂ и w_start = текущее окно + 1.
    if lifecycle.phase == NodePhase::Bootstrap {
        if is_genesis {
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

    loop {
        // M9 Phase 2: drain incoming Proposal envelopes от bootstrap. Decode window_index
        // и proposer_node_id напрямую из 3722-байтного header layout без полного
        // deserialize (signature валидация в M10). Для каждого нового окна — apply_proposal
        // с reconstructed singleton ProposalSettle (winner = bootstrap, confirmers = [bootstrap]).
        // Followers: current_window растёт в lockstep с Moscow.
        if let Some(ref mut handle) = network_handle {
            let bootstrap_node_id = mt_state::derive_node_id(&params.bootstrap_node_pubkey);
            while let Ok(msg) = handle.incoming_rx.try_recv() {
                if msg.msg_type != MsgType::Proposal {
                    continue;
                }
                if msg.payload.len() != 3722 {
                    eprintln!(
                        "[consensus] Proposal envelope wrong size {} (expected 3722) — skip",
                        msg.payload.len()
                    );
                    continue;
                }
                let window_index = u64::from_le_bytes([
                    msg.payload[32],
                    msg.payload[33],
                    msg.payload[34],
                    msg.payload[35],
                    msg.payload[36],
                    msg.payload[37],
                    msg.payload[38],
                    msg.payload[39],
                ]);
                let mut winner_id = [0u8; 32];
                winner_id.copy_from_slice(&msg.payload[332..364]);
                let mut proposer_node_id = [0u8; 32];
                proposer_node_id.copy_from_slice(&msg.payload[364..396]);

                if proposer_node_id != bootstrap_node_id {
                    eprintln!("[consensus] Proposal от не-bootstrap proposer, skip");
                    continue;
                }
                if window_index <= current {
                    continue;
                }
                let mut applied_count = 0u64;
                while current < window_index {
                    let next_w = current + 1;
                    let settle = ProposalSettle {
                        window_w: next_w,
                        winner_id,
                        cemented_confirmers: vec![proposer_node_id],
                    };
                    let _post_state_root = apply_proposal(
                        &mut state.accounts,
                        &mut state.nodes,
                        &state.candidates,
                        &settle,
                        params,
                    );
                    current = next_w;
                    applied_count += 1;
                }
                save_current_window(&data_dir, current)?;
                eprintln!(
                    "[consensus] applied {applied_count} window(s) from peer Proposal → current_window={current}"
                );
            }
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

        let window_start = Instant::now();
        let next_window = current + 1;

        let next_t_r = vdf_step_chunked(&timechain.t_r, effective_d, "TimeChain VDF", next_window);

        match lifecycle.phase {
            NodePhase::Bootstrap => unreachable!("Bootstrap → CandidateVdf transition выше"),
            NodePhase::CandidateVdf => {
                lifecycle.candidate_endpoint = vdf_step_chunked(
                    &lifecycle.candidate_endpoint,
                    effective_d,
                    "Candidate VDF",
                    next_window,
                );
                lifecycle.candidate_progress = lifecycle
                    .candidate_progress
                    .checked_add(1)
                    .expect("candidate_progress overflow at u64::MAX");

                if lifecycle.candidate_progress >= lifecycle.target_chain_length {
                    let cba_w_start_minus_2 =
                        cemented_bundle_aggregate(lifecycle.w_start.saturating_sub(2), &[]);
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
                    let cba_w_p_minus_2 =
                        cemented_bundle_aggregate(next_window.saturating_sub(2), &[]);
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
                // SPEC DEVIATION DEV-012: singleton-only proposal generation.
                // Этот блок реализует proposal где my_node — единственный confirmer
                // (included_bundles = {my_bundle}, included_reveals = {my_reveal}),
                // что корректно ТОЛЬКО когда state.nodes == {my_node}. При наличии
                // других узлов в NodeTable spec требует cemented_sum = Σ chain_length
                // confirmer-узлов ≥ quorum(active_chain_length) — то есть сбор
                // BundledConfirmation от peer-ов через сеть (M9 Phase 2). Этот сбор
                // ещё не реализован (line 162-169: «log only; Phase 2 = apply_proposal»).
                // Пока M9 Phase 2 не подключён, минoritarian валидатор в multi-node
                // NodeTable работает как passive follower: не producит proposal,
                // break 'active_arm падает в post-match cleanup (candidate_expiry +
                // selection_event + next_d + save_progress) — узел остаётся жив.
                let is_singleton = state.nodes.len() == 1 && state.nodes.get(&my_node).is_some();
                if !is_singleton {
                    eprintln!(
                        "[active W={current}] singleton невозможен (NodeTable={} узлов), пропуск окна — жду peer Proposal (M9 Phase 2)",
                        state.nodes.len()
                    );
                    break 'active_arm;
                }
                let active_chain_length: u64 = state.nodes.iter().map(|n| n.chain_length).sum();
                let cba_w_minus_2 = cemented_bundle_aggregate(current.saturating_sub(2), &[]);
                let endpoint = compute_endpoint(&timechain.t_r, &cba_w_minus_2, &my_node, current);

                let mut reveal = VdfReveal {
                    node_id: my_node,
                    window_index: current,
                    endpoint,
                    signature: Signature::from_array([0u8; SIGNATURE_SIZE]),
                };
                let mut reveal_scope = Vec::new();
                reveal.encode_signed_scope(&mut reveal_scope);
                reveal.signature =
                    sign(&identity.node_sk, &reveal_scope).map_err(NodeError::Crypto)?;
                validate_reveal(
                    &reveal,
                    &state.nodes,
                    &timechain.t_r,
                    &cba_w_minus_2,
                    current,
                )
                .map_err(|e| NodeError::InvalidArguments(format!("validate_reveal: {e:?}")))?;
                let r_hash = reveal_hash(&reveal);

                let mut bc = BundledConfirmation {
                    node_id: my_node,
                    endpoint: timechain.t_r,
                    window_index: current,
                    op_hashes: Vec::new(),
                    reveal_hashes: vec![r_hash],
                    signature: Signature::from_array([0u8; SIGNATURE_SIZE]),
                };
                let mut bc_scope = Vec::new();
                bc.encode_signed_scope(&mut bc_scope);
                bc.signature = sign(&identity.node_sk, &bc_scope).map_err(NodeError::Crypto)?;
                validate_bundle(&bc, &state.nodes, &timechain.t_r)
                    .map_err(|e| NodeError::InvalidArguments(format!("validate_bundle: {e:?}")))?;
                let bc_h = bundle_hash(&bc);

                let my_node_record = state.nodes.get(&my_node).cloned().ok_or_else(|| {
                    NodeError::InvalidArguments("active phase но узел не в NodeTable".into())
                })?;
                let cemented_chain_length = my_node_record.chain_length;
                if !is_cemented(cemented_chain_length, active_chain_length) {
                    return Err(NodeError::InvalidArguments(format!(
                        "singleton cementing: cemented={cemented_chain_length}, active={active_chain_length}, quorum={}",
                        quorum(active_chain_length)
                    )));
                }

                let snapshot = my_node_record.chain_length_snapshot.max(1);
                let _weight = lottery_weight(my_node_record.chain_length, snapshot);
                let _term = seniority_term(my_node_record.chain_length, snapshot);
                let _ticket =
                    weighted_ticket_node(&endpoint, my_node_record.chain_length, snapshot);

                // Proposal-level Merkle roots строятся как Sparse Merkle Tree
                // глубины 256 (тот же primitive что для state-уровня), индексация
                // по natural keys per spec, раздел "Структура proposal-level
                // Merkle roots":
                //   - included_bundles_root: ключ = confirmer_node_id,
                //     значение = (confirmer_node_id || bundle_hash)
                //   - included_reveals_root: ключ = reveal_author_node_id,
                //     значение = (reveal_author_node_id || reveal_hash)
                //   - control_root: ключ = nodereg_hash,
                //     значение = canonical_bytes(control_object)
                // Empty marker для всех трёх = empty_internal(TREE_DEPTH).
                // Singleton: один confirmer = my_node, один reveal = my_node;
                // control_set пустой (нет cemented NodeRegistration в singleton).
                let included_bundles_root = {
                    let mut tree = SparseMerkleTree::new();
                    let mut bundle_meta = Vec::with_capacity(64);
                    bundle_meta.extend_from_slice(&my_node);
                    bundle_meta.extend_from_slice(&bc_h);
                    tree.insert(my_node, &bundle_meta);
                    tree.root()
                };
                let included_reveals_root = {
                    let mut tree = SparseMerkleTree::new();
                    let mut reveal_meta = Vec::with_capacity(64);
                    reveal_meta.extend_from_slice(&my_node);
                    reveal_meta.extend_from_slice(&r_hash);
                    tree.insert(my_node, &reveal_meta);
                    tree.root()
                };
                let control_root = empty_internal(TREE_DEPTH);

                // Header формируется с placeholder values для post-apply полей
                // (state_root / account_root / node_root). Подпись вычисляется
                // ровно один раз — после apply_proposal — над финальным
                // signed_scope с post-apply значениями. Spec, раздел "Proposal":
                // "signature ML-DSA-65 над signed_scope(header) (Правило R1)";
                // все поля канонически вычислимы из cemented set, значит state_root
                // в подписи обязан быть post-apply.
                let mut header = ProposalHeader {
                    prev_proposal_hash,
                    window_index: current,
                    protocol_version: 1,
                    control_root,
                    node_root: [0u8; 32],
                    candidate_root: state.candidates.root(),
                    account_root: [0u8; 32],
                    state_root: [0u8; 32],
                    timechain_value: timechain.t_r,
                    included_bundles_root,
                    included_reveals_root,
                    winner_endpoint: endpoint,
                    winner_id: my_node,
                    proposer_node_id: my_node,
                    target: u128::MAX,
                    fallback_depth: 1,
                    signature: Signature::from_array([0u8; SIGNATURE_SIZE]),
                };

                let settle = ProposalSettle {
                    window_w: current,
                    winner_id: my_node,
                    cemented_confirmers: vec![my_node],
                };
                let post_state_root = apply_proposal(
                    &mut state.accounts,
                    &mut state.nodes,
                    &state.candidates,
                    &settle,
                    params,
                );

                header.state_root = post_state_root;
                header.account_root = state.accounts.root();
                header.node_root = state.nodes.root();
                let mut header_scope = Vec::new();
                header.encode_signed_scope(&mut header_scope);
                header.signature =
                    sign(&identity.node_sk, &header_scope).map_err(NodeError::Crypto)?;

                // M9 Phase 1: broadcast ProposalHeader всем connected peers через
                // network thread. Followers получают envelope с window_index и
                // в M9 Phase 2 будут running apply_proposal locally.
                if let Some(ref handle) = network_handle {
                    let mut header_bytes = Vec::with_capacity(3722);
                    header.encode(&mut header_bytes);
                    let envelope =
                        ProtocolMessage::new(MsgType::Proposal, header.window_index, header_bytes);
                    if let Err(e) = handle.broadcast_tx.send(envelope) {
                        eprintln!(
                            "[consensus] broadcast Proposal w={} failed: {e}",
                            header.window_index
                        );
                    } else {
                        eprintln!(
                            "[consensus] broadcast Proposal window={} → peers",
                            header.window_index
                        );
                    }
                }

                let recomputed = compute_state_root(
                    &state.nodes.root(),
                    &state.candidates.root(),
                    &state.accounts.root(),
                );
                if recomputed != post_state_root {
                    panic!(
                        "state_root self-verify failed: header={:02x?} recomputed={:02x?}",
                        post_state_root, recomputed
                    );
                }

                store
                    .archive_proposal(&header)
                    .map_err(|e| NodeError::InvalidArguments(format!("archive_proposal: {e:?}")))?;
                store.save_meta_last_cemented(current).map_err(|e| {
                    NodeError::InvalidArguments(format!("save_meta_last_cemented: {e:?}"))
                })?;
                prev_proposal_hash = proposal_hash(&header);

                session_emitted = session_emitted.saturating_add(params.emission_moneta);
                session_windows += 1;
            },
        }

        let _ = apply_candidate_expiry(&mut state.candidates, next_window);
        if is_selection_window(next_window, params) {
            let active = state.nodes.len() as u64;
            let cba = cemented_bundle_aggregate(next_window.saturating_sub(2), &[]);
            let activated = apply_selection_event(
                &mut state.candidates,
                &mut state.nodes,
                &mut state.accounts,
                &timechain.t_r,
                &cba,
                active,
                next_window,
                params,
            );
            if !activated.is_empty() {
                println!(
                    "[selection W={next_window}] активировано {} узл(ов)",
                    activated.len()
                );
                if lifecycle.phase == NodePhase::Registered && state.nodes.contains(&my_node) {
                    lifecycle.phase = NodePhase::Active;
                    println!("[active W={next_window}] phase Registered → Active");
                }
            }
        }

        if next_window > 0 && next_window % params.tau2_windows == 0 {
            let median_permille = 1000u32;
            let new_d = next_d(timechain.current_d, median_permille, params);
            if new_d != timechain.current_d {
                println!(
                    "[next_d W={next_window}] D: {} → {} (median_permille={median_permille})",
                    timechain.current_d, new_d
                );
                timechain.current_d = new_d;
                if args.d_test_override.is_none() {
                    effective_d = new_d;
                }
            }
        }

        timechain.t_r = next_t_r;
        timechain.last_window = next_window;
        current = next_window;

        let balance = state
            .accounts
            .get(&my_account)
            .map(|a| a.balance)
            .unwrap_or(0);

        let phase_marker = match lifecycle.phase {
            NodePhase::Bootstrap => "Bootstrap",
            NodePhase::CandidateVdf => "CandidateVdf",
            NodePhase::Registered => "Registered",
            NodePhase::Active => "Active",
        };
        let window_duration = window_start.elapsed();

        // Per-window лог = только progress bar строка от vdf_step_chunked
        // (in-place updates через `\r`, финал с `\n`). T_r/balance/phase
        // не печатаются здесь — оператор видит их через `montana-node status`.
        let _ = phase_marker;
        let _ = window_duration;
        let _ = balance;

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

fn vdf_step_chunked(prev: &Hash32, d: u64, label: &str, window: u64) -> Hash32 {
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
