# Montana Reference Implementation — Audit Package

**Spec target:** Montana Protocol v35.25.1 + Montana Network v1.1.0 + Montana App v3.12.0 — см. [VERSION.md](VERSION.md)
**Last verified:** 2026-05-20 (CISO-as-a-Service 2026-05-19 response sync; MONT-001 spec patch; MONT-002 online IBT nonce + replay tracking in mt-net / mt-net-transport)
**Audit-ready status:** **M1 + M2 + M3 + M4 + M5 + M6 + M9 layers — READY FOR EXTERNAL AUDIT**. M8 node binary remains pre-mainnet/in progress; DEV-012 multi-node proposal apply is the current mainnet blocker.

---

## TL;DR — что готово к внешнему аудиту

| Layer | Готов? | Крейты | LOC | Tests + invariants | Open findings |
|-------|--------|--------|-----|---------------------|---------------|
| **M1 foundational** (cryptography + identity recovery) | ✅ **READY** | mt-codec, mt-crypto, mt-crypto-native, mt-mnemonic | ~2000 | 100+ unit + 13 security + 51 NIST KAT byte-exact | 0 (12/12 closed) |
| **M2 state foundation** (consensus state primitives) | ✅ **READY** | mt-merkle, mt-genesis, mt-state, mt-timechain | 1821 | 95+ unit + 60 determinism invariants | 0 (4/4 closed) |
| **M3 apply_proposal layer** (account operations + emission + state transition) | ✅ **READY** | mt-account | 2556 | 89 unit + 29 determinism invariants | 0 (3/3 internal closed: M3-1/M3-2/M3-3 + 10/10 external #3+VERIFIED closed: M3-A-1..M3-A-5 / F-5 / M3-A-3 / AUDIT-sync / P-C-1 F-19 reopen / P-C-2 history; M2-3/M2-13 superseded v34 monetary refactor) |
| **M4 consensus mechanics** (lottery + proposal acceptance + node admission) | ✅ **READY** | mt-lottery, mt-consensus, mt-entry | 3858 | 187 unit + 85 determinism invariants | 0 (1/1 internal closed: M4-1; 7/7 external #4 closed: M4-MED-1/2 + M4-LOW-3..7 + M4-INFO-10) |
| **M5 persistence** (filesystem state + proposal archive + crash recovery) | ✅ **READY** | mt-store | 955 | 27 unit + 17 determinism invariants | 0 (manual scan clean; v34 monetary refactor удалила MonetaryState persistence) |
| **Cross-implementation conformance** | ✅ **READY** | Domain registry sync (spec ↔ code, см. VERSION.md) | — | NIST ACVP 66 byte-exact (KeyGen 50 + SigGen 15 + ctx-equivalence 1) + Recovery flow | 0 (F-1 spec patch closed) |
| **M6 network layer** (wire format + transport + IBT + Dandelion + mesh + SF) | ✅ **READY** | mt-net, mt-net-transport | ~3300 | 127 tests: mt-net 112 + mt-net-transport 15 (включая 3 e2e two-node) | 0 (P-C1..P-C8 + MONT-002 nonce replay closure) |
| **M9 conformance suite** | ✅ **READY** | mt-conformance | ~150 | 2 unit byte-exact verify | 0 (envelope A1-A3 + PoW F1-F2 + IBT B1) |
| **M7 Fast Sync** | ⏳ TODO | mt-sync | — | — | (не реализовано) |
| **M8 Node binary** (montana-node production multi-node path) | 🔄 in progress | montana-node | ~600 | partial | DEV-012 open; DEV-013 closed |

**Audit firm engagement:** возможен прямо сейчас на полный scope **M1 + M2 + M3 + M4 + M5 + M6 + M9**. M7 Fast Sync + M8 production multi-node node binary — defer to отдельной audit фазы.

**iOS application audit:** см. отдельный package `iOS/Apps/Montana/AUDIT.md` — Phase 2 in progress, требует Phase 2.1+ implementation (4-6 недель) перед external firm engagement.

---



---

## TL;DR — M6 Phase C closure (2026-05-02)

**mt-net-transport** (~470 LOC) — libp2p-based transport layer:
- `src/codec.rs` — MontanaCodec для libp2p request-response с MAX_PROTOCOL_PAYLOAD_BYTES enforcement (Genesis Decree authoritative bound)
- `src/behaviour.rs` — MontanaBehaviour wrapper (request-response для FastSync/PeerList/BatchLookup/RangeSubscribe; one-way gossip — Phase C.5+)
- `src/transport.rs` — build_swarm() helper с TCP→TLS 1.3 (rustls)→Noise→Yamux upgrade chain
- `src/ibt_upgrade.rs` — classify_proof() для access level determination (Node/Candidate/Account) with online_session_nonce + used_online_nonces replay tracking
- `tests/e2e_two_node_handshake.rs` — Manual Validation Gate scenario 6 PASS (Ping/Pong через full transport chain)
- `tests/e2e_proposal_exchange.rs` — scenario 7 PASS (synthetic Proposal payload + 512 KiB boundary test)

**mt-conformance** (~150 LOC) — M9 standalone test vectors crate для cross-implementation verification:
- VectorEnvelope (A1/A2/A3 byte-exact)
- VectorIbtSeed (B1 после P-C2 rename mt-tunnel→mt-tunnel-online; Network v1.1.0 adds online_session_nonce)
- VectorPow (F1/F2 target derivation)
- Public API: `all_envelope_vectors()`, `all_pow_vectors()`, `ibt_b1_online_proof()`

**Capability checklist [C-5] для libp2p 0.56.0:** 8/8 PASS (TCP+TLS 1.3+Noise+Yamux+Swarm primitives; async tokio; rustls + snow constant-time; Linux+macOS+Windows; IPFS+Filecoin+Polkadot 5+ years production; MIT/Apache 2.0; ~120 transitive deps acceptable за изоляцию через own crate).

---

## 1. Audit Chain

Hybrid Rust + C architecture для cryptography (M1). Pure Rust для state foundation (M2). Three layers, each auditable independently.

### M1 — Foundational cryptography + identity recovery

#### Layer 1 — Rust shim (own audit responsibility)

| Crate | Path | Lines (exact) | Scope |
|-------|------|---------------|-------|
| `mt-codec` | [crates/mt-codec/src/lib.rs](crates/mt-codec/src/lib.rs) | ~290 | Canonical encoding traits + Domain separators registry (32 domains, sync с актуальным spec target из VERSION.md byte-exact). Все consensus hash compositions через explicit domain separator + NUL byte (`SHA-256(domain ‖ 0x00 ‖ parts)` self-delimiting per P1 external finding). |
| `mt-crypto` | [crates/mt-crypto/src/lib.rs](crates/mt-crypto/src/lib.rs) | **662** | Public API: `PublicKey`, `SecretKey`, `Signature`, `Mlkem*`, `keypair_from_seed`, `keypair_from_seed_mlkem`, `sign`, `verify`, `hash`, `sha256_raw`. `CryptoError` enum + `Result<_, CryptoError>` API. **Heap-allocated SK через `Box<[u8; N]>` + `mlock` против swap-out + `Drop+zeroize`** для secret types. Все **7** `unsafe` blocks с `// SAFETY:` комментариями: 4 FFI sites (`keypair_from_seed`, `sign`, `verify`, `keypair_from_seed_mlkem`) + 3 mlock/munlock (`Drop for SecretKey`, `alloc_locked_secret_box`, `Drop for MlkemSecretKey`). Test-only `keypair()` через OS CSPRNG (`getrandom`). |
| `mt-crypto-native` (Rust binding) | [crates/mt-crypto-native/src/lib.rs](crates/mt-crypto-native/src/lib.rs) | 49 | `extern "C"` декларации FFI к Layer 2. Только `pub const` константы и FFI signatures, никакой логики. Включает `mt_sign_mldsa_ctx` для FIPS context support. |
| `mt-mnemonic` | [crates/mt-mnemonic/src](crates/mt-mnemonic/src) | 937 | 24-word mnemonic recovery flow: PBKDF2-HMAC-SHA-256 (iter=2²⁰), HKDF-Expand per-role derivation, ML-DSA seed (32B) + ML-KEM seed (64B) generation. Wordlist binding SHA-256 verified. |

#### Layer 2 — Own thin C wrapper (own audit responsibility)

| File | Lines (exact) | Scope |
|------|---------------|-------|
| [crates/mt-crypto-native/csrc/mt_crypto.c](crates/mt-crypto-native/csrc/mt_crypto.c) | 457 | Wrapping OpenSSL EVP_PKEY API: `mt_keypair_from_seed_mldsa`, `mt_keypair_from_seed_mlkem`, `mt_sign_mldsa`, `mt_sign_mldsa_ctx` (FIPS context support), `mt_verify_mldsa`, `mt_self_test`. Использует `OSSL_PKEY_PARAM_ML_DSA_SEED` (FIPS 204 §3.1 ξ ∈ B³²) и `OSSL_PKEY_PARAM_ML_KEM_SEED` (FIPS 203 §6.1 d ‖ z, 64B) для deterministic KeyGen. `OSSL_SIGNATURE_PARAM_DETERMINISTIC=1` для FIPS 204 Algorithm 2 deterministic Sign. |
| [crates/mt-crypto-native/csrc/mt_crypto.h](crates/mt-crypto-native/csrc/mt_crypto.h) | 67 | C API declarations + размеры primitives + 13 status codes (1 success + 12 errors). |
| [crates/mt-crypto-native/build.rs](crates/mt-crypto-native/build.rs) | 45 | Vendored OpenSSL build через `openssl-src`, `cc::Build` с `-Wall -Wextra -Wpedantic -Werror`, cross-compile корректность через `CARGO_CFG_TARGET_OS`. |

**Total own audit surface (Layer 1 + Layer 2): 1280 lines** (662 Rust shim + 49 FFI bindings + 457 C wrapper + 67 C header + 45 build script). Verify counts:
```
cd "<repo-root>" && wc -l crates/mt-crypto/src/lib.rs crates/mt-crypto-native/src/lib.rs crates/mt-crypto-native/csrc/mt_crypto.c crates/mt-crypto-native/csrc/mt_crypto.h crates/mt-crypto-native/build.rs
```

#### Layer 3 — Underlying production C library (vendor audit responsibility)

| Component | Version | Source | Audit history |
|-----------|---------|--------|---------------|
| OpenSSL | 3.5.5 LTS | [openssl-src 300.5.5+3.5.5](https://crates.io/crates/openssl-src) — vendored, byte-pinned exact version | OpenSSL Foundation governance, FIPS 140-3 validated, decades production deployment в TLS world (Apache HTTP, nginx, OpenSSH, Linux kernel, …), supported до **April 2030** (LTS) |

**Layer 3 НЕ в нашем audit scope** — auditor проверяет только наш способ использования OpenSSL EVP API (Layer 2), не реализацию самих ML-DSA/ML-KEM/SHA-256.

### M2 — State foundation (audit-ready 2026-04-26)

| Crate | Path | Lines | Scope | Audit findings |
|-------|------|-------|-------|----------------|
| `mt-merkle` | [crates/mt-merkle/src/lib.rs](crates/mt-merkle/src/lib.rs) | 474 | Sparse Merkle Tree (depth 256), `empty_internal()` precomputed cache (OnceLock), `leaf_hash`/`internal_hash` (SHA-256 domain-separated через mt-codec::domain), `SparseMerkleTree::insert/root` через `BTreeMap` для canonical iteration order, `verify_proof` для inclusion + absence proofs. **0 unsafe blocks. 0 panic!. 0 f32/f64. 0 SystemTime/RNG. BTreeMap не HashMap.** | Pass 1-12 clean (manual scan); 10 automated determinism invariants |
| `mt-genesis` | [crates/mt-genesis/src/lib.rs](crates/mt-genesis/src/lib.rs) | **353** | Genesis Decree + `ProtocolParams` SSOT (4094B encoded), `genesis_app_id()` (SHA-256 domain-separated), `genesis_params()` через `OnceLock` (singleton, thread-safe), `compute_genesis_state_hash()`. Const `emission_moneta = 13 × 10⁹ nɈ` per spec v34+. **0 unsafe. 0 panic. Только read-only constants + deterministic hash.** | Pass 1-12 clean; automated determinism invariants |
| `mt-state` | [crates/mt-state/src/lib.rs](crates/mt-state/src/lib.rs) | **647** | AccountTable (2059B records) / NodeTable (2098B) / CandidatePool (2082B) через `BTreeMap<id, Record>` + `SparseMerkleTree`, `derive_account_id` / `derive_node_id` (SHA-256 domain-separated), `compute_state_root` (SHA-256 of node_root ‖ candidate_root ‖ account_root), `is_active` predicate. **0 unsafe. 0 panic! 0 HashMap (BTreeMap canonical sort). 0 f64.** | Pass 1-12 clean; automated determinism invariants |
| `mt-timechain` | [crates/mt-timechain/src/lib.rs](crates/mt-timechain/src/lib.rs) | 347 | TimeChain VDF (`vdf_step` = SHA-256^d, `vdf_verify` re-computes), `next_d` Adaptive D через participation-ratio feedback (integer permille per [I-9]), `cemented_bundle_aggregate(W, node_ids)` per [I-8] Network-Bound Unpredictability (3 ветви: genesis 0×32, empty marker, sorted node_ids hash). **0 unsafe. 0 panic. 0 HashMap. 0 f64. Все integer арифметика per [I-9].** | Pass 1-12 clean; 19 automated determinism invariants |

**Total M2 audit surface:** 1821 lines code + 60 automated determinism invariants.

### M3 — apply_proposal layer (audit-ready 2026-04-27)

| Crate | Path | Lines | Scope | Audit findings |
|-------|------|-------|-------|----------------|
| `mt-account` | [crates/mt-account/src/lib.rs](crates/mt-account/src/lib.rs) | **2556** | 4 user opcodes (`Transfer 0x02` / `ChangeKey 0x03` / `Anchor 0x04` / `TransferActivation 0x0A`) с byte-exact canonical encoding (TRANSFER_SIZE / CHANGE_KEY_SIZE / ANCHOR_SIZE / TRANSFER_ACTIVATION_SIZE). `validate_*` для каждого opcode (full validation per spec table). `apply_*` с **checked arithmetic** (`checked_sub`/`checked_add` + descriptive panic для protocol invariant breach). `op_hash` через R2 SHA-256(`mt-op` ‖ signed_scope), signature excluded. `settle_window(cemented_ops)` сортирует по `op_hash` lex asc. `apply_proposal` orchestrates Steps 2/3.5/3.6/4 (steps 1, 3a, 3b stubbed → M4 mt-entry). `apply_emission` зачисляет `EMISSION_moneta` (const) operator-у winner-узла. `reward_moneta(params) = params.emission_moneta`, `supply_moneta(W) = emission × (W+1)` closed-form. `build_genesis_state` + `genesis_state_root` для bootstrap. **0 unsafe. 0 panic! без `protocol invariant` justification. 0 HashMap. 0 f64. 0 SystemTime.** | Pass 1-12 clean; automated determinism invariants |

**Total M3 audit surface:** 2556 lines code + 29 automated determinism invariants.

### M4 — Consensus mechanics (audit-ready 2026-04-27)

| Crate | Path | Lines | Scope | Audit findings |
|-------|------|-------|-------|----------------|
| `mt-lottery` | [crates/mt-lottery/src/lib.rs](crates/mt-lottery/src/lib.rs) | **1715** | BundledConfirmation R1/R2 (signed_scope + bundle_hash через `mt-bundle` domain), VdfReveal R1/R2 (`mt-vdf-reveal` domain), `compute_endpoint` lottery formula (`mt-lottery` domain) с [I-8] cemented_bundle_aggregate(W-2) binding, validate_bundle/validate_reveal с full structural + signature verification, `log2_q64` (Q64.64 fixed-point integer log с degree-3 Remez minimax polynomial, max error 2^-10.62, binding coefficients B0..B3), `ln_q64`, `weighted_ticket_node` u128 integer division, `determine_winner` argmin canonical rule (ticket asc, class asc, id lex asc), `quorum` 67% ceiling integer formula `(67×X+99)/100`, `is_cemented`. **0 unsafe. 0 panic!. 2 controlled `expect()` (try_into на slice длины 16 — protocol invariant). 0 HashMap. 0 f64. 0 SystemTime.** | M4-1 closure (Vec.len() as u16 silent truncation → BundleError::TooManyOps/TooManyReveals + validate cap + debug_assert defense-in-depth); 34 automated determinism invariants |
| `mt-consensus` | [crates/mt-consensus/src/lib.rs](crates/mt-consensus/src/lib.rs) | **1089** | ProposalHeader R1/R2 (3722 B fixed-size, 17 полей, signed_scope без signature, `mt-proposal` domain), validate_header (a-f структурные правила: fallback_depth ≥ 1, window_index = prev+1, protocol_version monotone + ≤ local_max, proposer registered + Mldsa65 suite, signature verify), canonical_proposer / fallback_proposer (Lookback Leadership: proposer_W = winner_{W-2}; genesis bootstrap; cascade by depth), compute_control_set (filter cemented_window > prev AND ≤ W, sort by (window asc, op_hash lex asc)), validate_proposer_is_canonical / validate_bundles_threshold / validate_included_reveals / validate_winner (Canonical acceptance), finalization_status (Cemented/Rejected per is_cemented), leader_penalty_excluded_node. **0 unsafe. 0 panic. 0 prod unwrap/expect. 0 HashMap. 0 f64. 0 SystemTime.** | Pass 1-12 clean (manual scan); 27 automated determinism invariants |
| `mt-entry` | [crates/mt-entry/src/lib.rs](crates/mt-entry/src/lib.rs) | **1054** | NodeRegistration R1/R2 (5344 B, opcode 0x11, `mt-nodereg` domain), validate_noderegistration (3 структурные правила: suite supported, signature verify, node_id unique в Node Table + Candidate Pool, operator account exists + не is_node_operator), `candidate_vdf_init` per [I-8] (`mt-candidate-vdf-init` domain composing T_r + cba(W-2) + node_id), `compute_expiry_window` 3τ₂, `apply_candidate_expiry` (apply_proposal Step 3a), selection_slots (1% cap через ADMISSION_DIVISOR=130), `selection_sort_key` (`mt-selection` domain), rank_candidates_for_selection (canonical sort), `apply_selection_event` (apply_proposal Step 3b: insert into Node Table + mark operator + remove from Candidate Pool), `required_vdf_length` (Adaptive VDF integer permille per [I-9]), `nr_sort_key` (`mt-nodereg-sort` domain), `apply_noderegistrations_batch` (apply_proposal Step 1: incremental sort+apply с pending growth). **0 unsafe. 0 panic. 0 prod unwrap/expect. 0 HashMap. 0 f64.** | Pass 1-12 clean; 24 automated determinism invariants |

**Total M4 audit surface:** 3858 lines code + 85 automated determinism invariants. Domain separators: `mt-bundle`, `mt-vdf-reveal`, `mt-lottery`, `mt-proposal`, `mt-nodereg`, `mt-candidate-vdf-init`, `mt-selection`, `mt-nodereg-sort` — все consensus-critical hash compositions через canonical domains (per [I-8] + [I-10] SSOT).

### M5 — Persistence (audit-ready 2026-04-27)

| Crate | Path | Lines | Scope | Audit findings |
|-------|------|-------|-------|----------------|
| `mt-store` | [crates/mt-store/src/lib.rs](crates/mt-store/src/lib.rs) | **955** | Filesystem-backed state persistence (pure std::fs, без RocksDB/sled — minimum deps): `FsStore::open` (создаёт `proposals/` subdirectory), save/load AccountTable / NodeTable / CandidatePool через canonical_encode/decode pair (round-trip byte-exact verified), Proposal archive (`proposals/{window:020}.bin`, `archive_proposal` + `get_proposal_by_window` byte-exact decode), Crash recovery (`meta_last_cemented.bin` u64 LE; `verify_consistency` проверяет что meta-указанный proposal существует в archive — иначе StoreError::NotFound), Pruning (`prune_proposals_before` удаляет файлы window < threshold). Все decode_X функции проверяют `bytes.len() != EXPECTED_SIZE` ДО чтения (StoreError::CorruptedLength). **0 unsafe. 0 panic. 0 prod unwrap/expect. 0 HashMap. 0 f64.** | Pass 1-12 clean; automated determinism invariants |

**Total M5 audit surface:** 955 lines code + 17 automated determinism invariants.

**M3 + M4 + M5 orchestration ordering** (важно для каллера):
1. M3: `settle_window(account_table, cemented_user_ops, window_w)` — apply cemented user ops к state
2. M4: `apply_noderegistrations_batch` (Step 1) → `apply_candidate_expiry` (Step 3a) → `apply_selection_event` (Step 3b) — node admission flow
3. M3: `apply_proposal(account_table, node_table, candidate_pool, input, params)` — Steps 2 (emission) → 3.5 (chain_length++) → 3.6 (checkpoint rotation) → 4 (state_root)
4. M5: `archive_proposal(header)` → `save_meta_last_cemented(window)` → `save_account_table` / `save_node_table` / `save_candidate_pool` — persist на disk

Settle разделено от apply_proposal (M3 design choice); steps 1/3a/3b делегированы M4 mt-entry. M5 finalization атомарность через meta-pointer pattern (если crash между archive и meta — verify_consistency detects).

**M3 apply_proposal orchestration ordering** (важный invariant для каллера):
1. `settle_window(account_table, cemented_user_ops, window_w)` — apply cemented ops к state (Transfer/Anchor/ChangeKey/TransferActivation) **ДО** apply_proposal
2. `apply_proposal(account_table, node_table, candidate_pool, input, params)` — Steps 2 (emission) → 3.5 (chain_length++) → 3.6 (checkpoint rotation) → 4 (state_root)

Settle разделено от apply_proposal по design — orchestration ordering invariant виден каллеру explicitly, скрытие в одной функции рискует silent fork. Документировано в module-level comment перед `pub fn apply_proposal`.

---

## 2. Conformance Proofs

### NIST FIPS 204/203 byte-exact conformance (M1)

**66 differential test cases** vs NIST CAVP ACVP-Server published vectors:

| Test | Source | Cases | Status |
|------|--------|-------|--------|
| ML-DSA-65 KeyGen byte-exact | [NIST ACVP-Server](https://github.com/usnistgov/ACVP-Server) `gen-val/json-files/ML-DSA-keyGen-FIPS204` | 25 | ✅ PASS |
| ML-KEM-768 KeyGen byte-exact | [NIST ACVP-Server](https://github.com/usnistgov/ACVP-Server) `gen-val/json-files/ML-KEM-keyGen-FIPS203` | 25 | ✅ PASS |
| ML-DSA-65 SigGen deterministic external pure (1 empty + 14 non-empty context, 0..255B) | [NIST ACVP-Server](https://github.com/usnistgov/ACVP-Server) `gen-val/json-files/ML-DSA-sigGen-FIPS204` tgId=3 | **15** | ✅ PASS |
| `mt_sign_mldsa` ≡ `mt_sign_mldsa_ctx` (empty ctx) equivalence | own correctness check | 1 | ✅ PASS |

**Reproduction:**
```
cd "<repo-root>" && cargo test -p mt-crypto-native --test nist_acvp_kat -- --nocapture
```

**Fixtures location:** [crates/mt-crypto-native/tests/fixtures/nist_acvp/](crates/mt-crypto-native/tests/fixtures/nist_acvp/)

### SHA-256 NIST FIPS 180-4 conformance (M1)

| Test | Source | Status |
|------|--------|--------|
| SHA-256("abc") == ba7816bf...15ad | FIPS 180-4 §B.1 | ✅ PASS (in `mt-crypto/src/lib.rs::tests::hash_nist_vector_abc`) |

### Internal correctness baselines (M1)

| Test | Path | Status |
|------|------|--------|
| ML-DSA-65 zero-seed deterministic baseline | [crates/mt-crypto-native/tests/regression_baselines.rs](crates/mt-crypto-native/tests/regression_baselines.rs) | ✅ PASS |
| ML-KEM-768 zero-seed + ones-seed deterministic baselines | same | ✅ PASS |
| ML-DSA-65 sign roundtrip + tamper detection | same | ✅ PASS |
| 5 mt-mnemonic KAT vectors (entropy → mnemonic → master_seed → per-role keypair) | [crates/mt-mnemonic/tests/keygen_vectors.rs](crates/mt-mnemonic/tests/keygen_vectors.rs) | ✅ PASS |
| End-to-end recovery flow (entropy → identity round-trip) | [crates/mt-mnemonic/tests/e2e_recovery.rs](crates/mt-mnemonic/tests/e2e_recovery.rs) | ✅ PASS |
| `mt_crypto::self_test()` (KeyGen determinism + Sign/Verify + KAT 1 byte-exact conformance check) | [crates/mt-crypto/src/lib.rs](crates/mt-crypto/src/lib.rs) | ✅ PASS |

### Security invariants (M1, Pass 17 enforcement через automated tests)

Regression detection для security-critical properties — если будущий рефакторинг ломает invariant, тест fails в CI ДО merge.

| Invariant | Test | Status |
|-----------|------|--------|
| `SecretKey: !Clone` (no accidental copies) | [crates/mt-crypto/tests/security_invariants.rs](crates/mt-crypto/tests/security_invariants.rs)::secret_key_is_not_clone | ✅ |
| `MlkemSecretKey: !Clone` | same::mlkem_secret_key_is_not_clone | ✅ |
| `SecretKey: !PartialEq` (no timing leak via ==) | same::secret_key_no_partial_eq_to_prevent_timing_leak | ✅ |
| `MlkemSecretKey: !PartialEq` | same::mlkem_secret_key_no_partial_eq_to_prevent_timing_leak | ✅ |
| `SecretKey` heap-allocated (size = pointer) | same::secret_key_is_heap_allocated | ✅ |
| `MlkemSecretKey` heap-allocated | same::mlkem_secret_key_is_heap_allocated | ✅ |
| `SecretKey` имеет Drop impl | same::secret_key_needs_drop | ✅ |
| `MlkemSecretKey` имеет Drop impl | same::mlkem_secret_key_needs_drop | ✅ |
| FFI fills SK с non-zero bytes | same::secret_key_filled_by_ffi_keygen | ✅ |
| FFI fills MlkemSK с non-zero bytes | same::mlkem_secret_key_filled_by_ffi_keygen | ✅ |
| No `println!`/`log::*` macros на SK bytes в lib коде | same::no_println_or_log_on_secret_bytes_in_lib_code | ✅ |
| `PublicKey: Clone` (sanity — public material clone-able) | same::public_key_can_be_cloned | ✅ |
| `Signature: Clone` (sanity) | same::signature_can_be_cloned | ✅ |

**13/13 security invariants PASS.**

### Determinism invariants (M2 — все 4 crates)

Automated regression detection per [I-3] determinism + [I-9] integer arithmetic + [I-10] SSOT + [I-8] Network-Bound Unpredictability.

| Crate | Test file | Cases | Status |
|-------|-----------|-------|--------|
| `mt-merkle` | [tests/determinism_invariants.rs](crates/mt-merkle/tests/determinism_invariants.rs) | 10 | ✅ PASS (empty_internal determinism + level uniqueness, leaf/internal hash determinism, internal_hash order-sensitivity, SMT empty root + insertion-order-independent root, root-changes-on-insert, type sizes) |
| `mt-genesis` | [tests/determinism_invariants.rs](crates/mt-genesis/tests/determinism_invariants.rs) | 7 | ✅ PASS (genesis_app_id determinism, genesis_params singleton stable pointer, ProtocolParams encoded size constant + encoding determinism, compute_genesis_state_hash determinism + dependence on state_root, SSOT singleton consistency) |
| `mt-state` | [tests/determinism_invariants.rs](crates/mt-state/tests/determinism_invariants.rs) | — | ✅ PASS (derive_account_id / derive_node_id determinism + dependence + cross-distinctness, ACCOUNT/NODE/CANDIDATE_RECORD_SIZE matches encoded, encoding determinism, AccountTable/NodeTable/CandidatePool insertion-order-independent root, compute_state_root determinism + order-sensitivity + dependence на каждый input root, is_active boundary at 2×τ₂ inclusive + saturating_sub safety, WINNER_CLASS_NODE SSOT, empty tables consistent root, insert/remove inverse) |
| `mt-timechain` | [tests/determinism_invariants.rs](crates/mt-timechain/tests/determinism_invariants.rs) | 19 | ✅ PASS (vdf_step zero=identity + determinism + dependence на iterations и prev, vdf_verify accept/reject correct/wrong claim/iteration count, vdf_chain composition associative, next_d dead-zone unchanged + above-high-increases + below-low-decreases + determinism, cemented_bundle_aggregate window<2 = genesis zero, empty marker distinct from non-empty, change-on-window/node_ids, **canonical sort input-order independence per [I-8]**) |

**60/60 M2 invariants PASS.**

### Spec ↔ Code byte-exact alignment (M0+M1+M2)

Verified per critic audit (commit `9387900`):

| Проверка | Spec value | Code value | Status |
|----------|-----------|-----------|--------|
| ML-DSA-65 pubkey size | 1952B | `PUBLIC_KEY_SIZE = 1952` | ✅ |
| ML-DSA-65 secretkey | 4032B | `SECRET_KEY_SIZE = 4032` | ✅ |
| ML-DSA-65 signature | 3309B | `SIGNATURE_SIZE = 3309` | ✅ |
| ML-DSA-65 seed | 32B | `KEYPAIR_SEED_SIZE = 32` | ✅ |
| ML-KEM-768 ek | 1184B | `MLKEM_PUBLIC_KEY_SIZE = 1184` | ✅ |
| ML-KEM-768 dk | 2400B | `MLKEM_SECRET_KEY_SIZE = 2400` | ✅ |
| ML-KEM-768 seed | 64B (d‖z) | `MLKEM_SEED_SIZE = 64` | ✅ |
| AccountRecord size | 2059B | `ACCOUNT_RECORD_SIZE = 2059` | ✅ |
| NodeRecord size | 2098B | `NODE_RECORD_SIZE = 2098` | ✅ |
| CandidateRecord size | 2082B | `CANDIDATE_RECORD_SIZE = 2082` | ✅ |
| ProposalHeader size | 3722B | `PROPOSAL_HEADER_SIZE = 3722` | ✅ |
| ProtocolParams size | 4094B (spec v34) | `PARAMS_ENCODED_SIZE = 4094` | ✅ |
| Sparse Merkle Tree depth | 256 | `TREE_DEPTH = 256` | ✅ |
| Emission pin | 13 Ɉ const per окно | `emission_moneta = 13_000_000_000` | ✅ |
| Domain registry sync (32 domains) | spec list | code mt-codec const list | ✅ |

### Security Cards per crypto primitive (M1)

Каждый primitive с secret material имеет detailed Security Card в [docs/security-cards.md](docs/security-cards.md). Cards covered:

| Primitive | Card | Status |
|-----------|------|--------|
| `SecretKey` (ML-DSA-65) | Card 1 — 8/8 Pass 17 checks closed | ✅ closed |
| `MlkemSecretKey` (ML-KEM-768) | Card 2 — 8/8 closed | ✅ closed |
| `keypair_from_seed` (ML-DSA KeyGen) | Card 3 — heap+mlock+stack hygiene closed | ✅ closed |
| `keypair_from_seed_mlkem` (ML-KEM KeyGen) | Card 4 — heap+mlock+stack hygiene closed | ✅ closed |
| `sign` (ML-DSA-65 deterministic) | Card 5 — 8/8 closed | ✅ closed |
| `verify` (ML-DSA-65) | Card 6 — no secret material, minimal card | ✅ closed |

---

## 3. Threat Model & Scope Boundaries

### In scope — что аудитор должен проверить

**M1 — cryptography + identity:**
1. **FFI memory safety** — Layer 1 `unsafe` blocks (4 sites), buffer sizes, pointer validity
2. **OpenSSL EVP API misuse** — параметры EVP_PKEY_CTX_set_params, error handling, EVP_DigestSignInit determinism flag
3. **Secret hygiene** — `Drop+zeroize` для `SecretKey`/`MlkemSecretKey`, no `Clone`/`Copy` на secret types, no leak в logs/dumps, **heap-allocated bytes (Box) с mlock** против swap-out, **stack hygiene** — FFI пишет напрямую в heap-allocated locked Box, никаких stack temporary buffers с secret bytes (см. [docs/security-cards.md](docs/security-cards.md) Cards 1-6)
4. **Result API correctness** — `sign`/`keypair_from_seed` corruption resistance: malformed SK через `from_array(arbitrary_bytes)` → `Err(CryptoError::InvalidSecretKey)`, не panic
5. **Deterministic semantics** — FIPS 204 Algorithm 2 deterministic Sign correctly invoked via `OSSL_SIGNATURE_PARAM_DETERMINISTIC=1` (consensus determinism per Montana [I-3])
6. **Recovery flow correctness** — mnemonic → master_seed (PBKDF2-HMAC-SHA-256) → per-role HKDF derivation → byte-exact reproducible identity (Montana spec sections "Ключи → Мнемоника и seed", "Per-role key derivation")
7. **Cross-compile correctness** — build.rs использует `CARGO_CFG_TARGET_OS` env var (не `cfg!(target_os)`)
8. **Reproducible builds** — Docker container с pinned base image (`debian:bookworm-slim@sha256:40b107342c492725bc7aacbe93a49945445191ae364184a6d24fedb28172f6f7`), byte-identical между независимыми runs (CI gate `reproducible_release`)

**M2 — state foundation:**
9. **Sparse Merkle Tree correctness** — empty_internal level uniqueness + cache consistency + verify_proof для inclusion + absence proofs
10. **Genesis Decree SSOT** — ProtocolParams singleton consistency, encoded size invariance под struct changes, frozen constants
11. **State table determinism** — BTreeMap canonical sort guarantee для AccountTable / NodeTable / CandidatePool, insertion-order-independent state_root
12. **Account/Node ID derivation** — SHA-256 domain-separated через mt-codec::ACCOUNT / ::NODE registry, cross-distinctness same pubkey
13. **Monetary policy correctness** — const emission `reward_moneta = EMISSION_moneta = 13 × 10⁹ nɈ` per spec «Эмиссия», closed-form `supply_moneta(W) = EMISSION_moneta × (W + 1)`
14. **TimeChain VDF correctness** — vdf_step composition associativity, vdf_verify byte-exact against re-computed chain
15. **Adaptive D feedback** — next_d dead-zone semantics + integer permille arithmetic per [I-9]
16. **cemented_bundle_aggregate per [I-8]** — Network-Bound Unpredictability через canonical sort node_ids + window binding, защита от input-order grinding

### Out of scope — НЕ предмет этого аудита

1. **OpenSSL internal correctness** — вне scope; ответственность OpenSSL Foundation. Аудитор может предположить FIPS 204/203 Algorithm 1/2/16 implementations OpenSSL корректны (audited+deployed at scale)
2. **Consensus protocol correctness M3-M5** — отдельные layers (mt-account, mt-consensus, mt-lottery, mt-entry, mt-store) — internal-tested 255+ tests green, но не audit-prepared (TODO для следующих audit phases)
3. **Network protocol** — M6+ (mt-net не существует)
4. **Application layer** — отдельный workspace в будущем (Juno agent, messaging, файловое хранилище)
5. **Hardware side-channel attacks** — software-only assumption; embedded deployment audit отдельный

### Known limitations (deferred с обоснованием)

0. **Genesis bootstrap ceremony pending** (M2-1 finding из external audit Claude Opus 4.7 #2) — 4 поля `ProtocolParams` хранят placeholder zeros и финализируются только при Genesis ceremony перед mainnet:
   - `bootstrap_account_pubkey: [0u8; PUBLIC_KEY_SIZE]`
   - `bootstrap_node_pubkey: [0u8; PUBLIC_KEY_SIZE]`
   - `target_zero: [0u8; 32]` (initial VDF target)
   - `genesis_content_data_hash: [0u8; 32]`

   **Severity:** блокер mainnet, **НЕ блокер аудита кода**. Layout / encoding / SSOT / singleton корректны и audit-ready. Финализация = выбор seed, генерация keypairs через `mt_crypto::keypair_from_seed`, безопасное хранение privkey (multi-party ceremony либо single trusted party — design decision автора).

   **Programmatic check:** `mt_genesis::is_genesis_bootstrap_finalized(params) -> bool` для operator deployment scripts. Test `bootstrap_keypairs_finalized` остаётся `#[ignore]` до ceremony, после ceremony — снимается ignore. Pre-ceremony test `is_genesis_bootstrap_finalized_pre_ceremony_returns_false` явно фиксирует текущее состояние.

   **Closure path:** Genesis ceremony plan — отдельный milestone в `ROADMAP.md` перед mainnet.

1. **Sign with non-empty context** — текущий `mt_sign_mldsa` API не принимает context parameter, использует empty context (соответствует Montana usage pattern, но не FIPS 204 full API surface). NIST ACVP cases с context bytes (14/15 в external pure deterministic group) не проверяются. Closure path: расширить FFI signature, когда понадобится FIPS context support (M6+).

2. **Verify NIST KAT** — отдельный test set для FIPS 204 SigVer не добавлен в этой session (sign-baseline + sign-NIST KAT byte-exact match косвенно подтверждает Verify correctness через round-trip, но direct Verify NIST KAT остаётся как enhancement).

3. **ML-KEM-768 Encapsulate/Decapsulate NIST KAT** — текущий M1-F scope только KeyGen (Encapsulate/Decapsulate понадобятся в M6+ application layer encryption). NIST KAT для encapDecap имеется в fixtures dir, тесты добавляются когда понадобится.

---

## 4. Build & Reproduction

### Toolchain

```
rustc 1.70+ (pinned via rust-toolchain.toml)
Cargo workspace
OpenSSL 3.5.5 LTS (vendored через openssl-src)
```

### Optional security audit tooling (для Step H verification)

```
cargo install cargo-audit --locked
```

Один раз на audit machine. После — команда `cargo audit` доступна.

### Build from source

```
cd "<repo-root>"
cargo build --all --release
```

Все 4 обязательные проверки CI:
```
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test --all
cargo build --all --release
```

### Reproducible release build (Docker)

**Prerequisites:** Docker ≥ 20.10 installed and running, ~30 минут wall-clock для двух clean builds, ~5 GB free disk.

```
docker build --file docker/release-build.dockerfile --tag montana-release:audit .
```

CI gate `reproducible_release` (см. .github/workflows/ci.yml) выполняет два независимых build с `--no-cache`, asserts byte-identity hashes на каждом push в main. Auditor может верифицировать через CI history без локального запуска (избегая Docker prerequisite).

### Run audit-relevant tests

**M1 NIST ACVP KAT (cross-implementation conformance):**
```
cd "<repo-root>" && cargo test -p mt-crypto-native --test nist_acvp_kat -- --nocapture
```

**M1 Internal correctness baselines:**
```
cd "<repo-root>" && cargo test -p mt-crypto-native --test kat_independent -- --nocapture
cd "<repo-root>" && cargo test -p mt-crypto -- --nocapture
cd "<repo-root>" && cargo test -p mt-mnemonic --test keygen_vectors -- --nocapture
cd "<repo-root>" && cargo test -p mt-mnemonic --test e2e_recovery -- --nocapture
```

**M1 Security invariants (Pass 17):**
```
cd "<repo-root>" && cargo test -p mt-crypto --test security_invariants -- --nocapture
```

**M2 Determinism invariants:**
```
cd "<repo-root>" && cargo test -p mt-merkle --test determinism_invariants -- --nocapture
cd "<repo-root>" && cargo test -p mt-genesis --test determinism_invariants -- --nocapture
cd "<repo-root>" && cargo test -p mt-state --test determinism_invariants -- --nocapture
cd "<repo-root>" && cargo test -p mt-timechain --test determinism_invariants -- --nocapture
```

**Полный M1+M2 audit chain:**
```
cd "<repo-root>" && cargo test -p mt-codec -p mt-crypto-native -p mt-crypto -p mt-mnemonic -p mt-merkle -p mt-genesis -p mt-state -p mt-timechain
```

---

## 5. Audit History

| Date | Audit | Findings | Closure |
|------|-------|----------|---------|
| 2026-04-27 | **Spec bump v33.1.6 → v34.0.0** — major monetary policy refactor (geometric step-up baseline → const linear emission) | Architectural simplification, не security audit findings | ProtocolParams удалены 4 поля (`r_genesis_moneta`, `monetary_epoch_windows`, `inflation_num`, `inflation_den`) + добавлен `emission_moneta: u128 = 13×10⁹ nɈ`. PARAMS_ENCODED_SIZE 4118 → 4094. mt-state удалена `MonetaryState` struct + carry-recurrence machinery. mt-account `apply_emission` упрощён: `reward = params.emission_moneta` const, `monetary_epoch_tick` удалён. `supply_moneta(W) = E × (W+1)` closed-form O(1) — eliminates class of bugs (epoch boundary off-by-one, carry overflow). 288/288 affected tests PASS. Driver: упрощение audit surface для production firm engagement. |
| 2026-04-27 | **External audit #4 Claude Opus 4.7 (1M context)** — incremental M4+M5 scope (mt-lottery, mt-consensus, mt-entry, mt-store) — independent SHA-256 oracle (Python hashlib) cross-check 4 hash compositions + ad-hoc ln_q64 mathematical verification (T141253) | 10 findings (0 CRITICAL/HIGH, 2 MEDIUM, 5 LOW, 3 INFO); score **8.5/10**; 325/325 M4+M5 tests PASS под single-core/single-process | **All 10 findings closed конструкцией.** Commits: `cb81d4c` M4-MED-1 window_index u32→u64 unification (spec v33.1.4→v33.1.5), `f46d48a` M4-LOW-7 3 hardcoded const → ProtocolParams [C-1] SSOT (spec v33.1.5→v33.1.6 + admission_divisor field), `64cefca` M4-MED-2 validate_winner genesis-aware contract docs, `53ba832` M4-LOW-3 log2_q64 panic-free byte extraction, `e8450bb` M4-LOW-4 checked_add validate_header window monotone, `c41f043` M4-LOW-5 saturating_mul quorum, `845d74d` M4-LOW-6 positive functional tests TooManyOps/TooManyReveals, `bcf5e9b` M5-LOW-8 cleanup orphan .tmp at FsStore::open + anti-regression test, `022c5ff` M4-INFO-10 canonical_proposer degraded-mode docs. |
| 2026-04-27 | **External audit #3 VERIFIED Claude Opus 4.7 (1M context)** — incremental M3 mt-account scope + полная zero-trust verification закрытий M1/M2 + fresh NIST CAVP source download (T124438; supersedes incomplete first pass T121239 в которой аудитор по собственному признанию срезал углы). 785/785 cargo test --all PASS. | 10 findings: 8 из incomplete first pass (1 HIGH M3-A-4, 3 LOW M3-A-1/M3-A-2/F-5, 4 INFO M3-A-3/M3-A-5/M3-A-7/M3-A-8) + 2 из VERIFIED pass (P-C-1 F-19 reopen HIGH, P-C-2 audit history sync MEDIUM). Score **8.5/10**; expected 9/10 после P-C-1 + P-C-2. | **All 10 findings closed конструкцией.** Commits: `4c14685` M3-A-4 ValidationContext, `047cc45` M3-A-1+M3-A-2 checked arithmetic, `66df1a1` M3-A-5+F-5 docs/telemetry TODO, `e61c479` M3-A-3 binding test vector, `5d3506c` Cargo.lock sync, `b96085d` P-C-1 SAFETY convention для 4 const-cast sites + Rust shim cross-references, `<this>` P-C-2 audit history sync. **131 tests PASS** в mt-account (96 unit + 35 determinism); 4 anti-regression теста (validate_dispatcher cooldown enforcement, apply_chain_length overflow panic, apply_checkpoint_rotation underflow panic, genesis_candidate_root binding `empty_internal(256)`). |
| 2026-04-26 | **External audit #2 Claude Opus 4.7 (1M context)** — incremental M2 scope (mt-merkle, mt-genesis, mt-state, mt-timechain) | 17 findings (1 HIGH M2-1 Genesis bootstrap pubkeys placeholder, 5 MEDIUM, 11 LOW/INFO); score **8/10**; 60/60 determinism invariants verified | M2-3 + M2-13 изначально закрыты в M3 milestone (binding test pin 41/40 + MonetaryState в compute_state_root). **Superseded** spec bump v33.1.6 → v34.0.0 monetary policy refactor (geometric step-up → const linear emission): pin 41/40 inflation_num/den и MonetaryState carry-recurrence удалены как artifacts old policy. Новые equivalent invariants: `reward_moneta(W) = EMISSION_moneta` const closed-form, `supply_moneta(W) = EMISSION × (W+1)` linear closed-form. M2-1 re-classified как known limitation (mainnet deployment blocker, не code audit blocker). Остальные documentation-only findings закрыты в audit-package iterations. |
| 2026-04-26 | **External audit #1 Claude Opus 4.7 (1M context)** — initial M1 layer (mt-crypto, mt-crypto-native, mt-mnemonic) — independent NIST CAVP source download + byte-exact verification | 19 findings (3 HIGH + 4 MEDIUM + 12 LOW/INFO); score **8/10**; 0 cryptographic vulnerabilities | **14/19 closed конструкцией** (commit `6ff26b3` + closure доделан в `b96085d` для F-19): F-2 cargo fmt clean, F-3 stale RustCrypto refs cleanup, F-4 SAFETY comments к 3 mlock/munlock blocks, F-5 расширенный SAFETY-комментарий, F-6 keypair() переход на OS CSPRNG (`getrandom`), F-7 zeroize intermediate PBKDF2/HKDF/HMAC buffers, F-8 SigGen NIST KAT расширен 1→15 cases с новым `mt_sign_mldsa_ctx` API, F-9 rename kat_independent → regression_baselines, F-12 split_whitespace вместо split(' '), F-18 cc parallel feature removed, F-19 const-cast OpenSSL convention в SAFETY (изначально incomplete; полностью закрыт `b96085d` после VERIFIED audit reopen). **5 deferred** с явным rationale (F-10 NIST в self_test, F-14 formal constant-time, F-15 cargo-fuzz infrastructure, F-16 threshold sigs M6+, F-17 serde_json dev-dep — auditor сам "не приоритетно"). Expected score после closure: 9/10. |
| 2026-04-26 | M0+M1+M2 critic spec-vs-code audit (`Протокол/Code/CRITIC.md` v1.6.0) — verification полного соответствия кода M0/M1/M2 спеке v33.1.2 | 4 findings (F-1 mt-recovery-fingerprint domain spec drift, F-2 stale VERSION.md Implementation field, F-3 false positive snyат критиком, F-4 controlled halts documentation) | All 4 closed (commit `9387900`): F-1 spec patch v33.1.2 → v33.1.3 (Domain registry sync), F-2 VERSION.md updated на «M0..M5 closed», F-4 audit-checklist §K Controlled halts section |
| 2026-04-26 | M1-F audit package critic review (`Протокол/Code/CRITIC.md` v1.4.0) — verification of AUDIT.md + reproduction commands + claims accuracy | 5 findings (F-A1..F-A5) — все documentation-only, не security | All 5 closed: F-A1 unsafe count 3→4 (verify added in list), F-A2 cargo audit verified+install instructions, F-A3 line counts exact (568/40/375/56/45=1084), F-A4 print_sk gate accurately described (env var M1_DUMP_SK=1), F-A5 Docker reproduction prerequisites added |
| 2026-04-26 | M1-F internal critic audit (`Протокол/Code/CRITIC.md` v1.4.0) | 7 findings (F-1..F-7) | 6 closed конструкцией, F-3 closed через NIST ACVP differential testing — **all 7 closed**. Commits: `9f2ba93` (F-4 cross-compile), `71896f6` (F-6 separate error codes), `3333738` (F-1 zeroize), `e1164ad` (F-2 Result API + F-5 cfg-gate keypair + F-7 implicit), `6b7ff30` (F-3 NIST KAT) |
| 2026-04-26 | M2 batch 2+3 audit prep — manual critic Pass 1-12 для mt-state + mt-timechain | 0 findings (manual scan clean) | 24 + 19 automated determinism invariants добавлены |
| 2026-04-26 | M2 batch 1 audit prep — manual critic Pass 1-12 для mt-merkle + mt-genesis | 0 findings (manual scan clean) | 10 + 7 automated determinism invariants добавлены |
| 2026-04-21 | External critic finding — domain separation prefix-collision | 5 findings (P1-P5) | All closed: NUL separator pattern, SK leak gate, label coherence, empty_internal binding, stale RECOVERY disclosure. Commit `d762cec`, spec v29.13.0. |

**Audit-ready status (2026-04-27):** **zero open code-side findings** в M1 + M2 + M3 layers across **3 independent external audits** (T201805 + T232707 + T124438 by Claude Opus 4.7 1M; T124438 — VERIFIED replacement of incomplete first-pass T121239). Total cumulative **46 findings** across audits — all code-side closed конструкцией; deferred items explicitly classified (M2-1 Genesis bootstrap pubkeys = mainnet deployment, не code audit; F-14/F-15 formal verification + fuzzing = post-MVP scope). M3 score 8.5/10 → 9/10 ready после P-C-1 + P-C-2 closures (commits `b96085d` + this). Documentation accuracy verified through 3 incremental audits включая VERIFIED zero-trust pass. Cross-implementation conformance verified через spec ↔ code byte-exact alignment + independent NIST CAVP source download (65 KAT cases byte-exact с fresh source 2026-04-27).

---

## 6. Spec & Documentation

| Document | Path | Scope |
|----------|------|-------|
| Protocol spec | см. `VERSION.md` (Spec target / Spec path) — авторитетный источник | Полный protocol specification, источник истины |
| App spec | `Протокол/Montana App v3.9.2.md` (актуальный на дату; см. историю VERSION.md) | Application layer specification |
| Architect role | [CLAUDE.md](CLAUDE.md) | Code architect role definition (v1.12.0) |
| Critic role | [CRITIC.md](CRITIC.md) | Code critic role definition (v1.6.0 — Pass 17 Security Card mandatory) |
| Roadmap | [ROADMAP.md](ROADMAP.md) | Implementation roadmap, milestones, status |
| Spec version SSOT | [VERSION.md](VERSION.md) | Single source of truth для spec target version |
| Pre-audit checklist | [docs/audit-checklist.md](docs/audit-checklist.md) | 11 категорий self-attestation (A-K) + reproduction one-liners для аудитора + sign-off table |
| Security Cards | [docs/security-cards.md](docs/security-cards.md) | Pass 17 mandatory Security Cards для 6 crypto primitives |

---

## 7. Pre-audit self-attestation

Перед началом external audit — см. [docs/audit-checklist.md](docs/audit-checklist.md) для полного 11-категорийного чеклиста.

**M1 + M2 final closure self-attestation (2026-04-26):**

**Findings closure:**
- [x] Все 7 M1-F audit findings закрыты конструкцией (F-1..F-7)
- [x] Все 5 audit-package findings закрыты (F-A1..F-A5)
- [x] Все 4 M0+M1+M2 critic spec-vs-code findings закрыты (F-1..F-4)
- [x] **Total: 16/16 findings closed, 0 открытых блокеров**

**Conformance:**
- [x] NIST ACVP differential testing PASS (51/51 KAT byte-exact)
- [x] Spec ↔ code byte-exact alignment verified (16 ключевых constants/sizes table)
- [x] Domain registry sync (актуальный spec target ↔ mt-codec, см. VERSION.md)

**Security:**
- [x] 13/13 security invariants PASS (Pass 17 enforcement)
- [x] 6/6 Security Cards заполнены и closed (`docs/security-cards.md`)
- [x] Heap+mlock+zeroize для secret types verified

**Determinism:**
- [x] 60/60 M2 determinism invariants PASS
- [x] 0 HashMap (real usage) в consensus path; BTreeMap canonical sort
- [x] 0 SystemTime/Instant::now/RNG в consensus path
- [x] 0 f32/f64 в consensus path

**Build & CI:**
- [x] `cargo audit` clean (verified 2026-04-26: 0 vulnerabilities, 0 warnings, 39 deps)
- [x] `cargo fmt --all -- --check` clean
- [x] `cargo clippy --all-targets -- -D warnings` clean
- [x] `cargo test --all` 564+ PASS / 0 FAIL
- [x] `cargo build --all --release` clean
- [x] Reproducible builds verified through CI gate `reproducible_release`

**Documentation:**
- [x] Public API stable (mt-crypto types/signatures unchanged через M1-E + M1-F migrations)
- [x] Audit chain documented (Layer 1/2/3 explicit, exact line counts)
- [x] Threat model documented (in/out of scope explicit)
- [x] Known limitations documented с closure path
- [x] Reproduction commands verified working (each one tested 2026-04-26)
- [x] Documentation accuracy verified through critic review of audit package itself

---

## Contact

Audit findings, questions, reports — fork репозиторий, open PR с patch + reproduction instructions.

Architect role + critic role files в репозитории — peer-reviewable methodology.
