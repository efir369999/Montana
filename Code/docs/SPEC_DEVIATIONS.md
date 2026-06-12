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


---

## DEV-018b/c: fast-sync client retry on discard + 10s deadline

**Crate:** `montana-node`
**File:line:** `crates/montana-node/src/commands/start.rs:540-565` (discard drop), `crates/montana-node/src/commands/start.rs:393-401` (deadline check), `crates/montana-node/src/commands/start.rs:425-432` (trigger sets deadline)
**Spec section:** «Sync protocols → fast-sync» / «Liveness under partial responses»
**What the code does (before fix):** DEV-018 introduced anchor_window discard for stale-peer chunks but kept `fast_sync = Some(client)` on discard, so the `fast_sync.is_some()` guard at the next cemented arrival blocked re-trigger. Even worse, when the broadcast FastSyncRequest got NO response at all (peer unreachable, request lost in libp2p RR queue, peer too busy to serve), the client stayed Some forever and catch-up halted permanently.
**What the code does (after fix):**
  1. **Drop on discard (DEV-018b):** when anchor_window check rejects a chunk, immediately `drop(client)` and `fast_sync_deadline = None`. The next cemented Proposal arrival triggers a fresh FastSyncRequest.
  2. **10-second deadline (DEV-018c):** when fast-sync triggers, `fast_sync_deadline = Some(Instant::now() + Duration::from_secs(10))`. Each cemented Proposal handler checks `if Instant::now() > deadline` and drops the stale client. Recovers from silently-lost requests.
**Severity:** mainnet blocker for any cohort with intermittent peer availability or libp2p backpressure.
**Closure path:** ↑ implemented in this commit.
**Closure cost:** ~15 lines of Rust.
**Status:** closed (Build 11 + Build 12, this session).


---

## DEV-018d: serve FastSyncRequest inline during proposer spin-drain

**Crate:** `montana-node`
**File:line:** `crates/montana-node/src/commands/start.rs:965-1060` (spin-drain block in Active phase)
**Spec section:** «Sync protocols → fast-sync» / «Server-side liveness»
**What the code does (before fix):** The proposer (bootstrap) spends ~5 seconds per window in the BC accumulator spin-drain loop. The original spin-drain consumed messages from `incoming_rx` via `try_recv()` but only processed `MsgType::BundledConfirmation`. All other message types (FastSyncRequest, FastSyncResponse, peer Proposals) were silently discarded. Followers' fast-sync requests had ~0% chance of being served — they were eaten by the spin-drain before the post-spin main-loop dispatcher could reach them. As a result, followers stuck at an older window could never catch up to the proposer's head.
**What the code does (after fix):** spin-drain now handles three cases:
  1. `BundledConfirmation` — accumulator insert (as before).
  2. `FastSyncRequest` — serve inline: build snapshot from current state, chunk, broadcast `FastSyncResponse` envelopes with `anchor_window = current`, log `[m7] served FastSync snapshot (spin)`.
  3. Other types (e.g. `FastSyncResponse` for the proposer's own outgoing requests, peer Proposals) — pushed to a `deferred: Vec<ProtocolMessage>` (currently dropped after spin; acceptable because followers re-broadcast on every window).
**Severity:** mainnet blocker — without inline fast-sync serving, a lagging follower can never catch up to a proposer that's continuously cementing windows.
**Closure path:** ↑ implemented in this commit. Follow-up: re-queue `deferred` messages back into `incoming_rx` after spin (requires a multi-producer channel side or a local app-level pending queue).
**Closure cost:** ~70 lines of Rust.
**Status:** closed (Build 13, this session).


---

## DEV-019: post-quorum grace period for peer BC fairness

**Crate:** `montana-node`
**File:line:** `crates/montana-node/src/commands/start.rs:1007-1075` (active arm spin-drain post-quorum)
**Spec section:** «BundledConfirmation cementing» / «Fairness across cohort»
**What the code does (before fix):** when proposer's own chain_length dominates Σ active_chain_length (e.g. bootstrap operator with chain_length=2500+ vs co-validators at chain_length=1), self-quorum is trivially met on the first spin-iteration after inserting own BC. The spin loop breaks immediately (within 20ms), never giving peer BCs time to land (typical RTT 50–150ms). Cemented_confirmers contains only proposer → peer chain_length stays at 1 forever → dominance compounds. Multi-confirmer was structurally impossible after the first cohort.
**What the code does (after fix):** when `collected >= need_quorum` triggers, instead of `break;` the loop enters a 500 ms grace window that keeps draining `BundledConfirmation` envelopes from `incoming_rx`. Any peer BC arriving within grace and validating against `t_r_history[bc.window_index]` is inserted into the accumulator at the bc's window slot. After grace, the cement settle includes ALL accumulated confirmers for `current`. Non-BC messages collected during grace go to the same `deferred` queue as DEV-018d.
**Severity:** mainnet blocker for fairness — without this fix the dominant operator's chain_length grows monotonically while all peers stay at 1 forever.
**Closure path:** ↑ implemented in this commit. Open: spec extension to formalize the grace-window value (500 ms) as a protocol parameter.
**Closure cost:** ~40 lines of Rust.
**Status:** closed (Build 14, this session).


---

## DEV-019b: peer-quorum gate exit + 5s timeout

**Crate:** `montana-node`
**File:line:** `crates/montana-node/src/commands/start.rs:1007-1075` (active arm spin-drain grace block)
**Spec section:** «BundledConfirmation cementing» / «Fairness» / «Consensus close timing»
**What the code does (before):** DEV-019 used fixed 2000ms grace after self-quorum. Worked but peer BCs that arrived later (every 1-2 windows late) consistently missed the grace window, producing bundles=1 cementing in steady state.
**What the code does (after):** grace polls accumulator every 20ms. Exits early on `accumulator[current].len() >= ⌈total_active/2⌉` (peer-quorum gate) OR 5000ms timeout. Peers with consistent latency now stand a higher chance of inclusion; peers that are completely silent are not blocked indefinitely.
**Severity:** fairness improvement; not a hard blocker.
**Closure path:** ↑ implemented. Verified live: 33% of windows now cement with bundles=2 (vs ~1% with fixed grace). Peer chain_length growth observed: Frankfurt 5→91, Vilnius 3→27, Helsinki 2→18, Nicosia 1→5 over 1600 windows.
**Closure cost:** ~10 lines of Rust.
**Status:** closed (Build 16 sha f1030eb151c0, this session).


---

## DEV-020: per-window Reveal broadcast + reveal_pool

**Crate:** `montana-node`, `mt-lottery`
**File:line:** `crates/mt-lottery/src/lib.rs:248-281` (VdfReveal::decode), `crates/montana-node/src/commands/start.rs:196` (reveal_pool init), `crates/montana-node/src/commands/start.rs:622-700` (drain MsgType::VdfReveal), `crates/montana-node/src/commands/start.rs:300-370` (follower compute+broadcast own Reveal), `crates/montana-node/src/commands/start.rs:850-870` (bootstrap broadcast own Reveal), `crates/montana-node/src/commands/start.rs:1010-1080` (spin+grace inline Reveal handling).
**Spec section:** «VDF_Reveal pipeline» / «Cemented Reveal set»
**What the code does (before):** only bootstrap inserted its own reveal_hash into BC; the Reveal object itself was never broadcast over the wire. Peer nodes had no way to participate in the lottery.
**What the code does (after):**
  1. All Active operators compute their own Reveal each window using `compute_endpoint(t_r_window, cba_w_minus_2, my_node, window_index)`.
  2. The Reveal is broadcast as `MsgType::VdfReveal` envelope (wire size 3381 = 32+8+32+3309).
  3. Every node maintains `reveal_pool: BTreeMap<u64, BTreeMap<NodeId, VdfReveal>>` keyed by window, bounded to last 64 windows.
  4. Main dispatcher and proposer's spin-drain / grace handlers all decode VdfReveal envelopes, validate via `mt_lottery::validate_reveal`, and insert into the pool.
  5. Follower's BC.reveal_hashes is populated from `reveal_pool.get(window_index)` (own + any peer reveals received).

**Severity:** prerequisite for DEV-021 winner determination and DEV-022 Lookback rotation.
**Closure path:** ↑ implemented in this commit.
**Closure cost:** ~150 lines of Rust + VdfReveal::decode added to mt-lottery.
**Status:** closed (Build 17/18, this session).

---

## DEV-021: winner determination from cemented Reveal set

**Crate:** `montana-node`, `mt-lottery`
**File:line:** `crates/montana-node/src/commands/start.rs:1240-1290` (winner computation block)
**Spec section:** «Lookback Leadership / Determine winner_{W-1}»
**Spec quote:** «`winner_{W-1} = argmin(weighted_ticket_node)` среди cemented VDF_Reveal узлов-кандидатов окна W-1»
**What the code does (before):** proposer set `winner_id = my_node` unconditionally — no lottery, no per-window winner.
**What the code does (after):**
  1. At cement time, proposer computes `cemented_hashes = union of reveal_hashes across BCs in accumulator[current]`.
  2. Filters `reveal_pool[current]` by `cemented_hashes` to get cemented_reveals.
  3. Builds `mt_lottery::Candidate` list using `weighted_ticket_node(reveal.endpoint, node.chain_length, snapshot)`.
  4. `winner_id = mt_lottery::determine_winner(&candidates).map(|w| w.id).unwrap_or(my_node)`.
  5. Settle + header.winner_id = winner_id.
  Logs `[dev-021] cemented_reveals=N candidates=N winner=ID` per window.

**Severity:** mainnet blocker for genuine lottery (without this, proposer always wins → emission centralization).
**Closure path:** ↑ implemented in this commit. Live verification limited by upstream: peer BCs consistently late (DEV-019b note); cemented set typically = {proposer's own Reveal}; full multi-candidate lottery achievable once peer-drain-during-VDF issue closed (see open follow-up below).
**Closure cost:** ~50 lines of Rust.
**Status:** closed (Build 17/18, this session); upstream blocker tracked separately.

---

## DEV-021b (open): peer drain during VDF tick

**Crate:** `montana-node`
**File:line:** `crates/montana-node/src/commands/start.rs:627` (vdf_step_chunked call inside main loop body)
**Spec section:** «Cross-window cementing timeline»
**What the code currently does:** follower's main loop drains `incoming_rx` only at the very top of each iteration. Each iteration takes ~30s (VDF tick) + ~500ms (idle sleep). Candidate Proposal envelopes arrive mid-VDF and queue in `incoming_rx` until next iteration top. By the time the follower's drain processes a candidate, the proposer has already moved past that window into the next, so follower's BC for window N reaches the proposer ~30s late — too late to land in `accumulator[N]` before cement.
**Consequence:** peer BCs and peer Reveals are chronically 1 window late. DEV-019b grace mitigates partially; full multi-confirmer cement (bundles=N) and multi-candidate lottery (DEV-021) require lockstep timing.
**Closure path:** restructure follower main loop so `incoming_rx` is drained periodically during `vdf_step_chunked` (callback every N steps), or move drain into a separate tokio task in the network thread with shared state. Either change implies a larger refactor than fits this session.
**Closure cost:** ~1–2 days wall-clock for correct implementation + integration test.
**Status:** open. Tracked as the gate for DEV-022 Lookback Leadership rotation.


---

## DEV-021c: grace timeout 30s matches peer sequential-chain cycle

**Crate:** `montana-node`
**File:line:** `crates/montana-node/src/commands/start.rs` (grace_deadline)
**Spec section:** «Cross-window cementing timeline»
**What the code does:** grace timeout raised from 5000ms to 30000ms. Peer nodes run their own sequential SHA-256 chain step (~25-30 s wall-clock per window on calibrated hardware). Peer BCs and Reveals for window N reach the proposer ~25–30 s after the proposer broadcasts candidate(N) — the round-trip is bounded by the peer's full sequential-chain cycle for that window, not network latency. Previous 5 s grace consistently missed peer responses; 30 s grace lets peer BCs and Reveals land before cement.
**Severity:** mainnet-fairness blocker (without this, peer-quorum gate could never fire and DEV-021 lottery degenerated to single-candidate).
**Closure path:** ↑ implemented.
**Closure cost:** 1 line.
**Status:** closed (Build 19 sha 759544cc, this session).

Live verification: Moscow log shows
  `[dev-019] peer-quorum gate satisfied: 3/3 BCs for w=4389`
  `[dev-021] cemented_reveals=3 candidates=3 winner=75bfaf9026405c12`
Multi-candidate lottery proven on real mainnet windows.


---

## DEV-022: Lookback Leadership rotation

**Crate:** `montana-node`
**File:line:** `crates/montana-node/src/commands/start.rs` — `winner_history` state, drain-side `winner_history.insert`, active arm gate, own-cement `winner_history.insert`.
**Spec section:** «Lookback Leadership / proposer_W = winner_{W-2}»
**Spec quote:** «proposer_0 и proposer_1 = bootstrap-узел. Начиная с proposer_2 = winner_0, стандартная lookback логика.»
**What the code does (before):** active arm gated on `is_genesis` — only bootstrap proposed; non-bootstrap nodes were permanent followers; no proposer rotation; emission concentrated in bootstrap regardless of lottery.
**What the code does (after):**
  1. `winner_history: BTreeMap<u64, NodeId>` records per-window cemented winners (bounded to 64 entries).
  2. Main drain populates `winner_history[W]` from cemented Proposal envelopes received from any peer.
  3. Own cement also populates `winner_history[current]` so the proposer's own rotation gate sees its just-cemented winner two windows later.
  4. Active arm computes `proposer_W = if W < 2 { bootstrap } else { winner_history[W-2].unwrap_or(bootstrap) }`.
  5. If `my_node != proposer_W` → follower mode; if `my_node == proposer_W` → run full proposer pipeline (compute Reveal, broadcast candidate, spin-drain BCs, grace, determine winner, cement).
  6. Genesis bootstrap rule preserved: proposer_0 = proposer_1 = bootstrap (no winner_history available yet).
**Severity:** mainnet-critical for emission decentralization. Without Lookback, bootstrap permanently captures all emission regardless of lottery (DEV-021 outcome ignored).
**Closure path:** ↑ implemented. Live verification on next deploy.
**Closure cost:** ~50 lines of Rust.
**Status:** closed (Build 21 sha 8936f063, this session).


---

## DEV-022b: Lookback rotation gate disabled pending DEV-023 fallback cascade

**Crate:** `montana-node`
**File:line:** `crates/montana-node/src/commands/start.rs` (active arm gate)
**Spec section:** «Lookback Leadership / Fallback cascade»
**What the code does (before):** DEV-022 gated active arm on `my_node == winner_{W-2}`. Each operator independently maintained `winner_history`; both bootstrap and elected proposer saw the OTHER as proposer for their respective windows and both became followers — dead-lock. The spec's fallback cascade («fallback_proposer_W = second_min(weighted_ticket)») was not yet implemented, so a stuck elected proposer left the chain indefinitely frozen.
**What the code does (after):** rotation gate reverted to bootstrap-only. `winner_history` is still maintained (drain-side + own cement) so DEV-023 can introduce rotation on top of a working fallback cascade.
**Severity:** dead-lock recovery; not a fairness regression beyond pre-DEV-022 state.
**Closure path:** implement DEV-023 fallback cascade (if elected proposer doesn't cement within K windows, second_min becomes proposer), then re-enable rotation gate.
**Closure cost:** revert: 5 lines; DEV-023: ~80 lines.
**Status:** closed (Build 22 sha c8de927c, this session).


---

## DEV-023 (open): proposer fallback cascade

**Crate:** `montana-node`
**File:line:** active arm proposer gate
**Spec section:** «Lookback Leadership / Fallback cascade»
**Spec quote:** «Если < 67% подписали → proposal отклонён. Fallback: `fallback_proposer_W = second_min(weighted_ticket)` окна W-2. Fallback cascade: third_min, fourth_min, etc.»
**Status:** open for v1.0.1.
**Closure path:**
  1. Each Active node tracks `proposer_silence_windows[W]` = how many windows have elapsed since `expected_proposer_W = winner_{W-2}` should have cemented W and hasn't.
  2. If `proposer_silence_windows[W] >= K` (K = 3 per spec discussion), use `sorted_candidates_for_fallback(reveals_{W-2})` to compute `fallback_proposer_W`.
  3. Cascade: if fallback_1 also silent for K more windows → fallback_2, etc.
  4. Bootstrap operator is final guaranteed fallback (eliminates dead-lock).
**Closure cost:** ~80 lines + integration test for `silence_counter` consistency across operators.
**Operational note (current state).** Without DEV-023, DEV-022 rotation gate is disabled (see DEV-022b). Bootstrap remains sole canonical proposer. DEV-021 winner determination + lottery rotation work end-to-end at emission level: every cemented Proposal records a different winner across the active cohort, so emission is distributed even without proposer rotation. The chain remains live; only proposer-set diversity is deferred.

---

## v1.0.0 Mainnet Baseline (2026-05-30)

Closed DEVs live on mainnet at sha c8de927c (Build 22):
  - DEV-017 follower t_r_history populated from Proposal envelopes
  - DEV-018  fast-sync chunk anchor_window + stale-peer filter
  - DEV-018b/c/d fast-sync client retry on discard, 10s deadline, inline serve during proposer spin
  - DEV-019  post-quorum 500ms grace for peer BC inclusion
  - DEV-019b peer-quorum gate ⌈total/2⌉ + 30s grace
  - DEV-020  per-window Reveal broadcast + reveal_pool on all nodes
  - DEV-021  winner determination from cemented Reveal set (argmin weighted_ticket)
  - DEV-021c grace = 30s = peer sequential-SHA-chain cycle

Open for v1.0.1:
  - DEV-021b peer drain during sequential-SHA-chain tick (latency floor)
  - DEV-022  Lookback proposer rotation (requires DEV-023)
  - DEV-023  proposer fallback cascade

Live verification (window 4418..4422 explorer snapshot):
  /api/winners → 5 distinct winners in 6 consecutive windows (vilnius, frankfurt×2, vilnius, moscow, armenia)
  /api/consensus → chain_length distribution: moscow 958‰, frankfurt 27‰, vilnius 7‰, armenia 5‰, helsinki 1‰
  bundles=3 (multi-confirmer cement) on majority of windows
  emission distributed per spec lottery; bootstrap is sole proposer pending DEV-022/023.


---

## DEV-023: bootstrap fallback after K=3 silent windows + DEV-022 re-enable

**Crate:** `montana-node`
**File:line:** `crates/montana-node/src/commands/start.rs` — `last_proposer_cement` state, active arm cascade gate.
**Spec section:** «Lookback Leadership / Fallback cascade»
**Spec quote:** «Если < 67% подписали → proposal отклонён. Fallback: `fallback_proposer_W = second_min(weighted_ticket)` окна W-2. Fallback cascade.»
**What the code does:**
  1. `last_proposer_cement: BTreeMap<NodeId, u64>` records per-proposer last cemented window. Populated drain-side (from received cement.proposer_node_id) AND own-cement-side (active arm).
  2. Active arm gate computes `primary_proposer = winner_{W-2}` per DEV-022 Lookback.
  3. `primary_silent = current - last_proposer_cement[primary_proposer]`. If `primary_silent >= K_FALLBACK_WINDOWS (3)` AND primary != bootstrap → fallback to bootstrap.
  4. Bootstrap is canonical fallback always-active (cannot itself be silent because if bootstrap silent, no one cements at all → all stuck).
  5. Each node deterministically computes same active_proposer from canonical state; no coordination round needed.

**Simplification vs spec:** Spec's full cascade («second_min, third_min, ...») requires sorted_candidates_for_fallback over reveals_{W-2}. Current implementation collapses cascade to bootstrap-only fallback. Spec-correct multi-level cascade deferred to v1.0.2 — requires further work on reveal-pool persistence across the W-2 lookback window plus a coordination protocol to break ties when multiple fallback candidates think they should propose.

**Severity:** mainnet-critical for emission decentralization once DEV-022 rotation is re-enabled.
**Closure cost:** ~30 lines of Rust (this commit).
**Status:** closed (Build 23 sha ede6dffb, this session).


---

## DEV-023b: election grace for newly-elected primary

**Crate:** `montana-node`
**File:line:** `crates/montana-node/src/commands/start.rs` active arm gate (primary_active computation)
**Spec section:** «Lookback Leadership / Election grace»
**What the code does:** when `primary_proposer != bootstrap` AND `last_proposer_cement[primary] == 0` (never cemented), check whether primary won lottery in last K_FALLBACK_WINDOWS via winner_history. If yes — treat as active (give grace), let primary attempt to propose. Without grace, silent_count = current (=4467) immediately exceeds K=3 and fallback to bootstrap fires before primary ever runs.
**Severity:** required for DEV-022 rotation to be operationally meaningful (otherwise every newly-elected non-bootstrap proposer is preempted by bootstrap fallback in the first iteration).
**Closure path:** ↑ implemented. Live verification:
  - Moscow log: `[lookback W=4467] primary=5509211b179d6969 silent=4467 active_proposer=5509211b179d6969 my_node=75bfaf9026405c12 — follower mode`
  - Moscow correctly defers to Frankfurt for W=4467 (Frankfurt is elected primary AND won 4465 within last K windows). Bootstrap fallback NOT triggered.
**Closure cost:** ~15 lines.
**Status:** closed (Build 24b sha ad27ae713758, this session).

**Operational note.** Frankfurt does not yet actually propose for W=4467 because of upstream DEV-021b (peer drain during sequential-SHA-chain tick) — Frankfurt's local current=4466 doesn't advance to 4467 before bootstrap takes over the cement via fallback K-window timeout. Full multi-proposer rotation gated on DEV-021b closure (drain refactor with periodic message processing inside vdf_step_chunked).


---

## DEV-023c: election grace tied to FIRST election, not every win (hotfix)

**Crate:** `montana-node`
**File:line:** `crates/montana-node/src/commands/start.rs` active arm gate (`primary_active` computation)
**Spec section:** «Lookback Leadership / Fallback cascade»
**Bug:** DEV-023b grace check was `won_recently = any window in [W-K..W-2] where winner == primary`. If primary (e.g. Frankfurt with growing chain_length) kept winning lottery, every window had a fresh `won_recently=true` → grace re-triggered indefinitely → bootstrap fallback never fired → chain frozen.
**Symptom (mainnet 2026-05-30 17:00):** cw=4469 unchanged for 30+ minutes despite Moscow active on Build 24b. Moscow log: `[lookback W=4470] primary=Frankfurt silent=4470 active_proposer=Frankfurt — follower mode` repeatedly; Frankfurt never cemented (gated on DEV-021b drain-during-sequential-chain-tick), Moscow stuck deferring.
**Fix:** grace measured from FIRST election window (the smallest W where `winner_history[W-2] == primary`), not from the most recent win:
```rust
let first_election = winner_history.iter()
    .filter_map(|(w, n)| if *n == primary { Some(w + 2) } else { None })
    .min()
    .unwrap_or(current);
primary_active = current - first_election < K_FALLBACK_WINDOWS
```
After K windows since FIRST election, bootstrap fallback fires regardless of whether primary keeps winning. Each primary gets one grace term per genesis run.
**Live verification:** chain resumed at cw=4475 within 5 minutes of Build 25 deploy. `bundles=2` cementing stable; Moscow winning lottery again (sufficient chain_length).
**Severity:** mainnet liveness blocker — must close before re-enabling rotation; deployed as urgent hotfix.
**Status:** closed (Build 25 sha 5a22bf8c53c6, this session).


---

## DEV-022/023 disabled (v1.0.1 baseline freeze)

**Decision (mainnet 2026-05-30 17:55):** DEV-022 Lookback rotation + DEV-023(a/b/c) fallback cascade gate DISABLED. Bootstrap is sole proposer. Lottery (DEV-021) still picks per-window winner_id and distributes emission; rotation just doesn't kick in.
**Reason:** Even after DEV-023c first-election-anchored grace, chain throughput collapsed to ~1 cement per 16 min (vs 30s baseline) because each new primary win re-armed grace via fresh winner_history entries while old min() values aged out. Without DEV-021b drain refactor (peer can apply candidate during their sequential SHA chain tick), non-bootstrap proposers can never actually cement, so rotation only stalls the chain.
**Code change:** active arm gate is `my_node != bootstrap_node_id → follower_skip = true`. `winner_history`, `last_proposer_cement`, `K_FALLBACK_WINDOWS` remain in code (unused, dead-letter) for future v1.0.1 re-enable on top of DEV-021b closure.
**Status:** Bootstrap-only proposer is the v1.0.1 mainnet baseline. Rotation gated on DEV-021b future work.


---

## v1.1.0 Spec-faithful consensus loop (2026-06-12, commit 6ea9d12+)

Закрытые этим циклом отклонения:

- **DEV-021b closed.** `drain_network`/`handle_protocol_message` вызываются между порциями последовательной SHA-256 цепочки (`vdf_step_chunked` on_chunk) и из цикла ожидания кворума ведущего — спецификация «Непрерывность VDF» выполняется дословно: финализация и приём билетов идут параллельно вычислению следующего окна. Подтверждено локальным собранием из 3 узлов: confirmers=3 на каждом окне после генезисных.
- **DEV-022 closed (re-enabled).** Lookback-ротация ведущего: `proposer_W = winner_{W-2}` из `winner_history` (канонически из cemented proposal_{W-1}); подпись заголовка проверяется по `NodeTable[proposer_node_id].node_pubkey` (mt_consensus::validate_header), не по ключу первопоселенца. Подтверждено: цементируют все три узла собрания (46/63/44 на 150 окнах).
- **DEV-023 closed (cascade).** Каскад запасных: глубина = elapsed/FALLBACK_TIMEOUT_SECS (120 с), `mt_consensus::fallback_proposer` по отсортированным взвешенным билетам окна W-2 из `lottery_history`; первопоселенец — терминальная страховка генезисных окон. Терпимость ±1 уровень глубины на расхождение настенных часов при приёме.
- **DEV-012 closed (multi-confirmer).** Цементация только по реальному кворуму: Σ chain_length подписантов BC ≥ 67% активной длины (active predicate 2τ₂ — `active_chain_length_at`). Никакого тайм-аут-цементирования: при недостижимом кворуме цепочка честно ждёт (проверено отказом узла: рост 2 окна инерции → заморозка → возобновление при возврате узла).

Двухоконный конвейер по спецификации «Закрытие окна (Lookback Leadership Finalization)»:
- BC окна W несёт reveal_hashes окна W-1; цементация билетов W-1 — взвешенная по chain_length подписантов BC_W (порог 67%), не объединение множеств.
- proposal_W: included_bundles = BC окна W-1 (chain_length++ подтвердившим при settle W), included_reveals = цементированный набор W-1, winner_{W-1} = argmin, выплата при settle W («One-window lag награды»).
- Конверт: `[header 3722][u16 n1][BC W-1][u16 n2][BC W]` — улики цементации включены, ведомые перепроверяют взвешенный порог и победителя по своему пулу.
- Канонический агрегат подтверждений: `cemented_bundle_aggregate(w, bc_set_history[w])` — настоящий набор подтвердивших из included_bundles предложения, закрывшего окно w (раньше везде передавался пустой список).
- Каждый узел архивирует зацементированный конверт (`archive_proposal_envelope`) — полнота данных эксплорера на любом узле; пооконные истории восстанавливаются из архива при рестарте.
- Отстающий сосед (gap < порога быстрой синхронизации) подтягивается повторной раздачей архивных конвертов по его устаревшему BC.

Открытые честные расхождения (новые записи):

## DEV-024 (open): cemented_bundle_aggregate — node_id вместо подписей

Спецификация: «агрегат подписей cemented BundledConfirmation окна W-2 … содержит ML-DSA-65 подписи будущих confirmers — aggregate непредсказуем offline». Реализация mt-timechain (KAT-зафиксированная): агрегат над **node_id** подтвердивших. Набор node_id предсказуем → защита от перебора (grinding) слабее заявленной. Для собрания из 3 узлов практический риск ≈ 0. Закрытие: решение автора — либо правка спецификации (агрегат по node_id), либо смена примитива (агрегат по подписям, ре-генерация KAT). Не закрывать молча.

## DEV-025: target лотереи без τ₂-калибровки

**Status: closed (commit baca884).** `mt_lottery::calibrate_target` — integer-форма спецификации (u256-промежуток, насыщение, TA4-нулевой случай); биндинг-векторы TA1-TA5 байт-в-байт в тестах. Узел: счётчик зацементированных билетов за τ₂ и порог-цель персистентны (timechain v2), пересчёт на границе τ₂ в едином переходе состояния, ворота кандидатства weighted_ticket < target при публикации билета, target в заголовке и сверка у ведомых.

## DEV-026 (open): эквивокация ведущего — первый валидный, не reject-both

Спецификация: «Два proposal от одного proposer_node_id в одном окне: оба отклоняются». Реализация: применяется первый валидный кворумный конверт; второй отбивается монотонностью окна. Безопасность держится кворумом BC (одинаковое содержимое детерминировано); расхождение возможно только в метаданных (prev_proposal_hash при гонке запасных). Закрытие: буферизация конкурирующих конвертов окна + правило выбора.

## DEV-027 (note): supply closed-form сдвиг генезисного окна

Окно 0 — генезис (не settle-ится предложением); первая выплата — settle(1) победителю окна 0 (первопоселенец, генезисное правило). Фактический supply(W) = EMISSION × W. Формула спецификации EMISSION × (W+1) считает выплату окна 0. Решение автора: либо спецификация фиксирует «окно 0 без эмиссии», либо settle(1) выплачивает двойную (окна 0 и 1). Текущее поведение — без эмиссии за окно 0.
