# Pre-audit self-attestation checklist — M1+M2+M3+M4+M5+M6+M9 layers

Заполняется архитектором перед каждым external audit engagement. Все пункты должны быть `[x]` либо иметь явное обоснование почему `[ ]`.

---

## A. Conformance proofs

- [x] **NIST FIPS 204 ML-DSA-65 KeyGen byte-exact** vs ACVP-Server published vectors (25 cases)
- [x] **NIST FIPS 203 ML-KEM-768 KeyGen byte-exact** vs ACVP-Server published vectors (25 cases)
- [x] **NIST FIPS 204 ML-DSA-65 SigGen deterministic byte-exact** для empty context (1 case)
- [x] **NIST FIPS 180-4 SHA-256** vector "abc" → ba7816bf...15ad
- [ ] **NIST FIPS 204 ML-DSA-65 SigVer** (deferred — косвенно подтверждено через round-trip; direct NIST KAT не добавлен в M1-F)
- [ ] **NIST FIPS 203 ML-KEM-768 Encapsulate/Decapsulate** (deferred — Montana M1-F scope только KeyGen, encapDecap нужен в M6+)

## B. Code surface

- [x] **Layer 1 Rust shim** **662 строк** (`crates/mt-crypto/src/lib.rs`), все **7** `unsafe` blocks с `// SAFETY:` комментариями: 4 FFI sites (`fn keypair_from_seed`, `fn sign`, `fn verify`, `fn keypair_from_seed_mlkem`) + 3 mlock/munlock (`impl Drop for SecretKey`, `fn alloc_locked_secret_box`, `impl Drop for MlkemSecretKey`). Точные строки сверять через `grep -n "unsafe " crates/mt-crypto/src/lib.rs` (line refs не фиксируем — drift при growth).
- [x] **Layer 2 own C wrapper** **457 строк** (`mt_crypto.c`) + **67 строк** (`mt_crypto.h`), focused EVP API wrapping, `-Wall -Wextra -Wpedantic -Werror`
- [x] **Layer 3 vendored OpenSSL 3.5.5 LTS** через `openssl-src = "=300.5.5+3.5.5"` byte-pinned
- [x] Total own audit surface (Layer 1 + Layer 2) **1280 строк** (662 Rust shim + 49 FFI bindings + 457 C wrapper + 67 C header + 45 build script) — small enough для thorough review. Точные числа: `wc -l crates/mt-crypto/src/lib.rs crates/mt-crypto-native/src/lib.rs crates/mt-crypto-native/csrc/mt_crypto.{c,h} crates/mt-crypto-native/build.rs`.
- [x] No `serde` auto-derive в consensus-critical types (custom `CanonicalEncode` trait)

## C. Memory safety + secret hygiene (Pass 17 enforcement)

- [x] **Drop+zeroize** для `SecretKey` (4032B) и `MlkemSecretKey` (2400B)
- [x] **Heap-allocated через Box** — secret bytes живут в heap, не stack inline (verified `secret_key_is_heap_allocated` test: `size_of::<SecretKey>() == 8` pointer)
- [x] **mlock applied** на heap pages для secret bytes (best-effort через `alloc_locked_secret_box`); fallback assumption: encrypted swap (FileVault macOS / LUKS Linux)
- [x] **Stack hygiene при FFI** — `keypair_from_seed*` пишет напрямую в heap-locked Box; никаких stack temporary buffers с secret bytes (verified в [crates/mt-crypto/src/lib.rs](../crates/mt-crypto/src/lib.rs) — функции `keypair_from_seed` и `keypair_from_seed_mlkem`, search by name)
- [x] **munlock в Drop** перед heap dealloc (best-effort, errno ignored)
- [x] No `Clone`/`Copy` derive на secret types (compile-time enforced via security_invariants test)
- [x] No `PartialEq`/`Eq` на secret types (предотвращает timing leak через ==)
- [x] No secret bytes в logs / stdout / stderr (file-content scan через `no_println_or_log_on_secret_bytes_in_lib_code` test)
- [x] `mt-examples/m1_crypto.rs::print_sk` gated через env var (`M1_DUMP_SK=1` opt-in; по умолчанию SK bytes redacted; механизм: `dump_sk_enabled()` функция в [m1_crypto.rs](../crates/mt-examples/examples/m1_crypto.rs#L39-L41) проверяется в `print_sk` строка 76)
- [x] FFI buffer sizes match contract (PUBLIC_KEY_SIZE / SECRET_KEY_SIZE / SIGNATURE_SIZE constants used consistently)
- [x] **13 security invariants automated** в [crates/mt-crypto/tests/security_invariants.rs](../crates/mt-crypto/tests/security_invariants.rs) — regression detection
- [x] **6 Security Cards заполнены** в [docs/security-cards.md](security-cards.md) — Pass 17 mandatory enforcement

## D. Error surface

- [x] **`sign` / `keypair_from_seed` / `keypair_from_seed_mlkem`** возвращают `Result<_, CryptoError>` (не panic)
- [x] **CryptoError enum** с 11 variants (Display + std::error::Error impls)
- [x] No `unwrap()` / `expect()` в lib коде кроме явного internal invariant с комментарием
- [x] No silent error swallowing (`.ok()`, `let _ = ...`)

## E. Determinism (consensus path)

- [x] **FIPS 204 Algorithm 2 deterministic Sign** через `OSSL_SIGNATURE_PARAM_DETERMINISTIC=1`
- [x] No `f32`/`f64` в crypto path (consensus determinism per Montana [I-3] + [I-9])
- [x] No `HashMap`/`HashSet` iteration order dependency
- [x] No `SystemTime::now`/`Instant::now` в consensus path (только в test/tool helper `keypair()` который gated `#[cfg(any(test, feature = "testing"))]`)

## F. Misuse resistance

- [x] **`SecretKey::from_array(arbitrary_bytes)`** при последующем `sign()` возвращает `Err(CryptoError::InvalidSecretKey)`, не panic (F-7 closure через F-2 Result API)
- [x] `keypair()` (weak entropy test helper) **не доступен** в production binary (cfg-gate)
- [x] Public type fields private (no struct literal construction обходящая validation)
- [x] No `Default` impl для types requiring real crypto material

## G. Build & reproducibility

- [x] **`Cargo.lock`** committed
- [x] **Точные версии всех dependencies** (`=X.Y.Z`)
- [x] **`rust-toolchain.toml`** pinned
- [x] **Docker reproducible build** с pinned base image digest
- [x] **CI gate `reproducible_release`** проверяет byte-identity между двумя независимыми runs
- [x] **Cross-compile correctness** через `CARGO_CFG_TARGET_OS` env var

## H. Dependencies

- [x] **Crypto deps** все production-grade (OpenSSL 3.5.5 LTS, sha2 0.10.9, no pre-1.0 в consensus path)
- [x] **No "USE AT YOUR OWN RISK" libraries** в production paths
- [x] **`cargo audit`** clean (verified 2026-04-26: 0 vulnerabilities, 0 warnings, 39 dependencies scanned). Prerequisite: `cargo install cargo-audit --locked` (один раз). Verify: `cd "<repo-root>" && cargo audit`
- [x] **`cargo tree -p mt-crypto | grep -iE "ml-dsa|ml-kem|hybrid-array"`** → 0 hits (RustCrypto pre-1.0 deps удалены полностью per M1-F migration)
- [x] **License compatibility** — все deps MIT / Apache-2.0 / BSD / ISC

## I. Documentation

- [x] **`AUDIT.md`** в корне репозитория с audit chain + threat model + reproduction commands
- [x] **Threat model** explicit: in scope / out of scope / known limitations с closure paths
- [x] **Spec references** в коде через `// spec, раздел "<name>"` без версии (single source of truth — `VERSION.md`)
- [x] **Manual Validation Gate scenarios** documented в `ROADMAP.md`
- [x] **Architect + critic roles** ([CLAUDE.md](../CLAUDE.md), [CRITIC.md](../CRITIC.md)) в репозитории — peer-reviewable methodology

## J. Open findings

- [x] **Zero open audit findings** в M1 foundational layer (per AUDIT.md §5)
- [x] All 7 M1-F audit findings (F-1..F-7) закрыты конструкцией
- [x] All 5 audit-package findings (F-A1..F-A5) закрыты конструкцией
- [x] All 4 M0+M1+M2 critic findings (F-1..F-4 from `b4a00b1` audit) закрыты:
      F-1 (mt-recovery-fingerprint domain spec drift) → spec patch v33.1.2 → v33.1.3;
      F-2 (VERSION.md stale Implementation field) → updated to M0..M5 closed;
      F-3 (false positive снят критиком в самом audit);
      F-4 (controlled halts documentation) → этот раздел K ниже
- [x] Manual Validation Gate scenarios 0/1 status: ✅ Ready (зеленая под external audit)

## K. Controlled halts (documented panic sites)

Список всех `panic!`/`assert!` в lib коде Montana с обоснованием. Все они — **controlled halts при protocol-invariant violation**, НЕ attacker-triggered, НЕ silent failures. Auditor должен verify что каждый panic site:
- (a) reachable только при invariant violation от trusted source (Genesis params, frozen const)
- (b) imeет explicit comment с обоснованием
- (c) не открыт для attacker-controlled input

| Site | Файл:строка | Trigger | Обоснование |
|------|-------------|---------|-------------|
| `apply_transfer*` balance underflow | [crates/mt-account/src/lib.rs](../crates/mt-account/src/lib.rs) `fn apply_transfer{,_activation}` | `sender.balance.checked_sub(amount)` returns None | Protocol invariant breach: `validate_transfer*` гарантирует `balance >= amount` ДО apply. Halt = caller вызвал apply без validate (programmer error либо memory corruption). Не attacker-triggered: validate-then-apply pattern enforced. |
| `apply_transfer*` receiver/operator balance overflow | [crates/mt-account/src/lib.rs](../crates/mt-account/src/lib.rs) `fn apply_transfer/apply_emission` | `balance.checked_add(amount)` returns None at u128::MAX | Encoded arithmetic horizon — суммарный баланс per-account достиг u128::MAX (~3.4×10³⁸ nɈ). Не достижим под const emission `EMISSION_moneta = 13 × 10⁹ nɈ` per окно. Documented halt. |
| `apply_*` op_height/account_chain_length overflow | [crates/mt-account/src/lib.rs](../crates/mt-account/src/lib.rs) `fn apply_*` | `u32` field counters достигли u32::MAX (~4.29 млрд operations per account) | Encoded arithmetic horizon: 4.29 млрд operations per account, не достижим в реалистичный срок. Documented halt. |
| `window_w_to_u32` cast overflow | [crates/mt-account/src/lib.rs](../crates/mt-account/src/lib.rs) `fn window_w_to_u32` | `window_w: u64 > u32::MAX` при cast в AccountRecord field | AccountRecord использует u32 для window-полей (encoded size optimization 4B vs 8B). Horizon = ~4.29 млрд окон ≈ 8000 лет at 60 sec/window. Documented halt. |

**Все sites:**
- ✅ Имеют explicit panic message с обоснованием
- ✅ Reachable только при achieved arithmetic horizon либо protocol invariant breach
- ✅ Не attacker-triggered
- ✅ Halt = correct behavior per spec — protocol upgrade required либо validate-then-apply violated (programmer error)

`mt-crypto` panics gated через `assert_eq!(r, MT_OK, ...)` уже converted в `Result<_, CryptoError>` в M1-F closure (commit `e1164ad`) — там panic-в-lib больше нет.

`mt-examples` test helpers могут panic через `.expect("...")` — это test scaffolding, вне production audit scope (per critic role Scope §«НЕ входит в scope: mt-examples test helpers»). Production binary вызовы из mt-examples (m1_crypto demo) panicи через `.expect("HKDF-derived seed cannot fail KeyGen")` — internal invariant, не attacker-triggered.

## L0. Test strength augmentations (Pass 22)

**Mutation testing (recommended, не block для audit):**

```
cargo install cargo-mutants --locked
cargo mutants --package mt-lottery --package mt-consensus --package mt-entry --package mt-account --package mt-store
```

Mutation testing вводит синтетические `mutations` (изменения арифметики,
boundary conditions, removed function calls) в production code и проверяет
есть ли test что catches каждый mutation. **Surviving mutations** = weak
tests (могут passing на broken code).

**Текущий статус:** не run автоматически в CI. Recommended pre-mainnet
benchmark: ≥80% mutation kill rate для consensus path (M3-M4 крейты).

**Применимость к external audit:** auditor может запросить mutation
report как evidence test strength. Закрытие требует:
1. Run cargo-mutants
2. Analyze surviving mutations
3. Add test cases или strengthen assertions

Closure cost ~1-2 рабочих дня (run + analyze + augment tests). Не блокер
для current external audit engagement — это test quality improvement
metric, не correctness gap.

## L. M3 Storage Cards (per persistent state table)

Per родительский `Протокол/CLAUDE.md` Storage Card invariant: каждая
persistent state table обязана иметь Storage Card до статуса «closed».
Закрывает Gate 14 ([I-14] state lifecycle) для apply_proposal layer.

### AccountTable Storage Card

```
Таблица:                          AccountTable (mt_state::AccountTable)
Operation создающая запись:       TransferActivation (opcode 0x0A)
Платит creation cost:             sender (existing account, sponsor pattern)
Размер записи (bytes):            2059 (ACCOUNT_RECORD_SIZE)
Secondary resources per record:   SMT leaf hash 32B + потенциальный merkle path

Cost per record:                  амount > 0 (sender выбирает) — нет fixed
                                   creation fee, sender отправляет любую
                                   amount получателю который при этом получает
                                   AccountRecord
Cost barrier для anti-spam:       НЕ через денежный барьер (нарушило бы
                                   [I-15] time-based scarcity); защита через
                                   cooldown 1 TransferActivation per sender
                                   per τ₂ (см. validate_transfer_activation
                                   spec rule (e) [I-15] cooldown enforcement)

Lifecycle condition:              нет explicit removal в M3 scope
                                   (CloseAccount opcode 0x0B — M11 milestone,
                                   pending spec finalization payload format)
Lifecycle threshold:              N/A в текущей версии
[I-14] путь:                      3 (rate-limit через [I-15] time scarcity);
                                   barriers через time, не money

Existing pruning consistent:      yes (нет pruning, по design до M11)
[I-14] compliance status:         pending M11 — closure path в ROADMAP §M11
                                   «CloseAccount финализация». Rate-limit
                                   через cooldown сейчас limits attacker
                                   account creation rate; explicit deletion
                                   ждёт spec finalization opcode 0x0B.

Conservation invariant (per-op):  Σ delta_balance == 0 для Transfer/
                                   TransferActivation (sender -= amount,
                                   receiver += amount, atomic)
Storage growth invariant per τ₂:  ≤ active_chain_length / τ₂ × max_accounts_
                                   per_τ₂ (rate-limited через [I-15])
Storage cap:                      нет explicit hard cap — relies on
                                   [I-15] time-based + future M11 deletion
```

**Sabotage budget analysis:** атакующий $1M / $100k / $10k без profit motive,
максимизирующий state bytes:
- Stake-protected: TransferActivation требует sender с balance > 0 + cooldown;
  атакующий создаёт `N` accounts с initial balance, ждёт τ₂ окон, повторяет
  активацию каждого отдельным sponsor. Real cost = TC required для
  bootstrapping N senders × cooldown overhead.
- Per-τ₂ limit: 1 activation per existing account → max N accounts на
  attacker = `existing_accounts × (windows / τ₂)`. Линейный, не
  exponential growth.
- Для bloat 1 GB AccountTable: 1 GB / 2059 B ≈ 487K accounts. При 1
  activation per account per τ₂ = 487K окон ≈ 6.7 days (при τ₂ = 20160 ×
  60sec). Эта оценка предполагает что attacker уже владеет ≈ 487K sender
  accounts — что само по себе требует bootstrapping.
- M11 CloseAccount позволит deletion, что smaller surface для accumulated
  bloat но не предотвратит short-term spike.

[I-15] time-based + cooldown — текущий primary mitigation. M11 — secondary
explicit cleanup path после spec finalization.

### NodeTable + CandidatePool Storage Cards

Доступ через mt-state types (NODE_RECORD_SIZE = 2098, CANDIDATE_RECORD_SIZE
= 2082). Lifecycle / cost analysis для этих таблиц — domain mt-entry (M4
audit scope, отдельный milestone).

---

## Reproduction one-liners для аудитора

**NIST KAT cross-implementation conformance proof:**
```
cd "<repo-root>" && cargo test -p mt-crypto-native --test nist_acvp_kat -- --nocapture
```

**Internal correctness baselines:**
```
cd "<repo-root>" && cargo test -p mt-crypto-native -p mt-crypto -p mt-mnemonic
```

**Recovery flow end-to-end:**
```
cd "<repo-root>" && cargo test -p mt-mnemonic --test e2e_recovery -- --nocapture
```

**M2 Determinism invariants (mt-merkle / mt-genesis / mt-state / mt-timechain):**
```
cd "<repo-root>" && cargo test -p mt-merkle -p mt-genesis -p mt-state -p mt-timechain --test determinism_invariants -- --nocapture
```

Ожидание: SMT root determinism, Genesis singleton stability,
state table BTreeMap canonical sort,
VDF + cemented_bundle_aggregate per [I-8].

**M3 Determinism invariants (mt-account):**
```
cd "<repo-root>" && cargo test -p mt-account --test determinism_invariants -- --nocapture
```

Ожидание: Transfer/ChangeKey/Anchor/TransferActivation encoded
sizes, op_hash determinism (R2 invariant), apply_* determinism, validate
rejection patterns, settle_window order independence, genesis state
determinism, reward/supply consistency, apply_proposal determinism,
controlled panic on protocol breach (checked arithmetic).

**M4 Determinism invariants (mt-lottery / mt-consensus / mt-entry):**
```
cd "<repo-root>" && cargo test -p mt-lottery -p mt-consensus -p mt-entry --test determinism_invariants -- --nocapture
```

Ожидание: 32 + 27 + 24 = **83 PASS** — bundle_hash/reveal_hash R2 stability,
compute_endpoint [I-8] binding, log2_q64 / ln_q64 / weighted_ticket_node
monotonicity, determine_winner argmin canonical (M4-1 closure: TooManyOps
validation barrier), proposal_hash R2, canonical_proposer / fallback_proposer
Lookback Leadership cascade, compute_control_set canonical sort, validate_*
acceptance, finalization_status, NodeRegistration R2, candidate_vdf_init
[I-8], selection_slots / selection_sort_key, required_vdf_length Adaptive
VDF, distinct domain separators between three sort_key compositions.

**M5 Determinism invariants (mt-store):**
```
cd "<repo-root>" && cargo test -p mt-store --test determinism_invariants -- --nocapture
```

Ожидание: AccountTable / NodeTable / CandidatePool
save/load roundtrip (root byte-equal), CorruptedLength detection, crash
recovery (meta + verify_consistency), prune_proposals, byte-exact equality
для identical input, BTreeMap canonical sort persistence-stable, full state
cycle (open → populate → save → close → reopen → load), R5 atomic rename
verification (no `<name>.tmp` после save, atomic overwrite).

**M5 Pseudo-fuzz harness (mt-store wire decoders):**
```
cd "<repo-root>" && cargo test -p mt-store --test fuzz_decoders -- --nocapture
```

Ожидание: 5 PASS — `decode_account_record` / `decode_node_record` /
`decode_candidate_record` / `decode_proposal_header` /
`load_meta_last_cemented` прогоняются на 7500+ pseudo-random byte arrays
различных длин (deterministic Xorshift64). Invariant: never panic,
always возвращает Result; valid length → Ok, mismatch → StoreError::CorruptedLength.

Pseudo-fuzz используется вместо libfuzzer-sys / cargo-fuzz из-за nightly
toolchain dependency (workspace pinned на stable). При появлении nightly
target — заменить на coverage-guided fuzzing в crates/mt-store/fuzz/
fuzz_targets/.

**M4 External SHA-256 oracle (Pass 25 Independent Oracle):**
```
cd "<repo-root>" && python3 scripts/oracle_python_sha256.py
cd "<repo-root>" && cargo test -p mt-lottery --test external_oracle -p mt-entry --test external_oracle
```

Ожидание (Python): 4 hardcoded hex digests + distinct domains PASS +
input sensitivity PASS.

Ожидание (Rust tests): 4 PASS — `compute_endpoint`, `candidate_vdf_init`,
`selection_sort_key`, `nr_sort_key` byte-exact match Python `hashlib.sha256`
output. Cross-impl conformance verified — независимая reference не от
Rust SHA-256 (sha2 crate) → защита от drift между Rust impl и spec
formula.

**Reproducible release build verification:**

*Prerequisites:* Docker ≥ 20.10 installed and running, bash (для process substitution), ~30 минут wall-clock для двух clean builds, ~5 GB free disk.

```
cd "<repo-root>" && docker build --no-cache --file docker/release-build.dockerfile --tag mt-audit-1 . && docker build --no-cache --file docker/release-build.dockerfile --tag mt-audit-2 . && diff <(docker run --rm mt-audit-1 sha256sum /usr/local/bin/*) <(docker run --rm mt-audit-2 sha256sum /usr/local/bin/*)
```

*Альтернатива без Docker prerequisite:* верифицировать через CI history — каждый push в `main` запускает CI job `reproducible_release` (см. [.github/workflows/ci.yml](../.github/workflows/ci.yml)), который выполняет тот же двойной build с byte-identity assertion. Auditor может проверить green CI runs за период как evidence.

**Build sanity:**
```
cd "<repo-root>" && cargo fmt --all -- --check && cargo clippy --all-targets -- -D warnings && cargo build --all --release
```

---


---

## H. M6 Network layer (mt-net + mt-net-transport)

- [x] **mt-net wire format byte-exact** — ProtocolMessage envelope (14 B header + payload), 18 message types in registry, IBT online + mesh proof, Bootstrap PoW, Uniform Framing (1024 B fixed), 12 structured payloads (FastSync*, PeerList*, BatchLookup*, RangeSubscribe*, Bye)
- [x] **mt-net 110 unit + integration tests pass** — `cargo test -p mt-net --features testing`
- [x] **mt-net-transport libp2p TCP+TLS 1.3+Noise+Yamux upgrade chain** — verified through `cargo test -p mt-net-transport --features testing` (14 tests pass)
- [x] **Manual Validation Gate scenario 6 PASS** — two-node handshake e2e (commit `9a15f49`); `tests/e2e_two_node_handshake.rs::two_node_request_response_ping_pong`
- [x] **Manual Validation Gate scenario 7 PASS** — proposal exchange e2e + 512 KiB boundary (commit `04f8d29`); `tests/e2e_proposal_exchange.rs::{proposal_envelope_round_trip, large_payload_near_max_limit}`
- [x] **[C-5] libp2p capability checklist 8/8 PASS** — TCP+TLS 1.3+Noise+Yamux+Swarm primitives, async tokio, rustls + snow constant-time, Linux+macOS+Windows, IPFS+Filecoin+Polkadot 5+ years production, MIT/Apache 2.0
- [x] **Backpressure rules B1-B6 enforced** — max_protocol_payload_bytes (1 MiB) + max_sf_ciphertext_bytes (64 KiB) reject до allocation per spec
- [x] **Critic-fix bundle P-C1..P-C8 closed** — domain registry SSOT в mt-codec, prefix-free rename mt-tunnel→mt-tunnel-online, 5 fuzz harnesses scaffolded, try_new constructors, Bye forward-compat, ibt_mesh_verify O(1) path, no unwrap/expect в lib code
- [x] **5 fuzz harnesses scaffolded** в `crates/mt-net/fuzz/fuzz_targets/` — fuzz_decode_envelope/frame/mesh_frame/sf_envelope/payloads (запуск через `cargo +nightly fuzz run`)

## I. M9 Conformance suite (mt-conformance)

- [x] **mt-conformance crate created** — public binding test vectors для cross-implementation byte-exact verification
- [x] **2 unit tests pass** — `envelope_vectors_byte_exact` + `pow_target_byte_exact`
- [x] **Initial vectors covered** — envelope A1/A2/A3 + IBT B1 (after P-C2 rename) + Bootstrap PoW F1/F2 target derivation
- [x] **iOS port мirrored** — `iOS/Apps/Montana/MontanaTests/MTConformanceVectors.swift` byte-exact mirror Rust crate
- [ ] **Expansion** (12 TBD-A markers) — defer until app-layer payload format finalization (BatchLookupRequest/Response, RangeSubscribeResponse query/result/blob entry types)

## Sign-off

| Role | Name | Date | Status |
|------|------|------|--------|
| Architect | (architect role per CLAUDE.md v1.15.0) | 2026-05-02 | ✅ Self-attested ready (M6 + M9 added) |
| Critic | (critic role per CRITIC.md v1.7.0) | 2026-05-02 | ✅ All findings closed (8 P-C1..P-C8 + 5 P-S1..P-S5 critic-fix bundle) |
| Author | (project owner) | — | Ожидает решения об audit firm engagement |
| External auditor | (TBD) | — | Pending engagement |

---

**Status:** READY FOR EXTERNAL AUDIT (M1 + M2 + M3 + M4 + M5 + M6 + M9 layers scope, 16 крейтов, ~14640 LOC, 255+ invariants + 14 e2e network tests, 53/53 findings closed: 40 prior + 13 P-C1..P-C8 + P-S1..P-S5 critic-fix bundle; spec v35.23.0 — M6 transport closure + Genesis Decree network params restructure).
