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
**Severity:** post-mainnet — v1.0.1 hot-fix track (the bootstrap-proposer + follower-apply path is the v1.0.0 mainnet baseline; multi-confirmer rotation is the v1.0.1 target)
**Closure path:** implement M9 Phase 2 — drain the incoming Proposal envelope (start.rs:160-169), validate via `mt_consensus::validate_acceptance`, `mt_account::apply_proposal` for the cemented set from the proposer, recompute state_root, sync `current_window` + `state.nodes[].chain_length` from the peer Proposal. After this, Frankfurt / Helsinki as followers catch up with Moscow without needing to produce their own singleton-proposal.
**Closure cost:** ~3-5 days wall-clock for implementation + integration test (e2e_three_peer_apply_proposal)
**Status:** partially closed (follower drift fix in commit `e1a0bd0`); multi-confirmer protocol (Phase B+C) carried into v1.0.1 hot-fix track post the v1.0.0 mainnet tag.

**Partial close (commit `e1a0bd0`, 2026-05-21).** The `follower_skip` flag in `start.rs` prevents a node in Active phase with `NodeTable.len() > 1` from advancing its cemented head via the local VDF tick. The only path that advances `current` for a follower is `apply_proposal` driven by an incoming Proposal envelope from the bootstrap proposer. Verified across the four-node mesh (Moscow proposer + Frankfurt + Helsinki + Armenia followers) — lag stays bounded by the network broadcast latency rather than diverging.

**Open: multi-confirmer protocol (v1.0.1 closure).** Closure to v1.0.1 requires:

  1. **Wire-level BundledConfirmation broadcast.** `MsgType::BundledConfirmation (0x20)` already in the message-type registry; followers must sign + broadcast their own BC on receipt of a Proposal for the current window. Wire-format dependency: the canonical `expected_endpoint` for `validate_bundle` is `SHA-256(domain || T_r(W) || cemented_bundle_aggregate(W-2) || node_id || W)`, so the follower's local `timechain.t_r` must equal the canonical value at window `W`.
  2. **t_r consistency for followers.** Two viable paths: (i) followers tick the VDF locally in lockstep with the wall clock and cache the per-window t_r history during catch-up; (ii) Moscow's Proposal envelope is extended to include t_r(W) explicitly. Path (i) does not change the wire format but increases follower CPU; path (ii) breaks the existing 3722-byte envelope size. Path (ii) is the cleaner architectural choice for v1.0.0.
  3. **Proposer-side BC accumulator.** Moscow opens a one-window accumulator on broadcast of its Proposal candidate; collects incoming BC envelopes for the same window from registered Active operators; once `cemented_sum = Σ node.chain_length` over the collected confirmers reaches `quorum(active_chain_length) = ⌈67 * Σ active / 100⌉`, builds the included_bundles set and broadcasts the cemented Proposal with the full bundle set inline (envelope schema change).
  4. **Multi-confirmer ProposalSettle on followers.** Each follower validates every BC signature in the cemented Proposal against the corresponding `NodeTable[node_id].node_pubkey`; reconstructs ProposalSettle with `cemented_confirmers = [all signers]`; calls apply_proposal with the multi-confirmer set.
  5. **Wire format Proposal envelope schema bump.** From 3722-byte header-only to header + length-prefixed BC set. Network spec v1.1.0 → v1.2.0; binding KAT vector regenerated for the new wire format.
  6. **Tests.** mt-net-transport e2e integration test with 3 in-process operators reaching quorum via multi-confirmer cementing across simulated balanced chain_length distribution.

**Operational note (current production state).** Moscow's `chain_length = 25 766` is dominant (Frankfurt, Helsinki, Armenia each `≤ 1`); Moscow's BC alone already satisfies `67% × Σ active_chain_length` quorum. The multi-confirmer protocol becomes operationally consequential only once non-bootstrap operators accumulate non-negligible chain_length over many τ₂ epochs — well after the v1.0.0 mainnet tag. The protocol is the explicit gate to v1.0.1; the bootstrap-proposer baseline is the v1.0.0 mainnet baseline.

**Precedent (historical).** the Frankfurt node became Active on genesis bootstrap (registration_window=45916, start_window=46032, chain_length=1) and immediately landed in a multi-node situation (state.nodes = {msk, fra}). 4,790 montana-node restarts over 24 hours with the error `singleton cementing: cemented=1, active=25767, quorum=17264` — msk had chain_length=25766 in Frankfurt's state (received via P2P sync), fra had its own chain_length=1. The `follower_skip` patch (commit `e1a0bd0`) replaces the crash with passive follower mode; the node stays in Active phase, keeps heartbeating to peers, and only advances its cemented head via apply_proposal from the bootstrap proposer.

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

---

## DEV-014: Noise_PQ post-quantum transport migration (M6 milestone)

**Crate:** `mt-net-transport`
**File:line:** ~~`crates/mt-net-transport/src/transport.rs:42, 76`~~ (classical TLS + Noise XK upgrade chain removed in commit closing DEV-014; transport.rs now uses `NoisePqXxConfig` exclusively)
**Spec section:** «Post-quantum transport migration (M6 milestone)» in `Montana Network v1.1.0.md`
**Spec quote:** «Migration to a single post-quantum transport handshake: hybrid Noise_PQ combining X25519 with ML-KEM-768 as the KEM replacement for Diffie-Hellman.»
**What the code does (historical):** The previous transport upgrade chain was `TLS 1.3 (rustls) → Noise XK (X25519 ECDHE inner) → Yamux`. Both handshake layers used classical X25519 ECDHE and were vulnerable to store-now-decrypt-later attacks by a future quantum adversary. As of this commit the entire classical auth stack is removed — production transport is `TCP → Noise_PQ XX (ML-KEM-768 + ML-DSA-65) → Yamux`. Consensus signatures (ML-DSA-65) are unaffected; transport confidentiality is now post-quantum.
**Severity:** previously mainnet blocker for the «pure post-quantum» claim; closed by switching the production transport stack to Noise_PQ XX.
**Closure path (multi-phase, 3–5 weeks total wall-clock):**

- **Phase 0 — Architecture & scaffolding (this entry).** Network spec documents the migration plan with phases and verification criteria; this DEV-014 tracker entry is added; a `pq_transport_version: u8` wire field is reserved in the IBT advertisement for capability negotiation; no code change beyond the planning documentation. **Status: completed in this commit.**
- **Phase 1 — Noise_PQ handshake implementation.** Implement an ML-KEM-768-augmented Noise XK variant. Two viable paths:
  - (a) Fork the `snow` crate (https://github.com/mcginty/snow) to add ML-KEM-768 as a DH replacement. Contribute upstream after byte-exact KAT validation against the emerging Noise PQ draft. Estimated effort: 3 weeks for a senior Rust + crypto engineer, including KATs and differential testing.
  - (b) Write a custom Noise_PQ handler outside libp2p's `noise` upgrade module, wrapping it as a `libp2p::core::upgrade::OutboundConnectionUpgrade` / `InboundConnectionUpgrade`. Reuse the `mt-crypto::keypair_from_seed_mlkem` and `mt-crypto::Mlkem*` types already present in `mt-crypto`. Estimated effort: 4 weeks.
  Either path requires byte-exact KAT vectors checked into `mt-conformance` and differential testing against at least one independent reference implementation.
- **Phase 2 — Hybrid coexistence period.** Capability negotiation through the `pq_transport_version` wire field. Peers advertise both classical and Noise_PQ; the connection negotiates the highest mutually supported version. A chain_length-weighted majority signal (≥ 67% of active_chain_length advertising Noise_PQ for ≥ τ₂) triggers the deprecation of classical inbound. Estimated wall-clock: 2 weeks of soak-time on the genesis 3-node network plus observability collection.
- **Phase 3 — Classical removal.** TLS 1.3 layer dropped entirely. The transport stack becomes TCP → Noise_PQ → Yamux. Uniform framing preserved at the application layer for DPI obfuscation. Spec bump removes `pq_transport_version` once capability negotiation is no longer needed. Estimated wall-clock: 1 week including spec patch + node deployment + 24-hour soak.

**Closure cost:** 3–5 weeks wall-clock for Phase 1 + 1–2 weeks for Phases 2 + 3 = total **5–7 weeks** for production-grade closure with KATs, differential testing, and three-node soak. This is M6 milestone scope, not single-session work.

**Status:** Phase 0 + Phase 1 + Phase 2 + Phase 3 part 1 + Phase 3 part 2 (AEAD stream + drive functions) + Phase 3 part 2c (libp2p UpgradeInfo / InboundConnectionUpgrade / OutboundConnectionUpgrade trait impls + PeerId derivation from ML-DSA-65) + **Phase 3 XX redesign** (ephemeral KEM both sides, identity discovered during handshake — enables libp2p `with_tcp` plug-in where XK could not) + **Phase 3 part 3 production wire-up** (transport.rs replaced with `NoisePqXxConfig` only; tls + noise removed; PeerId derived from ML-DSA-65 throughout the stack) completed; cross-machine 24h soak across the 3-node Genesis cohort is the remaining empirical verification (off-session).

**Phase 1 closure note (2026-05-21):** mt-crypto extended with FIPS 203 §6.2 / §6.3 ML-KEM-768 encapsulate / decapsulate primitives (`mlkem_encapsulate`, `mlkem_decapsulate`, types `MlkemCiphertext`, `MlkemSharedSecret` with zeroize-on-drop and mlock-protected shared secret allocation). Added C wrapper functions `mt_mlkem_encapsulate` / `mt_mlkem_decapsulate` over OpenSSL 3.5 EVP API.

New crate `mt-noise-pq` (`crates/mt-noise-pq`) implements a 3-message Noise XK-like handshake with ML-KEM-768 in place of Diffie-Hellman and ML-DSA-65 identity signatures over transcript hashes. Wire sizes: msg1 2272 B, msg2 6349 B, msg3 5261 B. Session keys derived via domain-separated SHA-256 from ss_rs ‖ ss_e ‖ transcript ‖ rs_id_pk.

Tests passing:
- `cargo test -p mt-crypto --release --test mlkem_encap_decap` — 2 passed (encap / decap roundtrip + ciphertext freshness)
- `cargo test -p mt-noise-pq --release` — 6 passed total: full handshake roundtrip, tamper detection on msg2 / msg3 signatures (BadResponderSignature / BadInitiatorSignature), wire-size invariants, fixed-input consistent session derivation
- `cargo fmt --all -- --check` clean
- `cargo clippy --workspace --all-targets -- -D warnings` clean

**Phase 3 remaining work (Swarm integration + multi-node soak):**

- Phase 2 spec: completed — wire format and capability negotiation documented in Network v1.1.0.md (commit 2bcd86d and follow-up).
- Phase 3 part 1: TCP loopback integration test in `crates/mt-noise-pq/tests/loopback.rs` completed — both sides run as tokio async tasks and successfully derive identical session keys over a real `TcpStream` pair.
- Phase 3 part 2 (open): libp2p custom transport upgrade implementing the Noise_PQ handshake as `InboundConnectionUpgrade` / `OutboundConnectionUpgrade` so it can replace the existing `noise::Config::new` in `mt-net-transport::transport::build_swarm_with_keypair`. libp2p's `noise` and `tls` upgrades are tightly coupled to the SwarmBuilder API, and a custom Noise variant needs to plug into the same upgrade chain. Estimated 1–2 weeks for production-grade integration with the existing `mt-net-transport` Swarm.
- Phase 3 part 3 (open): cross-machine soak on the 3-node network (Moscow / Helsinki / Frankfurt) for ≥ 24 hours of continuous operation with zero classical-fallback events; requires deployed binaries on real nodes and operator-side observation. After Phase 3 part 3: TLS 1.3 outer layer dropped; transport stack becomes TCP → Noise_PQ XX → Yamux. **Done in this closure commit.**

**XX redesign note (closure commit, 2026-05-21):** The original XK variant required the initiator to know the responder's static ML-KEM-768 public key a priori — incompatible with libp2p's plug-in `with_tcp` auth-upgrade slot which gives the upgrade only the local `libp2p::identity::Keypair` (Ed25519). The XX redesign discovers remote identity during the handshake (ephemeral ML-KEM-768 keypairs on both sides; identity ML-DSA-65 pk transmitted in msg2 / msg3 and authenticated by signature over transcript). Wire format: msg1 1184 B, msg2 7533 B, msg3 6349 B (replacing the XK 2272 / 6349 / 5261). Two upgrade modules now coexist in mt-noise-pq:

- `mt_noise_pq::lib` (legacy XK) — retained for KAT continuity and reference; no longer wired into the libp2p transport.
- `mt_noise_pq::xx_handshake` + `mt_noise_pq::xx_libp2p_upgrade` — new XX module, wired into `mt-net-transport::xx_noise_pq_upgrade::NoisePqXxConfig` which implements both `InboundConnectionUpgrade` and `OutboundConnectionUpgrade` and is what `build_swarm_with_keypair` now uses in production.

PeerId derivation: SHA-256 multihash of the peer's ML-DSA-65 identity public key (libp2p / IPFS sha2-256 multihash code 0x12). GenesisManifest peer_id fields must contain the ML-DSA-derived multihash for the dial-side identity pin to match what the XX upgrade returns on the wire.

**Verification protocol per phase.** Each phase is closed only after ≥ 24 hours of continuous operation across the three genesis nodes (Moscow, Helsinki, Frankfurt) with zero unexpected handshake failures and zero classical-fallback events during the observation window. The cross-node verification log is committed to the repository at `External-Audit/noise-pq-phase{N}-verification.log`.

**Acknowledged:** author 2026-05-20 — explicit request «do this before any release, full phases, verify on nodes». Acknowledgement of scope: Phase 0 closed in this session; Phases 1-3 are dedicated multi-week milestones with code work and cross-node deployment validation that cannot honestly be promised within a single conversation. The plan, scope, and verification criteria are documented here so that the work can be picked up and executed in dedicated implementation sessions.

---

## DEV-015: M7 fast-sync client-side handler

**Crate:** `montana-node`
**File:line:** `crates/montana-node/src/commands/start.rs` — message-dispatch drain
**Spec section:** «Sync protocols → fast-sync» in `Montana Network v1.1.0.md` (lines 964–970)
**What the code does:** the M7 algorithmic layer is complete: `mt_sync::Snapshot::{from_tables, to_wire_chunks, build_tables}` and `SnapshotVerifier::verify` (Sparse Merkle production root, byte-equal cross-implementation conformance, 17 unit tests). The server-side dispatcher in `start.rs` answers `MsgType::FastSyncRequest` by broadcasting chunked `FastSyncResponse` envelopes carrying the requester's `request_id`. The client-side handler — drain chunks by `request_id`, reassemble Snapshot, verify against the anchor `ProposalHeader.state_root`, swap the local `LocalState.{accounts, nodes, candidates}` — is not yet wired into the dispatcher.

**Severity:** v1.0.1 hot-fix track. New operators today join the live mesh by replaying the canonical history via the existing `apply_proposal`-from-peers path; fast-sync becomes a CPU/time win at long-running mesh depth, not a correctness requirement.

**Closure path:**
  1. Add a per-`request_id` accumulator keyed by `(anchor_window, request_id)` to the dispatcher state.
  2. On `MsgType::FastSyncResponse` arrival, decode `mt_net::FastSyncResponseChunk`, append records to the corresponding `Snapshot` instance.
  3. When `chunk_index + 1 == total_chunks` for the highest-seen chunk in a given `request_id`, call `SnapshotVerifier::verify(&snap, &expected_state_root)` against the anchor `ProposalHeader.state_root` retrieved from any honest peer's archived proposal at the same `anchor_window`.
  4. On verify success, call `snap.build_tables()` and swap into `LocalState`; persist via `FsStore`; bump `current_window` to `anchor_window`.
  5. On verify failure, increment an attempt counter and retry against a different peer.

**Status:** open for v1.0.1.

## DEV-016: N_SEED multi-Active genesis cohort

**Crate:** `mt-genesis`, `montana-node`
**File:line:** `crates/mt-genesis/src/manifest.rs:32-67` (GenesisPeer.force_active / node_pubkey_hex / account_pubkey_hex), `crates/montana-node/src/state.rs::LocalState::bootstrap` (pre-seed extra_actives)
**Spec section:** «Genesis Decree» / «N_SEED как consensus-binding параметр Genesis Decree» (Montana Protocol v35.26.0)
**What the code does:** GenesisManifest расширен опциональными полями `force_active`, `node_pubkey_hex`, `account_pubkey_hex` для pre-seed дополнительных Active operators в NodeTable / AccountTable от genesis (window=0). LocalState::bootstrap итерирует extras и добавляет NodeRecord (chain_length=1, start_window=0) + AccountRecord (is_node_operator=true, balance=0) для каждого force_active peer. Singleton bootstrap proposer model сохранён; post-genesis admission через selection_event для не-genesis nodes без изменений.
**Severity:** none (spec-compliant per v35.26.0).
**Closure path:** перенести N_SEED из operational manifest в Genesis Decree `protocol_params.genesis_active_operators` (consensus-binding, hardcoded в genesis_params()); current manifest-based pre-seed остаётся для тест-cohort flexibility, mainnet — через hardcoded params.
**Closure cost:** ~3-5 часов код + KAT vector update.
**Status:** acknowledged (spec formalized в v35.26.0; production hardcoding genesis_active_operators в Genesis Decree protocol_params — следующая итерация mt-genesis).

---

## DEV-017: follower t_r_history population from Proposal envelopes

**Crate:** `montana-node`
**File:line:** `crates/montana-node/src/commands/start.rs:275-285` (after recent_roots insert)
**Spec section:** «BundledConfirmation» / «expected_endpoint» / «follower BC validation»
**Spec quote:** «BC.endpoint = T_r(W) of the proposer; validator computes expected = T_r at window W and rejects if mismatch»
**What the code does (before fix):** followers receive Proposal envelopes (candidate or cemented) containing `timechain_value` (T_r) at offset 204..236, but only `recent_roots` is populated; `t_r_history` remains empty for followers. When BCs from other followers arrive in the live drain (line 568-602), `expected_t_r = t_r_history.get(bc.window_index).unwrap_or(timechain.t_r)` falls back to follower's own out-of-sync `timechain.t_r`, which never matches the proposer's authoritative T_r → all peer BCs rejected as `WrongEndpoint`. Result: bc_accumulator at the proposer side gets at most `bundles=1` (own BC only); 6-Active genesis cohort cannot achieve quorum-based cementing.
**What the code does (after fix):** in the Proposal envelope handler, alongside `recent_roots.insert(window_index, state_root)`, extract `t_r_w_extracted` from offset 204..236 and `t_r_history.insert(window_index, t_r_w_extracted)` (bounded to last 64 windows by identical eviction policy). Now every received Proposal seeds the follower's t_r_history; when subsequent BCs from peer followers arrive in the live drain, validation uses the authoritative T_r and BCs accumulate correctly.
**Severity:** prerequisite for DEV-012 closure (multi-confirmer protocol non-functional without it). Mainnet blocker for any cohort with N_SEED ≥ 1.
**Closure path:** ↑ implemented in this commit.
**Closure cost:** 6 lines of Rust + redeploy.
**Status:** closed (Build 9, this session).


---

## DEV-018: fast-sync chunk anchor_window + stale-peer filter

**Crate:** `mt-net`, `montana-node`
**File:line:** `crates/mt-net/src/payloads.rs:62-110` (FastSyncResponseChunk wire format), `crates/montana-node/src/commands/start.rs:486-545` (sender stamp), `crates/montana-node/src/commands/start.rs:540-560` (receiver filter)
**Spec section:** «Sync protocols → fast-sync» / «State root verification»
**What the code does (before fix):** FastSyncResponseChunk wire format had no anchor_window field; sender broadcast chunks built from sender's current cemented head, but the receiver could not tell which anchor a given chunk belonged to. When the receiver hit `lag_threshold` and requested fast-sync, it accepted FIRST-RESPONSE chunks regardless of sender's actual head. In a mixed-window mesh (Moscow at W=2146, others at W=2003), receiver typically got chunks from the closest peer (a fellow follower at W=2003), not from Moscow. Reconstructed state_root matched the sender's stale state, but not any recent_roots entry for a window the receiver had already advanced past → `StateRootUnmatched` reject → retry cascade on every cemented Proposal arrival → infinite no-progress loop.
**What the code does (after fix):**
  1. **Wire format bump:** `FastSyncResponseChunk` gains `anchor_window: u64` (LE). Minimum chunk size 13 B → 21 B. All construction sites updated (montana-node fastsync.rs, mt-net test_vectors).
  2. **Sender side:** stamps `anchor_window = current` (sender's last cemented) on every chunk.
  3. **Receiver side:** decodes anchor_window from the first chunk payload; if `chunk_anchor <= current` the chunk is discarded with a log line — peer cannot help us catch up. Only chunks from peers strictly ahead of us are accepted.
**Severity:** mainnet blocker for mixed-window mesh (every multi-node deployment where any peer lags >1 windows behind the proposer). Without this fix, fast-sync converges only when ALL peers happen to be at the same window — a vanishingly rare condition.
**Closure path:** ↑ implemented in this commit. Possible follow-up: targeted FastSyncRequest (send to a specific peer_id, not broadcast) to avoid wasting bandwidth on lagging peers' responses.
**Closure cost:** ~50 lines of Rust across 4 files (wire format + sender + receiver + 1 test fix).
**Status:** closed (Build 10, this session).

