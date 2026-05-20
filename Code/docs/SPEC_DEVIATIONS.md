# Spec Deviations

Single source of truth for all known deviations of the implementation from the Montana spec. Introduced in v1.13.0 of the code-architect role ([C-10] Mandatory deviation tracker).

Each `// SPEC DEVIATION DEV-NNN: ...` comment in code refers to a specific entry below. The pre-commit hook (`scripts/pre-commit.sh`) checks the counts.

Closed `DEV-N` entries are kept in this file as a historical record with `Status: closed (commit <sha>)`.

---

## DEV-001: NodeRegistration with vdf_chain_length=0

**Crate:** `montana-node`
**File:line:** `crates/montana-node/src/registration.rs:8-22` (build_node_registration)
**Spec section:** «NodeRegistration» / «Adaptive VDF» / «Step 1: incremental apply»
**Spec quote:** «`if NR.vdf_chain_length >= required: apply; N += 1; else: reject`», `required_vdf_length(pending=0, active=0, τ₂)` → `tau2_windows = 20160`
**What the code does:** `vdf_chain_length=0` (or user-provided), with no `≥ τ₂` check, bypasses `apply_noderegistrations_batch` via a manual `CandidatePool::insert`
**Severity:** mainnet blocker ([I-9] / [C-7] violation, bypass of the canonical apply pipeline)
**Closure path:** implement the candidate VDF phase in `start.rs` — the node ticks VDF until `vdf_chain_length ≥ τ₂_windows`, then automatically forms a NodeRegistration with the correct `vdf_chain_length` and calls `apply_noderegistrations_batch` through the canonical pipeline
**Closure cost:** ~14 days wall-clock on an M-class Mac (VDF physics, not code) + ~4 hours of code
**Status:** closed (commit `fb204ef` mt-local-node: byte-exact rewrite via canonical apply_proposal)

---

## DEV-002: proof_endpoint = candidate_vdf_init(zeros, zeros, node_id)

**Crate:** `montana-node`
**File:line:** `crates/montana-node/src/registration.rs:11`
**Spec section:** «Step 2: Candidacy» / «[I-8] compliance»
**Spec quote:** «`candidate_vdf_init = SHA-256("mt-candidate-vdf-init" || timechain_value(W_start) || cemented_bundle_aggregate(W_start - 2) || node_id)`»
**What the code does:** `candidate_vdf_init(&[0u8; 32], &[0u8; 32], &node_id)` — timechain_value and cba both zeros (placeholder)
**Severity:** mainnet blocker ([I-8] violation — no canonical unpredictable-offline binding)
**Closure path:** at the time of forming a NodeRegistration use the **real** `timechain.t_r` and `cemented_bundle_aggregate(W_start - 2, &cemented_node_ids_at_W_start_minus_2)` from the local node state
**Closure cost:** ~1 hour of code after DEV-001 closure
**Status:** closed (commit `fb204ef` mt-local-node: byte-exact rewrite via canonical apply_proposal)

---

## DEV-003: Lottery missing — winner = first node by lex

**Crate:** `montana-node`
**File:line:** `crates/montana-node/src/commands/start.rs:104-120`, `commands/advance.rs` similarly
**Spec section:** «Lottery» / «τ₁ Winner»
**Spec quote:** «winner = `argmin(weighted_ticket_node)` among cemented `VDF_Reveal` candidate nodes; `weighted_ticket_node = ln_q64(endpoint) / lottery_weight`»
**What the code does:** `state.nodes.iter().next()` — the first node by `node_id` lex order, **with no VDF_Reveal formed, no endpoint, no weighted_ticket**
**Severity:** mainnet blocker (consensus-critical logic ignored, [I-8] violation)
**Closure path:** implement per window: form a `VDF_Reveal` (`mt_lottery::VdfReveal`) with `endpoint = SHA-256("mt-lottery" || T_r || cba || node_id || W LE)`, sign with `node_sk`; compute `weighted_ticket_node` via `mt_lottery::weighted_ticket_node`; for singleton — sole candidate — argmin is trivial and correct **through the canonical API**
**Closure cost:** ~6 hours of code
**Status:** closed (commit `fb204ef` mt-local-node: byte-exact rewrite via canonical apply_proposal)

---

## DEV-004: BundledConfirmation never formed

**Crate:** `montana-node`
**File:line:** `crates/montana-node/src/commands/start.rs:113-117`
**Spec section:** «Confirmer threshold» / «BundledConfirmation» / «apply_proposal Step 3.5»
**Spec quote:** «chain_length is incremented on a cemented `BundledConfirmation`», quorum = `(67 × X + 99) / 100` of active_chain_length
**What the code does:** `chain_length += 1` directly, with no BC formed, no signature over `op_hashes / reveal_hashes`, no quorum cementing
**Severity:** mainnet blocker (chain_length is bluntly incremented on the basis of a non-existent rule)
**Closure path:** form a `mt_lottery::BundledConfirmation` with `op_hashes[]` (from Account Table cemented operations) + `reveal_hashes[]` (from cemented VDF_Reveal of the previous window) + signature `node_sk`; cementing via quorum (for singleton — 100% by itself, checked via `mt_lottery::is_cemented`)
**Closure cost:** ~8 hours of code
**Status:** closed (commit `fb204ef` mt-local-node: byte-exact rewrite via canonical apply_proposal)

---

## DEV-005: ProposalHeader not formed, Step 4 of apply_proposal bypassed

**Crate:** `montana-node`
**File:line:** `crates/montana-node/src/commands/start.rs:102-128`
**Spec section:** «Proposal header» / «Canonical acceptance» / «apply_proposal Step 4»
**Spec quote:** «the winner forms a `ProposalHeader` (1080 bytes) with `included_bundles + included_reveals + state_root`, signs it, archives it. The validator recomputes state_root and compares.»
**What the code does:** directly `account.balance += 13_000_000_000` bypassing `apply_emission`; ProposalHeader is not formed; `archive_proposal` is not called
**Severity:** mainnet blocker (full Step 4 of apply_proposal bypassed)
**Closure path:** form a `mt_consensus::ProposalHeader` with the correct fields (`canonical_proposer`, `included_bundles`, `included_reveals`, `state_root`), `validate_acceptance`, emission via `mt_account::apply_proposal`, `mt_store::archive_proposal`
**Closure cost:** ~12 hours of code
**Status:** closed (commit `fb204ef` mt-local-node: byte-exact rewrite via canonical apply_proposal)

---

## DEV-006: state_root is not cross-checked between proposer and validator

**Crate:** `montana-node`
**File:line:** N/A (absence of code)
**Spec section:** «Verification» / «Proposal finality»
**Spec quote:** «Proposal finality — signature of `proposer_node_id` on the proposal header. Verification — independent recomputation of state_root.»
**What the code does:** state_root recompute does not exist. Singleton mode — the node is its own proposer and validator, but cross-check is still required for regular self-verification (protection against disk / memory corruption)
**Severity:** medium (singleton has no 2 nodes for cross-check, but self-verification is mandatory)
**Closure path:** after forming `ProposalHeader.state_root`, recompute `compute_state_root(account_root, node_root, candidate_root)` independently and compare byte-exact; mismatch → panic (corruption detected)
**Closure cost:** ~1 hour of code
**Status:** closed (commit `fb204ef` mt-local-node: byte-exact rewrite via canonical apply_proposal)

---

## DEV-007: next_d is not invoked at the τ₂ boundary

**Crate:** `montana-node`
**File:line:** `crates/montana-node/src/commands/start.rs:95-160`
**Spec section:** «Adaptation of D via participation-ratio feedback»
**Spec quote:** «D is adapted at the τ₂ boundary via canonical chain observation»
**What the code does:** `timechain.current_d` is fixed at `D₀=252M`, `next_d` is not invoked
**Severity:** mainnet blocker for a long-running node (>14 days)
**Closure path:** keep `participation_history: Vec<u32>` (permille per window) in the timechain state; at every τ₂ boundary compute the median + `next_d(current_d, median, params)`; update `timechain.current_d`; for singleton: participation_ratio = always 1000 → median=1000 → every τ₂ D × 1.03
**Closure cost:** ~3 hours of code
**Status:** closed (commit `fb204ef` mt-local-node: byte-exact rewrite via canonical apply_proposal)

---

## DEV-008: selection_event with zeros in advance.rs vs real T_r in start.rs

**Crate:** `montana-node`
**File:line:** `crates/montana-node/src/commands/advance.rs:55-72`
**Spec section:** «Selection event sort_key»
**Spec quote:** «`sort_key(c) = SHA-256("mt-selection" || timechain_value(W) || cemented_bundle_aggregate(W-2) || c.node_id)`»
**What the code does:** `let placeholder = [0u8; 32]` for both `t_r` and `cba`. Silent divergence between my own commands — `start.rs` uses the real `timechain.t_r`, `advance.rs` uses zeros. Same state, different seeds → different ranking → different winners for multi-candidate.
**Severity:** mainnet blocker (silent divergence between execution paths)
**Closure path:** delete `advance.rs` entirely — for byte-exact spec there is no «fast simulation», only real execution
**Closure cost:** ~10 minutes (delete the file + dispatch update)
**Status:** closed (commit `fb204ef` mt-local-node: byte-exact rewrite via canonical apply_proposal)

---

## DEV-009: apply_proposal entirely bypassed — every step is implemented via manual insert / update

**Crate:** `montana-node`
**File:line:** `crates/montana-node/src/commands/start.rs:95-160`, `advance.rs:45-95`
**Spec section:** «State transition → apply_proposal»
**Spec quote:** «Steps 1, 2, 3a, 3b, 4 in canonical order»
**What the code does:** directly modifies `AccountTable` / `NodeTable` / `CandidatePool` outside of any `apply_proposal`. Each window is an ad-hoc set of shortcuts, not a canonical state transition.
**Severity:** mainnet blocker (silent divergence between implementation and spec on a per-window basis)
**Closure path:** replace the ad-hoc path with the canonical `apply_proposal` pipeline via `mt_account::apply_proposal(&mut account_table, &mut node_table, &mut candidate_pool, &proposal_input, params)`. Singleton mode forms a valid `ProposalInput` for every window and calls the canonical pipeline.
**Closure cost:** ~16 hours of code (depends on DEV-001..DEV-006)
**Status:** closed (commit `fb204ef` mt-local-node: byte-exact rewrite via canonical apply_proposal)

---

## DEV-011: hardware calibration of initial D for the target window time

**Crate:** `montana-node`
**File:line:** `crates/montana-node/src/commands/start.rs` function
              `calibrate_d_for_target_window` + first run of start
**Spec section:** «Engines → TimeChain VDF — oscillator», «Calibration of D₀»
**Spec quote:** «Mainnet calibration `D₀` targets τ₁ ≈ 60 seconds wall-clock
                 on median commodity hardware (engineering target, not a protocol
                 invariant)»
**What the code does:** on the first run of the node (timechain.bin did not exist)
                    it runs benchmark vdf_step(zeros, 10M) → measures
                    the hardware SHA-256 rate → calibrates `current_d` so that
                    a window ≈ 60 s wall-clock on this machine.
**What the spec says:** spec `D₀ = 252M` — engineering calibration target for
                 median commodity. Per-node actual wall-clock varies ×20
                 (Apple Silicon ~53s, idle x86_64 VPS ~68s, loaded ~1145s).
                 Adaptive D feedback at the τ₂ boundary automatically
                 adjusts D to the median network rate.
**Severity:** cosmetic — D in the genesis node = local state, not shared
              consensus invariant with other nodes (there are none).
              When new network nodes appear, their D will be calibrated
              independently or synchronized via canonical params.d0.
**Closure path:** when finalizing multi-node M6+ — nodes will sync via
                  canonical D from the Genesis Decree or negotiate via
                  network consensus. Hardware calibration remains for
                  the genesis node as the initial value.
**Closure cost:** acknowledged as a permanent feature for the genesis node,
                  no closure required
**Status:** acknowledged (genesis-node local hardware calibration —
            an explicit operator choice, not a silent shortcut)
**Acknowledged:** author 2026-04-28 «make my node produce roughly
                  a 60-second window»

---

## DEV-010: genesis bootstrap mode without Candidate VDF (auto-detected)

**Crate:** `montana-node`
**File:line:** `crates/montana-node/src/state.rs:32-66` (LocalState::bootstrap),
              `crates/montana-node/src/node_lifecycle.rs:48-92` (NodeLifecycle::fresh_for + is_bootstrap_node),
              `crates/montana-node/src/commands/start.rs:74-93` (Bootstrap → CandidateVdf transition)
**Spec section:** «Genesis Decree» / «bootstrap_node_pubkey» / «Node activation»
**Spec quote:** «`bootstrap_node_pubkey: [u8; PUBLIC_KEY_SIZE]` in `protocol_params` —
                 the first node of the network is activated via genesis state, not via
                 the Candidate VDF + selection event cycle»
**What the code does:** automatic detection of genesis vs candidate per spec:
  - `NodeLifecycle::is_bootstrap_node(identity, params)` compares
    `identity.node_pk` with `params.bootstrap_node_pubkey` byte by byte
  - If `bootstrap_node_pubkey == [0u8; PUBLIC_KEY_SIZE]` (placeholder
    pre-Genesis-ceremony) — **any** node is treated as genesis (singleton
    legacy mode for the M5 development phase). This branch stops applying
    after the Genesis ceremony when `bootstrap_node_pubkey` is finalized
    with a concrete value
  - If `bootstrap_node_pubkey` is finalized + `identity.node_pk` matches —
    genesis path: phase=Active immediately, NodeRecord for self in NodeTable
  - If `bootstrap_node_pubkey` is finalized + `identity.node_pk` does NOT match —
    standard candidate path: phase=Bootstrap → CandidateVdf on the first window
    → Registered (via apply_noderegistrations_batch once vdf_chain_length
    ≥ τ₂) → Active (via apply_selection_event at the nearest W % selection_interval == 0).
    The node does NOT appear in NodeTable bootstrap state — it is added only
    via canonical apply_selection_event.
**Severity:** acknowledged feature pre-Genesis-ceremony; production-ready
              after the ceremony (auto-detection via canonical apply_proposal pipeline
              for non-bootstrap nodes works byte-exact spec).
**Closure path:** Genesis ceremony — set `params.bootstrap_node_pubkey`
                  to a real value. After that DEV-010 closes
                  automatically: the `is_bootstrap_node` check will identify exactly
                  one genesis node; the rest will go through the standard candidate path.
**Closure cost:** 0 after the Genesis ceremony (the code already implements auto-detection)
**Status:** acknowledged (auto-detection in code, pre-ceremony placeholder
            activates the singleton legacy branch for M5; post-ceremony — production
            spec compliance)
**Acknowledged:** author 2026-04-28 — «do we automatically detect
                  the genesis node by conditions and the others?» → fix v1.15.0 [C-13]
                  enforcement: the correct path immediately, without asking the author

---

## History

| Role version | Date | Action |
|---|---|---|
| v1.13.0 | 2026-04-28 | File created. DEV-001..DEV-009 opened for `montana-node` Stages 1-5. Author's decision: byte-exact rewrite. |
| v1.13.0 | 2026-04-28 | DEV-001..DEV-009 closed via canonical apply_proposal pipeline rewrite. |
| v1.13.0 | 2026-04-28 | DEV-010 added: genesis bootstrap mode (node starts Active without Candidate VDF) — explicit acknowledged deviation, author's decision. |
| v1.14.0 | 2026-05-20 | DEV-013 closed: online IBT proof includes `online_session_nonce`; `OnlineNonceTracker` rejects replay within current / previous slot. |


---

## DEV-012: singleton-only proposal generation in Active phase

**Crate:** `montana-node`
**File:line:** `crates/montana-node/src/commands/start.rs:265-292` (Active phase guard)
**Spec section:** «BundledConfirmation» / «apply_proposal Step 3.5 cementing» / «Singleton consensus»
**Spec quote:** «`cemented_sum = Σ chain_length of nodes whose BundledConfirmation entered included_bundles`. An object is cemented when `cemented_sum ≥ quorum(active_chain_length)`, where `quorum = (67 × active + 99) / 100`.» (mt-consensus/src/lib.rs:327, mt-lottery/src/lib.rs:503-510)
**What the code does:** the Active phase in start.rs forms a proposal in which my_node is the sole confirmer (`included_bundles = {my_bundle}`, cemented_sum = my_node.chain_length). This is correct ONLY when `state.nodes == {my_node}` (1 node in NodeTable, my own). In a multi-node NodeTable my_node.chain_length < quorum(Σ_chain_length) → `is_cemented` returns false → the node crashes with `singleton cementing: cemented=X, active=Y, quorum=Z`. The DEV-012 guard adds a check `state.nodes.len() == 1 && state.nodes.contains(&my_node)`; on failure it skips the proposal block (break 'active_arm) and does not crash.
**Severity:** mainnet blocker (M9 Phase 2 = apply_proposal from peers is not implemented, multi-node consensus does not work)
**Closure path:** implement M9 Phase 2 — drain the incoming Proposal envelope (start.rs:160-169), validate via `mt_consensus::validate_acceptance`, `mt_account::apply_proposal` for the cemented set from the proposer, recompute state_root, sync `current_window` + `state.nodes[].chain_length` from the peer Proposal. After this, Frankfurt / Helsinki as followers catch up with Moscow without needing to produce their own singleton-proposal.
**Closure cost:** ~3-5 days wall-clock for implementation + integration test (e2e_three_peer_apply_proposal)
**Status:** open

**Precedent:** the Frankfurt node became Active on genesis bootstrap (registration_window=45916, start_window=46032, chain_length=1) and immediately landed in a multi-node situation (state.nodes = {msk, fra}). 4,790 montana-node restarts over 24 hours with the error `singleton cementing: cemented=1, active=25767, quorum=17264` — msk had chain_length=25766 in Frankfurt's state (received via P2P sync), fra had its own chain_length=1. The guard prevents the crash loop; the node stays in Active phase, keeps heartbeating to peers, and waits for M9 Phase 2.

---

## DEV-013: online IBT proof formula — code behind spec (online_session_nonce)

**Crate:** `mt-net`
**File:line:** `crates/mt-net/src/ibt.rs` (online_proof / verify_online_proof; the exact line depends on the current implementation — see `cargo grep mt-tunnel-online`)
**Spec section:** «Identity-Bound Tunnel (IBT)» in `Montana Network v1.1.0.md` (after bump v1.0.0 → v1.1.0 for MONT-002 closure)
**Spec quote:** «`proof = ML-DSA-65_sign(client_privkey, "mt-tunnel-online" || server_node_id || floor(current_window_index / 2) || online_session_nonce)` where `online_session_nonce` 32B — generated by the client from CSPRNG for each handshake, transmitted in the plain part of the IBT advertisement alongside the proof.»
**What the code does:** `mt-net::ibt::ibt_online_proof` and `ibt_online_verify` accept `online_session_nonce: [u8; 32]` and include it in the signed message. `mt-net::ibt::OnlineNonceTracker` keeps `used_online_nonces[client_pubkey]` with pruning by current / previous window slot and a bounded per-client set. `mt-net-transport::ibt_upgrade::classify_proof` invokes the verifier + nonce tracker before issuing the access level.
**Severity:** closed for MONT-002 (MITM replay of the same online proof within the 2-window slot is rejected as `IbtError::ReplayedNonce`)
**Status:** closed (mt-net / mt-net-transport: online_session_nonce in signed scope + used_online_nonces tracking)

**Acknowledged:** the wire-level handshake envelope in transport integration must pass `online_session_nonce` alongside the proof; the API already requires the nonce, so without it the call site will not compile.
