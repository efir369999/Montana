# Montana

> **Open-source post-quantum reference blockchain.** Sequential-delay TimeChain consensus over SHA-256.
> Time-as-scarcity instead of fees. Baseline billion-account target, with M7 fast-sync benchmarks pending.
> Pre-mainnet **v0.2** spec package — Rust implementation `0.1.1`.
> Rust, dual-licensed Apache-2.0 / MIT.

---

## 🏛 Three-layer spec architecture (since v0.2)

The protocol is now specified as three layered documents — each independently auditable:

| Layer | Spec | Lines | Scope |
|-------|------|-------|-------|
| 1. Protocol | [`Montana Protocol v35.25.1.md`](Montana%20Protocol%20v35.25.1.md) | 4416 | State machine, crypto primitives (ML-DSA-65, ML-KEM-768, SHA-256), sequential-delay TimeChain, Lottery, Account/Node tables, Genesis Decree, apply_proposal pipeline, операции консенсуса |
| 2. Network | [`Montana Network v1.1.0.md`](Montana%20Network%20v1.1.0.md) | 3673 | libp2p транспорт, Identity-Bound Tunnel (IBT), Transport Randomness, PeerRecord, Mesh Transport, sync protocols, Threat Model сетевого слоя, Binding KAT vectors, Final Gate audit M6 |
| 3. App | [`Montana App v3.12.0.md`](Montana%20App%20v3.12.0.md) | 3706 | UI, Wallet, Messenger (Double Ratchet PQ), Channels, Contacts, Profile, Junona AI agent, Browser, Premium, экономика приложений |

Layer dependency direction: Protocol (low) ← Network (mid) ← App (high). Each layer depends on layers below it; no upward dependency.

---

> **Live network status:** [efir.org/explorer/](https://efir.org/explorer/) — dashboard трёх узлов (Moscow / Helsinki / Frankfurt), auto-refresh каждые 60 сек. Текущий snapshot: [STATUS.md](STATUS.md).

## 📨 Open call to AI agents and security researchers

This is a public invitation. Montana is a from-scratch post-quantum blockchain — every primitive, every consensus rule, every byte of state was designed ground-up against quantum adversaries and audit-grade rigor. We publish this pre-mainnet reference implementation against the v0.2 spec package **before** mainnet so that the protocol can be broken, fixed, and hardened by the people who will actually use it.

**You are explicitly invited to:**

- 🚀 **Deploy a node** on any Linux VPS — one command, ~5 minutes, ~5 GiB disk, ~1 GiB RAM. See [`Code/AGENTS.md`](Code/AGENTS.md) → *Deploy*.
- 🔬 **Run stress / chaos / fuzz suites** against your node. We have not. Find panic, leak, race, OOM, byte-divergence between runs. See [`Code/AGENTS.md`](Code/AGENTS.md) → *Stress test*.
- 🔍 **Audit the code against the spec.** [`Code/docs/SPEC_DEVIATIONS.md`](Code/docs/SPEC_DEVIATIONS.md) lists known deviations, acknowledgments, and closures. The spec is the single source of truth: [`Montana Protocol v35.25.1.md`](Montana%20Protocol%20v35.25.1.md) + [`Montana Network v1.1.0.md`](Montana%20Network%20v1.1.0.md) + [`Montana App v3.12.0.md`](Montana%20App%20v3.12.0.md).
- 📨 **Send findings** as GitHub Issues or Pull Requests. No NDA, no engagement contract. The protocol gets stronger or it does not ship.

**What this is NOT:**

- Not a token launch. No premine, no presale, no ICO. Mainnet has no schedule.
- Not «Web3» marketing. Read the spec — math first, branding second.
- Not yet-another-EVM-fork. From-scratch state machine, from-scratch consensus, from-scratch crypto stack.

**Why this matters:**

1. **Quantum threat is timing, not theory.** Bitcoin and Ethereum sign with ECDSA. Shor's algorithm breaks ECDSA. NIST PQC finalized ML-KEM and ML-DSA in 2024 (FIPS 203/204). Montana uses ML-DSA-65 for consensus signatures and ML-KEM-768 at the application layer; transport PQ confidentiality is tracked separately through the Noise_PQ migration.
2. **No fees.** Anti-spam through *time* (window-rate-limits, chain_length, seniority gating), not money. The economic primitive is time elapsed, not balance held.
3. **Built toward billion-account scale.** `AccountRecord` is 2 059 bytes, so 1B active accounts imply about 2.06 TB of state; M7 fast-sync benchmarks are the gate for claiming comfortable onboarding at that scale.

---

## 📨 Открытое обращение к ИИ-агентам и исследователям

Это публичное приглашение. Montana — пост-квантовый блокчейн, спроектированный с нуля: каждый примитив, каждое правило консенсуса, каждый байт состояния — против квантовых противников и аудиторских критериев. Мы публикуем pre-mainnet референс-реализацию под v0.2 spec package **до** запуска mainnet чтобы протокол мог быть сломан, починен и закалён теми кто будет им пользоваться.

**Вы приглашены:**

- 🚀 **Развернуть узел** на любом Linux VPS — одна команда, ~5 минут, ~5 GiB диска, ~1 GiB ОЗУ. См. [`Code/AGENTS.md`](Code/AGENTS.md) → *Deploy*.
- 🔬 **Прогнать stress / chaos / fuzz сюиты** против своего узла. Мы — не прогнали. Ищите panic, утечки, race-conditions, OOM, расхождения байт между запусками.
- 🔍 **Аудитить код против спеки.** [`Code/docs/SPEC_DEVIATIONS.md`](Code/docs/SPEC_DEVIATIONS.md) перечисляет известные отклонения, acknowledgments и закрытия. Спека — single source of truth: [`Montana Protocol v35.25.1.md`](Montana%20Protocol%20v35.25.1.md) + [`Montana Network v1.1.0.md`](Montana%20Network%20v1.1.0.md) + [`Montana App v3.12.0.md`](Montana%20App%20v3.12.0.md).
- 📨 **Шлите findings** как GitHub Issues или Pull Requests. Никаких NDA, никаких engagement contracts.

**Что это НЕ:**

- Не token-launch. Никакого премайна, presale, ICO. У mainnet нет даты.
- Не «Web3»-маркетинг. Читайте спеку — математика первична, бренд вторичен.
- Не yet-another-EVM-fork. Своя state machine, свой консенсус, свой крипто-стек, всё с нуля.

**Зачем это нужно:**

1. **Квантовая угроза — вопрос времени, не теории.** Bitcoin и Ethereum подписывают через ECDSA. Алгоритм Шора ломает ECDSA. NIST PQC финализировал ML-KEM и ML-DSA в 2024 (FIPS 203/204). Montana использует ML-DSA-65 для consensus-подписей и ML-KEM-768 на application layer; transport PQ confidentiality закрывается отдельно через миграцию Noise_PQ.
2. **Без комиссий.** Анти-спам через *время* (окно-лимиты, chain_length, seniority-gating), не через деньги. Экономический примитив — прошедшее время, а не удержанный баланс.
3. **К масштабу миллиарда аккаунтов.** `AccountRecord` занимает 2 059 байт, поэтому 1B активных аккаунтов дают около 2.06 TB state; M7 fast-sync benchmarks — gate для уверенного заявления про onboarding на этом масштабе.

---

## ⚡ Quick start

**Узел Montana + VPN endpoint на чистом Linux VPS, одной командой:**

```bash
git clone https://github.com/efir369999/Montana.git /opt/montana && \
sudo bash /opt/montana/Code/scripts/install-vps-full.sh
```

**Только узел:**

```bash
sudo bash /opt/montana/Code/scripts/install-vps.sh
```

**Только VPN endpoint:**

```bash
sudo bash /opt/montana/Code/montana-vpn/install.sh
```

Полный installer выводит 24-словную recovery мнемонику для узла + VLESS URL для VPN. Сохрани мнемонику сразу — единственный backup.

---

## 🗺 Где что лежит

| Путь | Что |
|------|-----|
| [`Code/AGENTS.md`](Code/AGENTS.md) | **Точка входа для ИИ-агентов.** Deploy + stress-test + report findings |
| [`Montana Protocol v35.25.1.md`](Montana%20Protocol%20v35.25.1.md) | Полная спецификация протокола (whitepaper) |
| [`Montana App v3.12.0.md`](Montana%20App%20v3.12.0.md) | Спецификация клиентского приложения |
| [`Code/`](Code/) | Rust workspace — 17 crates, 9 milestones |
| [`Code/montana-vpn/`](Code/montana-vpn/) | Reality-VPN endpoint (опционально, рядом с узлом) |
| [`Code/scripts/install-vps-full.sh`](Code/scripts/install-vps-full.sh) | Узел + VPN одной командой |
| [`Agents/`](Agents/) | Роли ИИ-агентов протокола (АРХИТЕКТОР-СПЕКИ, КРИТИК-СПЕКИ, КООРДИНАТОР, etc.) |
| [`Code/AUDIT.md`](Code/AUDIT.md) | Audit package для external firm engagement |
| [`Code/ROADMAP.md`](Code/ROADMAP.md) | 9 milestones, M1-M6+M9 ready, M7-M8 in progress |
| [`Code/docs/SPEC_DEVIATIONS.md`](Code/docs/SPEC_DEVIATIONS.md) | Known deviations, acknowledgments, and closures (pre-mainnet node phase) |
| [`SECURITY.md`](SECURITY.md) | Security policy, как репортить уязвимости |
| [`Генезис.md`](%D0%93%D0%B5%D0%BD%D0%B5%D0%B7%D0%B8%D1%81.md) | Genesis-послание автора (cypherpunk-style, аналог Bitcoin Genesis headline). Будет вшито в Genesis Decree протокола. |
| [`Archive/`](Archive/) | Исторические версии спецификации |

## Status

**M1 + M2 + M3 + M4 + M5 + M6 + M9 — ready for external audit firm engagement.**

| Layer | Status | Tests |
|-------|--------|-------|
| M1 foundational primitives | ✅ ready | 100+ unit + 51 NIST KAT |
| M2 state foundation | ✅ ready | 95+ unit + 60 invariants |
| M3 apply_proposal | ✅ ready | 89 unit + 29 invariants |
| M4 consensus mechanics | ✅ ready | 187 unit + 85 invariants |
| M5 persistence | ✅ ready | 27 unit + 17 invariants |
| M6 network | ✅ ready | 127 tests: mt-net 112 + mt-net-transport 15, incl. 3 e2e two-node |
| M9 conformance | ✅ ready | 2 byte-exact verify |
| M7 fast sync | ⏳ TODO | — |
| M8 node binary | 🔄 in progress | partial; DEV-012 multi-node proposal apply remains open |

## License

Dual-licensed under Apache-2.0 OR MIT, at your choice.

- [`LICENSE`](LICENSE) — Apache-2.0 (root, applies to spec + Agents/ + supporting files)
- [`Code/LICENSE-APACHE`](Code/LICENSE-APACHE) — Apache-2.0 (Rust workspace)
- [`Code/LICENSE-MIT`](Code/LICENSE-MIT) — MIT (Rust workspace, choose either)

## Contact

- 🐛 **Issues / Findings:** [github.com/efir369999/Montana/issues](https://github.com/efir369999/Montana/issues)
- 📜 **Pull Requests:** прямые PRs приветствуются
- 📄 **Whitepaper (Сатоши style):** [`Whitepaper Montana.md`](Whitepaper Montana.md) — academic paper в стиле Bitcoin paper, для рассылки в [metzdowd cryptography list](metzdowd-email.txt)
- 🚫 **Никаких email/Discord/Telegram** — публичный on-record review

---

*Pre-mainnet. Break it, fix it, send PRs. Время — это элегантные деньги.*
