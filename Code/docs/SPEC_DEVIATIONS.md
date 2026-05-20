# Spec Deviations

Single source of truth для всех известных отклонений реализации от спеки Montana. Введён v1.13.0 роли архитектора кода ([C-10] Mandatory deviation tracker).

Каждый `// SPEC DEVIATION DEV-NNN: ...` в коде ссылается на конкретный entry ниже. Pre-commit hook (`scripts/pre-commit.sh`) сверяет количество.

Закрытые `DEV-N` оставляются в файле как историческая запись со `Status: закрыто (commit <sha>)`.

---

## DEV-001: NodeRegistration с vdf_chain_length=0

**Crate:** `montana-node`
**File:line:** `crates/montana-node/src/registration.rs:8-22` (build_node_registration)
**Spec section:** «NodeRegistration» / «Adaptive VDF» / «Шаг 1: incremental apply»
**Spec quote:** «`if NR.vdf_chain_length >= required: apply; N += 1; else: reject`», `required_vdf_length(pending=0, active=0, τ₂)` → `tau2_windows = 20160`
**Что делает код:** `vdf_chain_length=0` (либо user-provided), без проверки `≥ τ₂`, обходит `apply_noderegistrations_batch` через ручной `CandidatePool::insert`
**Severity:** блокер mainnet ([I-9] / [C-7] violation, обход canonical apply pipeline)
**Closure path:** реализовать candidate VDF phase в `start.rs` — узел тикает VDF до `vdf_chain_length ≥ τ₂_windows`, после чего автоматически формирует NodeRegistration с правильным `vdf_chain_length` и вызывает `apply_noderegistrations_batch` через canonical pipeline
**Closure cost:** ~14 дней wall-clock на M-class Mac (физика VDF, не код) + ~4 часа кода
**Status:** закрыто (commit `fb204ef` mt-local-node: byte-exact rewrite через canonical apply_proposal)

---

## DEV-002: proof_endpoint = candidate_vdf_init(zeros, zeros, node_id)

**Crate:** `montana-node`
**File:line:** `crates/montana-node/src/registration.rs:11`
**Spec section:** «Шаг 2: Кандидатура» / «[I-8] compliance»
**Spec quote:** «`candidate_vdf_init = SHA-256("mt-candidate-vdf-init" || timechain_value(W_start) || cemented_bundle_aggregate(W_start - 2) || node_id)`»
**Что делает код:** `candidate_vdf_init(&[0u8; 32], &[0u8; 32], &node_id)` — timechain_value и cba как zeros (placeholder)
**Severity:** блокер mainnet ([I-8] violation — нет canonical unpredictable-offline binding)
**Closure path:** на момент формирования NodeRegistration использовать **реальные** `timechain.t_r` и `cemented_bundle_aggregate(W_start - 2, &cemented_node_ids_at_W_start_minus_2)` из локального state узла
**Closure cost:** ~1 час кода после DEV-001 closure
**Status:** закрыто (commit `fb204ef` mt-local-node: byte-exact rewrite через canonical apply_proposal)

---

## DEV-003: Лотерея отсутствует — winner = первый node по lex

**Crate:** `montana-node`
**File:line:** `crates/montana-node/src/commands/start.rs:104-120`, `commands/advance.rs` аналогично
**Spec section:** «Лотерея» / «Победитель τ₁»
**Spec quote:** «winner = `argmin(weighted_ticket_node)` среди cemented `VDF_Reveal` узлов-кандидатов; `weighted_ticket_node = ln_q64(endpoint) / lottery_weight`»
**Что делает код:** `state.nodes.iter().next()` — первый узел по `node_id` lex order, **без формирования VDF_Reveal, без endpoint, без weighted_ticket**
**Severity:** блокер mainnet (consensus-critical логика игнорирована, [I-8] violation)
**Closure path:** реализовать per окно: формирование `VDF_Reveal` (`mt_lottery::VdfReveal`) с `endpoint = SHA-256("mt-lottery" || T_r || cba || node_id || W LE)`, подпись `node_sk`; вычисление `weighted_ticket_node` через `mt_lottery::weighted_ticket_node`; для singleton — единственный кандидат — argmin тривиален и corretct **через каноничный API**
**Closure cost:** ~6 часов кода
**Status:** закрыто (commit `fb204ef` mt-local-node: byte-exact rewrite через canonical apply_proposal)

---

## DEV-004: BundledConfirmation никогда не формируется

**Crate:** `montana-node`
**File:line:** `crates/montana-node/src/commands/start.rs:113-117`
**Spec section:** «Confirmer threshold» / «BundledConfirmation» / «apply_proposal Step 3.5»
**Spec quote:** «chain_length инкрементируется при cemented `BundledConfirmation`», quorum = `(67 × X + 99) / 100` от active_chain_length
**Что делает код:** `chain_length += 1` напрямую без формирования BC, без подписи `op_hashes/reveal_hashes`, без quorum cementing
**Severity:** блокер mainnet (chain_length грубо инкрементируется на основании несуществующего правила)
**Closure path:** формирование `mt_lottery::BundledConfirmation` с `op_hashes[]` (от Account Table cemented operations) + `reveal_hashes[]` (от cemented VDF_Reveal предыдущего окна) + подпись `node_sk`; cementing через quorum (для singleton — сам себе 100%, проверка через `mt_lottery::is_cemented`)
**Closure cost:** ~8 часов кода
**Status:** закрыто (commit `fb204ef` mt-local-node: byte-exact rewrite через canonical apply_proposal)

---

## DEV-005: ProposalHeader не формируется, Step 4 apply_proposal обойдён

**Crate:** `montana-node`
**File:line:** `crates/montana-node/src/commands/start.rs:102-128`
**Spec section:** «Proposal header» / «Canonical acceptance» / «apply_proposal Step 4»
**Spec quote:** «winner формирует `ProposalHeader` (1080 байт) с `included_bundles + included_reveals + state_root`, подписывает, archive-ит. Validator пересчитывает state_root и сверяет.»
**Что делает код:** напрямую `account.balance += 13_000_000_000` минуя `apply_emission`; ProposalHeader не формируется, `archive_proposal` не вызывается
**Severity:** блокер mainnet (full Step 4 apply_proposal обойдён)
**Closure path:** реализовать формирование `mt_consensus::ProposalHeader` с правильными полями (`canonical_proposer`, `included_bundles`, `included_reveals`, `state_root`), `validate_acceptance`, эмиссия через `mt_account::apply_proposal`, `mt_store::archive_proposal`
**Closure cost:** ~12 часов кода
**Status:** закрыто (commit `fb204ef` mt-local-node: byte-exact rewrite через canonical apply_proposal)

---

## DEV-006: state_root не cross-check между proposer и validator

**Crate:** `montana-node`
**File:line:** N/A (отсутствие кода)
**Spec section:** «Верификация» / «Финальность proposal»
**Spec quote:** «Финальность proposal — подпись `proposer_node_id` на proposal header. Верификация — независимый пересчёт state_root.»
**Что делает код:** state_root recompute не существует. Singleton mode — узел сам себе proposer и validator, но cross-check всё равно необходим для регулярного self-verification (защита от corruption диска / памяти)
**Severity:** средний (singleton не имеет 2 узлов для cross-check, но self-verification обязателен)
**Closure path:** после формирования `ProposalHeader.state_root`, повторно вычислить `compute_state_root(account_root, node_root, candidate_root)` независимо и сверить byte-exact; mismatch → panic (corruption detected)
**Closure cost:** ~1 час кода
**Status:** закрыто (commit `fb204ef` mt-local-node: byte-exact rewrite через canonical apply_proposal)

---

## DEV-007: next_d не вызывается на τ₂ boundary

**Crate:** `montana-node`
**File:line:** `crates/montana-node/src/commands/start.rs:95-160`
**Spec section:** «Адаптация D через participation-ratio feedback»
**Spec quote:** «D адаптируется на границе τ₂ через каноническое chain observation»
**Что делает код:** `timechain.current_d` фиксируется на `D₀=252M`, `next_d` не вызывается
**Severity:** блокер mainnet для long-running узла (>14 дней)
**Closure path:** хранить `participation_history: Vec<u32>` (permille per окно) в timechain state; на каждой τ₂ boundary вычислить median + `next_d(current_d, median, params)`; обновить `timechain.current_d`; для singleton: participation_ratio = всегда 1000 → median=1000 → каждые τ₂ D × 1.03
**Closure cost:** ~3 часа кода
**Status:** закрыто (commit `fb204ef` mt-local-node: byte-exact rewrite через canonical apply_proposal)

---

## DEV-008: selection_event с zeros в advance.rs vs реальный T_r в start.rs

**Crate:** `montana-node`
**File:line:** `crates/montana-node/src/commands/advance.rs:55-72`
**Spec section:** «Selection event sort_key»
**Spec quote:** «`sort_key(c) = SHA-256("mt-selection" || timechain_value(W) || cemented_bundle_aggregate(W-2) || c.node_id)`»
**Что делает код:** `let placeholder = [0u8; 32]` для обоих `t_r` и `cba`. Silent divergence между моими же командами — `start.rs` использует реальный `timechain.t_r`, `advance.rs` использует zeros. Один и тот же state, разные seeds → разные ranking → разные winners для multi-candidate.
**Severity:** блокер mainnet (silent divergence между путями исполнения)
**Closure path:** удалить `advance.rs` целиком — для byte-exact spec нет «быстрой симуляции», есть только реальное исполнение
**Closure cost:** ~10 минут (удаление файла + dispatch update)
**Status:** закрыто (commit `fb204ef` mt-local-node: byte-exact rewrite через canonical apply_proposal)

---

## DEV-009: apply_proposal целиком обойдён — все steps реализованы ручным insert/update

**Crate:** `montana-node`
**File:line:** `crates/montana-node/src/commands/start.rs:95-160`, `advance.rs:45-95`
**Spec section:** «State transition → apply_proposal»
**Spec quote:** «Steps 1, 2, 3a, 3b, 4 в каноническом порядке»
**Что делает код:** напрямую модифицирует `AccountTable`/`NodeTable`/`CandidatePool` вне любого `apply_proposal`. Каждое окно — это ad-hoc набор shortcut'ов, не каноническая state transition.
**Severity:** блокер mainnet (silent divergence реализации от спеки на per-окно basis)
**Closure path:** заменить ad-hoc на canonical `apply_proposal` pipeline через `mt_account::apply_proposal(&mut account_table, &mut node_table, &mut candidate_pool, &proposal_input, params)`. Singleton mode формирует валидный `ProposalInput` для каждого окна и вызывает canonical pipeline.
**Closure cost:** ~16 часов кода (зависит от DEV-001..DEV-006)
**Status:** закрыто (commit `fb204ef` mt-local-node: byte-exact rewrite через canonical apply_proposal)

---

## DEV-011: hardware calibration initial D под target window time

**Crate:** `montana-node`
**File:line:** `crates/montana-node/src/commands/start.rs` функция
              `calibrate_d_for_target_window` + первый запуск start
**Spec section:** «Двигатели → TimeChain VDF — осциллятор», «Калибровка D₀»
**Spec quote:** «Mainnet calibration `D₀` нацелена на τ₁ ≈ 60 секунд wall-clock
                 на median commodity hardware (engineering target, не protocol
                 invariant)»
**Что делает код:** при первом запуске узла (timechain.bin не существовал)
                    запускает benchmark vdf_step(zeros, 10M) → измеряет
                    hardware SHA-256 rate → calibrate `current_d` так чтобы
                    окно ≈ 60 сек wall-clock на этой машине.
**Что в спеке:** spec `D₀ = 252M` — engineering calibration target для
                 median commodity. Per-узел actual wall-clock varies ×20
                 (Apple Silicon ~53s, idle x86_64 VPS ~68s, loaded ~1145s).
                 Adaptive D feedback на τ₂ boundary автоматически
                 подстраивает D под median network rate.
**Severity:** косметический — D в genesis узле = local state, не shared
              consensus invariant с другими узлами (других нет).
              При появлении новых узлов сети их D будет calibrated
              самостоятельно либо synchronized через canonical params.d0.
**Closure path:** при finalize multi-node M6+ — узлы будут sync через
                  canonical D из Genesis Decree либо negotiate через
                  network consensus. Hardware calibration остаётся для
                  genesis узла как initial value.
**Closure cost:** acknowledged как permanent feature для genesis узла,
                  не требует closure
**Status:** acknowledged (genesis-узел local hardware calibration —
            explicit operator choice, не silent shortcut)
**Acknowledged:** автор 2026-04-28 «сделай так чтобы мой узел генерил
                  примерно в 60 секунд окно»

---

## DEV-010: genesis bootstrap mode без Candidate VDF (auto-detected)

**Crate:** `montana-node`
**File:line:** `crates/montana-node/src/state.rs:32-66` (LocalState::bootstrap),
              `crates/montana-node/src/node_lifecycle.rs:48-92` (NodeLifecycle::fresh_for + is_bootstrap_node),
              `crates/montana-node/src/commands/start.rs:74-93` (Bootstrap → CandidateVdf transition)
**Spec section:** «Genesis Decree» / «bootstrap_node_pubkey» / «Активация узла»
**Spec quote:** «`bootstrap_node_pubkey: [u8; PUBLIC_KEY_SIZE]` в `protocol_params` —
                 первый узел сети активируется через genesis state, не через
                 Candidate VDF + selection event цикл»
**Что делает код:** автоматическое определение genesis vs candidate per spec:
  - `NodeLifecycle::is_bootstrap_node(identity, params)` сравнивает
    `identity.node_pk` с `params.bootstrap_node_pubkey` byte-by-byte
  - Если `bootstrap_node_pubkey == [0u8; PUBLIC_KEY_SIZE]` (placeholder
    pre-Genesis-ceremony) — **любой** узел трактуется как genesis (singleton
    legacy mode для M5 development phase). Эта ветка перестаёт применяться
    после Genesis ceremony когда `bootstrap_node_pubkey` финализирован
    конкретным значением
  - Если `bootstrap_node_pubkey` finalized + `identity.node_pk` совпадает —
    genesis path: phase=Active immediately, NodeRecord для self в NodeTable
  - Если `bootstrap_node_pubkey` finalized + `identity.node_pk` НЕ совпадает —
    standard candidate path: phase=Bootstrap → CandidateVdf на первом окне
    → Registered (через apply_noderegistrations_batch когда vdf_chain_length
    ≥ τ₂) → Active (через apply_selection_event на ближайшем W % selection_interval == 0).
    Узел НЕ появляется в NodeTable bootstrap state — добавляется только
    через canonical apply_selection_event.
**Severity:** acknowledged feature pre-Genesis-ceremony; production-ready
              после ceremony (auto-detection через canonical apply_proposal pipeline
              для не-bootstrap узлов работает byte-exact spec).
**Closure path:** Genesis ceremony — установить `params.bootstrap_node_pubkey`
                  в реальное значение. После этого DEV-010 закрывается
                  автоматически: проверка `is_bootstrap_node` определит ровно
                  один genesis узел, остальные пройдут стандартный candidate path.
**Closure cost:** 0 после Genesis ceremony (код уже implements auto-detection)
**Status:** acknowledged (auto-detection в коде, pre-ceremony placeholder
            активирует singleton legacy ветку для M5; post-ceremony — production
            spec compliance)
**Acknowledged:** автор 2026-04-28 — «у нас автоматически определяется
                  genesis узел с условиями и остальные?» → fix v1.15.0 [C-13]
                  enforcement: правильный путь немедленно, без вопроса автору

---

## История

| Версия роли | Дата | Действие |
|---|---|---|
| v1.13.0 | 2026-04-28 | Файл создан. Открыты DEV-001..DEV-009 для `montana-node` Этапы 1-5. Решение автора: byte-exact rewrite. |
| v1.13.0 | 2026-04-28 | DEV-001..DEV-009 закрыты через canonical apply_proposal pipeline rewrite. |
| v1.13.0 | 2026-04-28 | DEV-010 added: genesis bootstrap mode (узел стартует Active без Candidate VDF) — explicit acknowledged deviation, decision автора. |
| v1.14.0 | 2026-05-20 | DEV-013 закрыт: online IBT proof включает `online_session_nonce`; `OnlineNonceTracker` rejects replay within current/previous slot. |


---

## DEV-012: singleton-only proposal generation в Active phase

**Crate:** `montana-node`
**File:line:** `crates/montana-node/src/commands/start.rs:265-292` (Active phase guard)
**Spec section:** «BundledConfirmation» / «apply_proposal Step 3.5 cementing» / «Singleton consensus»
**Spec quote:** «`cemented_sum = Σ chain_length узлов чьи BundledConfirmation попали в included_bundles`. Объект cemented когда `cemented_sum ≥ quorum(active_chain_length)`, где `quorum = (67 × active + 99) / 100`.» (mt-consensus/src/lib.rs:327, mt-lottery/src/lib.rs:503-510)
**Что делает код:** Active-фаза в start.rs формирует proposal где my_node — единственный confirmer (`included_bundles = {my_bundle}`, cemented_sum = my_node.chain_length). Это корректно ТОЛЬКО когда `state.nodes == {my_node}` (1 узел в NodeTable, мой собственный). В multi-node NodeTable my_node.chain_length < quorum(Σ_chain_length) → `is_cemented` возвращает false → узел падает с `singleton cementing: cemented=X, active=Y, quorum=Z`. Гард DEV-012 добавляет проверку `state.nodes.len() == 1 && state.nodes.contains(&my_node)`; при неудаче пропускает proposal-блок (break 'active_arm), не падает.
**Severity:** блокер mainnet (M9 Phase 2 = apply_proposal от peer-ов не реализован, multi-node консенсус не работает)
**Closure path:** реализовать M9 Phase 2 — drain incoming Proposal envelope (start.rs:160-169), validate через `mt_consensus::validate_acceptance`, `mt_account::apply_proposal` для cemented set от proposer-а, рекомпьют state_root, sync `current_window` + `state.nodes[].chain_length` из peer Proposal. После этого Frankfurt/Helsinki как followers догоняют moscow без необходимости producить собственные singleton-proposal.
**Closure cost:** ~3-5 дней wall-clock на implementation + integration test (e2e_three_peer_apply_proposal)
**Status:** открыто

**Прецедент:** Frankfurt узел стал Active при genesis bootstrap (registration_window=45916, start_window=46032, chain_length=1) и сразу попал в multi-node ситуацию (state.nodes = {msk, fra}). 4 790 рестартов montana-node за 24 часа с ошибкой `singleton cementing: cemented=1, active=25767, quorum=17264` — msk имел chain_length=25766 в state Frankfurt'а (получено через P2P-синхронизацию), fra собственный chain_length=1. Гард предотвращает crash loop; узел остаётся в Active phase, продолжает heartbeat к peer-ам, ждёт M9 Phase 2.

---

## DEV-013: online IBT proof formula — code behind spec (online_session_nonce)

**Crate:** `mt-net`
**File:line:** `crates/mt-net/src/ibt.rs` (online_proof / verify_online_proof, точная строка зависит от текущей реализации — см. `cargo grep mt-tunnel-online`)
**Spec section:** «Identity-Bound Tunnel (IBT)» в `Montana Network v1.1.0.md` (после bump v1.0.0 → v1.1.0 для MONT-002 closure)
**Spec quote:** «`proof = ML-DSA-65_sign(client_privkey, "mt-tunnel-online" || server_node_id || floor(current_window_index / 2) || online_session_nonce)` где `online_session_nonce` 32B — генерируется клиентом из CSPRNG для каждого handshake, передаётся в plain части IBT advertisement рядом с proof.»
**Что делает код:** `mt-net::ibt::ibt_online_proof` и `ibt_online_verify` принимают `online_session_nonce: [u8; 32]` и включают его в signed message. `mt-net::ibt::OnlineNonceTracker` хранит `used_online_nonces[client_pubkey]` с pruning по current/previous window slot и bounded per-client set. `mt-net-transport::ibt_upgrade::classify_proof` вызывает verifier + nonce tracker до выдачи access level.
**Severity:** закрыто для MONT-002 (MITM replay того же online proof в пределах 2-window slot rejected как `IbtError::ReplayedNonce`)
**Status:** закрыто (mt-net / mt-net-transport: online_session_nonce in signed scope + used_online_nonces tracking)

**Acknowledged:** Wire-level handshake envelope в transport integration должен передавать `online_session_nonce` рядом с proof; API уже требует nonce, поэтому без него callsite не компилируется.
