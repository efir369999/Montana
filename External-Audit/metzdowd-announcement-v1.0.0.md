# Metzdowd Cryptography List announcement — Montana v1.0.0 mainnet

**Audience:** cryptography@metzdowd.com
**Subject line:** `[ANN] Montana v1.0.0 — first mainnet release of a post-quantum sequential-delay-function blockchain (Noise_PQ XX, ML-KEM-768 + ML-DSA-65)`

---

Hello list,

Montana v1.0.0 was tagged today (2026-05-22) as the first mainnet release of a post-quantum blockchain whose consensus is anchored on a sequential delay function over SHA-256 rather than on a verifiable delay function in the Boneh / Pietrzak / Wesolowski sense. The reference implementation is a Rust workspace of eighteen crates; sources, release notes, and the running infrastructure are public on-record.

Release: https://github.com/efir369999/Montana/releases/tag/v1.0.0

What I am posting for review:

1. **Transport.** Production handshake is Noise_PQ XX over TCP, followed by Yamux. ML-KEM-768 ephemeral key encapsulation on both sides, ML-DSA-65 identity signatures over the transcript hash, ChaCha20-Poly1305 AEAD on the established session. Wire format byte-exact to the Network specification v1.1.0 (msg1 1184 B / msg2 7533 B / msg3 6349 B). PeerId derived as the SHA-256 multihash (libp2p sha2-256 multihash code 0x12) of each peer's ML-DSA-65 identity public key. The handshake state machine and the libp2p integration live in `Code/crates/mt-noise-pq/src/xx_handshake.rs` and `Code/crates/mt-net-transport/src/xx_noise_pq_upgrade.rs`. Loopback and three-peer end-to-end tests are in `Code/crates/mt-noise-pq/tests/` and `Code/crates/mt-net-transport/tests/three_peer_e2e.rs`.

2. **Sequential delay over SHA-256.** Montana's `t_r(W)` ticks via a deterministic `D`-iteration SHA-256 chain (`Code/crates/mt-timechain/src/lib.rs::vdf_step`). The window endpoint is the cemented-bundle aggregate over the preceding two τ₂ epochs (`cemented_bundle_aggregate`). I explicitly do not claim this is a VDF — there is no Pietrzak / Wesolowski proof of correct evaluation. The asymmetry between the proposer and the verifier is the asymmetry the spec calls out as part of the threat model.

3. **State integrity.** State is held in three Sparse Merkle Trees indexed by AccountId, NodeId, and CandidateId, all depth-256, all rooted via a domain-separated `state_root = SHA-256("mt-state-root" || node_root || candidate_root || account_root)`. M7 fast-sync delivers the state at an anchor window as canonical-encoded record chunks; the verifier byte-equals the proposer's `state_root` through the production SMT path. The implementation, tests, and the byte-equal cross-implementation conformance proof are in `Code/crates/mt-sync/src/snapshot.rs`.

4. **Sixteen-finding audit response.** The CISO-as-a-Service Team review of 2026-05-19 is closed. Disposition matrix at `External-Audit/montana-response-to-2026-05-19-audit.md`. Two findings are rejected with spec citation (MONT-003 race condition, WP-8 sub-claim); twelve are accepted and fixed by construction in the whitepaper (WP-1..WP-12); MONT-001 closed by spec patch (explicit constant-time requirement on the ML-DSA-65 / ML-KEM-768 rows), MONT-002 closed by `online_session_nonce` in the IBT online proof, MONT-004 documented as the pre-mainnet operating state, DEV-014 closed by the Noise_PQ XX migration itself.

5. **What is open and not in v1.0.0.** Multi-confirmer cementing in the Active phase (DEV-012 Phase B + C) and the M7 client-side handler (DEV-015) are scheduled for v1.0.1 hot-fix. The bootstrap-proposer + follower-apply path is the v1.0.0 mainnet baseline. An independent constant-time audit of the production crypto path in `mt-crypto-native` (MONT-001) is the explicit external-audit ask after the mainnet tag.

What I am asking for from the list:

- Independent review of the Noise_PQ XX transcript binding and the ML-DSA-65 sign-after-decapsulation ordering in `xx_handshake.rs`.
- Independent review of the SHA-256 sequential delay function reduction in `mt-timechain` and its absence of a Pietrzak / Wesolowski proof of correct evaluation, particularly the implications for the lookback leadership rule.
- Independent constant-time review of the `mt-crypto-native` C bindings over OpenSSL 3.5 LTS for ML-DSA-65 and ML-KEM-768 (the row-level constant-time requirement is spec-stated; the bindings are exercised in `Code/crates/mt-crypto-native/tests/`).
- Findings posted as GitHub issues at https://github.com/efir369999/Montana/issues with the `mainnet-v1.0.0` label, or as plaintext replies to this list — both are read.

The live mesh — Moscow / Frankfurt / Helsinki / Yerevan — has been running on Noise_PQ XX since 2026-05-21 15:54 UTC and is reachable at https://efir.org/explorer/. The install path is `git clone https://github.com/efir369999/Montana.git /opt/montana && sudo bash /opt/montana/Code/scripts/install-vps.sh` on any clean Linux VPS; the new node negotiates `/montana/noise-pq-xx/1.0.0` with each Genesis peer within roughly sixteen minutes of the install.

— Montana protocol team
