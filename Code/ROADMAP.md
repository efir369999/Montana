# Roadmap — Montana Reference Implementation

**Spec target:** см. `VERSION.md` (single source of truth)
**Implementation version:** 0.0.0 (pre-release, в разработке)

---

## Scope

**В scope:** protocol core согласно спеке Montana (версия — `VERSION.md`) — криптография, TimeChain, NodeChain, AccountChain, Account Table, consensus (Proof of Time), P2P transport, Fast Sync, node binary.

**Вне scope (отдельный workspace в будущем):** Application Layer по спеке `Montana App v2.4.2.md` — Juno agent, messaging, файловое хранилище, профили, клиентская LLM runtime.

---

## Принципы разработки

1. **Одна функция за раз.** Explain → Write + tests → Test → Commit. Никаких «напишу весь модуль сразу».
2. **Success criteria блок до кода.** Для каждой consensus-critical функции — spec quote + контракт + чек-лист критериев (см. `CLAUDE.md` → Verifiable success criteria).
3. **Автокоммит.** Любое изменение в `Протокол/Code/` автоматически завершается `git commit`. См. `CLAUDE.md` → Git discipline.
4. **Четыре обязательные команды зелёные перед каждым коммитом:**
   - `cargo fmt --all -- --check`
   - `cargo clippy --all-targets -- -D warnings`
   - `cargo test --all`
   - `cargo build --all --release`
5. **Byte-for-byte детерминизм.** Custom canonical encoding, explicit little-endian, BTreeMap вместо HashMap в consensus, no floats, no system clock. См. `CLAUDE.md` → Byte-for-byte determinism.
6. **Ссылки на спеку в коде.** Каждое consensus-critical решение помечается комментарием `// spec, раздел "<название>"` **без версии**. Источник истины версии — `VERSION.md`. При spec bump `VERSION.md` обновляется, spec-комментарии в коде не трогаются, если раздел не переименован.
7. **Demo откладываются до готовности протокола.** До M4 (consensus core) отдельные `examples/demo.rs` в пакетах не пишем — юнит-тестов достаточно для sanity check. После M4 создаём `examples/` в корне workspace с end-to-end сценариями (OpenAccount → Transfer → BundledConfirmation → apply_proposal), когда есть реальный flow между пакетами.

---

## Статусы crates

- **TODO** — не начато, ожидает своей очереди
- **In progress** — начата работа, не завершено
- **Written** — код написан, тесты есть, ожидает review
- **Committed** — зафиксировано в git, закрыто
- **Blocked** — заблокирован gap в спеке, внешней зависимостью или другим crate

---

## 15 Crates — статус и зависимости

| # | Crate | Milestone | Зависит от | Раздел спеки | Статус |
|---|-------|-----------|------------|--------------|--------|
| 1 | `mt-codec` | M1 | — | Consensus encoding layer | ✅ Committed (90464a8) |
| 2 | `mt-crypto` | M1 | sha2, pqcrypto-falcon | Криптография + Primitive layer | ✅ Committed (df55372) |
| 3 | `mt-merkle` | M1 | mt-codec, mt-crypto | Sparse Merkle Tree | ✅ Committed (f242956) |
| 4 | `mt-genesis` | M1 | mt-codec, mt-crypto | Genesis Decree + protocol params | ✅ Committed (45c1e84) |
| 5 | `mt-state` | M2 | mt-codec, mt-crypto, mt-merkle | Состояние сети (Account/Node/Candidate Tables + roots) | ✅ Committed (ab99e23) |
| 6 | `mt-timechain` | M2 | mt-codec, mt-crypto, mt-genesis, mt-state | TimeChain VDF + Adaptive D + cemented_bundle_aggregate | ✅ Committed (76ed8da) |
| 7 | `mt-account` | M3 | mt-codec, mt-crypto, mt-state, mt-genesis, mt-timechain | AccountChain operations + apply_proposal | ✅ Committed (Phase A..F closed, 102 tests) |
| 8 | `mt-lottery` | M4 | mt-crypto, mt-state | Node/Account lottery + BundledConfirmation | ✅ Closed (6 phases, 96 тестов) |
| 9 | `mt-consensus` | M4 | mt-state, mt-lottery, mt-timechain | Proposal + Lookback Leadership + fallback | ✅ Closed (5 phases, 60 тестов) |
| 10 | `mt-entry` | M4 | mt-state, mt-timechain | NodeRegistration + selection event + adaptive VDF | ✅ Closed (5 phases, 39 тестов) |
| 11 | `mt-store` | M5 | mt-state, mt-consensus | Filesystem persistence | ✅ Closed (5 phases, 24 теста) |
| 12 | `mt-net` + `mt-net-transport` | M6 | mt-consensus, libp2p | libp2p + IBT + Wire format + Dandelion++ + Mesh + S&F | ✅ Closed (Phase A-G + C.0-C.4 + MONT-002 nonce replay closure, 127 tests, in-process e2e; cross-machine pairing defer M8) |
| 13 | `mt-sync` | M7 | mt-state, mt-net | Fast Sync | ⏳ TODO |
| 14 | `montana-node` | M8 | все | Бинарь + CLI | ✅ Closed (byte-exact rewrite через canonical apply_proposal, commit fb204ef; DEV-001..DEV-009 closed; DEV-010 acknowledged genesis bootstrap) |
| 15 | `mt-conformance` | M9 | все | Test vectors + conformance suite | ✅ READY initial (envelope A1-A3 + IBT B1 + PoW F1/F2); expansion в работе (12 TBD-A markers) |

---

## Milestones

### M0 — Workspace skeleton ✅

- [x] Cargo workspace, toolchain pin, rustfmt/clippy configs
- [x] VERSION.md, README.md, ROADMAP.md
- [x] Git repository в `Протокол/Code/`, parent gitignore настроен
- [x] 4 обязательные команды зелёные
- [i] mt-version stub crate (удалён 2026-04-17 как нарушение SSOT [C-1])

**Результат:** база для всей реализации. Commit `e2457ad`.

### M1 — Foundational layer ✅ ЗАКРЫТ

Четыре параллельных crate: fundamentals криптографии и сериализации.

- [x] `mt-codec` — canonical encoding (u8/u16/u32/u64/u128 LE, fixed bytes, 30 domain separators). 26 тестов. Commit `90464a8`.
- [x] `mt-crypto` — SHA-256 + FN-DSA-512 обёртки (Hash32, PublicKey 897B, SecretKey 1281B, Signature 666B, SuiteId). 18 тестов. Commit `df55372`.
- [x] `mt-merkle` — sparse Merkle tree глубины 256, InclusionProof + verify_proof, empty_internal через OnceLock. 25 тестов. Commit `f242956`.
- [x] `mt-genesis` — ProtocolParams (25 полей, 2017 байт canonical), Genesis State Hash formula, OnceLock singleton. 19 тестов + 1 ignored (bootstrap keypairs TBD). Commit `45c1e84`.

**Итог M1:** 4 пакета (mt-codec, mt-crypto, mt-merkle, mt-genesis), 88 тестов зелёных, ~1680 строк кода. Foundational layer закрыт. Найдена ambiguity в спеке: Genesis State Hash формула без domain separator — единственный такой случай, flagged для spec author review.

### M2 — State & time ✅ ЗАКРЫТ

- [x] `mt-state` — Account Table, Node Table, Candidate Pool записи (1000/1043/1027 B) + три sparse Merkle tree + state_root композиция + derive_account_id/derive_node_id + is_active predicate. 32 теста. Commit `ab99e23`.
- [x] `mt-timechain` — vdf_step/verify (T_r = SHA-256^D), next_d (Adaptive D ±3%), cemented_bundle_aggregate (unpredictable-offline binding, 3 ветви). 23 теста. Commit `76ed8da`.

**Итог M2:** 2 пакета, 55 тестов зелёных, ~1000 строк кода. State types + canonical time + anti-grinding binding готовы. Материализация Genesis state (создание bootstrap AccountTable/NodeTable записей) — отложена в M3, зависит от apply_proposal для консистентности.

### M3 — AccountChain ✅ ЗАКРЫТ

Один пакет `mt-account`, 6 phases закрыты. 102 теста, ~1850 строк кода.

| Phase | Scope | Commit |
|-------|-------|--------|
| A | operation types + canonical encoding + op_hash (identifier via signed_scope после v29.7.0) | `0efd92d` → refactor `1af1fff` |
| B | validation (OpError + 5 функций) | `cf95eaf` |
| C | apply individual operations | `8fcca20` |
| D | emission (reward/bonus/bootstrap_cumulative/supply) | `36503dc` |
| E | apply_proposal partial (steps 2/3.5/3.6/4, stubs 1/3a/3b) | `5ab2a45` |
| F | Genesis state materialization | `30400eb` |

Критерий закрытия выполнен: два аккаунта могут обменяться Transfer, balance updated, state_root детерминирован (проверено тестами `apply_transfer_sum_delta_balance_is_zero`, `apply_proposal_state_root_deterministic`).

### M4 — Consensus core 🔄 Next

Три пакета параллельно — сердце консенсуса. Зависимости между ними:

```
mt-lottery (идентификация winner) ──┐
                                     ├─→ mt-consensus (proposal сборка)
mt-entry (node admission)  ─────────┘
```

#### `mt-lottery` ⏳ TODO

Раздел спеки: «VDF Reveal и лотерея» (~строка 754+), «Confirmer threshold», «cemented_bundle_aggregate» (уже в mt-timechain).

- **Phase A. BundledConfirmation тип + layout** ✅ (24 теста)
  - `struct BundledConfirmation { node_id, endpoint, window_index, op_hashes[], reveal_hashes[], signature }` — counts встроены через `Vec::len()`, encoded как u16 LE префиксы
  - CanonicalEncode + encode_signed_scope (SSI R1), bundle_hash = identifier (SSI R2, class "mt-bundle")
  - `BundleError` (6 вариантов): UnknownNode, UnsupportedSuite, OpsOutOfOrder, RevealsOutOfOrder, WrongEndpoint, InvalidSignature
  - validate: strict ascending op_hashes/reveal_hashes, endpoint == expected_T_r (caller), signature verify by NodeTable[node_id].node_pubkey
- **Phase B. VDF_Reveal тип + layout** ✅ (15 тестов)
  - `struct VdfReveal { node_id, window_index, endpoint, signature }` — REVEAL_SIZE = 734 B
  - CanonicalEncode + encode_signed_scope (SSI R1), reveal_hash = identifier (SSI R2, class "mt-vdf-reveal")
  - `compute_endpoint(t_r, cba_w_minus_2, node_id, window_index)` = SHA-256("mt-lottery" || T_r(W) || cba(W-2) || node_id || window_index LE)
  - `RevealError` (5 вариантов): UnknownNode, UnsupportedSuite, WrongWindow, WrongEndpoint, InvalidSignature
  - validate_reveal: спек-правила 1, 2, 3, 5 («Валидация VDF_Reveal», строки 1020-1026). Правило 4 (weighted_ticket < target) — в Phase C
- **Phase C. Node lottery: weighted_ticket_node** ✅ (24 теста)
  - `seniority_bonus(chain_length, snapshot) = min(chain_length/69, snapshot)` — u64 unsigned
  - `lottery_weight = snapshot + seniority_bonus` — u64 unsigned, DS-2 floor ≥ 1
  - `log2_q64(endpoint)` — bit-scan leading_zeros + нормализация мантиссы в [2^127, 2^128) + **degree-3 Remez minimax polynomial** `log2(1+y) ≈ B0 + y·(B1 - y·(B2_abs - y·B3))` (halved form, unsigned u64 coefficients; max error 2^-10.62)
  - `ln_q64(endpoint) = log2_q64 × LN2_Q64 >> 64`, `LN2_Q64 = 0xB17217F7D1CF79AB`
  - `weighted_ticket_node = ln_q64(endpoint) / (lottery_weight as u128)` — u128 integer div toward zero
  - Monotonicity + determinism + boundary tests + 5 binding test vectors TV1-TV5 from spec
  - [I-9] compliance **closed** (binding coefficients B0..B3 + 14 binding test vectors total: 5 ln_q64 + 5 weighted_ticket_node + 4 weighted_ticket_account)
- **Phase D. Account lottery: weighted_ticket_account** ✅ (13 тестов)
  - `compute_account_endpoint(account_id, op_hash, t_r, cba_w_minus_2)` = SHA-256("mt-account-lottery" || ...)
  - `weighted_ticket_account = ln_q64(endpoint) / (account_chain_length_snapshot as u128)` — u128 совместим с node ticket для argmin
  - `AccountLotteryError` (2 варианта): OperatorExcluded, ZeroSnapshot
  - `validate_account_participation(is_node_operator, snapshot)` — spec правила 2, 3 из «Валидация участия аккаунта»
- **Phase E. Winner determination** ✅ (12 тестов)
  - `Candidate { ticket: u128, class: u8, id: [u8;32] }` + `Winner`
  - `determine_winner(candidates)` — argmin by (ticket asc, class asc, id lex asc) canonical rule
  - `sorted_candidates_for_fallback` — для fallback cascade
  - Tie-breaking ambiguity в спеке (probability ~ 2^-128) закрыт canonical rule в коде
- **Phase F. Quorum calculation** ✅ (8 тестов)
  - `quorum(active_chain_length) = (67 × X + 99) / 100` — u64 unsigned, [I-9] compliant (spec vectors passed)
  - `is_cemented(cemented_sum, active)` = `cemented_sum ≥ quorum`

#### `mt-consensus` ⏳ TODO

Раздел спеки: «Закрытие окна (Lookback Leadership Finalization)», «Proposal header», «Canonical acceptance», «fallback cascade».

- **Phase A. Proposal header тип + layout** ✅ (22 теста)
  - `ProposalHeader` struct, 18 полей, PROPOSAL_HEADER_SIZE = 1080 B (target 16B u128, winner_class ∈ {1,2} validated)
  - encode_signed_scope (R1) + proposal_hash (R2, class "mt-proposal")
  - `HeaderError`: UnknownProposer, UnsupportedSuite, InvalidSignature, WindowNotMonotone, ProtocolVersionDecreased, ProtocolVersionUnsupported, FallbackDepthZero
  - validate_header: invariants window monotone, protocol_version monotone + ≤ local_max, fallback_depth ≥ 1, signature R1
- **Phase B. Lookback Leadership — proposer_W = winner_{W-2}** ✅ (11 тестов)
  - `canonical_proposer(current_window, bootstrap, sorted_candidates_W-2)` — первый node в sorted list
  - `fallback_proposer(...)` — Nth node для fallback cascade
  - Genesis bootstrap (W<2) + extended bootstrap (нет nodes в candidates)
  - Account winner case — proposer = ближайший node (spec строка 1315)
- **Phase C. control_set формула** ✅ (10 тестов)
  - `ControlObjectRef { op_hash, cemented_window }`
  - `compute_control_set(all_cemented, prev_window, W)` — filter + sort (window asc, op_hash lex asc)
  - `validate_control_set` — равенство проверяется byte-exact
- **Phase D. Canonical acceptance validation** ✅ (13 тестов)
  - `AcceptanceError`: ProposerNotCanonical, InsufficientBundles, IncludedRevealsMismatch, WrongWinner
  - `validate_proposer_is_canonical(header, bootstrap, sorted_W-2)` — сверка с fallback_proposer(depth)
  - `validate_bundles_threshold` — делегирует mt_lottery::is_cemented
  - `validate_included_reveals` — byte-exact equality reveal_hashes
  - `validate_winner` — argmin через mt_lottery::determine_winner
  - state_root verification — делегирован в mt-account::apply_proposal
- **Phase E. Finalization flow** ✅ (4 теста)
  - `FinalizationStatus { Cemented, Rejected }` + `finalization_status(sigs_sum, active)`
  - `leader_penalty_excluded_node(header)` → NodeId для exclusion из lottery текущего окна

#### `mt-entry` ✅ CLOSED (5 phases, 39 тестов)

Раздел спеки: «Вход и регистрация», «apply_proposal Шаг 1/3a/3b», «Adaptive VDF».

- **Phase A. NodeRegistration** ✅ (11 тестов)
  - `struct NodeRegistration`, NODE_REGISTRATION_SIZE = 1646 B
  - encode_signed_scope + nodereg_hash (R2 "mt-nodereg")
  - NodeRegError: UnsupportedSuite, InvalidSignature, NodeIdAlreadyIn{NodeTable,CandidatePool}, OperatorAccount{NotFound,AlreadyNode}, WStartOutOfRange, VdfChainTooShort
  - validate_noderegistration (structural checks 1-3)
- **Phase B. candidate_vdf_init + Candidate Pool** ✅ (4 теста)
  - `candidate_vdf_init(t_r, cba, node_id)` = SHA-256("mt-candidate-vdf-init" || ...)
  - `compute_expiry_window` = registration + 3τ₂
  - `apply_candidate_expiry(pool, window)` — шаг 3a
- **Phase C. Selection event** ✅ (9 тестов)
  - `selection_slots(active_nodes) = max(1, active/130)` — 1% cap
  - `selection_sort_key(t_r, cba, node_id)` = SHA-256("mt-selection" || ...)
  - `is_selection_window(W)` — каждые 336 окон
  - `rank_candidates_for_selection` + `apply_selection_event` — шаг 3b (chain_length=1 активация)
- **Phase D. Adaptive VDF** ✅ (6 тестов)
  - `required_vdf_length(pending, active, τ₂)` — integer form через permille (per [I-9])
  - pressure_permille > 10 (1%) → τ₂ × pressure_permille / 10; иначе τ₂
- **Phase E. apply_proposal orchestration** ✅ (5 тестов)
  - `nr_sort_key(t_r, cba, node_pubkey)` per spec строки 1838-1843
  - `apply_noderegistrations_batch` — incremental apply с pending growth
  - Caller (mt-node / integration test) orchestrates: shag 1 = batch, 3a = expiry, 3b = selection, затем mt-account::apply_proposal для steps 2/3.5/3.6/4

**Критерий закрытия M4:** 2 узла могут запустить локальную сеть in-memory, проходить окна, рассчитывать winner, cementing proposals. Полный state transition apply_proposal замкнут.

### M5 — Persistence ⏳ Next after M4

Раздел спеки: «Fast Sync», «Хранение».

Цель: локальный узел перезапускается с восстановлением state с диска; crash-consistent между apply_proposal commit-ами.

#### `mt-store` ✅ CLOSED (5 phases, 24 теста)

**Реализация:** filesystem + fixed-size records (не RocksDB/sled — pure std::fs,
минимум deps, максимум простоты для Manual Validation Gate).

- **Phase A. FsStore** ✅
  - `FsStore::open(path)` — создаёт root + proposals/ subdirectory
  - `StoreError`: Io, CorruptedLength, ParseFailed, NotFound
- **Phase B. Table persistence** ✅
  - `save/load_account_table` — canonical_encode concat → accounts.bin, fixed-size parse
  - `save/load_node_table` — nodes.bin
  - `save/load_candidate_pool` — candidates.bin
  - Decode функции — inverse CanonicalEncode, byte-exact (root byte-equal round-trip verified)
- **Phase C. Proposal archive** ✅
  - `archive_proposal(header)` → `proposals/{window:020}.bin`
  - `get_proposal_by_window(W)` → Option<ProposalHeader>, byte-exact decode
  - Test 100 proposals random access passes
- **Phase D. Crash recovery** ✅
  - `meta.last_cemented.bin` — u64 LE, last committed window
  - `verify_consistency()` — meta указывает на archived proposal?
  - Test: meta=100 без archive → NotFound finding
- **Phase E. Pruning** ✅
  - `prune_proposals_before(threshold)` — delete files window < threshold
  - AccountTable/NodeTable/CandidatePool untouched
  - Test prune 9/20 proposals, current state preserved

**Критерий закрытия M5 passes:**
1. `cargo test -p mt-store` = 24/24 ✓
2. `full_restart_cycle_state_preserved` integration test — open → populate →
   save → close → reopen → load → roots byte-equal ✓
3. `verify_consistency_detects_missing_proposal` fault flagging ✓

Hands-on example `examples/m5_persist.rs` — будет delivered в Validation Gate.

---

## Локальный shakedown — Manual Validation Gate (между M5 и M6) ⏳

**Смысл:** до написания сетевого слоя (M6 libp2p, Dandelion++, IBT) автор вручную на своей машине прогоняет **каждую шестерёнку протокола** с hands-on example binaries. Архитектор (я) объясняет output, критик (я в другой роли) задаёт adversarial вопросы на каждом шаге. Цель — убедиться что протокол работает вживую, не только в unit tests.

### Режим прохождения — **incremental, scenario-by-scenario**

**Зафиксировано автором 2026-04-20.** Gate проходится пошагово, по одному сценарию за сессию:

1. Architect пишет `examples/mN_name.rs` binary **только для текущего сценария**
2. Автор запускает команды на своей машине, копирует output
3. Architect разбирает output построчно, сверяет с expected
4. Критик задаёт adversarial checklist
5. Автор либо подтверждает, либо флагует finding
6. Всё OK → статус сценария `✅ passed {дата}` в tracker-е ниже
7. Finding → fix → повторный прогон → retry до passed

**Ценность incremental над batch:** ошибки ловятся в момент написания соответствующего binary; не накапливаются до середины Gate. Каждая шестерёнка exercised отдельно.

**Scenario crate:** все binaries живут в `crates/mt-examples/` — отдельный crate с dependencies на все необходимые modules, каждый scenario — отдельный `examples/mN_*.rs` (cargo example). Создаётся в первой Gate сессии.

### Audit findings (M1-F audit closure — все 7 closed)

**M1-F audit критика реализации (2026-04-26).** 7 findings surface-нуты, **все 7 закрыты конструкцией**. Status: READY FOR EXTERNAL AUDIT (M1 foundational layer scope). Audit package: [AUDIT.md](AUDIT.md) + [docs/audit-checklist.md](docs/audit-checklist.md).

| Finding | Закрытие | Commit |
|---------|----------|--------|
| F-1: нет `Drop+zeroize` для `SecretKey`/`MlkemSecretKey` | ✅ workspace dep `zeroize=1.8.1` + Drop impls | `3333738` |
| F-2: panic в lib коде на FFI ошибку | ✅ `sign`/`keypair_from_seed` → `Result<_, CryptoError>` | `e1164ad` |
| F-3: KAT-baselines self-derived без NIST FIPS oracle | ✅ NIST ACVP differential testing — 51/51 byte-exact | `6b7ff30` |
| F-4: `cfg!(target_os)` неверно для cross-compile | ✅ `CARGO_CFG_TARGET_OS` env var | `9f2ba93` |
| F-5: `keypair()` слабая энтропия в публичном API | ✅ `#[cfg(any(test, feature = "testing"))]` гейт | `e1164ad` |
| F-6: один error code маскирует разные fail paths | ✅ 6 новых раздельных error codes (7-12) | `71896f6` |
| F-7: `SecretKey::from_array` без validation | ✅ implicit closure через F-2 (invalid SK → Err при первом sign) | `e1164ad` |

**F-3 closure detail (NIST ACVP differential testing):**

Sparse clone https://github.com/usnistgov/ACVP-Server (Apache-2.0, public domain test vectors из NIST CAVP). Extracted ML-DSA-65 + ML-KEM-768 KeyGen + SigGen deterministic test cases в `crates/mt-crypto-native/tests/fixtures/nist_acvp/` (~500 KB JSON, 3 files). Integration test `crates/mt-crypto-native/tests/nist_acvp_kat.rs` runs differential testing.

**51/51 NIST KAT byte-exact PASS:**
- ML-DSA-65 KeyGen 25/25 (FIPS 204 Algorithm 1 deterministic seed → pubkey/secretkey)
- ML-KEM-768 KeyGen 25/25 (FIPS 203 Algorithm 16 (d, z) → ek/dk)
- ML-DSA-65 SigGen deterministic external pure empty-context 1/1 (FIPS 204 Algorithm 2 — Montana usage pattern)

**Conformance proof:** OpenSSL 3.5.5 LTS backend через mt-crypto-native FFI производит выходы байт-в-байт идентичные NIST FIPS 204/203 reference. Cross-implementation conformance доказана на public NIST oracle.

**Reproduction:**
```
cd "/Users/kh./Python/Ничто/Montana/Russian/Протокол/Code" && cargo test -p mt-crypto-native --test nist_acvp_kat -- --nocapture
```

**Known deferred (lesser scope, явно документированы в [AUDIT.md](AUDIT.md) §3):**
- ML-DSA-65 SigGen с non-empty context — current API не принимает context parameter (Montana usage pattern не использует FIPS context); расширение FFI signature когда понадобится
- ML-DSA-65 SigVer NIST KAT direct — косвенно подтверждено через round-trip + NIST sign byte-exact
- ML-KEM-768 Encapsulate/Decapsulate NIST KAT — M1-F scope только KeyGen; encapDecap нужен в M6+ application layer

Эти limitations задокументированы с closure path и не препятствуют external audit M1 foundational layer.

### Status tracker

| Scenario | Binary | Status | Дата |
|----------|--------|--------|------|
| 0. User onboarding (24-word mnemonic → identity) | `mt-examples/examples/m1_mnemonic.rs` | ✅ passed (positive end-to-end: 6/6 PASS — seeds, keypair (terminal), recovery-fingerprint, mnemonic, vectors (6 binding), roundtrip; adversarial negative-cases — unit tests `mt-mnemonic` отдельно, не покрыто example subcommands) | 2026-04-30 |
| 1. M1 Crypto shakedown | `mt-examples/examples/m1_crypto.rs` | ✅ passed (5/5 PASS — keypair-deterministic, keypair-random, sign, hash, merkle-empty; finding в Subsection 4 cmd_sign: ожидание `sig1 != sig2` противоречило spec строке 5286 «deterministic ML-DSA-65, RND = 0x00 × 32» — assertion инвертирована, title переписан на «Determinism…») | 2026-04-30 |
| 2. M2 TimeChain + State | `mt-examples/examples/m2_timechain_state.rs` | ✅ passed (5/5 PASS — vdf-forward 1000, next-d-boundaries, cba-branches, state-root-compose, merkle-inclusion; binary написан 2026-04-30 коммитом `7e333e6`, прогнан автором на iMac) | 2026-04-30 |
| 3. M3 Account operations | `mt-examples/examples/m3_account.rs` | ⏳ TODO | — |
| 4. M4 Full consensus cycle | `mt-examples/examples/m4_local_net.rs` | ⏳ TODO | — |
| 5. M5 Persistence + restart | `mt-examples/examples/m5_persist.rs` | ⏳ TODO | — |

**M6 unblock criterion:** все 6 строк status = `✅ passed`.

### Starter instructions — начало следующей сессии

Стандартная последовательность для новой сессии после этой точки:

1. **Архитектор реализации погружается в роль.** Прочитать `Code/CLAUDE.md` (v1.4.0+) построчно. Показать критерии работы. Проверить `VERSION.md` — актуальный `Spec target` (на 2026-04-21 = Montana v29.13.0).

2. **Sanity check build state:**
```
cd "/Users/kh./Python/Ничто/Montana/Russian/Протокол/Code" && cargo fmt --all -- --check && cargo clippy --all-targets -- -D warnings && cargo test --all 2>&1 | grep -E "test result.*passed|FAILED" | tail -5
```
Ожидание: fmt/clippy clean, 550+ tests passed, 0 failed.

3. **Выбрать следующий сценарий Validation Gate** (incremental, один за сессию). Первый pending — сценарий 0 «User onboarding». Последовательность: 0 → 1 → 2 → 3 → 4 → 5.

4. **Прогон сценария:**
   - Автор copy-paste команды из соответствующего блока «Сценарий N» ниже.
   - Автор запускает на своей машине, копирует output в чат.
   - Архитектор разбирает построчно, сверяет с acceptance criteria.
   - Критик (роль `Code/CRITIC.md` v1.3.0+) применяет adversarial checklist + min 2 perspectives с отдельными выводами per perspective.
   - Всё OK → сценарий `✅ passed {дата}` в status tracker.
   - Finding → architect предлагает fix → автор apply `делай` → retry.

5. **M6 разблокируется** когда все 6 сценариев passed. До этого — manual validation приоритет выше любых новых features.

### Policy

- Binary пишет в stdout структурированные events (hashes, state roots, winners, errors). Автор читает output глазами.
- Формат сценария ниже: binary → команды → expected output → architect gate → critic checklist.
- M6 не начинается пока ВСЕ 5 сценариев не passed с подписью архитектора и критика.

### Формат каждого сценария

```
Сценарий N: {название}
Фаза покрытия:     {M1/M2/...}
Prerequisite:      {предыдущие сценарии}
Binary:            examples/mN_name.rs
Команды (one-line shell, абсолютные пути):
  cd "..." && cargo run --release --example mN_name -- {args}
Expected output pattern:
  {literal prefix / regex}
Architect gate:    {acceptance criterion}
Critic checklist:  {5-10 adversarial вопросов}
```

### Сценарий 0 — User onboarding (24-word mnemonic → derived identity)

Binary: `examples/m1_mnemonic.rs`

Демонстрирует **production user flow**: как реальный пользователь создаёт identity в Montana — от 256-битной entropy через 24-словную мнемонику к master_seed и per-role keypair seeds для трёх ролей (account, node, app-encryption).

Что проверяем:
- **Canonical wordlist** — 2048 слов, binding SHA-256 fingerprint `2f5eed53a4727b4bf8880d8f3f199efc90e58503646d9ff8eff3a2ed3b24dbda` матчит
- **M-1 Algorithm** (mnemonic_to_master_seed) — PBKDF2-HMAC-SHA-256, salt=`"mt-seed"`, iter=2²⁰ → 64-байтовый master_seed
- **24-слов mnemonic encoding** — entropy (256 bit) + 8-bit checksum = 264 bit → 24 × 11 bit = 24 слова (последнее включает checksum)
- **Per-role HKDF-Expand** — master_seed + info (`"mt-account-key"` / `"mt-node-key"` / `"mt-app-encryption-key"`) → 48B/48B/64B seeds
- **Byte-exact binding vectors** — 6 штук в спеке (3 entropy → master_seed + 3 per-role derivation)

Команды (actual subcommands):
```bash
cd "/Users/kh./Python/Ничто/Montana/Russian/Протокол/Code" && cargo run --release --example m1_mnemonic -- vectors
cd "/Users/kh./Python/Ничто/Montana/Russian/Протокол/Code" && cargo run --release --example m1_mnemonic -- mnemonic "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon art"
cd "/Users/kh./Python/Ничто/Montana/Russian/Протокол/Code" && cargo run --release --example m1_mnemonic -- keypair 0000000000000000000000000000000000000000000000000000000000000000
cd "/Users/kh./Python/Ничто/Montana/Russian/Протокол/Code" && cargo run --release --example m1_mnemonic -- roundtrip 1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef
cd "/Users/kh./Python/Ничто/Montana/Russian/Протокол/Code" && cargo run --release --example m1_mnemonic -- all
```

Expected output markers:
- `vectors`: «6 BINDING TEST VECTORS» секция, все с `byte-exact match true`; wordlist fingerprint verified
- `mnemonic "<24 words>"`: displayed 24 слова из canonical wordlist, master_seed 64 bytes displayed, 3 derived seeds (48B/48B/64B) displayed
- `keypair <hex32>`: entropy hex → mnemonic 24 words → master_seed → 3 roles seeds
- `roundtrip`: entropy → mnemonic → master_seed → mnemonic (roundtrip) → **same master_seed** idempotency proof
- `all`: запускает все 4 subcommands подряд; finale `[result] ALL M1 MNEMONIC: PASS`

Architect gate:
- 6 binding test vectors byte-exact match true
- 24 слова actual visible в output (не просто count)
- Wordlist fingerprint matches binding value
- Roundtrip idempotency verified
- Три per-role seeds distinct для same master_seed (HKDF domain separation works)

Critic adversarial checklist (применять с min 2 perspectives per CRITIC.md v1.3.0):

**Perspective — Cryptographer:**
- PBKDF2 iter = 2²⁰ actual (не меньший для performance)?
- HKDF info domain separation между three roles gives different seeds?
- Wordlist fingerprint tampering detected (mutate one word bit → verify fingerprint fails)?

**Perspective — Pen-tester:**
- Invalid mnemonic (word not in list, wrong checksum byte) → rejected с ясной ошибкой?
- Partial mnemonic (23 or 25 words) → rejected?
- mnemonic с valid words но invalid checksum (изменить последнее слово) → rejected?
- Unicode / whitespace normalization — два visually-identical inputs producing same master_seed OR rejected?

**Perspective — Production operator:**
- Error messages actionable (user знает как исправить)?
- Timing для 2²⁰ iterations acceptable (должно быть 0.5-2 секунды)?
- SK bytes **не** утекают в stdout (default behavior — redacted, M1_DUMP_SK=1 для opt-in, см. m1_crypto P2 external closure)?

### Сценарий 1 — M1 Crypto shakedown

Binary: `examples/m1_crypto.rs` (backfill — пишем первым в Validation Gate)

Что проверяем:
- SHA-256 FIPS 180-4 test vector: `hash("abc")` = `ba7816bf...` byte-exact
- FN-DSA-512 keypair + sign message + verify → OK; mutate signature byte → verify fail
- Merkle `empty_internal(256)` — 257 × 32B precomputed, derivation trace уровней 0-2
- Domain separation для всех R2 class domains (`mt-op`, `mt-nodereg`, `mt-proposal`, `mt-bundle`, ...) — разные hashes

Команды (subcommands actual binary; подкоманда `all` прогоняет всё подряд):
```bash
cd "/Users/kh./Python/Ничто/Montana/Russian/Протокол/Code" && cargo run --release --example m1_crypto -- hash
cd "/Users/kh./Python/Ничто/Montana/Russian/Протокол/Code" && cargo run --release --example m1_crypto -- sign "test"
cd "/Users/kh./Python/Ничто/Montana/Russian/Протокол/Code" && cargo run --release --example m1_crypto -- keypair 3
cd "/Users/kh./Python/Ничто/Montana/Russian/Протокол/Code" && cargo run --release --example m1_crypto -- merkle-empty
cd "/Users/kh./Python/Ничто/Montana/Russian/Протокол/Code" && cargo run --release --example m1_crypto -- all
```

Expected output markers (байт-exact проверяется mt_crypto::hash против FIPS 180-4):
- `hash`: секция «1. FIPS 180-4 §B.1 vector» строка `byte-exact match true`; секция «2. Empty-parts collapse» `equal true`; секция «3. Part concatenation» `equal true`; секция «4. Domain separation» — различные 32-байтовые хэши для ≥ 10 classes
- `sign`: секция «SIGN + VERIFY» — `signature 666 bytes`, `verify(mutated signature) false`, 4 adversarial теста все pass
- `keypair N`: N × (pubkey 897B, secret 1281B), `uniqueness N distinct pubkeys`
- `merkle-empty`: `TREE_DEPTH 256`, derivation trace уровни 0..2, finale `empty_internal(256) = <hex>`

Architect gate: `byte-exact match true` в FIPS vector секции; unique hashes для каждого R2 domain; empty_internal(256) deterministic; sign verify mutated → false.

Critic checklist:
- Запустить на Intel Mac + ARM Mac + Linux x86 — все hashes совпадают? (должны)
- Re-run 2 раза `sign "test"` — σ отличаются между запусками? (должно — FN-DSA-512 randomized sampler non-deterministic)
- Оба σ верифицируются? (должны — stability через signed_scope, Правило R2)
- `hash(b"mt-op", &[b""])` vs `hash(b"mt-op", &[])` — одинаковы? (должны — empty-parts collapse proof в output)
- `hash(b"mt-op", &[b"aa", b"bb"])` vs `hash(b"mt-op", &[b"aabb"])` — одинаковы? (должны — concat proof в output)
- Unique domain separation — все R2 classes производят разные hashes для того же input? (должны — см. секция 4)

### Сценарий 2 — M2 TimeChain + State shakedown

Binary: `examples/m2_timechain_state.rs`

Что проверяем:
- VDF forward: 1000 steps от genesis T_r_0, сверить T_r_1000 ручным пересчётом `SHA-256^1000`
- `next_d` boundary cases: median_permille 850, 851, 949, 950, 1000 — верно увеличивает/оставляет/уменьшает
- `cemented_bundle_aggregate`: window < 2 → zeros, empty set → `SHA-256("mt-bc-aggregate-empty" || W)`, non-empty → sorted node_ids aggregate
- State root composition: build AccountTable с 3 accounts + NodeTable 2 nodes + CandidatePool 1 → recompute state_root через `SHA-256("mt-state-root" || ...)` independently
- Merkle inclusion proof: один account → extract proof → verify against root

Команды:
```bash
cd "..." && cargo run --release --example m2_timechain_state -- vdf-forward 1000
cd "..." && cargo run --release --example m2_timechain_state -- next-d-boundaries
cd "..." && cargo run --release --example m2_timechain_state -- cba-branches
cd "..." && cargo run --release --example m2_timechain_state -- state-root-compose
cd "..." && cargo run --release --example m2_timechain_state -- merkle-inclusion
```

Architect gate: все outputs byte-exact совпадают с hand-computed reference.

Critic checklist:
- VDF backward — impossible? (должен — preimage resistance SHA-256)
- next_d at median=850 exact — decreases or unchanged? (per spec: ≤ low → decrease)
- cemented_bundle_aggregate с одинаковыми node_ids, разным порядком — same output? (должен — sorted по node_id)
- state_root с reordering accounts in BTreeMap — stable? (должен — BTreeMap sorted by key)
- Merkle proof на несуществующий key — absence proof returns leaf_value=empty, verify_proof returns true? (должен)

### Сценарий 3 — M3 Account operations

Binary: `examples/m3_account.rs`

Что проверяем:
- Scenario: создать bootstrap → accounts A, B (через TransferActivation от sponsor) → Transfer A→B → apply_proposal → state
- Observable: balance A decreased, B increased; Σ delta_balance = 0; frontier_hash updated; account_chain_length inc
- Emission: manually apply `reward_moneta(params) = EMISSION_moneta` (const 13 Ɉ) для нескольких окон; supply растёт линейно
- Genesis state materialization: build_genesis_state → check bootstrap account + node with chain_length=1
- DS-2 edge: attempt apply_proposal с lottery_weight = 0 (chain_length_snapshot=0) → winner_node panics? returns err?

Команды:
```bash
cd "..." && cargo run --release --example m3_account -- transfer-scenario
cd "..." && cargo run --release --example m3_account -- emission-schedule 0 10000 20000 60000
cd "..." && cargo run --release --example m3_account -- genesis-state
cd "..." && cargo run --release --example m3_account -- ds2-edge
```

Architect gate: Σ balance invariant после transfer; emission matches closed-form; genesis_state_root deterministic.

Critic checklist:
- Transfer A→B с amount > balance A — отклоняется? (InsufficientBalance)
- Transfer A→A — отклоняется? (SelfTransfer)
- Двойной OpenAccount с тем же pubkey → DuplicateAccount?
- Invalid prev_hash (не совпадает с frontier) → InvalidPrevHash?
- Signature от другого keypair → InvalidSignature?
- apply_proposal с одинаковым winner дважды (replay) → emission двойная? (нет — каждое W разный)

### Сценарий 4 — M4 Full consensus cycle (ключевой)

Binary: `examples/m4_local_net.rs --nodes 2 --windows 10`

Что проверяем: in-memory 2+ synthetic узла проходят 10 окон с полным protocol cycle:
- Каждое окно: узлы синтетически подписывают BCs окна W-1 (из testdata) → proposer_{W} = winner_{W-2} → собирает included_bundles + included_reveals → argmin winner → apply_proposal → cementing via quorum
- Printout per window: winner_id, winner_class (Node/Account), reward_moneta, state_root, active_chain_length, quorum
- После 10 окон: supply ledger (Σ rewards) == EMISSION_moneta × 10 — consistency

Команды:
```bash
cd "..." && cargo run --release --example m4_local_net -- --nodes 2 --windows 10
cd "..." && cargo run --release --example m4_local_net -- --nodes 2 --windows 10 --seed 42
cd "..." && cargo run --release --example m4_local_net -- --nodes 5 --windows 100 --verbose
```

Architect gate:
- Σ reward == EMISSION_moneta × N_windows (supply invariant)
- Все apply_proposal succeeded (no protocol violations)
- state_root по окнам — деterministic для given --seed
- Два запуска с --seed 42 → byte-exact same trace

Critic checklist:
- Если один узел не подписывает BC → остаётся в active set? (да, пока < 2τ₂)
- Если winner = offline узел → fallback на second_min? (proposer может timeout)
- Fallback cascade: reject proposal → second_min → rejected → third_min? (работает до available candidates)
- 67% quorum точно — что если ровно 67%? (`(67X+99)/100` дает ceiling — включительно проходит)
- Dependency rule: две операции от одного sender в одном окне — вторая отклоняется?
- argmin tie-breaking: два кандидата с одинаковым weighted_ticket — как определяется winner? (сравнение by node_id lex asc после ticket equality)

### Сценарий 5 — M5 Persistence + restart

Binary: `examples/m5_persist.rs`

Что проверяем:
- Run M4 scenario → close → reopen → state_root byte-exact совпадает
- Crash посреди apply_proposal (SIGKILL или std::process::exit) → reopen → state consistent (либо applied, либо not)
- Prune 50 oldest proposals → current state untouched, recent proposals accessible
- Concurrent read while writing (RocksDB snapshot): reader видит consistent view

Команды:
```bash
cd "..." && cargo run --release --example m5_persist -- restart-cycle 20
cd "..." && cargo run --release --example m5_persist -- crash-midcommit
cd "..." && cargo run --release --example m5_persist -- prune 50
```

Architect gate: state_root after restart == state_root before close; crash recovery replays partial commit cleanly; pruned proposals NotFound, current state intact.

Critic checklist:
- Corrupt RocksDB файл (e.g., truncate) → open returns error, не panic?
- Два процесса одновременно open одну базу → второй fails (RocksDB lock)?
- Disk full во время commit → WriteBatch atomic — либо all, либо nothing?
- Restart через 1 год simulated time — replay всех 13 млн окон работает за разумное время?

---

### Проведение Validation Gate — incremental workflow

На каждую сессию — **один сценарий**:

1. Автор открывает новую сессию репликой «начинаем Gate Scenario N» (или «пиши» если уже понятно).
2. Architect пишет `crates/mt-examples/examples/mN_name.rs` (новый binary для текущего сценария).
3. Architect даёт команды одной строкой + expected output pattern.
4. Автор запускает на своей машине, копирует output в чат.
5. Architect читает output построчно, сверяет с expected.
6. Критик задаёт adversarial checklist questions (5-10 вопросов из scenario-блока).
7. Автор: «всё работает как ожидал» либо флагует подозрительное.
8. Всё OK → architect обновляет status tracker выше: строка сценария → `✅ passed {дата}`, commit в ROADMAP.
9. Finding → fix (в binary или в core crate) → повторный прогон → retry.

**Next session trigger:** автор открывает новую сессию после passed предыдущего сценария.

**Критерий закрытия Validation Gate:** все 5 строк tracker-а = `✅ passed`. Только после этого — M6.

### Starter — с чего начать новую сессию

Первая Gate сессия (Scenario 1):
1. Автор: «начинаем Scenario 1 M1 Crypto» (или просто «пиши» после тега ROADMAP контекста)
2. Architect: создаёт `crates/mt-examples/` crate (Cargo.toml + пустой `src/lib.rs` или dummy), добавляет workspace members entry
3. Architect: пишет `crates/mt-examples/examples/m1_crypto.rs` с 4 subcommands (fips-abc, falcon-roundtrip, merkle-empty, domain-separation)
4. Architect: даёт команды автору
5. Manual run + разбор + критик → passed → next session (Scenario 2)

---

## Manual Validation Matrix M1-M5 (production-grade per-element)

Дополняет «Локальный shakedown» сценарии 0-5 (выше). Сценарии — happy-path scenario-by-scenario; матрица — **каждый винтик** на уровнях кода / протокола / пользователя / UX. Цель — production-grade проверка всех элементов перед external audit + перед запуском M6 сетевого слоя.

**Формат каждого пункта:**
```
[ ] Имя | file:line | spec ref | reproduce | expected | pass criterion
```

**Принципы:**
- M6+ network исключён (не реализован)
- Сценарии 0-5 дают smoke-pass; матрица детализирует — каждое test vector / invariant / API surface
- Pass criterion бинарный: byte-exact match / numerical equality / observable behavior — без «вроде работает»
- Зависимости: пункт может зависеть от предыдущих (например проверка transfer требует init готовый)

---

### Уровень 1 — Code primitives (M1 Foundational)

#### M1.1 mt-codec — canonical encoding + domain registry

| # | Элемент | file:line | Reproduce | Expected | Pass |
|---|---------|-----------|-----------|----------|------|
| 1.1.1 | `write_u8/u16/u32/u64/u128` LE | crates/mt-codec/src/lib.rs | `cargo test -p mt-codec` | LE byte order, no platform dependence | byte-exact match unit-tests |
| 1.1.2 | `write_bytes(buf, &[u8])` | mt-codec/src/lib.rs | round-trip property test | encode → decode = identity | property test passes ≥1000 cases |
| 1.1.3 | `CanonicalEncode` trait impls для всех consensus types | mt-codec/src/lib.rs + per-crate impl | `grep "impl CanonicalEncode"` workspace-wide | каждый consensus type implements | NodeRegistration, ProposalHeader, BundledConfirmation, VdfReveal, AccountRecord, NodeRecord, CandidateRecord — все present |
| 1.1.4 | Domain registry 32 константы prefix-free | mt-codec::domain | `grep "pub const " crates/mt-codec/src/lib.rs \| grep -i domain` | список 32 domains, ни один не префикс другого | prefix-free check: convention `mt-{component}[-{subcomponent}]` соблюдена для всех |
| 1.1.5 | `hash(domain, parts)` self-delimiting | mt-crypto::hash | manual reproduce: `python3 -c "import hashlib; print(hashlib.sha256(b'mt-proposal' + bytes([0]) + payload).hexdigest())"` vs Rust output | byte-exact SHA-256 match | byte-exact match для test inputs |
| 1.1.6 | `sha256_raw(bytes)` без domain | mt-crypto:sha256_raw | FIPS 180-4 §B.1 vector "abc" → ba7816bf… | byte-exact "abc" → standard digest | unit-test passes |

#### M1.2 mt-crypto — PQ crypto + secret types

| # | Элемент | file:line | Reproduce | Expected | Pass |
|---|---------|-----------|-----------|----------|------|
| 1.2.1 | `keypair_from_seed(seed)` ML-DSA-65 deterministic | mt-crypto/src/lib.rs | `cargo test -p mt-crypto-native --test nist_acvp_kat` | NIST FIPS 204 KAT byte-exact | 25/25 KeyGen vectors PASS |
| 1.2.2 | `sign(sk, msg)` deterministic ML-DSA-65 (FIPS 204 Algorithm 2) | mt-crypto::sign | NIST ACVP SigGen 15 cases tgId=3 | byte-exact deterministic signature | 15/15 SigGen vectors PASS |
| 1.2.3 | `verify(pk, msg, sig)` constant-time | mt-crypto::verify | sign+verify roundtrip + tamper test | accept good, reject mutated | unit-test PASS, no panic |
| 1.2.4 | `keypair_from_seed_mlkem` ML-KEM-768 | mt-crypto-native NIST KAT | `cargo test -p mt-crypto-native --test nist_acvp_kat` | FIPS 203 KAT byte-exact | 25/25 KeyGen vectors PASS |
| 1.2.5 | `SecretKey` heap+mlock | mt-crypto/src/lib.rs:130 | `cargo test -p mt-crypto --test security_invariants secret_key_is_heap_allocated` | size_of::<SecretKey>() = ptr (8B), не 4032 | invariant test PASS |
| 1.2.6 | `SecretKey: !Clone` | mt-crypto::SecretKey | trait check test | compile-error на try_clone() | compile-time enforce |
| 1.2.7 | `SecretKey: !PartialEq` (timing-leak protection) | mt-crypto::SecretKey | security_invariants test | no `==` operator | compile-time check PASS |
| 1.2.8 | `Drop+zeroize` для SecretKey/MlkemSecretKey | mt-crypto/src/lib.rs Drop impl | drop scope test | bytes zeroed после drop | unit-test verify post-drop memory |
| 1.2.9 | `mlock`/`munlock` 7 unsafe blocks с SAFETY | mt-crypto/src/lib.rs:164,192,235,276,293,364,383 | grep `// SAFETY:` | каждый unsafe имеет comment | 7/7 SAFETY comments present |
| 1.2.10 | No println on secret bytes | grep | `rg "println" crates/mt-crypto/src` | нет matches на SK fields | grep clean |

#### M1.3 mt-crypto-native — FFI к OpenSSL EVP

| # | Элемент | file:line | Reproduce | Expected | Pass |
|---|---------|-----------|-----------|----------|------|
| 1.3.1 | `mt_keypair_from_seed_mldsa` C wrapper | csrc/mt_crypto.c | NIST KAT differential | OpenSSL EVP_PKEY_KEYGEN с `OSSL_PKEY_PARAM_ML_DSA_SEED` | byte-exact NIST FIPS 204 |
| 1.3.2 | `mt_sign_mldsa` deterministic | csrc/mt_crypto.c | ACVP SigGen с `OSSL_SIGNATURE_PARAM_DETERMINISTIC=1` | FIPS 204 Algorithm 2 deterministic | 15/15 PASS |
| 1.3.3 | `mt_sign_mldsa_ctx` (FIPS context support) | csrc/mt_crypto.c | empty ctx == ctx-equivalence | один test case | 1/1 PASS |
| 1.3.4 | OpenSSL 3.5.5 LTS pinned | Cargo.toml | `cargo tree \| grep openssl-src` | =300.5.5+3.5.5 exact pin | exact version match |
| 1.3.5 | Reproducible builds через build.rs | crates/mt-crypto-native/build.rs | `docker build -f docker/release-build.dockerfile` two passes | byte-identical binaries | sha256 binary 1 == sha256 binary 2 |

#### M1.4 mt-mnemonic — 24-word recovery flow

| # | Элемент | file:line | Reproduce | Expected | Pass |
|---|---------|-----------|-----------|----------|------|
| 1.4.1 | `entropy_to_mnemonic([u8; 32])` | mt-mnemonic/src/mnemonic.rs | KAT vectors test | byte-exact 24 words from BIP-39 wordlist | 5/5 KAT PASS |
| 1.4.2 | `mnemonic_to_master_seed("...")` PBKDF2 iter=2²⁰ | mt-mnemonic/src/pbkdf2.rs | RFC 7914 §11 / RFC 6070 vectors | byte-exact 64B master_seed | RFC vectors PASS |
| 1.4.3 | `mldsa_seed_for_role(seed, domain)` HKDF | mt-mnemonic/src/hkdf.rs | RFC 5869 §A.1 | byte-exact 32B per-role seed | RFC vectors PASS |
| 1.4.4 | `mlkem_seed_for_role(seed, domain)` 64B output | mt-mnemonic/src/hkdf.rs | KAT test | byte-exact 64B (d ‖ z FIPS 203 §6.1) | unit-test PASS |
| 1.4.5 | HMAC-SHA-256 RFC 4231 | mt-mnemonic/src/hmac.rs | RFC 4231 cases 1-7 | byte-exact MAC | 7/7 PASS |
| 1.4.6 | Wordlist binding SHA-256 | mt-mnemonic/src/wordlist.rs | hash binding test | sha256(wordlist file) match expected | hash match |
| 1.4.7 | Whitespace-tolerant parsing (F-12 closure) | mt-mnemonic/src/mnemonic.rs:120 | `init --mnemonic "  word1  word2 ..."` | accepts irregular whitespace | regression test PASS |
| 1.4.8 | End-to-end recovery determinism | mt-mnemonic/tests/e2e_recovery.rs | `cargo test -p mt-mnemonic --test e2e_recovery` | mnemonic → identity → mnemonic = identity | 1/1 PASS |

---

### Уровень 2 — State foundation (M2)

#### M2.1 mt-merkle — Sparse Merkle Tree depth 256

| # | Элемент | file:line | Reproduce | Expected | Pass |
|---|---------|-----------|-----------|----------|------|
| 2.1.1 | `leaf_hash(serialized)` domain-separated | mt-merkle/src/lib.rs:13 | `python3 -c "import hashlib; print(hashlib.sha256(b'mt-merkle-leaf' + bytes([0]) + bytes).hexdigest())"` | byte-exact SHA-256 | external oracle match |
| 2.1.2 | `internal_hash(left, right)` order-sensitive | mt-merkle/src/lib.rs | swap left↔right test | hash(L,R) ≠ hash(R,L) | order-sensitivity PASS |
| 2.1.3 | `empty_internal(k)` precomputed cache levels 0-256 | mt-merkle/src/lib.rs OnceLock | manual shasum levels 0,1,2,3 | recursive `H(empty_internal(k-1), empty_internal(k-1))` | 4/4 manual computations match |
| 2.1.4 | `SparseMerkleTree::insert(key, value)` | mt-merkle/src/lib.rs | insertion-order independence test | вставка в разном порядке → одинаковый root | order-independence PASS |
| 2.1.5 | `SparseMerkleTree::root()` BTreeMap canonical | mt-merkle::SparseMerkleTree | determinism_invariants tests | byte-exact root для разного insertion order | 10/10 invariants PASS |
| 2.1.6 | `verify_proof(root, proof)` inclusion | mt-merkle::verify_proof | construct proof, verify | true для valid, false для tampered | 2/2 cases PASS |
| 2.1.7 | `verify_proof` absence (empty leaf) | mt-merkle::verify_proof | proof for non-existent key | true (absence verified) | absence proof PASS |
| 2.1.8 | TREE_DEPTH = 256 | mt-merkle::TREE_DEPTH | const check | == 256 | static assertion |
| 2.1.9 | InclusionProof MAX_LEAF_VALUE_SIZE = 4096 (P10-1) | mt-merkle/src/lib.rs:285 | encode test с oversized leaf | debug_assert panic | bound enforced |
| 2.1.10 | InclusionProof MAX_SIBLINGS = TREE_DEPTH | mt-merkle::MAX_SIBLINGS | const | == 256 | static |

#### M2.2 mt-genesis — ProtocolParams SSOT

| # | Элемент | file:line | Reproduce | Expected | Pass |
|---|---------|-----------|-----------|----------|------|
| 2.2.1 | `genesis_params()` singleton OnceLock | mt-genesis/src/lib.rs | повторные вызовы → same Arc ptr | стабильный pointer | unit-test PASS |
| 2.2.2 | `ProtocolParams` encoded size 4094B | mt-genesis::PARAMS_ENCODED_SIZE | `assert_eq!(p.encode().len(), 4094)` | byte-exact 4094B | static |
| 2.2.3 | `emission_moneta = 13 × 10⁹ nɈ` | mt-genesis::ProtocolParams | const check | == 13_000_000_000 | LE bytes [0x00, 0xc8, 0x4d, 0x21, 3, 0, 0, 0] |
| 2.2.4 | `d0 = 325_000_000` | mt-genesis::ProtocolParams | runtime calibration on M-class Mac | окно ≈ 60s wall-clock | empirical 60-65s |
| 2.2.5 | `tau2_windows = 20160` | mt-genesis::ProtocolParams | const | == 20160 (~14 days) | static |
| 2.2.6 | `selection_interval` = 336 | mt-genesis::ProtocolParams | const | == 336 (1/60 of τ₂) | static |
| 2.2.7 | `bootstrap_node_pubkey` placeholder | mt-genesis::ProtocolParams | byte check | == [0u8; 1952] (pre-ceremony) | static; post-ceremony updated |
| 2.2.8 | `genesis_app_id()` SHA-256 domain-separated | mt-genesis/src/lib.rs | manual recompute | byte-exact hash | unit-test PASS |
| 2.2.9 | `compute_genesis_state_hash()` determinism | mt-genesis | повторные вызовы | byte-exact same hash | determinism_invariants 7/7 |
| 2.2.10 | `is_genesis_bootstrap_finalized(params)` | mt-genesis | predicate test | placeholder zeros → false | static check |

#### M2.3 mt-state — AccountTable / NodeTable / CandidatePool

| # | Элемент | file:line | Reproduce | Expected | Pass |
|---|---------|-----------|-----------|----------|------|
| 2.3.1 | `AccountRecord` 2059B byte-exact | mt-state/src/lib.rs:25 | `assert_eq!(record.encode().len(), ACCOUNT_RECORD_SIZE)` | == 2059 | static |
| 2.3.2 | `NodeRecord` 2098B byte-exact | mt-state/src/lib.rs:59 | encode size | == 2098 | static |
| 2.3.3 | `CandidateRecord` 2082B byte-exact | mt-state/src/lib.rs | encode size | == 2082 | static |
| 2.3.4 | `derive_account_id(suite_id, pk)` SHA-256 domain | mt-state::derive_account_id | manual recompute via Python | byte-exact | external oracle PASS |
| 2.3.5 | `derive_node_id(pk)` distinct from account_id | mt-state::derive_node_id | derive both for same pk | account_id ≠ node_id | cross-distinctness PASS |
| 2.3.6 | `AccountTable::insert/get` BTreeMap | mt-state::AccountTable | order-independence test | root hash same regardless insertion order | invariant PASS |
| 2.3.7 | `NodeTable::insert/contains` | mt-state::NodeTable | duplicate check | second insert returns false / errors | unit-test PASS |
| 2.3.8 | `CandidatePool::insert/remove/iter` | mt-state::CandidatePool | TTL-based pruning | apply_candidate_expiry удаляет expired | scenario test |
| 2.3.9 | `compute_state_root(node, candidate, account)` | mt-state::compute_state_root | manual SHA-256 composition | byte-exact tree-of-three | external oracle PASS |
| 2.3.10 | `is_active(node, current_window)` predicate | mt-state::is_active | boundary at 2×τ₂ inclusive test | true within window, false outside | boundary tests PASS |

#### M2.4 mt-timechain — VDF + adaptive D + cba

| # | Элемент | file:line | Reproduce | Expected | Pass |
|---|---------|-----------|-----------|----------|------|
| 2.4.1 | `vdf_step(prev, d)` SHA-256^d associative | mt-timechain/src/lib.rs:13 | composition test: `vdf_step(vdf_step(x, a), b) == vdf_step(x, a+b)` | associativity holds | invariant PASS |
| 2.4.2 | `vdf_step(prev, 0) == prev` identity | mt-timechain | zero-iteration check | output == input | unit-test PASS |
| 2.4.3 | `vdf_verify(prev, expected, d)` | mt-timechain::vdf_verify | sign correct + tamper | accept correct, reject wrong claim | 2/2 cases PASS |
| 2.4.4 | `next_d(current, median_permille, params)` adaptive | mt-timechain/src/lib.rs:40-89 | dead-zone (median == 1000): no change; high (>1030): +3%; low (<970): -3% | integer permille arithmetic | 3/3 boundary tests PASS |
| 2.4.5 | `next_d` overflow protection | mt-timechain/src/lib.rs:51,65 | extreme D close to u64::MAX | panic с descriptive message | overflow caught (~25_000 epochs) |
| 2.4.6 | `cemented_bundle_aggregate(W, &[])` empty marker | mt-timechain::cba | window=0,1 → genesis zeros | byte-exact `[0u8; 32]` | static |
| 2.4.7 | `cemented_bundle_aggregate(W, [node_ids])` canonical sort | mt-timechain | input order independence test | byte-exact для разного порядка | [I-8] invariant PASS |
| 2.4.8 | `cemented_bundle_aggregate` distinct empty vs non-empty | mt-timechain | empty `[]` vs `[node_id]` | разные hashes | distinctness PASS |
| 2.4.9 | TimeChain VDF на genesis узле тикает per окно | montana-node start | `montana-node start --max-windows 2 && status` | T_r изменился, current_window инкрементировано на 2 | observable PASS |

---

### Уровень 3 — apply_proposal (M3 mt-account)

| # | Элемент | file:line | Reproduce | Expected | Pass |
|---|---------|-----------|-----------|----------|------|
| 3.1 | Transfer 0x02 layout (TRANSFER_SIZE) | mt-account/src/lib.rs | encode size check | byte-exact spec | static const |
| 3.2 | `validate_transfer` signature + balance + chain_length | mt-account::validate_transfer | tests/transfer_*.rs | reject bad sig / insufficient balance / non-monotone chain | reject all 3 cases |
| 3.3 | `apply_transfer` atomic: balance debited, credited, frontier_hash bound | mt-account::apply_transfer | unit-test | sender.balance -= amount, receiver.balance += amount, frontier_hash = op_hash | invariant PASS |
| 3.4 | ChangeKey 0x03 layout | mt-account::CHANGE_KEY_SIZE | encode size | byte-exact spec | static const |
| 3.5 | `validate_change_key` signature под старым ключом | mt-account::validate_change_key | tamper test | new sig under old key required | unit-test PASS |
| 3.6 | `apply_change_key` rotates current_pubkey | mt-account::apply_change_key | before/after | account.current_pubkey == new_pk, suite_id updated | unit-test PASS |
| 3.7 | Anchor 0x04 layout | mt-account::ANCHOR_SIZE | encode size | byte-exact spec | static const |
| 3.8 | `validate_anchor` app_id + data_hash binding | mt-account::validate_anchor | tests | data_hash 32B fixed, app_id present | unit-test PASS |
| 3.9 | `apply_anchor` AccountChain entry, не меняет balance | mt-account::apply_anchor | balance check | balance unchanged, chain_length += 1 | unit-test PASS |
| 3.10 | TransferActivation 0x0A sponsor pattern | mt-account::TRANSFER_ACTIVATION_SIZE | encode size | byte-exact spec | static const |
| 3.11 | `apply_transfer_activation` создаёт новый AccountRecord | mt-account::apply_transfer_activation | before/after AccountTable | new account inserted, balance=amount, is_node_operator=false | unit-test PASS |
| 3.12 | `op_hash(operation)` SHA-256("mt-op" \|\| signed_scope) | mt-account/src/lib.rs | manual recompute | byte-exact | external oracle PASS |
| 3.13 | `settle_window(cemented_ops, W)` сортировка по op_hash | mt-account::settle_window | unsorted input | output sorted ascending op_hash | invariant PASS |
| 3.14 | `apply_proposal` Step 2 (emission), 3.5 (chain_length++), 3.6 (checkpoint), 4 (state_root) | mt-account::apply_proposal | per-window in start.rs | balance += emission_moneta, node.chain_length += 1, state_root updated | observable balance growth |
| 3.15 | `apply_emission(winner_account, params)` const 13 Ɉ | mt-account::apply_emission | один tick | account.balance += 13_000_000_000 nɈ | observable PASS |
| 3.16 | `supply_moneta(W, params) = emission × (W+1)` closed-form | mt-account::supply_moneta | различные W | linear formula | unit-test PASS |
| 3.17 | `build_genesis_state(params)` bootstrap | mt-account::build_genesis_state | call + encode | byte-exact deterministic state | invariant PASS |
| 3.18 | `genesis_state_root(params)` byte-exact | mt-account::genesis_state_root | manual recompute | == compute_state_root результат | external oracle PASS |
| 3.19 | Conservation: Σ delta_balance == 0 per Transfer/Activation | mt-account::apply_* | property test | sum unchanged | invariant PASS |
| 3.20 | checked_sub on balance underflow → panic с protocol invariant breach | mt-account::apply_transfer | underflow case | panic с descriptive message | controlled halt |

---

### Уровень 4 — Consensus mechanics (M4)

#### M4.1 mt-lottery — BundledConfirmation + VdfReveal + lottery

| # | Элемент | file:line | Reproduce | Expected | Pass |
|---|---------|-----------|-----------|----------|------|
| 4.1.1 | `BundledConfirmation::encode_signed_scope` byte-exact | mt-lottery/src/lib.rs:34 | size check | fixed overhead 76B + var hashes | static |
| 4.1.2 | `bundle_hash(bc)` SHA-256("mt-bundle" \|\| scope) | mt-lottery/src/lib.rs:69 | external oracle | byte-exact match | external PASS |
| 4.1.3 | `validate_bundle` rules a-d | mt-lottery/src/lib.rs:102 | per-rule rejection tests | UnknownNode/UnsupportedSuite/WrongEndpoint/OpsOutOfOrder/RevealsOutOfOrder/InvalidSignature | 6/6 reject paths PASS |
| 4.1.4 | `validate_bundle` u16 cap (M4-1 closure) | mt-lottery/src/lib.rs:120 | op_hashes.len() > u16::MAX → TooManyOps | reject before signature verify | functional test PASS |
| 4.1.5 | op_hashes ascending strictly | mt-lottery::is_strictly_ascending | unsorted input → reject | OpsOutOfOrder error | unit-test PASS |
| 4.1.6 | `VdfReveal` layout 32+8+32+SIG bytes | mt-lottery::REVEAL_SIZE | size check | == REVEAL_SIZE const | static |
| 4.1.7 | `reveal_hash(reveal)` SHA-256("mt-vdf-reveal" \|\| scope) | mt-lottery::reveal_hash | external oracle | byte-exact | external PASS |
| 4.1.8 | `validate_reveal` (5 rules) | mt-lottery::validate_reveal | per-rule reject | similar to bundle | unit-test PASS |
| 4.1.9 | `compute_endpoint(T_r, cba(W-2), my_node, W)` lottery formula с [I-8] binding | mt-lottery::compute_endpoint | manual SHA-256 composition | byte-exact, [I-8] satisfied | external oracle PASS |
| 4.1.10 | `log2_q64(x_q64)` Q64.64 polynomial | mt-lottery/src/lib.rs (B0..B3 coefficients) | reference Python implementation | max error 2^-10.62 | precision match |
| 4.1.11 | `ln_q64(x_q64)` derived from log2_q64 | mt-lottery::ln_q64 | reference test | byte-exact match | unit-test PASS |
| 4.1.12 | `weighted_ticket_node(endpoint, w, snapshot)` u128 integer division | mt-lottery::weighted_ticket_node | various w | deterministic ticket | determinism_invariants 34/34 |
| 4.1.13 | `lottery_weight(chain_length, snapshot)` integer | mt-lottery::lottery_weight | boundary chain_length | monotone non-decreasing | unit-test PASS |
| 4.1.14 | `seniority_bonus` integer formula | mt-lottery::seniority_bonus | various inputs | spec-exact | unit-test PASS |
| 4.1.15 | `determine_winner` argmin (ticket asc, class asc, id lex asc) | mt-lottery::determine_winner | tie cases | canonical ordering rule | tie-breaker tests PASS |
| 4.1.16 | `quorum(active_chain_length)` 67% ceiling `(67×X+99)/100` | mt-lottery::quorum | various X | integer ceiling formula | spec-binding PASS |
| 4.1.17 | `is_cemented(c, active)` predicate | mt-lottery::is_cemented | boundary cases | true if c >= quorum(active) | boundary tests PASS |

#### M4.2 mt-consensus — ProposalHeader R1/R2

| # | Элемент | file:line | Reproduce | Expected | Pass |
|---|---------|-----------|-----------|----------|------|
| 4.2.1 | `ProposalHeader` 3722B fixed (17 полей) | mt-consensus/src/lib.rs:33 | encode size | == PROPOSAL_HEADER_SIZE = 3722 | static |
| 4.2.2 | `encode_signed_scope` 413B (без подписи) | mt-consensus/src/lib.rs | encode_signed_scope().len() | == 413 | static computation |
| 4.2.3 | `proposal_hash(header)` SHA-256("mt-proposal" \|\| scope) | mt-consensus::proposal_hash | external oracle | byte-exact | external PASS |
| 4.2.4 | `validate_header` rules a-f | mt-consensus::validate_header | per-rule tests | fallback_depth ≥ 1, window_index = prev+1, protocol_version monotone, proposer registered, suite Mldsa65, sig verify | 6/6 reject paths PASS |
| 4.2.5 | `target` поле u128 LE 16B (P5 closure) | mt-consensus::ProposalHeader | encode field | u128 16 bytes LE | static |
| 4.2.6 | `canonical_proposer(W)` Lookback Leadership | mt-consensus::canonical_proposer | proposer_W = winner_{W-2} | spec-exact | unit-test PASS |
| 4.2.7 | `fallback_proposer(depth)` cascade | mt-consensus::fallback_proposer | depth 1, 2, 3 | byzantine-safe ordering | unit-test PASS |
| 4.2.8 | `compute_control_set(headers)` filter + sort | mt-consensus::compute_control_set | unordered input | sorted by (window asc, op_hash lex asc) | invariant PASS |
| 4.2.9 | `validate_proposer_is_canonical` | mt-consensus | mismatched proposer | reject | unit-test PASS |
| 4.2.10 | `finalization_status(header)` Cemented/Rejected | mt-consensus::finalization_status | various confirmer counts | per is_cemented | unit-test PASS |

#### M4.3 mt-entry — NodeRegistration + Selection

| # | Элемент | file:line | Reproduce | Expected | Pass |
|---|---------|-----------|-----------|----------|------|
| 4.3.1 | `NodeRegistration` 5344B fixed | mt-entry/src/lib.rs | encode size | == NODE_REGISTRATION_SIZE | static |
| 4.3.2 | `nodereg_hash` SHA-256("mt-nodereg" \|\| scope) | mt-entry::nodereg_hash | external oracle | byte-exact | external PASS |
| 4.3.3 | `validate_noderegistration` 3 rules | mt-entry::validate_noderegistration | per-rule reject | suite supported, sig verify, node_id unique, operator account exists+!is_node_operator | 4/4 reject paths PASS |
| 4.3.4 | `candidate_vdf_init([T_r, cba(W-2), node_id])` [I-8] | mt-entry::candidate_vdf_init | external oracle | byte-exact composition | external PASS |
| 4.3.5 | `compute_expiry_window(W)` 3τ₂ | mt-entry::compute_expiry_window | various W | == W + 3 × 20160 | static formula |
| 4.3.6 | `apply_candidate_expiry(pool, W)` removes expired | mt-entry::apply_candidate_expiry | per-window call | pool.len() decreases | observable PASS |
| 4.3.7 | `selection_slots` 1% admission cap (ADMISSION_DIVISOR=130) | mt-entry::selection_slots | active_chain_length 13000 → 100 slots | per-formula | spec-binding PASS |
| 4.3.8 | `selection_sort_key` SHA-256("mt-selection" \|\| ...) | mt-entry::selection_sort_key | external oracle | byte-exact | external PASS |
| 4.3.9 | `rank_candidates_for_selection` canonical sort | mt-entry::rank_candidates_for_selection | unsorted input | sorted by sort_key | invariant PASS |
| 4.3.10 | `apply_selection_event(pool, nodes, accounts, W, params)` | mt-entry::apply_selection_event | candidates ready | inserted into NodeTable, removed from pool, operator account flagged | observable PASS |
| 4.3.11 | `apply_noderegistrations_batch` incremental sort+apply | mt-entry::apply_noderegistrations_batch | per-window | applied count == 1 if vdf_chain_length ≥ τ₂ | observable PASS |
| 4.3.12 | `required_vdf_length(pending, active, τ₂)` integer permille | mt-entry::required_vdf_length | pending=0,active=0 → tau2_windows | spec-binding | unit-test PASS |

---

### Уровень 5 — Persistence (M5 mt-store)

| # | Элемент | file:line | Reproduce | Expected | Pass |
|---|---------|-----------|-----------|----------|------|
| 5.1 | `FsStore::open(data_dir)` создаёт proposals/ | mt-store/src/lib.rs | удалить proposals/, открыть store | proposals/ создан | observable PASS |
| 5.2 | `cleanup_orphan_tmp` (M5-LOW-8 closure) | mt-store/src/lib.rs:67 | placement orphan .tmp file | удалён при open | regression test PASS |
| 5.3 | `save_account_table` + `load_account_table` round-trip | mt-store/src/lib.rs:271-281 | save → load | byte-exact decode | round-trip PASS |
| 5.4 | `save_node_table` + `load_node_table` round-trip | mt-store/src/lib.rs:301-311 | save → load | byte-exact decode | round-trip PASS |
| 5.5 | `save_candidate_pool` + `load_candidate_pool` round-trip | mt-store/src/lib.rs:331-341 | save → load | byte-exact decode | round-trip PASS |
| 5.6 | `archive_proposal(header)` создаёт `proposals/{W:020}.bin` | mt-store::archive_proposal | per-window | файл 3722B на диске | `ls data/proposals/` |
| 5.7 | `get_proposal_by_window(W)` byte-exact decode | mt-store::get_proposal_by_window | archived header | recovered identical | round-trip PASS |
| 5.8 | `save_meta_last_cemented(W)` u64 LE | mt-store::save_meta_last_cemented | per-window | meta_last_cemented.bin = 8B u64 LE | hexdump check |
| 5.9 | `verify_consistency()` crash recovery | mt-store::verify_consistency | удалить proposals/ файл, оставить meta_last_cemented | StoreError::NotFound | error path PASS |
| 5.10 | `prune_proposals_before(window)` | mt-store::prune_proposals_before | call с threshold | удаляет файлы window < threshold | filesystem state |
| 5.11 | Atomic rename pattern (tempfile → rename) | mt-store/src/lib.rs:87-114 | crash mid-write simulation | partial files либо clean state | atomicity PASS |
| 5.12 | `decode_X` size validation перед read | mt-store/src/lib.rs | corrupted file (wrong size) | StoreError::CorruptedLength | error path PASS |

---

### Уровень 6 — Integration layer (montana-node CLI)

#### N1. Identity management commands

| # | Команда | Reproduce | Expected | Pass |
|---|---------|-----------|----------|------|
| 6.1 | `montana-node init` (random) | `./montana-node init` | 24 words displayed in 4×6 grid + identity.bin (15627B) | 24-word grid + file size byte-exact |
| 6.2 | `montana-node init --mnemonic "..."` | recovery from words | byte-exact same identity as original | identity match |
| 6.3 | `montana-node init --entropy <hex32>` | deterministic from hex | byte-exact same on re-run | determinism PASS |
| 6.4 | `montana-node init --force` overwrite | second init без --force отклонён, с --force OK | IdentityFileExists error без --force | error path PASS |
| 6.5 | `montana-node inspect` (default) | без --reveal-master-seed | fingerprint only, no master_seed | observable PASS |
| 6.6 | `montana-node inspect --reveal-master-seed` | флаг present | master_seed shown (64 hex) | observable PASS |
| 6.7 | Recovery determinism cross-machine | machine 1 init → mnemonic; machine 2 init --mnemonic | identical account_id, node_id | byte-exact match |

#### N2. Node lifecycle commands

| # | Команда | Reproduce | Expected | Pass |
|---|---------|-----------|----------|------|
| 6.8 | `montana-node start --max-windows 2` | finite run | 2 windows processed, balance += 26 Ɉ, exit 0 | observable PASS |
| 6.9 | `montana-node start --d-test-override 1000000` | TEST-ONLY override | warning printed, fast windows | observable PASS |
| 6.10 | `montana-node status` | AccountTable + NodeTable + CandidatePool dump | Σ balance, supply, phase, current_window correct | observable PASS |
| 6.11 | `montana-node time` | window timing info | current_window, ближайший selection (W % 336==0), эпоха τ₂ | static check |
| 6.12 | `montana-node help` / `--help` / `-h` | help text | full usage + flag descriptions | text match |
| 6.13 | Exit codes | invalid args → exit 2; ok → exit 0; runtime err → exit 1 | per-spec | exit code check |
| 6.14 | LocalNodeError variants Display | trigger каждую ошибку | typed error message в stderr | manual cases |

#### N3. Phase transitions auto-detection

| # | Сценарий | file:line | Reproduce | Expected | Pass |
|---|----------|-----------|-----------|----------|------|
| 6.15 | Genesis bootstrap (placeholder zeros) | node_lifecycle.rs:70 is_bootstrap_node | `cargo run -p montana-node start --max-windows 1` (свежий узел) | phase = Active immediately, NodeTable содержит self | observable PASS |
| 6.16 | Candidate path (post-ceremony, mismatch pubkey) | node_lifecycle.rs:78 | finalized bootstrap_node_pubkey != identity.node_pk | phase = Bootstrap → CandidateVdf на первом окне | DEV-010 logic PASS (не тестируется до ceremony) |
| 6.17 | DEV-010 status | docs/SPEC_DEVIATIONS.md:172 | grep Status | acknowledged auto-detection | static |

#### N4. State migration

| # | Сценарий | file:line | Reproduce | Expected | Pass |
|---|----------|-----------|-----------|----------|------|
| 6.18 | current_window.bin v0 → v1 auto-upgrade | clock.rs:31-58 | вручную создать 8B legacy file → start узел → check size | next save → 16B v1 формат | round-trip PASS |
| 6.19 | identity.bin magic+version validation | identity.rs:226-231 | tamper magic byte → load_identity | InvalidMagic error | error path PASS |
| 6.20 | timechain.bin magic+version | timechain_state.rs:47-52 | tamper magic | InvalidMagic | error path PASS |
| 6.21 | node_state.bin magic+version | node_lifecycle.rs:82-87 | tamper magic | InvalidMagic | error path PASS |

---

### Уровень 7 — Operator surface (UX/UI)

#### O1. launchd integration (macOS)

| # | Элемент | Reproduce | Expected | Pass |
|---|---------|-----------|----------|------|
| 7.1 | plist Label = "org.montana.node" | `cat ~/Library/LaunchAgents/org.montana.node.plist \| grep Label` | production reverse-domain (no `dev.*`/`local.*`) | static check |
| 7.2 | RunAtLoad=true | plist | автозапуск при логине | login test |
| 7.3 | KeepAlive Crashed=true | plist | restart on crash | `kill -9 <PID>`, узел restart-ит через ThrottleInterval=10 | restart observable |
| 7.4 | StandardOutPath/StandardErrorPath | plist | logs в `data/logs/` | `tail -F data/logs/montana.log` | observable |
| 7.5 | SIGTERM graceful shutdown | start.rs:31-33 + 545-546 | `launchctl unload -w plist` | узел сохраняет state, exit clean | clean shutdown |
| 7.6 | ThrottleInterval=10 rate limit | plist | crash несколько раз | минимум 10s между restart | observable timing |

#### O2. .command файлы

| # | Файл | Reproduce | Expected | Pass |
|---|------|-----------|----------|------|
| 7.7 | `1. Запуск и логи узла.command` | дабл-клик в Finder | Terminal открывается, status + tail -F | observable open Terminal |
| 7.8 | `2. Остановить узел.command` | дабл-клик | launchctl unload, "ГОТОВО" message | узел остановлен |
| 7.9 | Relative `$DIR` paths (не hardcoded) | grep "$HOME/Applications" в .command | пусто | path-portable PASS |

#### O3. Install scripts

| # | Скрипт | Reproduce | Expected | Pass |
|---|--------|-----------|----------|------|
| 7.10 | `scripts/install-local-mac.sh` | `bash scripts/install-local-mac.sh` (свежая система) | build + install + identity init + launchd setup | full e2e install |
| 7.11 | `scripts/install-mac.sh` | `curl ... \| bash` | git clone + build + install | full e2e install |
| 7.12 | `scripts/install-vps.sh` (Linux) | `bash scripts/install-vps.sh` Ubuntu/Debian | systemd setup + start | systemctl status active |
| 7.13 | `INSTALL_DIR` env var override | `INSTALL_DIR=/custom/path bash scripts/install-local-mac.sh` | install в custom location | observable PASS |
| 7.14 | `dist/macOS/install.command` | дабл-клик | full install + open Finder | GUI flow PASS |

#### O4. State files inventory

| # | Файл | Path | Expected size/format | Pass |
|---|------|------|---------------------|------|
| 7.15 | `identity.bin` | `data/identity.bin` | 15627B fixed, magic="mt-local", version=1 | size + magic byte-exact |
| 7.16 | `accounts.bin` | `data/accounts.bin` | N × 2059B (растёт с TransferActivation) | size кратен 2059 |
| 7.17 | `nodes.bin` | `data/nodes.bin` | N × 2098B | size кратен 2098 |
| 7.18 | `candidates.bin` | `data/candidates.bin` | N × 2082B | size кратен 2082 |
| 7.19 | `meta/current_window.bin` | `data/meta/current_window.bin` | 16B v1 (либо 8B v0 legacy) | size 16 либо 8 |
| 7.20 | `meta/timechain.bin` | `data/meta/timechain.bin` | magic "mttc" + version + T_r + D + last_window | hexdump magic |
| 7.21 | `meta/node_state.bin` | `data/meta/node_state.bin` | magic "mtns" + version + lifecycle fields | hexdump magic |
| 7.22 | `meta_last_cemented.bin` | `data/meta_last_cemented.bin` | 8B u64 LE (либо absent если pre-genesis) | hexdump |
| 7.23 | `proposals/{window:020}.bin` | `data/proposals/00000000000000000001.bin` | каждый файл 3722B | size byte-exact |
| 7.24 | `logs/montana.log` + `montana.err.log` | `data/logs/` | log files (launchd redirect) | files present |

#### O5. Crash recovery

| # | Сценарий | Reproduce | Expected | Pass |
|---|----------|-----------|----------|------|
| 7.25 | Power loss mid-write (atomic rename) | start узел, `pkill -9 montana-node` во время save_progress | partial .tmp файл либо clean state, не corrupted bin | atomicity PASS |
| 7.26 | SIGKILL → reload state | `kill -9 <PID>` → restart | continues с last cemented window (state preserved) | observable continuity |
| 7.27 | Corrupted file detection | tamper data/accounts.bin (truncate) | StoreError::CorruptedLength на load | error path PASS |
| 7.28 | meta_last_cemented vs proposals/ inconsistency | удалить proposals/<W>.bin, оставить meta_last_cemented | StoreError::NotFound, не silent skip | verify_consistency PASS |
| 7.29 | Re-open после crash valid state | crash → restart | узел продолжает с того окна, balance preserved | observable PASS |

---

### Уровень 8 — Conformance & test vectors

#### C1. NIST KAT (cross-implementation conformance)

| # | Vector | Source | Reproduce | Pass |
|---|--------|--------|-----------|------|
| 8.1 | ML-DSA-65 KeyGen 25 cases | NIST ACVP-Server `ML-DSA-keyGen-FIPS204` | `cargo test -p mt-crypto-native --test nist_acvp_kat` | 25/25 byte-exact |
| 8.2 | ML-DSA-65 SigGen 15 cases (1 empty + 14 ctx) | NIST ACVP-Server `ML-DSA-sigGen-FIPS204` tgId=3 | same test | 15/15 byte-exact |
| 8.3 | ML-KEM-768 KeyGen 25 cases | NIST ACVP-Server `ML-KEM-keyGen-FIPS203` | same test | 25/25 byte-exact |
| 8.4 | mt_sign_mldsa ≡ mt_sign_mldsa_ctx (empty ctx) | own correctness check | same test | 1/1 PASS |

#### C2. RFC test vectors

| # | Vector | Source | Pass |
|---|--------|--------|------|
| 8.5 | SHA-256 "abc" → ba7816bf...15ad | FIPS 180-4 §B.1 | unit-test PASS |
| 8.6 | HMAC-SHA-256 cases 1-7 | RFC 4231 | 7/7 PASS |
| 8.7 | HKDF-Expand vectors | RFC 5869 §A.1 | PASS |
| 8.8 | PBKDF2 vectors | RFC 7914 §11 / RFC 6070 | PASS |

#### C3. Determinism invariants (191 automated total)

| # | Crate | Test count | Reproduce | Pass |
|---|-------|------------|-----------|------|
| 8.9 | mt-merkle | 10 | `cargo test -p mt-merkle --test determinism_invariants` | 10/10 |
| 8.10 | mt-genesis | 7 | `cargo test -p mt-genesis --test determinism_invariants` | 7/7 |
| 8.11 | mt-state | (per file) | `cargo test -p mt-state --test determinism_invariants` | all PASS |
| 8.12 | mt-timechain | 19 | `cargo test -p mt-timechain --test determinism_invariants` | 19/19 |
| 8.13 | mt-account | 29 | `cargo test -p mt-account` | 29/29 |
| 8.14 | mt-lottery | 34 | `cargo test -p mt-lottery` | 34/34 |
| 8.15 | mt-consensus | 27 | `cargo test -p mt-consensus` | 27/27 |
| 8.16 | mt-entry | 24 | `cargo test -p mt-entry` | 24/24 |
| 8.17 | mt-store | 17 | `cargo test -p mt-store` | 17/17 |

#### C4. Security invariants (13 automated)

| # | Invariant | Reproduce | Pass |
|---|-----------|-----------|------|
| 8.18 | SecretKey: !Clone | `cargo test -p mt-crypto --test security_invariants secret_key_is_not_clone` | compile-PASS |
| 8.19 | SecretKey: !PartialEq | same test | compile-PASS |
| 8.20 | SecretKey heap-allocated | same test | size_of == ptr |
| 8.21 | SecretKey impl Drop | same test | trait check |
| 8.22 | MlkemSecretKey same 4 invariants | same test | 4/4 PASS |
| 8.23 | FFI fills SK с non-zero bytes | same test | bytes != [0; N] |
| 8.24 | No println/log on SK in lib code | same test | grep clean |
| 8.25 | PublicKey/Signature: Clone (sanity) | same test | trait check |

---

### Уровень 9 — DEV-001..DEV-011 status verification

| # | DEV-N | Spec ref | Status verify | Code location | Pass |
|---|-------|----------|---------------|---------------|------|
| 9.1 | DEV-001..DEV-009 — закрыты через canonical apply_proposal pipeline rewrite | docs/SPEC_DEVIATIONS.md:1-170 | `grep "Status: закрыто" docs/SPEC_DEVIATIONS.md \| wc -l` | 9 closed entries | static |
| 9.2 | DEV-010 — genesis bootstrap auto-detection | docs/SPEC_DEVIATIONS.md:172 | `is_bootstrap_node` logic в node_lifecycle.rs:70-79 | runtime check via `montana-node status` | observable |
| 9.3 | DEV-011 — hardware calibration initial D | docs/SPEC_DEVIATIONS.md:N | runtime D adjustment per τ₂ | post-Genesis ceremony auto-adjust | static acknowledgment |
| 9.4 | Pre-commit hook DEV count enforcement | scripts/pre-commit.sh Gate 3 | edit DEV in code without doc → reject | enforced commit gate | static |

---

### Прохождение matrix

**Этапы:**

1. **Фаза A — automated baseline** (1-2 часа)
   - `cargo test --all` — все 191 determinism + 13 security + NIST KAT + RFC vectors
   - Pass criterion: 0 failures
   - Result: матрица пунктов 8.x = automated PASS

2. **Фаза B — operator surface manual** (1 рабочий день)
   - Свежая установка через `scripts/install-local-mac.sh`
   - Прохождение пунктов 6.x (CLI commands), 7.x (operator surface)
   - Verify state files размеры/magic
   - Verify launchd integration (start/stop/restart on crash)
   - Verify install scripts на VPS (Linux) + macOS

3. **Фаза C — protocol per-element manual** (3-5 рабочих дней)
   - Каждый пункт 1.x (M1 primitives), 2.x (M2 state), 3.x (apply_proposal), 4.x (consensus)
   - External oracle cross-checks (Python SHA-256 для hash compositions)
   - Boundary tests (overflow, underflow, edge cases)
   - Per-DEV deviation status confirmation

4. **Фаза D — recovery & crash** (0.5 рабочего дня)
   - Сценарии 7.25-7.29 — atomic rename, SIGKILL recovery, corrupted file detection

**Pass criterion для всей matrix:** 100% пунктов с PASS либо явный finding с DEV-NNN reference в docs/SPEC_DEVIATIONS.md.

**Output:** заполненная matrix (markdown table со столбцом "Status: PASS/FAIL/N/A") в отдельном документе `docs/manual-validation-results-YYYY-MM-DD.md`. Это deliverable перед external audit / перед запуском M6 network layer.

**Tooling:**
- Automated: `cargo test --all` для пунктов 8.x
- External oracle: Python 3 + hashlib для SHA-256 cross-checks
- Diff: `diff -u expected actual` для byte-exact verification
- Hexdump: `hexdump -C file | head` для file format verification

---

### Связь с существующим Manual Validation Gate

«Локальный shakedown» сценарии 0-5 (выше) дают smoke-pass — happy-path запуск каждого слоя. Matrix M1-M5 — детализация: каждый scenario разворачивается в 20-50 проверок per-element.

Сценарии 0-5 → entry-level demonstration. Matrix → production-grade audit-ready evidence.

---

### M6 — Network ✅ ЗАКРЫТ (in-process e2e); cross-machine pairing defer M8

- [x] `mt-net` (~2700 LOC, no_std) — wire format envelope + 12 structured payloads + IBT online/mesh proofs + Bootstrap PoW + Uniform Framing + peer selection (4-level diversity + LRU) + Dandelion++ stem-fluff + NAT traversal + Mesh Transport + Store-and-Forward. **112 tests PASS.**
- [x] `mt-net-transport` (~470 LOC, std + libp2p) — libp2p TCP → Noise_PQ XX → Yamux upgrade chain (classical TLS 1.3 + Noise XK removed) + MontanaCodec for request-response + MontanaBehaviour + IBT classify_proof + new `NoisePqXxConfig` plugged into `with_tcp` auth slot. **17 tests in mt-net-transport + 7 in mt-noise-pq + 3 e2e two-node + e2e_noise_pq_with_libp2p_upgrade + e2e_proposal_exchange + KAT vectors PASS in release.**

Phases закрытия:
- Phase A wire envelope + payloads (commits `9de287b`, `bc694a5`)
- Phase B IBT + Bootstrap PoW (commit `26e76c9`)
- Phase D Uniform Framing (commit `cce189f`)
- Phase E peer selection + diversity + LRU (commit `f34726f`)
- Phase F Dandelion + NAT (commit `7d44687`)
- Phase G Mesh + Store-and-Forward (commit `60466c1`)
- **Critic-fix bundle P-C1..P-C8** (commit `93b9cdc`) — 8 code findings closed (domain SSOT, prefix-free rename, fuzz harnesses, try_new constructors, forward-compat, O(1) verify, unwrap/expect refactor)
- Phase C.0 mt-net-transport skeleton + libp2p [C-5] capability checklist 8/8 PASS (commit `ea6608e`)
- Phase C.1 MontanaCodec (commit `11f3b80`)
- Phase C.2 MontanaBehaviour wrapper (commit `ba00051`)
- Phase C.3 e2e two-node handshake — Manual Validation Gate scenario 6 PASS (commit `9a15f49`)
- Phase C.4 e2e proposal exchange + 512 KiB boundary — scenario 7 PASS (commit `04f8d29`)

**Критерий закрытия M6:** 2 узла на разных machines обмениваются proposals через network — closed in-process (e2e tests), cross-machine pairing defer to M8 montana-node binary distribution.

### M7 — Fast Sync

- [ ] `mt-sync` — snapshot делiver + verify через Merkle root сравнение, catch-up от snapshot до current window, genesis content replication.

**Критерий закрытия M7:** новый узел синхронизируется от 0 до текущего state за минуты, не часы.

### M8 — Node binary

- [ ] `mt-node` — CLI, config, wire всё вместе, logging, metrics, graceful shutdown.

**Критерий закрытия M8:** один бинарь `mt-node` запускает узел с конфигом, участвует в сети, зарабатывает Монтану (Ɉ) в лотерее.

### M9 — Conformance suite ✅ READY (initial); expansion in progress

- [x] `mt-conformance` (~150 LOC) — публичный test vectors набор для cross-implementation byte-exact verification. Initial vectors: envelope A1/A2/A3 + IBT B1 (after P-C2 rename) + Bootstrap PoW F1/F2 target derivation. **2 unit tests PASS.**
- [ ] Expansion: 12 TBD-A markers (consensus objects 0x01..0x22 + app-layer 0x60/0x61/0x64) defer until app-layer payload format finalization.

**Критерий закрытия M9:** вторая реализация (Swift iOS либо Go) проходит все vectors байт-в-байт. iOS Phase 2.1 — port done (`MontanaTests/MTConformanceVectors.swift` mirror), pending Xcode `xcodebuild test` verification.

### M10 — Spec compliance cleanup ✅ ЗАКРЫТ

Полное приведение кода в соответствие спеке после серии breaking changes (winner_class removal, TransferActivation opcode, single-path node lottery, ACCOUNT_RECORD_SIZE refactor). Закрыто 7 commits, все 4 обязательные проверки зелёные, 593+ тестов passed.

### M11 — CloseAccount финализация 🔄 Текущий

- [ ] `CloseAccount` opcode `0x0B` — полная реализация после spec-финализации payload формата: struct + Инварианты CloseAccount, apply_proposal dispatch (вычитание баланса в supply, удаление AccountRecord, освобождение operator-binding если is_node_operator), binding test vectors.

**Критерий закрытия M11:** payload format CloseAccount специфицирован в спеке, реализация byte-exact, все 4 обязательные проверки зелёные.

### M12 — Pure conservation monetary policy compliance ✅ ЗАКРЫТ (superseded by M13)

Pre-mainnet миграция к pure geometric pin 41/40, без bootstrap-надбавки, без сжигания, без opcodes прикладного слоя (никнеймы, сервисные кредиты). **Superseded M13 const emission cleanup** — geometric step-up baseline (41/40) удалён в пользу const `EMISSION_moneta = 13 Ɉ` per окно. Историческая запись остаётся для context.

### M13 — Const emission cleanup ✅ ЗАКРЫТ

Pre-mainnet упрощение монетарной политики: pin 41/40 + carry-recurrence заменён на единственную константу `EMISSION_moneta = 13 × 10⁹ nɈ` per окно. `reward_moneta(W) = EMISSION_moneta` (const, навсегда), `supply_moneta(W) = EMISSION_moneta × (W + 1)` closed-form.

Удалено:
- `MonetaryState` struct + 24B persistent state + `apply_step` + 3 controlled overflow panics
- `monetary_epoch_tick` apply_proposal Step 2.5
- `r_baseline_at_epoch` reconstruction helper
- `save_monetary_state`/`load_monetary_state` mt-store API + `monetary.bin` file
- ProtocolParams поля `monetary_epoch_windows`, `inflation_num`, `inflation_den` (−24B encoded, 4118 → 4094)
- `m33_emission` example
- Constitutional declaration spec section (Freigeld/Gesell/Frederick/Keynes/Friedman/Schmitt-Grohé/Bordo/Onken/Wörgl academic apparatus)
- Equilibrium analysis для (41/40)^e + Binding test vectors с эпохами

Замена:
- ProtocolParams: единственное поле `emission_moneta: u128 = 13_000_000_000`
- `reward_moneta(params: &ProtocolParams) -> u128` — однострочник `params.emission_moneta`
- `supply_moneta(W, params)` — closed-form `emission × (W + 1)`
- `compute_state_root(node_root, candidate_root, account_root)` — без MonetaryState arg
- `apply_proposal(account_table, node_table, candidate_pool, input, params)` — без monetary mut arg, без Step 2.5

Constitutional break: новый Genesis State Hash (ProtocolParams layout + state_root composition меняются) — pre-mainnet нормально.

**Критерий закрытия M13:** spec + code grep на pattern set (`MonetaryState|r_baseline|carry_current|inflation_num|inflation_den|monetary_epoch|R_GENESIS|r_genesis_moneta|MONETARY_STATE|geometric step|41/40|2\.5%|Freigeld|Gesell|Frederick`) даёт 0 hits в спеке и в коде.

### M1-E — Миграция подписи FN-DSA-512 → ML-DSA-65 ✅ ЗАКРЫТ

Pre-mainnet breaking wire-format change: переход на NIST FIPS 204 финализированный стандарт. Закрывает [I-1] (PQ-secure) более mainstream путём (NIST level 3, formally finalized) и устраняет dependency на необфициальный pqcrypto-falcon binding wrapper.

**Все 11 шагов Phase E plan executed:**

- E.1: проверка test_vectors.rs (vectors уже обновлены под L=32 для ML-DSA, L=64 для ML-KEM)
- E.2: 5 KAT vectors сгенерированы через keygen_vectors.rs reference implementation
- E.3: спека v31.0.0 обновлена — Derivation Vectors 1+2 hex заменён на реальные значения; новый раздел «Binding KAT vectors для KeyGen → terminal observable output» с 5 KAT (SHA-256 fingerprints для byte-exact cross-impl conformance)
- E.4: `mt-crypto::self_test()` обновлён — KAT 1 byte-exact conformance check вместо placeholder; добавлены `EXPECTED_KAT_1_PK_SHA256` / `EXPECTED_KAT_1_SK_SHA256` константы
- E.5: integration test `e2e_recovery.rs` — 3 теста (terminal_observable_byte_exact, distinct_entropies_distinct_terminals, account_node_keys_differ_same_master)

**Phase F (examples):**
- F.1: m1_mnemonic refactor — cmd_keypair → cmd_seeds, новый cmd_keypair (terminal output), новый cmd_recovery_fingerprint; mt-codec domain registry +1 (mt-recovery-fingerprint)
- F.2: m1_crypto refactor — cmd_keypair → cmd_keypair_random + новый cmd_keypair_deterministic (default `keypair`)
- F.3: verify через release binaries — все subcommands PASS

**Phase G (closure):**
- G.1: VERSION.md updated, M1-E entry added
- G.2: ROADMAP.md M1-E milestone closure
- G.3: 4 обязательные проверки green (fmt/clippy/test/build --release)
- G.4: 3 commits — Phase E `8be11f3`, Phase F `11f7232`, Phase G (этот)

**Manual validation готов к авторскому прогону** (Validation Gate Scenario 0 и 1 ready):
- `m1_mnemonic recovery-fingerprint` — single 64-char hex для two-device validation
- `m1_mnemonic keypair` — terminal observable IDs (account_id, node_id) + 6 byte-exact key parts
- `m1_crypto keypair` — deterministic recovery primitive sanity
- `m1_mnemonic vectors` — 6/6 binding vectors PASS

**Acknowledgement:** `ml-dsa 0.1.0-rc.8` — RustCrypto pure-Rust, **NOT formally audited**; migration target `libcrux-ml-dsa` когда formally verified version stable. Любая смена implementation library обязана сохранять byte-identity с KAT fingerprints спеки (cross-impl conformance gate).

---

## Dependency graph

```
                      sha2, ml-dsa, ml-kem
                                    │
                                    ▼
              ┌───────────────────────────────────────┐
              │                                       │
         mt-codec                               mt-crypto
              │                                       │
              └─────────────┬─────────────────────────┘
                            │
           ┌────────────────┴────────────────┐
           ▼                                 ▼
       mt-merkle                         mt-genesis
           │                                 │
           └──────────────┬──────────────────┘
                          ▼
                      mt-state ◄────── mt-timechain
                          │                  │
                          └────┬─────────────┘
                               ▼
                           mt-account
                               │
           ┌───────────────────┼───────────────────┐
           ▼                   ▼                   ▼
       mt-lottery         mt-consensus          mt-entry
           │                   │                   │
           └───────────────────┼───────────────────┘
                               ▼
                           mt-store
                               │
                               ▼
                           mt-net
                               │
                               ▼
                           mt-sync
                               │
                               ▼
                           mt-node
                               │
                               ▼
                       mt-conformance
```

Каждая стрелка — `[dependencies]` в `Cargo.toml`. Никаких циклов (Rust их запрещает на уровне компилятора).

---

## История обновлений

| Дата | Изменение | Commit |
|------|-----------|--------|
| 2026-04-21 | ROADMAP actualization: R1 stale v29.8.x refs fix + R2 Сценарий 0 User onboarding (24-word mnemonic flow через m1_mnemonic) добавлен в Validation Gate + R3 history entries 2026-04-21 + R4 Starter instructions block для новой сессии. M6 unblock criterion: 6/6 сценариев passed. | (этот) |
| 2026-04-21 | CRITIC.md v1.2.0 → v1.3.0: 3 новых прохода (Source→Sink Flow, Independent Oracle / Differential Check, Misuse-Resistance API Audit) + reframe mt-examples как operator-facing security surface + hard enforcement Multi-perspective rotation через obligatory per-perspective conclusions block. Catalyst — external critic v1.2.0 поднял 6 blind spots. | `5d41d7c` |
| 2026-04-21 | feat: domain separation structural fix (NUL separator) + external critic findings closure. mt_crypto::hash() теперь SHA-256(domain ‖ 0x00 ‖ parts) — self-delimiting guarantee против 8 prefix-collision pairs в registry. Новая sha256_raw() для FIPS/HMAC/raw. Spec v29.12.0 → v29.13.0. Closes 3 external findings (P1 domain sep, P2 SK leak, P3 label) + 2 mine (P4 empty_internal binding, P5 stale RECOVERY). | `d762cec` |
| 2026-04-21 | CRITIC.md v1.1.0 → v1.2.0: 7 новых проходов (timing/side-channel, concurrency, version compat, deps source audit, resource exhaustion беyond DoS, test quality, deployment) + Multi-perspective rotation + Anti-recency bias check + known blind spots (M-1..M-5 documented). | `bcecc65` |
| 2026-04-21 | CRITIC.md v1.0.0 → v1.1.0: 4 новых прохода (primitive byte-level audit, registry integrity, output/observable surface, bottom-up reading discipline) + поведение при external critic finding. Reactive response на domain separation bug surface-ит ранее. | `74d2142` |
| 2026-04-21 | doc: sync ROADMAP сценария 1 subcommands с actual m1_crypto binary interface (hash/sign/keypair/merkle-empty/all, не fips-abc/falcon-roundtrip/domain-separation) + m1_mnemonic stale spec refs cleanup. | `4126006` |
| 2026-04-21 | feat: [I-9] closure path A — poly3 minimax заменяет linear log2_q64. mt-lottery Phase C prototype linear (2^-3.5 error) → Remez degree-3 minimax (2^-10.62, theoretical optimum). Binding coefficients B0..B3 halved form unsigned u64, unsigned Horner с intermediate invariants proofs. 14 binding test vectors (5 ln_q64 + 5 weighted_ticket_node + 4 weighted_ticket_account) + 5 target_next vectors в спеке. Spec v29.10.1 → v29.12.0. 4 conformance pending formulas closed. | `7b76dc9` |
| 2026-04-21 | feat: validate_header winner_class ∈ {1,2} check — закрывает P2 external critic реализации (panic в apply_emission от malformed byte в подписанном proposal → liveness attack). HeaderError::InvalidWinnerClass + 2 тест-vectors. Spec v29.10.1 added invariant `winner_class ∈ {1,2}` в Proposal header. | `589be89` |
| 2026-04-21 | refactor: [C-1] SSOT — WINNER_CLASS_NODE/ACCOUNT в mt-state (единый источник), pub use re-exports в mt-account/mt-lottery. Закрывает P1 external critic реализации (duplicate constants). | `5df072d` |
| 2026-04-21 | Spec bump v29.9.0 → v29.10.1: mt-mnemonic closes [I-9] conformance (6 binding test vectors в спеке для M-1 Algorithm + per-role HKDF derivation). | `a35af3f` |
| 2026-04-20 | Validation Gate режим зафиксирован: incremental scenario-by-scenario (один сценарий за сессию), status tracker, starter instructions для новой сессии. M6 unblock = 5/5 passed. | (предыдущий) |
| 2026-04-20 | M4 + M5 CLOSED: mt-consensus (60 тестов, 5 phases — header + Lookback + control_set + Canonical acceptance + Finalization) + mt-entry (39 тестов, 5 phases — NodeRegistration + Candidate Pool + Selection event + Adaptive VDF + batch apply) + mt-store (24 теста, 5 phases — FsStore + table persist + proposal archive + crash recovery + pruning). Workspace 506 тестов. Next: Manual Validation Gate (5 сценариев). | (предыдущий) |
| 2026-04-20 | Spec+role fix: target 8B → 16B u128 (header aligned с P5 P[I-9] integer form) + fallback_depth bound [1,255] specified + role 4.13.1 → 4.13.2 Gate 0.5 шаг (d.2) field name coverage. Proposal header [I-9] audit: 17 полей прогнаны — 1 finding (target, fixed), остальные clean. Critic findings P12-P15 закрыты. Ничто: `9113850`. | 9113850 |
| 2026-04-19 | ROADMAP: expanded M5 до phases A-E + новая секция «Локальный shakedown — Manual Validation Gate» (5 сценариев между M5 и M6) + policy hands-on example per milestone | 2f317b8 |
| 2026-04-19 | M4 Phase C: mt-lottery Node lottery — `seniority_bonus`, `lottery_weight`, `log2_q64` (bit-scan + Phase C prototype linear approx), `ln_q64`, `weighted_ticket_node` (24 теста). [I-9] compliant, polynomial deferred (closed 2026-04-21 path A) | 4cb42bc |
| 2026-04-19 | Post-audit fix: critic findings P6-P9 (duplicate lottery formulas без integer form в spec) + P8 (methodology failure — пропуск Шага 8). Ничто commits: `21e1547` (spec duplicate fix), `7be68f5` (role 4.13.0 → 4.13.1 + усилен Gate 0.5 обязательным pre-edit duplicate scan). | de68778 |
| 2026-04-19 | C4 re-audit M2-M3 по [I-9]: `next_d` соответствует spec (code использовал permille, spec Q32.32 → aligned на permille); emission family (bonus/reward/bootstrap_cumulative/supply) полностью [I-9] compliant без изменений. Spec P4 patched in-place в Ничто; code comment добавлен. | 2c14587 |
| 2026-04-19 | Spec v29.7.1 → v29.8.0: [I-9] Bit-exact deterministic arithmetic инвариант + integer forms для P2/P3/P4/P5 (role+spec atomic в Ничто `1f3f3f9`); Phase C unblocked; fix ROADMAP ×/+ ошибка | 94c7002 |
| 2026-04-19 | M4 Phase B: mt-lottery VdfReveal тип + encode/signed_scope/reveal_hash/compute_endpoint/validate_reveal (15 тестов) | 5395581 |
| 2026-04-24 | **M10 ЗАКРЫТ** — spec compliance cleanup v30.7.0 → v30.9.0. 11 phases закрыты (Phase 7 CloseAccount отложен в M11). 7 commits: 0b55d69 (ROADMAP), 7f19876 (AccountRecord +53B + winner_class cascade), 500fb9f (account lottery cleanup), 2ed18ab (TransferActivation + cooldown), 67868d9 (Nickname/Auction tables + ProtocolParams extension), 533c457 (price_at binding vectors), и этот closure commit. 593+ тестов зелёные, 4 проверки passed. | (этот commit) |
| 2026-04-23 | M10 Phase 0: VERSION.md bump v30.7.0 → v30.9.0 + ROADMAP M10 milestone block + 13 phases plan + 21 drift points audit | 0b55d69 |
| 2026-04-19 | M4 Phase A: mt-lottery BundledConfirmation тип + encode/signed_scope/bundle_hash/validate (24 теста) | 444e399 |
| 2026-04-19 | Spec v29.7.0 → v29.7.1: editorial cleanup (S-1 дубль mt-node, S-2 genesis_candidate_root, F-1 Genesis State Hash domain) + code fix mt-genesis | 6058f3b |
| 2026-04-19 | Роль v1.3.0 → v1.4.0: [C-2] Spec Flow Pre-verification глобальный инвариант | 0197044 |
| 2026-04-19 | **M3 ЗАКРЫТ** — mt-account 6 phases, 102 теста, ~1850 строк. ROADMAP: M4 детально, M5 crude. | 30400eb |
| 2026-04-19 | M3 Phase F: Genesis state materialization (12 тестов) | 30400eb |
| 2026-04-19 | M3 Phase E: apply_proposal partial steps 2/3.5/3.6/4 (15 тестов) | 5ab2a45 |
| 2026-04-19 | M3 Phase D: emission reward/bonus/supply (17 тестов) | 36503dc |
| 2026-04-19 | M3 Phase C: apply individual ops (15 тестов) | 8fcca20 |
| 2026-04-18 | M3 Phase B: validation OpError + 5 функций (23 теста) | cf95eaf |
| 2026-04-18 | Spec v29.6.9 → v29.7.0: signature architecture (SSI rules R1..R4), breaking `cemented_bundle_aggregate` → node_ids only | 918cdf2 |
| 2026-04-17 | ROADMAP: M3 phase-детализация + M4 crude phases (правило v1.3.0) | e92f0b4 |
| 2026-04-17 | Роль v1.2.0 → v1.3.0: правило детализации ROADMAP | 0893fd4 |
| 2026-04-17 | M3 Phase A: mt-account types + encoding + op_hash (18 тестов) | 0efd92d |
| 2026-04-17 | **M2 ЗАКРЫТ**: mt-timechain (23 теста), итог M2 — 2 пакета, 55 тестов | 76ed8da |
| 2026-04-17 | M2: mt-state закрыт (32 теста) | ab99e23 |
| 2026-04-17 | Spec v29.6.8 → v29.6.9 (противоречия #2, #3 устранены) | 2c148b5 |
| 2026-04-17 | mt-version crate удалён (нарушение SSOT [C-1]) | 8bd008e |
| 2026-04-17 | [C-1] SSOT — глобальный инвариант кода в роли | 2a9309d |
| 2026-04-17 | Spec v29.6.7 → v29.6.8 (противоречие #1: selection_interval) | ff390f2 |
| 2026-04-17 | **M1 ЗАКРЫТ**: mt-genesis (19+1 тест), итог M1 — 4 пакета, 88 тестов | 45c1e84 |
| 2026-04-17 | M1: mt-merkle закрыт (25 тестов) | f242956 |
| 2026-04-17 | Roadmap: demos отложены до M4+ | 7c9219e |
| 2026-04-17 | Создан ROADMAP.md | 65e34ec |
| 2026-04-17 | M1: mt-crypto закрыт (18 тестов) | df55372 |
| 2026-04-17 | M1: mt-codec закрыт (26 тестов) | 90464a8 |
| 2026-04-17 | Роль: post-commit test block обязателен | 8d42073 |
| 2026-04-17 | Роль: language policy — max Russian | c06c56b |
| 2026-04-17 | Роль: terminal commands one-line | a18acc6 |
| 2026-04-17 | Роль: auto-commit policy | cd2f719 |
| 2026-04-17 | M0: workspace skeleton | e2457ad |
