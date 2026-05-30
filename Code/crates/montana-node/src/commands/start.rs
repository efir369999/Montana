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
    bundle_hash, compute_endpoint, lottery_weight, quorum, reveal_hash, seniority_term,
    validate_bundle, validate_reveal, weighted_ticket_node, BundledConfirmation, VdfReveal,
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

    loop {
        // M9 Phase 2: drain incoming Proposal envelopes от bootstrap. Decode window_index
        // и proposer_node_id напрямую из 3722-байтного header layout без полного
        // deserialize (signature валидация в M10). Для каждого нового окна — apply_proposal
        // с reconstructed singleton ProposalSettle (winner = bootstrap, confirmers = [bootstrap]).
        // Followers: current_window растёт в lockstep с Moscow.
        if let Some(ref mut handle) = network_handle {
            // DEV-022: bootstrap_node_id hoisted to outer scope above
            let mut bc_count = 0usize;
            while let Ok(msg) = handle.incoming_rx.try_recv() {
                match msg.msg_type {
                    MsgType::Proposal => {
                        // M9 Phase 2: bootstrap Proposal envelope (3722 B header layout).
                        // Decode window_index + winner + proposer без полного deserialize
                        // (signature валидация в M10), apply_proposal with reconstructed
                        // singleton ProposalSettle. Followers stay in lockstep with Moscow.
                        if msg.payload.len() < 3722 {
                            eprintln!(
                                "[consensus] Proposal envelope wrong size {} (expected >= 3722) — skip",
                                msg.payload.len()
                            );
                            continue;
                        }
                        let is_cemented = msg.payload.len() > 3722;
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
                        // DEV-022: record cemented winner_id for Lookback proposer selection
                        winner_history.insert(window_index, winner_id);
                        // DEV-023: record proposer activity for fallback cascade decisions
                        last_proposer_cement.insert(proposer_node_id, window_index);
                        while winner_history.len() > 64 {
                            let oldest = *winner_history.keys().next().unwrap();
                            winner_history.remove(&oldest);
                        }

                        if proposer_node_id != bootstrap_node_id {
                            eprintln!("[consensus] Proposal от не-bootstrap proposer, skip");
                            continue;
                        }
                        // M10: cryptographic verification of the bootstrap
                        // signature over signed_scope (bytes 0..413). Rejects
                        // any forged or tampered Proposal — closes the M9
                        // Phase 2 deferred-signature gate symmetrically for
                        // the replay path and (via recent_roots) for fast-sync.
                        let mut sig_bytes = [0u8; mt_crypto::SIGNATURE_SIZE];
                        sig_bytes.copy_from_slice(&msg.payload[413..3722]);
                        let sig = mt_crypto::Signature::from_array(sig_bytes);
                        let bootstrap_pk =
                            mt_crypto::PublicKey::from_array(params.bootstrap_node_pubkey);
                        if !mt_crypto::verify(&bootstrap_pk, &msg.payload[0..413], &sig) {
                            eprintln!(
                                "[consensus] Proposal w={window_index} с невалидной подписью bootstrap — skip"
                            );
                            continue;
                        }
                        // Record this bootstrap Proposal's state_root as a trusted
                        // fast-sync anchor (offset 172..204), bounded to recent windows.
                        let mut sr = [0u8; 32];
                        sr.copy_from_slice(&msg.payload[172..204]);
                        recent_roots.insert(window_index, sr);
                        while recent_roots.len() > 64 {
                            let oldest = *recent_roots.keys().next().unwrap();
                            recent_roots.remove(&oldest);
                        }
                        // DEV-017 follower t_r history: extract proposer's T_r(W)
                        // from Proposal envelope (offset 204..236 = timechain_value
                        // field) so incoming BCs from other followers validate
                        // against the authoritative T_r, not the follower's own
                        // (out-of-sync) timechain.t_r.
                        let mut t_r_w_extracted = [0u8; 32];
                        t_r_w_extracted.copy_from_slice(&msg.payload[204..236]);
                        t_r_history.insert(window_index, t_r_w_extracted);
                        while t_r_history.len() > 64 {
                            let oldest = *t_r_history.keys().next().unwrap();
                            t_r_history.remove(&oldest);
                        }
                        // Candidate envelope (size == 3722, no bundles) is NOT applied:
                        // it serves as a notification "window W is being proposed,
                        // send me your BC". Active followers respond with a BC.
                        if !is_cemented {
                            // Active follower: compute own BC for this window and
                            // broadcast back to the proposer (and to peers).
                            let am_active_in_table = state.nodes.get(&my_node).is_some();
                            if am_active_in_table && my_node != bootstrap_node_id {
                                let mut t_r_w = [0u8; 32];
                                t_r_w.copy_from_slice(&msg.payload[204..236]);
                                let cba = mt_timechain::cemented_bundle_aggregate(
                                    window_index.saturating_sub(2),
                                    &[],
                                );
                                let endpoint = mt_lottery::compute_endpoint(
                                    &t_r_w,
                                    &cba,
                                    &my_node,
                                    window_index,
                                );
                                // DEV-020 follower Reveal: вычислить и broadcast
                                // собственный VdfReveal для текущего окна.
                                let mut own_reveal = VdfReveal {
                                    node_id: my_node,
                                    window_index,
                                    endpoint,
                                    signature: Signature::from_array([0u8; SIGNATURE_SIZE]),
                                };
                                let mut reveal_scope = Vec::new();
                                own_reveal.encode_signed_scope(&mut reveal_scope);
                                own_reveal.signature = sign(&identity.node_sk, &reveal_scope)
                                    .map_err(NodeError::Crypto)?;
                                let own_reveal_hash = mt_lottery::reveal_hash(&own_reveal);
                                // Store own reveal in pool
                                reveal_pool
                                    .entry(window_index)
                                    .or_default()
                                    .insert(my_node, own_reveal.clone());
                                while reveal_pool.len() > 64 {
                                    let oldest = *reveal_pool.keys().next().unwrap();
                                    reveal_pool.remove(&oldest);
                                }
                                // Broadcast Reveal envelope to peers
                                let mut reveal_payload = Vec::new();
                                own_reveal.encode(&mut reveal_payload);
                                let reveal_env = ProtocolMessage::new(
                                    MsgType::VdfReveal,
                                    window_index,
                                    reveal_payload,
                                );
                                let _ = handle.broadcast_tx.send(reveal_env);
                                // Build BC with reveal_hashes from pool (own + any peer reveals received earlier)
                                let mut bc_reveal_hashes: Vec<Hash32> = reveal_pool
                                    .get(&window_index)
                                    .map(|m| m.values().map(mt_lottery::reveal_hash).collect())
                                    .unwrap_or_default();
                                bc_reveal_hashes.sort(); // canonical order
                                let _ = own_reveal_hash;
                                let mut bc = BundledConfirmation {
                                    node_id: my_node,
                                    endpoint: t_r_w,
                                    window_index,
                                    op_hashes: Vec::new(),
                                    reveal_hashes: bc_reveal_hashes,
                                    signature: Signature::from_array([0u8; SIGNATURE_SIZE]),
                                };
                                let mut bc_scope = Vec::new();
                                bc.encode_signed_scope(&mut bc_scope);
                                bc.signature = sign(&identity.node_sk, &bc_scope)
                                    .map_err(NodeError::Crypto)?;
                                let mut bc_payload = Vec::new();
                                bc.encode(&mut bc_payload);
                                let envelope = ProtocolMessage::new(
                                    MsgType::BundledConfirmation,
                                    window_index,
                                    bc_payload,
                                );
                                if handle.broadcast_tx.send(envelope).is_ok() {
                                    eprintln!("[bc] broadcast own BC for window {window_index}");
                                }
                            }
                            // Candidate not advanced; cemented envelope will advance current.
                            continue;
                        }
                        // Cemented envelope: parse bundles, validate, multi-confirmer apply.
                        let mut bundles: Vec<BundledConfirmation> = Vec::new();
                        let payload = &msg.payload;
                        if payload.len() >= 3722 + 2 {
                            let mut bc_buf = [0u8; 2];
                            bc_buf.copy_from_slice(&payload[3722..3724]);
                            let bundle_count = u16::from_le_bytes(bc_buf) as usize;
                            let mut off = 3724;
                            let mut ok = true;
                            for _ in 0..bundle_count {
                                match BundledConfirmation::decode(&payload[off..]) {
                                    Ok((bc, used)) => {
                                        bundles.push(bc);
                                        off += used;
                                    },
                                    Err(e) => {
                                        eprintln!(
                                            "[consensus] cemented bundle decode failed: {e:?} — skip envelope"
                                        );
                                        ok = false;
                                        break;
                                    },
                                }
                            }
                            if !ok {
                                continue;
                            }
                        }
                        let mut t_r_w_cemented = [0u8; 32];
                        t_r_w_cemented.copy_from_slice(&msg.payload[204..236]);
                        let mut valid_confirmers: Vec<mt_state::NodeId> = Vec::new();
                        let mut any_invalid = false;
                        for bc in &bundles {
                            if mt_lottery::validate_bundle(bc, &state.nodes, &t_r_w_cemented)
                                .is_ok()
                            {
                                valid_confirmers.push(bc.node_id);
                            } else {
                                any_invalid = true;
                            }
                        }
                        if any_invalid {
                            eprintln!(
                                "[consensus] cemented w={window_index}: некоторые bundles не прошли validate, продолжаю с валидными {}",
                                valid_confirmers.len()
                            );
                        }
                        if valid_confirmers.is_empty() && !bundles.is_empty() {
                            eprintln!(
                                "[consensus] cemented w={window_index}: 0 валидных bundles — skip"
                            );
                            continue;
                        }
                        // Fallback: if bundles empty (legacy 3722-only envelope), treat as
                        // singleton with proposer as sole confirmer. Unreachable here since
                        // is_cemented = payload.len() > 3722; but defensive.
                        if valid_confirmers.is_empty() {
                            valid_confirmers.push(proposer_node_id);
                        }
                        // M7 fast-sync: if a snapshot is already in flight, ignore
                        // cemented proposals until apply.
                        // DEV-018c: drop stale in-flight client if no response within 10s.
                        // Without this, a single unanswered FastSyncRequest stalls catch-up
                        // forever — broadcast may be lost, peer may be unable to serve,
                        // chunks may be partial and never complete.
                        if let Some(deadline) = fast_sync_deadline {
                            if Instant::now() > deadline {
                                eprintln!("[m7] fast-sync deadline exceeded — drop client, retry");
                                fast_sync = None;
                                fast_sync_deadline = None;
                            }
                        }
                        if fast_sync.is_some() {
                            continue;
                        }
                        // Far behind → request a fast-sync snapshot instead of waiting
                        // for many cemented envelopes (one per window) to catch up.
                        if window_index.saturating_sub(current) > fast_sync_lag_threshold {
                            let mut fs_payload = Vec::new();
                            mt_net::FastSyncRequest {
                                anchor_window: window_index,
                                resume_offset: 0,
                            }
                            .encode(&mut fs_payload);
                            match handle.broadcast_tx.send(ProtocolMessage::new(
                                MsgType::FastSyncRequest,
                                msg.request_id,
                                fs_payload,
                            )) {
                                Ok(()) => {
                                    eprintln!(
                                        "[m7] {} windows behind (> {fast_sync_lag_threshold}) \u{2192} fast-sync anchored at window {window_index}",
                                        window_index.saturating_sub(current)
                                    );
                                    fast_sync = Some(mt_sync::FastSyncClient::new());
                                    fast_sync_deadline =
                                        Some(Instant::now() + std::time::Duration::from_secs(10));
                                },
                                Err(e) => eprintln!("[m7] FastSyncRequest broadcast failed: {e}"),
                            }
                            continue;
                        }
                        // One cemented envelope = one window advance.
                        if window_index != current + 1 {
                            eprintln!(
                                "[consensus] cemented w={window_index} gap (current={current}) — wait for sequential cemented or fast-sync"
                            );
                            continue;
                        }
                        let settle = ProposalSettle {
                            window_w: window_index,
                            winner_id,
                            cemented_confirmers: valid_confirmers.clone(),
                        };
                        let _post_state_root = apply_proposal(
                            &mut state.accounts,
                            &mut state.nodes,
                            &state.candidates,
                            &settle,
                            params,
                        );
                        current = window_index;
                        save_current_window(&data_dir, current)?;
                        eprintln!(
                            "[consensus] applied cemented Proposal w={current} (confirmers={})",
                            valid_confirmers.len()
                        );
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
                    },
                    MsgType::FastSyncRequest => {
                        // M7 server-side: peer requested a snapshot anchored at a window.
                        // Build a Snapshot from the live state, chunk into wire format,
                        // and broadcast the chunks. Requester filters by request_id;
                        // unrelated peers see the broadcast and drop it via msg_type +
                        // request_id mismatch.
                        match mt_net::FastSyncRequest::decode(&msg.payload) {
                            Ok(req) => {
                                let snap = mt_sync::Snapshot::from_tables(
                                    current,
                                    &state.accounts,
                                    &state.nodes,
                                    &state.candidates,
                                );
                                let chunks = snap.to_wire_chunks(32);
                                let total = chunks.len();
                                for chunk in chunks {
                                    let table_id_byte = match chunk.table_id {
                                        mt_sync::FastSyncTableId::Account => {
                                            mt_net::TableId::Account
                                        },
                                        mt_sync::FastSyncTableId::Node => mt_net::TableId::Node,
                                        mt_sync::FastSyncTableId::Candidate => {
                                            mt_net::TableId::Candidate
                                        },
                                        mt_sync::FastSyncTableId::Proposals => {
                                            mt_net::TableId::Proposals
                                        },
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
                                        // DEV-018: stamp current cemented head into every chunk
                                        // so receiver can: (a) discard chunks from peers at/below
                                        // its own current, (b) verify against the correct
                                        // recent_roots[anchor_window] entry, not any matching root.
                                        anchor_window: current,
                                        records: flat,
                                    };
                                    let mut payload = Vec::new();
                                    wire_chunk.encode(&mut payload);
                                    let envelope = ProtocolMessage::new(
                                        MsgType::FastSyncResponse,
                                        msg.request_id,
                                        payload,
                                    );
                                    if let Err(e) = handle.broadcast_tx.send(envelope) {
                                        eprintln!("[m7] fastsync response broadcast failed: {e}");
                                        break;
                                    }
                                }
                                eprintln!(
                                    "[m7] served FastSync snapshot: anchor_window={current} req={} chunks={total}",
                                    req.anchor_window
                                );
                            },
                            Err(e) => {
                                eprintln!("[m7] FastSyncRequest decode failed: {e:?}");
                            },
                        }
                    },
                    MsgType::FastSyncResponse => {
                        if let Some(mut client) = fast_sync.take() {
                            // DEV-018: peek chunk anchor_window before accepting. Drop
                            // chunks from peers at <= our current (they cannot help us
                            // catch up). This avoids the StateRootUnmatched cascade where
                            // we'd otherwise import a peer's stale snapshot, fail finalize,
                            // and retry on every cemented Proposal.
                            let chunk_anchor = mt_net::FastSyncResponseChunk::decode(&msg.payload)
                                .ok()
                                .map(|c| c.anchor_window)
                                .unwrap_or(0);
                            if chunk_anchor <= current {
                                eprintln!(
                                    "[m7] discard FastSyncResponse anchor={chunk_anchor} <= current={current} (peer not ahead) — drop client, retry on next cemented"
                                );
                                // Drop the client so the next cemented Proposal triggers
                                // a fresh FastSyncRequest. Keeping client = Some would
                                // block re-trigger via the `fast_sync.is_some()` guard
                                // above and we'd stall forever if the first response
                                // happens to come from a stale peer.
                                drop(client);
                                fast_sync_deadline = None;
                                continue;
                            }
                            let parsed = mt_net::FastSyncResponseChunk::decode(&msg.payload)
                                .map_err(|e| format!("decode: {e:?}"))
                                .and_then(|w| {
                                    crate::commands::fastsync::wire_chunk_to_sync(w)
                                        .map_err(|e| format!("wire: {e:?}"))
                                });
                            match parsed {
                                Ok(chunk) => match client.accept_chunk(chunk) {
                                    Ok(mt_sync::AcceptOutcome::Complete) => {
                                        match client.finalize(&recent_roots) {
                                            Ok((window, tables)) => {
                                                state.apply_fast_sync(
                                                    tables, &data_dir, window,
                                                )?;
                                                current = window;
                                                save_current_window(&data_dir, current)?;
                                                fast_sync_deadline = None;
                                                eprintln!("[m7] fast-sync complete \u{2192} state replaced, current_window={current}");
                                            },
                                            Err(e) => eprintln!(
                                                "[m7] fast-sync finalize rejected: {e:?} \u{2014} retry on next lag"
                                            ),
                                        }
                                    },
                                    Ok(mt_sync::AcceptOutcome::Progress { received, total }) => {
                                        eprintln!("[m7] fast-sync chunk {received}/{total}");
                                        fast_sync = Some(client);
                                    },
                                    Err(e) => eprintln!(
                                        "[m7] fast-sync chunk rejected: {e:?} \u{2014} discard, retry on next lag"
                                    ),
                                },
                                Err(reason) => {
                                    eprintln!("[m7] FastSyncResponse {reason}");
                                    fast_sync = Some(client);
                                },
                            }
                        }
                    },
                    MsgType::VdfReveal => {
                        // DEV-020: incoming peer Reveal — validate + insert into reveal_pool.
                        // Proposer на cement-time использует cemented Reveal set для winner
                        // determination (DEV-021).
                        if let Ok((rec_reveal, _)) = VdfReveal::decode(&msg.payload) {
                            let exp_t_r = t_r_history
                                .get(&rec_reveal.window_index)
                                .copied()
                                .unwrap_or(timechain.t_r);
                            let cba = mt_timechain::cemented_bundle_aggregate(
                                rec_reveal.window_index.saturating_sub(2),
                                &[],
                            );
                            if mt_lottery::validate_reveal(
                                &rec_reveal,
                                &state.nodes,
                                &exp_t_r,
                                &cba,
                                rec_reveal.window_index,
                            )
                            .is_ok()
                            {
                                let nid = rec_reveal.node_id;
                                let w = rec_reveal.window_index;
                                reveal_pool.entry(w).or_default().insert(nid, rec_reveal);
                                eprintln!(
                                    "[dev-020] accepted Reveal from {} for w={w}",
                                    hex16(&nid)
                                );
                                while reveal_pool.len() > 64 {
                                    let oldest = *reveal_pool.keys().next().unwrap();
                                    reveal_pool.remove(&oldest);
                                }
                            }
                        }
                    },
                    MsgType::BundledConfirmation => {
                        // DEV-012 Phase B: validate incoming BC and insert into accumulator.
                        // Quorum check + cementing is done at the top of the Active loop.
                        bc_count += 1;
                        match BundledConfirmation::decode(&msg.payload) {
                            Ok((bc, _used)) => {
                                // expected_endpoint = my own t_r at bc.window_index. For
                                // current-window BCs this is timechain.t_r; for past windows
                                // we'd need history. Simplification: validate against
                                // current t_r only; older BCs may fail and be ignored.
                                let expected_t_r = t_r_history
                                    .get(&bc.window_index)
                                    .copied()
                                    .unwrap_or(timechain.t_r);
                                if mt_lottery::validate_bundle(&bc, &state.nodes, &expected_t_r)
                                    .is_ok()
                                {
                                    let node_id = bc.node_id;
                                    let w = bc.window_index;
                                    bc_accumulator.entry(w).or_default().insert(node_id, bc);
                                    eprintln!(
                                        "[bc] accepted BC from {} for window {w}",
                                        hex16(&node_id)
                                    );
                                } else {
                                    eprintln!(
                                        "[bc] BC validate failed for {} w={}",
                                        hex16(&bc.node_id),
                                        bc.window_index
                                    );
                                }
                            },
                            Err(e) => eprintln!("[bc] decode failed: {e:?}"),
                        }
                    },
                    _ => {},
                }
            }
            if bc_count > 0 {
                eprintln!("[consensus] drained {bc_count} BundledConfirmation envelope(s) — DEV-012 Phase A scaffold (no validation yet)");
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

        // DEV-012 follower mode: set to true when this iteration is a follower
        // (Active + NodeTable.len() > 1), in which case the post-match epilogue
        // skips current_window advance — the only way the cemented head moves
        // is via apply_proposal from the bootstrap proposer at the start of
        // the next iteration. This keeps Frankfurt / Helsinki / Armenia in
        // lockstep with Moscow until M9 Phase 2 multi-confirmer is wired.
        let mut follower_skip = false;

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
                // DEV-012 multi-confirmer: bootstrap is the canonical proposer; any
                // Active node that is NOT the bootstrap stays a follower and contributes
                // a BC on incoming candidate Proposal. (Spec calls for lookback-based
                // proposer rotation in a future iteration; for the v1.0.0 cohort the
                // bootstrap-only proposer model is the deployed baseline.)
                // DEV-022 + DEV-023: Lookback rotation with bootstrap fallback.
                //   primary_proposer = winner_{W-2} (или bootstrap для W<2)
                //   если primary cemented within last K_FALLBACK_WINDOWS → primary propose
                //   иначе bootstrap takes over (canonical fallback, нет dead-lock)
                let primary_proposer: mt_state::NodeId = if current < 2 {
                    bootstrap_node_id
                } else {
                    winner_history
                        .get(&(current - 2))
                        .copied()
                        .unwrap_or(bootstrap_node_id)
                };
                let primary_last_cement = last_proposer_cement
                    .get(&primary_proposer)
                    .copied()
                    .unwrap_or(0);
                // DEV-023b election grace: if primary never cemented (last_cement=0)
                // BUT won lottery in last K windows, treat as active — give the freshly
                // elected proposer K windows to publish their first proposal. Without
                // this grace, silent_count = current (huge) immediately overrides
                // primary with bootstrap fallback and the elected node never gets a
                // chance.
                let primary_active = if primary_last_cement == 0
                    && primary_proposer != bootstrap_node_id
                {
                    let grace_start = current.saturating_sub(K_FALLBACK_WINDOWS).max(2);
                    let grace_end = current.saturating_sub(2);
                    (grace_start..=grace_end)
                        .any(|w| winner_history.get(&w).copied() == Some(primary_proposer))
                } else {
                    let primary_silent = current.saturating_sub(primary_last_cement);
                    primary_silent < K_FALLBACK_WINDOWS || primary_proposer == bootstrap_node_id
                };
                let primary_silent = current.saturating_sub(primary_last_cement);
                let active_proposer = if primary_active {
                    primary_proposer
                } else {
                    bootstrap_node_id
                };
                if my_node != active_proposer {
                    eprintln!(
                        "[lookback W={current}] primary={} silent={} active_proposer={} my_node={} — follower mode",
                        hex16(&primary_proposer),
                        primary_silent,
                        hex16(&active_proposer),
                        hex16(&my_node)
                    );
                    follower_skip = true;
                    break 'active_arm;
                }
                if my_node != bootstrap_node_id {
                    eprintln!(
                        "[lookback W={current}] my_node={} elected proposer (primary={} active={})",
                        hex16(&my_node),
                        hex16(&primary_proposer),
                        primary_active
                    );
                } else if !primary_active {
                    eprintln!(
                        "[fallback W={current}] bootstrap taking over (primary {} silent for {} windows)",
                        hex16(&primary_proposer),
                        primary_silent
                    );
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
                // DEV-020: proposer also broadcasts own Reveal envelope so followers
                // can include it in their BC.reveal_hashes for cement-time winner determination.
                reveal_pool
                    .entry(current)
                    .or_default()
                    .insert(my_node, reveal.clone());
                while reveal_pool.len() > 64 {
                    let oldest = *reveal_pool.keys().next().unwrap();
                    reveal_pool.remove(&oldest);
                }
                if let Some(ref handle) = network_handle {
                    let mut reveal_payload = Vec::new();
                    reveal.encode(&mut reveal_payload);
                    let _ = handle.broadcast_tx.send(ProtocolMessage::new(
                        MsgType::VdfReveal,
                        current,
                        reveal_payload,
                    ));
                }

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
                // DEV-012: cementing check is performed against the multi-confirmer
                // accumulator after broadcast+drain (below). The singleton-only check
                // is no longer correct in multi-Active mode.
                let _ = (cemented_chain_length, active_chain_length);

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
                // header.winner_id initially = my_node, overwritten below after DEV-021 winner computation
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

                // DEV-012: record T_r for this window so late-arriving BCs validate
                // against historical T_r (not Armenia's current after window advances).
                t_r_history.insert(current, timechain.t_r);
                while t_r_history.len() > 64 {
                    let oldest = *t_r_history.keys().next().unwrap();
                    t_r_history.remove(&oldest);
                }
                // Insert own BC into accumulator first.
                bc_accumulator
                    .entry(current)
                    .or_default()
                    .insert(my_node, bc.clone());

                // Multi-confirmer: if not singleton, broadcast candidate (3722) first,
                // then spin draining incoming for BCs from peers up to 800ms or quorum.
                if state.nodes.len() > 1 {
                    // Build a minimal candidate header (placeholder state_root) for the
                    // notification broadcast. Signed scope must include real T_r so peers
                    // compute matching BC.endpoint.
                    let candidate = ProposalHeader {
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
                    let mut cand_scope = Vec::new();
                    candidate.encode_signed_scope(&mut cand_scope);
                    let cand_sig =
                        sign(&identity.node_sk, &cand_scope).map_err(NodeError::Crypto)?;
                    let mut signed_cand = candidate.clone();
                    signed_cand.signature = cand_sig;
                    let mut cand_bytes = Vec::with_capacity(3722);
                    signed_cand.encode(&mut cand_bytes);
                    if let Some(ref handle) = network_handle {
                        let _ = handle.broadcast_tx.send(ProtocolMessage::new(
                            MsgType::Proposal,
                            current,
                            cand_bytes,
                        ));
                        eprintln!(
                            "[dev-012] broadcast candidate Proposal w={current} (NodeTable.len={}, awaiting BCs)",
                            state.nodes.len()
                        );
                    }

                    // Spin draining BCs up to 800ms.
                    let active_sum: u64 = state.nodes.iter().map(|n| n.chain_length).sum();
                    let need_quorum = quorum(active_sum);
                    let deadline = Instant::now() + Duration::from_millis(5000);
                    // DEV-018d: drain non-BC messages into a deferred queue so
                    // FastSyncRequest / FastSyncResponse / Proposal envelopes from
                    // peers are not silently dropped while the proposer spins
                    // waiting for BC quorum. Deferred messages are re-handled
                    // after spin completes (top of next main-loop iteration).
                    let mut deferred: Vec<ProtocolMessage> = Vec::new();
                    while Instant::now() < deadline {
                        if let Some(ref mut handle) = network_handle {
                            while let Ok(msg) = handle.incoming_rx.try_recv() {
                                if msg.msg_type == MsgType::BundledConfirmation {
                                    if let Ok((rec_bc, _)) =
                                        BundledConfirmation::decode(&msg.payload)
                                    {
                                        let exp_t_r = t_r_history
                                            .get(&rec_bc.window_index)
                                            .copied()
                                            .unwrap_or(timechain.t_r);
                                        if mt_lottery::validate_bundle(
                                            &rec_bc,
                                            &state.nodes,
                                            &exp_t_r,
                                        )
                                        .is_ok()
                                        {
                                            let nid = rec_bc.node_id;
                                            let w = rec_bc.window_index;
                                            bc_accumulator
                                                .entry(w)
                                                .or_default()
                                                .insert(nid, rec_bc);
                                            eprintln!(
                                                "[dev-012] accepted BC from {} for w={w} (current={current})",
                                                hex16(&nid)
                                            );
                                        }
                                    }
                                } else if msg.msg_type == MsgType::FastSyncRequest {
                                    // DEV-018d: serve fast-sync inline during spin.
                                    // Followers depend on the proposer to deliver
                                    // current-window snapshots; if we defer the request
                                    // they may have already retried with a different
                                    // anchor by the time we get back to the main drain.
                                    if let Ok(_req) = mt_net::FastSyncRequest::decode(&msg.payload)
                                    {
                                        let snap = mt_sync::Snapshot::from_tables(
                                            current,
                                            &state.accounts,
                                            &state.nodes,
                                            &state.candidates,
                                        );
                                        let chunks = snap.to_wire_chunks(32);
                                        let total = chunks.len();
                                        for chunk in chunks {
                                            let table_id_byte = match chunk.table_id {
                                                mt_sync::FastSyncTableId::Account => {
                                                    mt_net::TableId::Account
                                                },
                                                mt_sync::FastSyncTableId::Node => {
                                                    mt_net::TableId::Node
                                                },
                                                mt_sync::FastSyncTableId::Candidate => {
                                                    mt_net::TableId::Candidate
                                                },
                                                mt_sync::FastSyncTableId::Proposals => {
                                                    mt_net::TableId::Proposals
                                                },
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
                                                anchor_window: current,
                                                records: flat,
                                            };
                                            let mut payload = Vec::new();
                                            wire_chunk.encode(&mut payload);
                                            let envelope = ProtocolMessage::new(
                                                MsgType::FastSyncResponse,
                                                msg.request_id,
                                                payload,
                                            );
                                            if handle.broadcast_tx.send(envelope).is_err() {
                                                break;
                                            }
                                        }
                                        eprintln!(
                                            "[m7] served FastSync snapshot (spin): anchor={current} chunks={total}"
                                        );
                                    }
                                } else if msg.msg_type == MsgType::VdfReveal {
                                    // DEV-020 spin-drain: peer Reveals must land in pool
                                    // so winner determination at cement time sees them.
                                    if let Ok((rec_reveal, _)) = VdfReveal::decode(&msg.payload) {
                                        let exp_t_r = t_r_history
                                            .get(&rec_reveal.window_index)
                                            .copied()
                                            .unwrap_or(timechain.t_r);
                                        let cba_r = mt_timechain::cemented_bundle_aggregate(
                                            rec_reveal.window_index.saturating_sub(2),
                                            &[],
                                        );
                                        if mt_lottery::validate_reveal(
                                            &rec_reveal,
                                            &state.nodes,
                                            &exp_t_r,
                                            &cba_r,
                                            rec_reveal.window_index,
                                        )
                                        .is_ok()
                                        {
                                            let nid = rec_reveal.node_id;
                                            let w = rec_reveal.window_index;
                                            reveal_pool
                                                .entry(w)
                                                .or_default()
                                                .insert(nid, rec_reveal);
                                            eprintln!(
                                                "[dev-020] spin Reveal from {} for w={w}",
                                                hex16(&nid)
                                            );
                                        } else {
                                            eprintln!(
                                                "[dev-020] Reveal validate failed for {} w={}",
                                                hex16(&rec_reveal.node_id),
                                                rec_reveal.window_index
                                            );
                                        }
                                    }
                                } else {
                                    deferred.push(msg);
                                }
                            }
                        }
                        let collected: u64 = bc_accumulator
                            .get(&current)
                            .map(|m| {
                                m.keys()
                                    .filter_map(|id| state.nodes.get(id).map(|n| n.chain_length))
                                    .sum()
                            })
                            .unwrap_or(0);
                        if collected >= need_quorum {
                            // DEV-019b: when self-quorum trivially met (proposer's own
                            // chain_length dominates Σ), wait until at least ⌈N/2⌉ peer
                            // BCs land for current window OR 5000ms timeout. Without
                            // a peer-quorum gate, every peer that's even slightly
                            // late never gets credit → chain_length stays at 1 forever.
                            // The total operator count includes self; we need
                            // ⌈total/2⌉ confirmers including self in accumulator[current].
                            let total_active = state.nodes.len();
                            let peer_target = (total_active + 1) / 2; // ⌈total/2⌉
                            let grace_deadline = Instant::now() + Duration::from_millis(30000);
                            while Instant::now() < grace_deadline {
                                let current_count =
                                    bc_accumulator.get(&current).map(|m| m.len()).unwrap_or(0);
                                if current_count >= peer_target {
                                    eprintln!(
                                        "[dev-019] peer-quorum gate satisfied: {current_count}/{peer_target} BCs for w={current}"
                                    );
                                    break;
                                }
                                if let Some(ref mut handle) = network_handle {
                                    while let Ok(grace_msg) = handle.incoming_rx.try_recv() {
                                        if grace_msg.msg_type == MsgType::BundledConfirmation {
                                            if let Ok((rec_bc, _)) =
                                                BundledConfirmation::decode(&grace_msg.payload)
                                            {
                                                let exp_t_r = t_r_history
                                                    .get(&rec_bc.window_index)
                                                    .copied()
                                                    .unwrap_or(timechain.t_r);
                                                if mt_lottery::validate_bundle(
                                                    &rec_bc,
                                                    &state.nodes,
                                                    &exp_t_r,
                                                )
                                                .is_ok()
                                                {
                                                    let nid = rec_bc.node_id;
                                                    let w = rec_bc.window_index;
                                                    bc_accumulator
                                                        .entry(w)
                                                        .or_default()
                                                        .insert(nid, rec_bc);
                                                    eprintln!(
                                                        "[dev-019] grace BC from {} for w={w}",
                                                        hex16(&nid)
                                                    );
                                                }
                                            }
                                        } else if grace_msg.msg_type == MsgType::VdfReveal {
                                            if let Ok((rec_reveal, _)) =
                                                VdfReveal::decode(&grace_msg.payload)
                                            {
                                                let exp_t_r = t_r_history
                                                    .get(&rec_reveal.window_index)
                                                    .copied()
                                                    .unwrap_or(timechain.t_r);
                                                let cba_r = mt_timechain::cemented_bundle_aggregate(
                                                    rec_reveal.window_index.saturating_sub(2),
                                                    &[],
                                                );
                                                if mt_lottery::validate_reveal(
                                                    &rec_reveal,
                                                    &state.nodes,
                                                    &exp_t_r,
                                                    &cba_r,
                                                    rec_reveal.window_index,
                                                )
                                                .is_ok()
                                                {
                                                    let nid = rec_reveal.node_id;
                                                    let w = rec_reveal.window_index;
                                                    reveal_pool
                                                        .entry(w)
                                                        .or_default()
                                                        .insert(nid, rec_reveal);
                                                    eprintln!(
                                                        "[dev-020] grace Reveal from {} for w={w}",
                                                        hex16(&nid)
                                                    );
                                                }
                                            }
                                        } else {
                                            deferred.push(grace_msg);
                                        }
                                    }
                                }
                                std::thread::sleep(Duration::from_millis(20));
                            }
                            eprintln!(
                                "[dev-012] quorum reached w={current}: cemented_sum={collected} >= {need_quorum}"
                            );
                            break;
                        }
                        std::thread::sleep(Duration::from_millis(20));
                    }
                }

                // Build final settle from accumulator. Sorted by node_id for determinism.
                let confirmer_ids: Vec<mt_state::NodeId> = bc_accumulator
                    .get(&current)
                    .map(|m| {
                        let mut v: Vec<_> = m.keys().copied().collect();
                        v.sort();
                        v
                    })
                    .unwrap_or_else(|| vec![my_node]);
                // DEV-021: compute winner from cemented Reveal set.
                // Cemented reveal_hashes = union of reveal_hashes across BCs in accumulator[current].
                let mut cemented_hashes: std::collections::BTreeSet<Hash32> =
                    std::collections::BTreeSet::new();
                if let Some(bcs) = bc_accumulator.get(&current) {
                    for bc in bcs.values() {
                        for rh in &bc.reveal_hashes {
                            cemented_hashes.insert(*rh);
                        }
                    }
                }
                // Lookup Reveal objects from pool by hash
                let cemented_reveals: Vec<&VdfReveal> = reveal_pool
                    .get(&current)
                    .map(|m| {
                        m.values()
                            .filter(|r| cemented_hashes.contains(&mt_lottery::reveal_hash(r)))
                            .collect()
                    })
                    .unwrap_or_default();
                // Compute winner via argmin(weighted_ticket_node).
                // weighted_ticket_node(endpoint, chain_length, snapshot) → u128 ticket
                let candidates: Vec<mt_lottery::Candidate> = cemented_reveals
                    .iter()
                    .filter_map(|r| {
                        state.nodes.get(&r.node_id).map(|n| {
                            let snapshot = n.chain_length_snapshot.max(1);
                            let ticket = mt_lottery::weighted_ticket_node(
                                &r.endpoint,
                                n.chain_length,
                                snapshot,
                            );
                            mt_lottery::Candidate {
                                ticket,
                                class: mt_lottery::WINNER_CLASS_NODE,
                                id: r.node_id,
                            }
                        })
                    })
                    .collect();
                let winner_id = mt_lottery::determine_winner(&candidates)
                    .map(|w| w.id)
                    .unwrap_or(my_node);
                eprintln!(
                    "[dev-021] cemented_reveals={} candidates={} winner={}",
                    cemented_reveals.len(),
                    candidates.len(),
                    hex16(&winner_id)
                );
                let settle = ProposalSettle {
                    window_w: current,
                    winner_id,
                    cemented_confirmers: confirmer_ids.clone(),
                };
                let post_state_root = apply_proposal(
                    &mut state.accounts,
                    &mut state.nodes,
                    &state.candidates,
                    &settle,
                    params,
                );

                // DEV-021: write computed winner_id into header (was placeholder my_node)
                header.winner_id = settle.winner_id;
                header.state_root = post_state_root;
                header.account_root = state.accounts.root();
                header.node_root = state.nodes.root();
                let mut header_scope = Vec::new();
                header.encode_signed_scope(&mut header_scope);
                header.signature =
                    sign(&identity.node_sk, &header_scope).map_err(NodeError::Crypto)?;

                // DEV-012: broadcast CEMENTED envelope: [header(3722)][u16 bundle_count][N × BC].
                if let Some(ref handle) = network_handle {
                    let mut payload = Vec::with_capacity(3722 + 2 + 4096 * confirmer_ids.len());
                    header.encode(&mut payload);
                    let bundles_for_envelope: Vec<&BundledConfirmation> = {
                        let map = bc_accumulator.get(&current).cloned().unwrap_or_default();
                        let mut keys: Vec<_> = map.keys().copied().collect();
                        keys.sort();
                        keys.into_iter()
                            .filter_map(|k| bc_accumulator.get(&current).and_then(|m| m.get(&k)))
                            .collect::<Vec<_>>()
                    };
                    let bundle_count = bundles_for_envelope.len() as u16;
                    payload.extend_from_slice(&bundle_count.to_le_bytes());
                    for bc in &bundles_for_envelope {
                        bc.encode(&mut payload);
                    }
                    let envelope =
                        ProtocolMessage::new(MsgType::Proposal, header.window_index, payload);
                    if let Err(e) = handle.broadcast_tx.send(envelope) {
                        eprintln!(
                            "[consensus] broadcast CEMENTED Proposal w={} failed: {e}",
                            header.window_index
                        );
                    } else {
                        eprintln!(
                            "[consensus] broadcast CEMENTED Proposal window={} → peers (bundles={})",
                            header.window_index,
                            bundle_count
                        );
                    }
                }
                // Window cemented; drop its accumulator entry.
                bc_accumulator.remove(&current);

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
                // DEV-022: own cemented winner_id must populate winner_history so the
                // proposer rotation gate at W+2 sees the correct proposer (otherwise
                // proposer keeps thinking it's always self via unwrap_or fallback).
                winner_history.insert(current, header.winner_id);
                // DEV-023: own cement also updates per-proposer activity tracker
                last_proposer_cement.insert(my_node, current);
                while winner_history.len() > 64 {
                    let oldest = *winner_history.keys().next().unwrap();
                    winner_history.remove(&oldest);
                }
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

        if follower_skip {
            // Follower idle: do not advance the cemented head, do not tick the
            // local VDF (timechain.t_r unchanged). The only way `current`
            // advances is through the apply_proposal loop at the top of the
            // outer loop body (lines around 200), driven by an incoming
            // Proposal envelope from the bootstrap proposer. Brief sleep so
            // the outer loop does not spin while waiting for the next peer
            // broadcast.
            std::thread::sleep(std::time::Duration::from_millis(500));
            continue;
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
