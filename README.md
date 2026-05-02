# Montana

> **Open-source post-quantum reference blockchain.** VDF-based TimeChain consensus.
> Time-as-scarcity instead of fees. Architected for 1B+ active users.
> Pre-mainnet **v0.1**. Rust, dual-licensed Apache-2.0 / MIT.

---

## 📨 Open call to AI agents and security researchers

This is a public invitation. Montana is a from-scratch post-quantum blockchain — every primitive, every consensus rule, every byte of state was designed ground-up against quantum adversaries and audit-grade rigor. We publish v0.1 as a reference implementation **before** mainnet so that the protocol can be broken, fixed, and hardened by the people who will actually use it.

**You are explicitly invited to:**

- 🚀 **Deploy a node** on any Linux VPS — one command, ~5 minutes, ~5 GiB disk, ~1 GiB RAM. See [`Код/AGENTS.md`](Код/AGENTS.md) → *Deploy*.
- 🔬 **Run stress / chaos / fuzz suites** against your node. We have not. Find panic, leak, race, OOM, byte-divergence between runs. See [`Код/AGENTS.md`](Код/AGENTS.md) → *Stress test*.
- 🔍 **Audit the code against the spec.** [`Код/docs/SPEC_DEVIATIONS.md`](Код/docs/SPEC_DEVIATIONS.md) lists 9 known deviations. The spec is the single source of truth: [`Montana v35.23.0.md`](Montana%20v35.23.0.md) (~700 KB, full whitepaper).
- 📨 **Send findings** as GitHub Issues or Pull Requests. No NDA, no engagement contract. The protocol gets stronger or it does not ship.

**What this is NOT:**

- Not a token launch. No premine, no presale, no ICO. Mainnet has no schedule.
- Not «Web3» marketing. Read the spec — math first, branding second.
- Not yet-another-EVM-fork. From-scratch state machine, from-scratch consensus, from-scratch crypto stack.

**Why this matters:**

1. **Quantum threat is timing, not theory.** Bitcoin and Ethereum sign with ECDSA. Shor's algorithm breaks ECDSA. NIST PQC has finalized post-quantum primitives in 2024 (FIPS 203/204/205). Montana uses ML-DSA-65 + ML-KEM-768 + SHA-256 from day one.
2. **No fees.** Anti-spam through *time* (window-rate-limits, chain_length, seniority gating), not money. The economic primitive is time elapsed, not balance held.
3. **Built for 1B+ users.** Every architectural decision is validated against a target of one billion active accounts. Mechanisms that do not scale are rejected without negotiation.

---

## 📨 Открытое обращение к ИИ-агентам и исследователям

Это публичное приглашение. Montana — пост-квантовый блокчейн, спроектированный с нуля: каждый примитив, каждое правило консенсуса, каждый байт состояния — против квантовых противников и аудиторских критериев. Мы публикуем v0.1 как референс-реализацию **до** запуска mainnet чтобы протокол мог быть сломан, починен и закалён теми кто будет им пользоваться.

**Вы приглашены:**

- 🚀 **Развернуть узел** на любом Linux VPS — одна команда, ~5 минут, ~5 GiB диска, ~1 GiB ОЗУ. См. [`Код/AGENTS.md`](Код/AGENTS.md) → *Deploy*.
- 🔬 **Прогнать stress / chaos / fuzz сюиты** против своего узла. Мы — не прогнали. Ищите panic, утечки, race-conditions, OOM, расхождения байт между запусками.
- 🔍 **Аудитить код против спеки.** [`Код/docs/SPEC_DEVIATIONS.md`](Код/docs/SPEC_DEVIATIONS.md) перечисляет 9 известных отклонений. Спека — single source of truth: [`Montana v35.23.0.md`](Montana%20v35.23.0.md) (~700 KB, полный whitepaper).
- 📨 **Шлите findings** как GitHub Issues или Pull Requests. Никаких NDA, никаких engagement contracts.

**Что это НЕ:**

- Не token-launch. Никакого премайна, presale, ICO. У mainnet нет даты.
- Не «Web3»-маркетинг. Читайте спеку — математика первична, бренд вторичен.
- Не yet-another-EVM-fork. Своя state machine, свой консенсус, свой крипто-стек, всё с нуля.

**Зачем это нужно:**

1. **Квантовая угроза — вопрос времени, не теории.** Bitcoin и Ethereum подписывают через ECDSA. Алгоритм Шора ломает ECDSA. NIST PQC финализировал пост-квантовые примитивы в 2024 (FIPS 203/204/205). Montana использует ML-DSA-65 + ML-KEM-768 + SHA-256 с первого дня.
2. **Без комиссий.** Анти-спам через *время* (окно-лимиты, chain_length, seniority-gating), не через деньги. Экономический примитив — прошедшее время, а не удержанный баланс.
3. **Под 1B+ пользователей.** Каждое архитектурное решение валидируется против цели в один миллиард активных аккаунтов. Механизмы которые не масштабируются — отклоняются без обсуждения.

---

## ⚡ Quick start

**Узел Montana + VPN endpoint на чистом Linux VPS, одной командой:**

```bash
git clone https://github.com/efir369999/Montana.git /opt/montana && \
sudo bash /opt/montana/Код/scripts/install-vps-full.sh
```

**Только узел:**

```bash
sudo bash /opt/montana/Код/scripts/install-vps.sh
```

**Только VPN endpoint:**

```bash
sudo bash /opt/montana/Код/montana-vpn/install.sh
```

Полный installer выводит 24-словную recovery мнемонику для узла + VLESS URL для VPN. Сохрани мнемонику сразу — единственный backup.

---

## 🗺 Где что лежит

| Путь | Что |
|------|-----|
| [`Код/AGENTS.md`](Код/AGENTS.md) | **Точка входа для ИИ-агентов.** Deploy + stress-test + report findings |
| [`Montana v35.23.0.md`](Montana%20v35.23.0.md) | Полная спецификация протокола (whitepaper) |
| [`Montana App v3.11.0.md`](Montana%20App%20v3.11.0.md) | Спецификация клиентского приложения |
| [`Код/`](Код/) | Rust workspace — 17 crates, 9 milestones |
| [`Код/montana-vpn/`](Код/montana-vpn/) | Reality-VPN endpoint (опционально, рядом с узлом) |
| [`Код/scripts/install-vps-full.sh`](Код/scripts/install-vps-full.sh) | Узел + VPN одной командой |
| [`Агенты/`](Агенты/) | Роли ИИ-агентов протокола (АРХИТЕКТОР-СПЕКИ, КРИТИК-СПЕКИ, КООРДИНАТОР, etc.) |
| [`Код/AUDIT.md`](Код/AUDIT.md) | Audit package для external firm engagement |
| [`Код/ROADMAP.md`](Код/ROADMAP.md) | 9 milestones, M1-M6+M9 ready, M7-M8 in progress |
| [`Код/docs/SPEC_DEVIATIONS.md`](Код/docs/SPEC_DEVIATIONS.md) | 9 documented deviations (M5-singleton phase) |
| [`SECURITY.md`](SECURITY.md) | Security policy, как репортить уязвимости |
| [`Генезис.md`](%D0%93%D0%B5%D0%BD%D0%B5%D0%B7%D0%B8%D1%81.md) | Genesis-послание автора (cypherpunk-style, аналог Bitcoin Genesis headline). Будет вшито в Genesis Decree протокола. |
| [`Архив/`](Архив/) | Исторические версии спецификации |

## Status

**M1 + M2 + M3 + M4 + M5 + M6 + M9 — ready for external audit firm engagement.**

| Layer | Status | Tests |
|-------|--------|-------|
| M1 foundational primitives | ✅ ready | 100+ unit + 51 NIST KAT |
| M2 state foundation | ✅ ready | 95+ unit + 60 invariants |
| M3 apply_proposal | ✅ ready | 89 unit + 29 invariants |
| M4 consensus mechanics | ✅ ready | 187 unit + 85 invariants |
| M5 persistence | ✅ ready | 27 unit + 17 invariants |
| M6 network | ✅ ready | 110 unit + 14 incl. 3 e2e two-node |
| M9 conformance | ✅ ready | 2 byte-exact verify |
| M7 fast sync | ⏳ TODO | — |
| M8 node binary | 🔄 in progress | partial (9 documented SPEC_DEVIATIONS) |

## License

Dual-licensed under Apache-2.0 OR MIT, at your choice.

- [`LICENSE`](LICENSE) — Apache-2.0 (root, applies to spec + Агенты/ + supporting files)
- [`Код/LICENSE-APACHE`](Код/LICENSE-APACHE) — Apache-2.0 (Rust workspace)
- [`Код/LICENSE-MIT`](Код/LICENSE-MIT) — MIT (Rust workspace, choose either)

## Contact

- 🐛 **Issues / Findings:** [github.com/efir369999/Montana/issues](https://github.com/efir369999/Montana/issues)
- 📜 **Pull Requests:** прямые PRs приветствуются
- 📄 **Whitepaper (Сатоши style):** [`Whitepaper Montana.md`](Whitepaper Montana.md) — academic paper в стиле Bitcoin paper, для рассылки в [metzdowd cryptography list](metzdowd-email.txt)
- 🚫 **Никаких email/Discord/Telegram** — публичный on-record review

---

*Pre-mainnet. Break it, fix it, send PRs. Время — это элегантные деньги.*
