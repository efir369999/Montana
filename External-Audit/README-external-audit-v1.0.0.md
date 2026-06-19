# Montana v1.0.0 — external audit reading list

**Release tag:** `v1.0.0` (2026-05-22)
**Release URL:** https://github.com/efir369999/Montana/releases/tag/v1.0.0
**Live mesh:** https://efir.org/explorer/ — Moscow / Frankfurt / Helsinki / Yerevan
**Spec target:** Protocol v35.25.1 + Network v1.1.0 + App v3.12.0

This bundle is the curated entry point for an external cryptographer or systems reviewer landing on the repository after the mainnet announcement. The order below is the recommended reading order; each item names what is in scope and what is out of scope.

> **If you want to verify the v1.0.0 tag end-to-end from a fresh shell — open [`AUDIT-WALKTHROUGH-v1.0.0.md`](AUDIT-WALKTHROUGH-v1.0.0.md) first.** It is the runnable ten-step checklist with hash anchors, public probes, install path, and finding-submission flow. Everything below is the reading material the walkthrough cites.

---

## 1. The whitepaper

**File:** [`Whitepaper Montana.md`](../Whitepaper%20Montana.md)

The protocol-level claim. Adversary model, threat surface, sequential delay function over SHA-256 with the explicit non-SSHA disclaimer in §5, multi-class lottery, cementing rule and quorum, Identity-Bound Tunnels, the eight attack-class subsections of §5, and the cryptographic primitives table in §13.

What to look for: implicit assumptions in §5 attack-class subsections, particularly the cost asymmetry between proposer and verifier in §5.3 (sequential-delay grinding); the section §13 row-level constant-time requirement on ML-DSA-65 and ML-KEM-768.

---

## 2. The network specification

**File:** [`Montana Network v1.1.0.md`](../Montana%20Network%20v1.1.0.md)

The wire-level claim. Noise_PQ XX message-by-message byte layout (msg1 1184 B / msg2 7533 B / msg3 6349 B), Yamux session, message-type registry, envelope sizes, IBT mesh and online proofs with the `online_session_nonce` replay defence, multi-confirmer cementing protocol, fast-sync chunk format.

What to look for: the transcript-binding ordering between ML-KEM-768 decapsulation and the ML-DSA-65 signature in the XX handshake; whether the `online_session_nonce` design generalises to the multi-confirmer cementing envelope schema bump scheduled for v1.0.1.

---

## 3. The implementation

**Workspace root:** [`Code/`](../Code/)
**Workspace version:** 1.0.0 (18 crates pinned)

| Crate | Scope under audit |
|-------|-------------------|
| [`mt-crypto-native`](../Code/crates/mt-crypto-native/) | C bindings over OpenSSL 3.5 LTS for ML-DSA-65 and ML-KEM-768. Constant-time row-level requirement is the explicit external-audit ask. |
| [`mt-noise-pq`](../Code/crates/mt-noise-pq/) | Noise_PQ XX handshake state machine + libp2p UpgradeInfo / Inbound / Outbound trait impls. Loopback and three-peer end-to-end tests. |
| [`mt-net-transport`](../Code/crates/mt-net-transport/) | The libp2p Swarm wiring that selects `/montana/noise-pq-xx/1.0.0` for every connection. |
| [`mt-timechain`](../Code/crates/mt-timechain/) | Sequential SHA-256 delay function. `ssha_step` is the canonical name despite the non-SSHA status — kept for backward compatibility with the spec wording. |
| [`mt-consensus`](../Code/crates/mt-consensus/) | ProposalHeader layout, signed scope, R1 / R2 rules, validate_header. |
| [`mt-lottery`](../Code/crates/mt-lottery/) | BundledConfirmation, lottery weights, cementing rule, quorum, validate_bundle. |
| [`mt-state`](../Code/crates/mt-state/) | AccountTable / NodeTable / CandidatePool over Sparse Merkle, depth 256. `compute_state_root` domain-separated combiner. |
| [`mt-sync`](../Code/crates/mt-sync/) | M7 fast-sync: Snapshot, SnapshotVerifier (production SMT root), wire chunk encoding. 17 unit tests including byte-equal cross-implementation conformance proof. |
| [`montana-node`](../Code/crates/montana-node/) | The operator binary. Single-process, single-thread consensus loop; tokio runtime confined to the network thread. |

---

## 4. Specification deviations

**File:** [`Code/docs/SPEC_DEVIATIONS.md`](../Code/docs/SPEC_DEVIATIONS.md)

The complete, on-record list of every spec-vs-code deviation, each with severity, closure path, and current status. Current open entries:

- **DEV-012 Phase B+C** — multi-confirmer cementing carried into v1.0.1. The v1.0.0 mainnet baseline is bootstrap-proposer + follower-apply; the multi-confirmer protocol becomes operationally consequential only after non-bootstrap operators accumulate chain_length over many τ₂ epochs.
- **DEV-015** — M7 client-side handler (drain + verify + LocalState swap) carried into v1.0.1. New operators join the live mesh by replaying the canonical history via the existing `apply_proposal`-from-peers path.

All other DEV-NNN trackers are closed with citation.

---

## 5. Sixteen-finding audit response

**File:** [`montana-response-to-2026-05-19-audit.md`](montana-response-to-2026-05-19-audit.md)
**Original audit:** [`montana-security-review-consolidated-2026-05-19-en.md`](montana-security-review-consolidated-2026-05-19-en.md)

Disposition matrix for the CISO-as-a-Service Team consolidated review of 2026-05-19. Twelve accepted and fixed by construction in the whitepaper (WP-1..WP-12); two rejected with spec citation (MONT-003 race condition, WP-8 sub-claim); MONT-001 closed by spec patch (constant-time on PQ rows); MONT-002 closed by `online_session_nonce`; MONT-004 documented as pre-mainnet operating state; DEV-014 closed by the Noise_PQ XX migration itself.

---

## 6. The deep retrospective

**File:** [`montana-deep-retrospective-2026-05-21.md`](montana-deep-retrospective-2026-05-21.md)

Empirical record of the four-node mesh: where Frankfurt drifted, why, what the fix was, how convergence was re-established after the `follower_skip` patch. Useful for understanding what the live network actually looks like under operational load before the v1.0.0 tag.

---

## 7. Transport-layer identifier leakage analysis

**File:** [`transport-identifier-leakage.md`](transport-identifier-leakage.md)

Byte-by-byte comparison of MTProto (Telegram) and Noise_PQ XX (Montana) under the passive-observer threat model. Establishes that the Montana production transport has no plaintext long-term identifier on the wire, so the retroactive-correlation class of attacks that succeeds against MTProto `auth_key_id` is structurally not reachable against Montana. Cross-referenced from `Montana Network v1.1.0.md` §«Network layer — Threat Model».

---

## How to reach the maintainer

GitHub issues. No email, no Discord, no Telegram — public on-record review only. The repository's CI on the v1.0.0 tag (`cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace --release`) is green; any failure is the maintainer's regression and a blocking issue.
