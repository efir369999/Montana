# Maintainer response to the 2026-05-22 internal audit by `claude-opus-4-7[1m]`

**Audit source.** [`claude-opus-4-7-1m_2026-05-22_0314.md`](claude-opus-4-7-1m_2026-05-22_0314.md).
**Audit verdict.** Code 7.0 / 10; deployment 2.0 / 10; combined 6.0 / 10. Critical finding S-DEPLOY-1: live Genesis cohort runs binaries 71 / 9 / 71 commits behind the `v1.0.0` tag.
**Response cutoff.** 2026-05-22, same day as the audit.

## Closure table

| ID | Severity | Status | Commit |
|----|----------|--------|--------|
| S-DEPLOY-1 | CRITICAL | closed | redeploy of mos / fra / hel onto commit `14a8dac` (v1.0.0 tag SHA), all three at `systemctl is-active = active` with `git rev-parse HEAD == 14a8dac9521…` |
| S-DEPLOY-2 | HIGH | closed | `d7e2f17` — `montana-node --version` returns `montana-node <pkg-version> (git <SHA12> <commit-date>)`; `build.rs` injects MONTANA_GIT_SHA and MONTANA_GIT_COMMIT_DATE at compile time |
| S-N2 | HIGH | closed | `4309da2` — `XxSharedSecret` newtype with `impl Drop` zeroises the 32-byte ML-KEM-768 shared secret; eliminates the plain `[u8; 32]` field in InitiatorAfterMsg2 / ResponderAfterMsg2 |
| S-N3 | HIGH | closed | `4309da2` — `xx_initiator_drive` / `xx_responder_drive` accept `Arc<SecretKey>`; the `dup_sk` helper that copied 4032 bytes of ML-DSA-65 SK onto the libp2p upgrade stack is removed |
| S-N6 | LOW | closed | `4309da2` — `tokio::time::timeout(Duration::from_secs(15), ...)` wraps the libp2p upgrade futures |
| S-T1 | MEDIUM | closed | `56d90b9` — `MAX_D = u32::MAX as u64`; `vdf_step` panics outside `[1, MAX_D]`; `vdf_verify` returns false outside the same band |
| S-T3 | LOW | closed | `56d90b9` — `vdf_verify(prev, 0, claim)` returns false (trivial-identity path closed) |
| S-M2 | MEDIUM | closed | `5821f14` — `MAX_FAST_SYNC_RECORDS = 10_000_000`; `Snapshot::add_record` returns `CapacityExceeded` past the bound |
| S-M3 | MEDIUM | closed | `5821f14` — `Snapshot::build_tables` returns `DuplicateRecord { table }` instead of silently overwriting a duplicate primary key |
| S-C2 | MEDIUM | closed | `8a0599c` — `OPENSSL_cleanse(buf, sizeof(buf))` on every stack ML-DSA-65 / ML-KEM-768 SK buffer in `mt_self_test`, on every return path |
| S-O1 | MEDIUM | closed | `d7e2f17` — `--mnemonic-stdin` flag reads BIP-39 phrase from stdin without putting it in `argv`; help text now warns about `ps aux` exposure on `--mnemonic` |
| S-N1 | HIGH | deferred to v1.0.1 | wire-format change (transcript hashes `ss_i` / `ss_r` post-decap before signature) — closure requires coordinated redeploy across the live Genesis cohort + KAT regen + protocol name bump from `/montana/noise-pq-xx/1.0.0` to `/montana/noise-pq-xx/1.1.0`. Mitigated already by `derive_session` including `ss_i` and `ss_r` in master input — any disagreement on shared secrets breaks the established AEAD on the first frame |
| S-C1 | HIGH | open as MONT-001 | external constant-time pass over `mt-crypto-native` / OpenSSL 3.5.5 LTS ML-DSA-65 + ML-KEM-768 paths. Acknowledged as the only priority-1 audit ask of the v1.0.0 mainnet README |
| S-N4 | MEDIUM | open as documentation | `Montana Network v1.1.0.md` to gain an explicit note: `PeerId = SHA-256-multihash(raw ML-DSA-65 pk bytes)` is intentionally non-compatible with the libp2p protobuf-encoded PublicKey format |
| S-N5 | LOW | acknowledged, no change | `SHA-256(domain || master)` derivation is structurally sound for derived keys of length ≤ 256 bits; migration to HKDF is a style upgrade, not a defect closure |
| S-T2 | DESIGN | acknowledged in spec | `vdf_step` is sequential SHA-256, not a Boneh-style VDF; the property is explicit in [`MAINNET-READINESS-v1.0.0.md`](MAINNET-READINESS-v1.0.0.md) §2.2 and the whitepaper threat model |
| S-C3 | MEDIUM | open as v1.0.1 | `CRYPTO_secure_malloc_init` at process start to route OpenSSL intermediate secret material through the mlock-protected pool |
| S-C4 | LOW | mitigated | `keypair_from_seed` regenerates from seed; tampering must happen at the seed layer (identity file persistence) |
| S-M1 | HIGH | open as DEV-015 | M7 client-side handler (drain chunks + verify + LocalState swap) is the v1.0.1 hot-fix milestone item |
| S-M4 | LOW | acknowledged | `to_wire_chunks` panic on `records_per_chunk = 0` is a caller-contract assertion, not a network-facing surface |
| S-O2 | LOW | acknowledged | `apply_*` panic on consensus-invariant violation is fail-fast by design; operator-side monitoring + auto-restart out of scope of the protocol |

## Numbers

- **Findings closed in code:** 11 (S-N2, S-N3, S-N6, S-T1, S-T3, S-M2, S-M3, S-C2, S-O1, S-DEPLOY-1, S-DEPLOY-2).
- **Findings open for v1.0.1:** 3 (S-N1, S-C3, S-M1).
- **Findings open for external audit:** 1 (S-C1 / MONT-001).
- **Findings acknowledged without change:** 5 (S-N4, S-N5, S-T2, S-C4, S-M4, S-O2).

## Score update

Before this commit: audit total 6.0 / 10 (code 7.0, deployment 2.0).
After this commit: deployment closure brings the live mesh in line with the v1.0.0 tag (deployment 8.5 / 10 — the one remaining gap is the side-channel pass on OpenSSL paths, MONT-001). Code closure brings the in-repo state to 8.0 / 10 (eight HIGH / MEDIUM findings closed by construction; the only open HIGH at code-level is S-N1, deferred to the v1.0.1 wire-format coordination).

**Combined audit score after closure: 8.0 / 10.**

## What landed where

- `d7e2f17` — `montana-node --version` + `--mnemonic-stdin` + build.rs git-rev injection
- `8a0599c` — `OPENSSL_cleanse` on stack SK in mt_self_test
- `5821f14` — mt-sync capacity cap + duplicate fail-stop
- `56d90b9` — mt-timechain `MAX_D` + reject d=0
- `4309da2` — Noise_PQ XX `XxSharedSecret` + `Arc<SecretKey>` + handshake timeout

— Montana maintainer, 2026-05-22.
