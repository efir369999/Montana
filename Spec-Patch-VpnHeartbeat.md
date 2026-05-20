# Spec Patch: opcode `0x06 VpnHeartbeat`

**Целевая версия спеки:** Montana Protocol v35.25.1 → **v35.26.0**
**Дата проекта патча:** 2026-05-19
**Применимый раздел:** §11 «Account operations», §13 «apply_proposal»
**Зависимые инварианты:** [I-1] PQ-secure, [I-3] Determinism, [I-9] Bit-exact arithmetic, [I-13] Deflationary sink, [I-14] State lifecycle
**Связанные роли:** `Protocol/CLAUDE.md` v4.30.0+ (архитектор), `Protocol/CRITIC.md` v3.13.0+ (критик)

---

## Намерение патча

Закрытие глобальной слабости в текущей реализации Montana App + VPN-balance coordinator:

1. **Heartbeat начисление Ɉ происходит вне консенсуса.** Текущий Rust coordinator на узле Moscow ведёт `state.json` авторитарно, не реплицируется через TimeChain, не cemented. F-4 из аудит-пакета `Android/Внешний-аудит/07-Известные-ограничения.md`.

2. **Sybil-resistance работает только через Ed25519 TOFU pinning.** Это нарушает [I-1] PQ-secure (Shor на curve25519), хотя сейчас acceptable как Phase 2 stopgap. CF-Phase2-1.

3. **Migration к полному консенсусу требует нового opcode и operation type.**

Этот патч описывает opcode `0x06 VpnHeartbeat` — нормативно. После принятия:
- `mt-account` реализует `apply_vpn_heartbeat`
- `montana-node` принимает в mempool, cement через BundledConfirmation
- AccountRecord.balance обновляется через canonical apply pipeline
- Координатор Moscow становится **indexer** для быстрого чтения, не source-of-truth

---

## §1. Type byte registry — расширение

Текущее распределение (Protocol v35.25.0 §11.1):

| Byte | Operation | Класс |
|------|-----------|-------|
| `0x01` | _reserved (anchor-fee sponsor)_ | — |
| `0x02` | Transfer | value |
| `0x03` | ChangeKey | power |
| `0x04` | Anchor | value |
| `0x05` | NicknameBid | value (allocation) |

После v35.26.0:

| Byte | Operation | Класс |
|------|-----------|-------|
| `0x01` | _reserved_ | — |
| `0x02` | Transfer | value |
| `0x03` | ChangeKey | power |
| `0x04` | Anchor | value |
| `0x05` | NicknameBid | value (allocation) |
| **`0x06`** | **`VpnHeartbeat`** | **value (operator-credit)** |

---

## §2. Operation layout

```
operation VpnHeartbeat ::= {
    type_byte:              u8       = 0x06
    sender_account_id:      AccountId  (20B)     // адрес владельца кошелька-кредитуемого
    window_index:           u64        (8B LE)   // окно консенсуса в котором heartbeat валиден
    exit_node_id:           NodeId     (32B)     // public key валидатора-exit (узел VPN-каскада)
    duration_ms:            u32        (4B LE)   // intervalo миллисекунд с предыдущего heartbeat
    operator_signature:     Signature  (666B)    // FN-DSA-512 подпись sender-а над canonical_preimage
}
```

**Размер:** `1 + 20 + 8 + 32 + 4 + 666 = 731 bytes`.

### §2.1 Canonical preimage для подписи

```
domain_separator = ASCII("mt-vpn-heartbeat-v1")               (19B)
preimage = domain_separator
        || sender_account_id                                   (20B)
        || window_index_le8                                    (8B)
        || exit_node_id                                        (32B)
        || duration_ms_le4                                     (4B)
preimage_length = 19 + 20 + 8 + 32 + 4 = 83 bytes
```

`operator_signature = FN-DSA-512.Sign(sender_secret_key, preimage)`.

### §2.2 Инварианты VpnHeartbeat

1. **VH-1 type byte:** `type_byte == 0x06`.
2. **VH-2 sender exists:** `AccountTable[sender_account_id]` существует и `account_chain_length ≥ 1`.
3. **VH-3 window bounds:** `current_window − 1 ≤ window_index ≤ current_window` (heartbeat принимается только в текущем или предыдущем окне).
4. **VH-4 exit-node validity:** `NodeTable[exit_node_id]` существует и имеет `is_active = true` и `node_role ∈ {exit_node, dual_role}`.
5. **VH-5 signature verify:** `FN-DSA-512.Verify(AccountTable[sender_account_id].suite_pubkey, preimage, operator_signature) == true`.
6. **VH-6 duration upper bound:** `duration_ms ≤ MAX_HEARTBEAT_DURATION_MS = 30 000` ms (защита от backdate flooding).
7. **VH-7 duration lower bound:** `duration_ms ≥ MIN_HEARTBEAT_DURATION_MS = 4 000` ms (защита от heartbeat spam внутри окна).
8. **VH-8 monotonic nonce:** `duration_ms` накопляется через `AccountTable[sender].vpn_credited_seconds_in_window[window_index]` — heartbeat принимается только если суммарная сумма за окно ≤ `WINDOW_DURATION_MS = τ₁`.
9. **VH-9 exit-node cooldown:** `NodeTable[exit_node_id].last_heartbeat_processed_window ≥ window_index − 2` (защита от использования давно-offline exit-узлов).

---

## §3. apply_vpn_heartbeat

### §3.1 Шаги применения

```
apply_vpn_heartbeat(state, op):
    // Шаг 1. Валидация инвариантов VH-1..VH-9.
    require all VH-1..VH-9 pass

    // Шаг 2. Вычисление credit.
    rate_nj_per_ms = RATE_NJ_PER_MILLISECOND = 1   // 0.001 Ɉ/sec = 1 nɈ/ms
    credit_nj = op.duration_ms × rate_nj_per_ms     // integer arithmetic per [I-9]

    // Шаг 3. State mutation:
    state.account_table[op.sender_account_id].balance_nj += credit_nj
    state.account_table[op.sender_account_id].vpn_credited_seconds_x1000_in_window[op.window_index] += op.duration_ms
    state.node_table[op.exit_node_id].vpn_traffic_served_seconds += op.duration_ms / 1000

    // Шаг 4. Emission supply update:
    state.supply_nj += credit_nj                    // эмиссия Ɉ за VPN-работу

    // Шаг 5. Chain length:
    state.account_table[op.sender_account_id].account_chain_length += 1
    state.account_table[op.sender_account_id].frontier_hash = H_canon(op)
```

### §3.2 Возможные результаты

- **OK:** state mutated as above.
- **Err::AccountNotFound:** инвариант VH-2 нарушен.
- **Err::SignatureInvalid:** VH-5.
- **Err::WindowOutOfBounds:** VH-3.
- **Err::ExitNodeOffline:** VH-4 / VH-9.
- **Err::DurationOutOfBounds:** VH-6 / VH-7.
- **Err::WindowQuotaExceeded:** VH-8.

---

## §4. Эмиссия Ɉ за VPN — экономический анализ

### §4.1 Параметры

| Параметр | Значение | Обоснование |
|---|---|---|
| `RATE_NJ_PER_MILLISECOND` | `1 nɈ/ms` (= `0.001 Ɉ/sec`) | Соответствует текущему MVP-coordinator. Закрепляется как initial baseline. |
| `MIN_HEARTBEAT_DURATION_MS` | `4 000 ms` | Rate-limit per identity per heartbeat (см. Pass 18 critic). |
| `MAX_HEARTBEAT_DURATION_MS` | `30 000 ms` | Защита от backdate flooding. |
| `WINDOW_DURATION_MS` | `τ₁ = 30 000 ms` (один TimeChain window) | Используется как cap quota per window. |
| `MAX_HEARTBEATS_PER_WINDOW_PER_SENDER` | `WINDOW_DURATION_MS / MIN_HEARTBEAT_DURATION_MS = 7` | Pre-mainnet, может быть скорректировано. |

### §4.2 Maximum emission rate per validator

Допущения: один honest sender накапливает `WINDOW_DURATION_MS = 30 000 ms = 30 sec` credit per τ₁.

```
emission_per_sender_per_τ₁     = 30 000 nɈ          = 0.03 Ɉ
emission_per_sender_per_day    = 0.03 × (86400/30)  = 86.4 Ɉ/day
emission_per_sender_per_year   = 31536 Ɉ/year (theoretical maximum)
```

### §4.3 Sybil-attack defense

Sybil-attacker создаёт N fake accounts. Каждый требует:
- Открытие AccountRecord через первый Transfer (закрытие через [I-14] cost barrier — отдельный механизм)
- FN-DSA-512 signature на каждый heartbeat (computational cost не критичный, hardware-asymmetry attack)
- Активный VPN session на exit-узле (network bandwidth cost > VPN credit при честных rate)

**Деривация:** atacker экономика выгодна только если `revenue_per_account_per_day > cost_of_VPN_bandwidth_per_account_per_day`. Учитывая что VPN-bandwidth сам стоит более чем 0.001 Ɉ/sec (текущий TC price + cloud bandwidth pricing), attack экономически нерентабельна.

### §4.4 Соответствие [I-13] Deflationary sink

VpnHeartbeat — **value operation**, не burn. Эмиссия Ɉ за работу validator-а покрывается **существующими** механизмами sink ([I-13]):
- NicknameBid burn
- Anchor fee burn
- ChangeKey re-issuance cost

Баланс между emission (VpnHeartbeat) и burn (NicknameBid + Anchor) — open параметр для economic equilibrium (см. Pass 22 critic — equilibrium analysis).

---

## §5. AccountRecord — расширение поля

```
AccountRecord ::= {
    ...                                              // existing fields per v35.25
    vpn_credited_seconds_x1000:           u64        // суммарно за всю жизнь аккаунта
    vpn_credited_seconds_x1000_in_window: u32        // per current window (reset per τ₁)
    vpn_last_window_index:                u64        // последнее окно где аккаунт активен
}
```

**Дополнительно 24 байта на AccountRecord.**

### §5.1 NodeRecord — расширение поля

```
NodeRecord ::= {
    ...
    vpn_traffic_served_seconds: u64                  // суммарно за всю жизнь узла
}
```

**Дополнительно 8 байт на NodeRecord.**

### §5.2 [I-14] State lifecycle compliance

`vpn_credited_seconds_x1000_in_window` сбрасывается на каждой границе τ₁. Это **не persistent growth** — поле bounded, переиспользуется. ✅

`vpn_traffic_served_seconds` растёт линейно с активностью узла. Для exit-узлов это **expected behavior** (per [I-14] путь cost-based: NODE_REGISTRATION_STAKE покрывает storage cost для NodeRecord). ✅

---

## §6. Test vectors (binding для conformance)

### §6.1 Vector V6-1: канонический preimage

```
sender_account_id    = 2f8714b236118011647ec51d0ca6ad40d286bec7      (20B)
window_index         = 1779120000                                      (u64 LE)
exit_node_id         = b17dd919772d4268a7249b866b92d12b...             (32B placeholder)
duration_ms          = 5000                                            (u32 LE)

preimage_hex = 6d742d76706e2d68656172746265 6174 2d 76 31    [domain]
             | 2f8714b236118011647ec51d0ca6 ad40 d2 86 bec7  [sender]
             | 00ed2bc564010000                              [window le8]
             | b17dd919772d4268a7249b866b92 d12b ...         [exit_id]
             | 88130000                                       [duration le4]
preimage_size = 83 bytes
```

(Конкретный hex значения вычисляются при первом deploy implementation — это placeholder.)

### §6.2 Vector V6-2: balance delta integer arithmetic

```
duration_ms          = 5000 ms
rate_nj_per_ms       = 1
credit_nj            = 5000

Pre-state:  account.balance_nj = 0
Post-state: account.balance_nj = 5000
```

### §6.3 Vector V6-3: rejected — duration слишком большая

```
duration_ms = 35000
Expected: Err::DurationOutOfBounds  (VH-6 violation, 35000 > MAX = 30000)
```

---

## §7. Migration path Phase 2 → Phase 3

### §7.1 Координатор Moscow остаётся

После принятия opcode `0x06` координатор `mt-vpn-balance.service` продолжает работать как **indexer**:
- Принимает heartbeat от Android клиента через legacy REST API (Ed25519 signed).
- Транслирует в `VpnHeartbeat` opcode → sends в локальный montana-node mempool.
- Опционально: возвращает клиенту transaction hash как proof что operation попала в mempool.

### §7.2 Android client изменения

После принятия opcode:
- BIP39 seed → FN-DSA-512 keypair (вместо Ed25519). Требует Falcon JNI bridge.
- Heartbeat body расширяется: добавляется `signature` (FN-DSA-512, 666B) и поля `window_index`, `exit_node_id`, `duration_ms`.
- Координатор Moscow становится транспортом, не authority.

### §7.3 Backwards-compatible переходный период

В переходный период:
- Координатор принимает **обе** формы heartbeat: legacy (Ed25519) и новую (FN-DSA-512).
- Legacy heartbeats не cemented в TimeChain — only credited через координатор.
- Pre-mainnet принцип: при запуске mainnet legacy heartbeats **прекращают** работать, требуется update приложения.

---

## §8. Adversarial gates check (Gate 0..15)

### Gate 0 — Global invariants

- [I-1] PQ-secure: ✅ FN-DSA-512 (Falcon, lattice-based, NIST PQ winner)
- [I-2] Public financial layer: ✅ balance + duration_ms public
- [I-3] Determinism: ✅ integer arithmetic only, no floats
- [I-4] TimeChain independence: ✅ VpnHeartbeat зависит от TimeChain (window_index), не наоборот
- [I-5] Commodity hardware: ✅ FN-DSA-512 на ARM64 ~5-10ms sign, приемлемо
- [I-6] Regulatory compat: ✅ public balance, no privacy mixer
- [I-7] Minimal crypto surface: ✅ переиспользует FN-DSA-512 уже в спеке
- [I-8] Network-bound unpredictability: **N/A** (VpnHeartbeat не использует seed/lottery)
- [I-9] Bit-exact arithmetic: ✅ integer-only, test vectors §6
- [I-13] Deflationary sink: ✅ эмиссия покрывается existing burn механизмами
- [I-14] State lifecycle: ✅ window-window поле сбрасывается, persistent поля cost-bounded

### Gate 1 — Control plane separation

`VpnHeartbeat` — value operation (двигает balance, эмиссию). Не power. ✅

### Gate 2 — Temporal anchor audit

`window_index` ограничен `current_window ± 1` (VH-3). Нет precompute attack. ✅

### Gate 9 — Expiry math

Heartbeat не имеет expiry. Quota window-based (VH-8). ✅

### Gate 10 — Hardware asymmetry

VpnHeartbeat не использует canonical seed → grinding-attack не применим. ✅

### Gate 13 — Invariant enumeration

§2.2 содержит exhaustive list VH-1..VH-9. ✅

### Gate 14 — State lifecycle

§5.2 — все persistent поля либо bounded, либо cost-protected. ✅

### Gate 15 — Post-edit completeness

При принятии патча требуется:
- Удалить упоминания координатора Moscow как authoritative в Android `Внешний-аудит/05-Состояние-и-хранилище.md`
- Обновить mt-account реализацию: добавить apply_vpn_heartbeat
- Обновить mt-conformance test vectors
- Bump VERSION.md и Code/VERSION.md

---

## §9. Roadmap implementation

| Этап | Длительность | Описание |
|------|--------------|----------|
| **M-VPN-2 Phase A** | 1 week | Spec patch finalized + critic 22-pass audit |
| **M-VPN-2 Phase B** | 2 weeks | `mt-account::apply_vpn_heartbeat` + 50+ unit tests |
| **M-VPN-2 Phase C** | 1 week | `montana-node` accepts opcode в mempool + cemented |
| **M-VPN-2 Phase D** | 1 week | `mt-conformance` test vectors из §6 |
| **M-VPN-3 Phase A** | 1 week | Android pqcrypto-falcon JNI bridge research + prototype |
| **M-VPN-3 Phase B** | 2 weeks | Production Android FN-DSA-512 integration |
| **M-VPN-3 Phase C** | 1 week | Coordinator Moscow translates legacy → opcode |
| **M-VPN-4** | 1 week | Migration cutover (legacy heartbeats deprecated) |

**Total:** ~9 weeks. Зависит от availability `pqcrypto-falcon` Android bindings.

---

## §10. Status

**Spec patch:** draft pending critic review.
**Implementation:** **TODO** (M-VPN-2/3 milestones not started).
**Acknowledged dependency:** Falcon-512 Android JNI is non-trivial — alternative migration path = SLH-DSA (SPHINCS+) если pqcrypto-falcon не пригоден.

После принятия патча архитектором + критиком — bump спеки v35.25.0 → v35.26.0 с заменой §11.1 type byte registry и добавлением §11.2-VpnHeartbeat.
