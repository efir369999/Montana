# Montana v1.0.0 — external audit scope

**Release tag.** `v1.0.0` (2026-05-22). Annotated git tag at `a260ba9005c48763fadad0de5797bae48989215e`.
**Repository.** https://github.com/efir369999/Montana
**Workspace version.** Rust workspace, 18 crates, all pinned to `1.0.0`.
**Spec target.** Protocol v35.25.1 + Network v1.1.0 + App v3.12.0.

This document defines the boundaries of an external audit engagement against the v1.0.0 mainnet tag. It is structured for a cryptographer / systems reviewer who has read [`External-Audit/README-external-audit-v1.0.0.md`](README-external-audit-v1.0.0.md) and wants to understand exactly what is in scope, what is out of scope, and what evidence the maintainer commits to provide.

---

## 1. In scope (priority 1)

These items are the explicit external-audit asks accompanying the mainnet announcement.

### 1.1 Noise_PQ XX transport — transcript binding

**Files.**
- [`Code/crates/mt-noise-pq/src/xx_handshake.rs`](../Code/crates/mt-noise-pq/src/xx_handshake.rs) — handshake state machine (3 messages, 1184 / 7533 / 6349 bytes).
- [`Code/crates/mt-noise-pq/src/xx_libp2p_upgrade.rs`](../Code/crates/mt-noise-pq/src/xx_libp2p_upgrade.rs) — async drive functions for the libp2p upgrade.
- [`Code/crates/mt-net-transport/src/xx_noise_pq_upgrade.rs`](../Code/crates/mt-net-transport/src/xx_noise_pq_upgrade.rs) — `NoisePqXxConfig` + `InboundConnectionUpgrade` + `OutboundConnectionUpgrade` trait impls.
- [`Montana Network v1.1.0.md`](../Montana%20Network%20v1.1.0.md) — wire-format normative source.

**Audit asks.**
- Ordering: does the ML-DSA-65 signature cover the post-decapsulation transcript hash, not the pre-decapsulation hash? Is the binding sound against substitution attacks on the ML-KEM-768 ciphertext?
- Transcript opacity: does any field used in the transcript hash carry attacker-grindable content?
- PeerId binding: does the SHA-256 multihash (libp2p sha2-256 multihash code 0x12) over the ML-DSA-65 identity public key prevent identity-substitution attacks during the handshake?

**Existing tests.**
- [`Code/crates/mt-noise-pq/tests/loopback.rs`](../Code/crates/mt-noise-pq/tests/loopback.rs) — TCP loopback handshake.
- [`Code/crates/mt-net-transport/tests/three_peer_e2e.rs`](../Code/crates/mt-net-transport/tests/three_peer_e2e.rs) — three-peer end-to-end with the libp2p Swarm.

### 1.2 Sequential SHA-256 delay function reduction

**Files.**
- [`Code/crates/mt-timechain/src/lib.rs`](../Code/crates/mt-timechain/src/lib.rs) — `vdf_step`, `next_d`, `cemented_bundle_aggregate`.
- [`Whitepaper Montana.md`](../Whitepaper%20Montana.md) — §5 attack-class subsections + §13 cryptographic primitives table.

**Audit asks.**
- The implementation is explicitly **not** a VDF in the Boneh / Pietrzak / Wesolowski sense — no proof of correct evaluation. What is the reduction from the cementing rule to the unforgeability of `t_r(W)` under the asymmetry between proposer and verifier?
- Lookback leadership rule (`cemented_bundle_aggregate(W − 2)` as the seed for endpoint computation): is the two-window lookback sufficient against grinding when the proposer has hardware advantage ×K over a verifier?

### 1.3 Constant-time review of `mt-crypto-native`

**Files.**
- [`Code/crates/mt-crypto-native/`](../Code/crates/mt-crypto-native/) — C bindings over OpenSSL 3.5 LTS for ML-DSA-65 and ML-KEM-768.

**Audit asks.**
- Per-row constant-time requirement on ML-DSA-65 and ML-KEM-768 is spec-stated (`Montana Protocol v35.25.1.md`, §«Cryptographic implementation → Primitives layer»). Is the production crypto path constant-time end-to-end, or are there leaks via `if`-branches keyed on secret bits, memory access patterns on secret-indexed tables, or timing-variable arithmetic on secret material?
- Are the secret-material allocations protected by `mlock`? Are the `Drop` impls zeroing the memory before returning to the allocator?

**Existing evidence.**
- [`Code/docs/security-cards.md`](../Code/docs/security-cards.md) — Security Cards per primitive with secret-site enumeration (file currently in mixed-language form — see audit deviation F-006 below).
- [`Code/crates/mt-crypto/tests/security_invariants.rs`](../Code/crates/mt-crypto/tests/security_invariants.rs) — 13 automated security invariants.

---

## 2. In scope (priority 2)

These are spec-vs-code conformance asks, not crypto-primitive asks. Lower priority for the cryptographer, higher priority for the systems reviewer.

### 2.1 M7 fast-sync state-root verifier

**Files.**
- [`Code/crates/mt-sync/src/snapshot.rs`](../Code/crates/mt-sync/src/snapshot.rs) — `Snapshot`, `SnapshotVerifier`, `build_tables`, `from_tables`, `to_wire_chunks`.
- [`Code/crates/mt-state/src/lib.rs`](../Code/crates/mt-state/src/lib.rs) — `compute_state_root` + Sparse Merkle tables.

**Audit asks.**
- The verifier byte-equals the proposer's `state_root` for the same record set via the Sparse Merkle root. Tested across 17 unit tests including order-independence. Is the SMT depth-256 construction implementing the spec layout byte-exact?
- The decode path (`AccountRecord::decode` / `NodeRecord::decode` / `CandidateRecord::decode`) is fixed-size canonical; is the layout consistent with `CanonicalEncode` byte-for-byte?

### 2.2 Spec deviations log

**File.** [`Code/docs/SPEC_DEVIATIONS.md`](../Code/docs/SPEC_DEVIATIONS.md) — every spec-vs-code deviation with closure status.

**Currently open entries (carried into v1.0.1 hot-fix track).**
- **DEV-012 Phase B + C** — multi-confirmer cementing in the Active phase. The v1.0.0 mainnet baseline uses bootstrap-proposer + follower-apply; the multi-confirmer rotation is needed once non-bootstrap operators accumulate `chain_length`.
- **DEV-015** — M7 client-side handler (drain chunks + verify + LocalState swap). New operators on v1.0.0 join the live mesh by replaying the canonical history via the existing `apply_proposal`-from-peers path.

**Audit asks.**
- Are the closure-path descriptions in DEV-012 / DEV-015 sufficient to enable independent reproduction of the fix?
- Are there other deviations the auditor identifies in the v1.0.0 tree that are not yet logged in `SPEC_DEVIATIONS.md`?

---

## 3. Out of scope

These are explicitly **not** the v1.0.0 audit scope. Auditor is free to comment but the maintainer commits no closure path against findings here.

- **App-layer code.** The iOS and Android apps under `Montana/iOS/` and `Montana/Android/` are out of scope. The reference protocol implementation is the Rust workspace under `Code/`.
- **VPN sub-protocol.** The xray Reality VPN running on the same nodes is operationally bundled with the Montana mesh but is a separate sub-protocol and is not part of the consensus path.
- **Spec entities not yet implemented.** Entries in `Code/docs/SPEC_DEVIATIONS.md` marked `pending` or `open` with explicit closure paths are acknowledged design gaps; the auditor is free to comment, but the v1.0.0 audit asks no closure here.

---

## 4. Audit-pass deviations known to the maintainer

The internal critic pass against the role CRITIC.md v3.14.0 (see [`critic-audit-v1.0.0-mainnet.md`](critic-audit-v1.0.0-mainnet.md)) identified eight findings; two closed mechanically before the v1.0.0 tag push (F-001 README mainnet wording, F-002 Network spec gate framing), six escalated to the author. The two highest-impact open findings for an external reader are:

- **F-003 (high)** — `Montana Network v1.1.0.md` contains 1796 Cyrillic-character hits. The wire format and KAT-byte counts are unaffected; the prose blocks and ASCII layout diagrams contain Russian commentary that has not been re-authored in English yet. The auditor should be aware that mixed-language content in the network specification is a known deviation from the English-only intent of the published artifact set.
- **F-005..F-008 (medium / low)** — `Code/docs/audit-checklist.md`, `Code/docs/security-cards.md`, `Code/docs/build-from-source.md`, `Code/VERSION.md` History column — same class of mixed-language deviation in supporting documents.

The auditor is welcome to comment on these as a separate finding class; the maintainer commits to closure in a follow-up release, **not** as a blocker against the v1.0.0 audit pass.

---

## 5. Engagement model

- **Findings format.** GitHub issues at https://github.com/efir369999/Montana/issues with label `mainnet-v1.0.0`. Plain-text replies on the Metzdowd Cryptography List (`cryptography@metzdowd.com`) are also read.
- **Confidentiality.** None. Public on-record review only. The maintainer does not accept findings under embargo.
- **Severity scale.** Critical / High / Medium / Low / Observational. Maintainer commits to acknowledge any finding within seven days of submission and publish a written disposition within thirty days.
- **No bug-bounty.** No financial incentive is offered. The repository is dual-licensed Apache-2.0 / MIT; the protocol is non-token.

---

## 6. Reproducibility artefacts

- **CI on the v1.0.0 tag.** `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace --release` — all green.
- **Live mesh.** [efir.org/explorer/](https://efir.org/explorer/) — Moscow, Frankfurt, Helsinki, Yerevan online + any external operators that have joined.
- **Install path.** `git clone https://github.com/efir369999/Montana.git /opt/montana && sudo bash /opt/montana/Code/scripts/install-vps.sh`. End-to-end onboarding verified at ~16 minutes from clone to live heartbeat exchange.
- **Build reproducibility.** [`Code/docs/build-from-source.md`](../Code/docs/build-from-source.md) — pinned to OpenSSL 3.5.5 LTS via `openssl-src = "=300.5.5+3.5.5"` for byte-exact crypto-native rebuild.
- **Spec deviation log.** [`Code/docs/SPEC_DEVIATIONS.md`](../Code/docs/SPEC_DEVIATIONS.md).

— Montana maintainer, 2026-05-22.
