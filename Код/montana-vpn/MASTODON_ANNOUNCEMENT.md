# Mastodon-анонс — Montana v0.1

Готовый текст для рассылки в Mastodon (формат — как Meta анонсировала LLaMa: лаконично, технически, без маркетинга, со ссылками на код, whitepaper и инструкции для агентов).

Два варианта — RU и EN. Каждый влезает в один Mastodon-пост (≤500 символов основной части), длинная часть пускается тредом.

---

## RU — короткий пост (главный)

```
Публикуем Montana v0.1 — open-source референс-имплементацию post-quantum блокчейна с VDF-консенсусом и time-as-scarcity монетарной моделью. Без комиссий, без ECDSA, под нагрузку 1B+ пользователей.

Reference на Rust под Apache-2.0. Pre-mainnet, ищем агентов и исследователей которые поднимут узлы и пришлют находки.

🔗 github.com/efir369999/Montana
📄 Whitepaper: спека v35.23.0 в репо
🤖 AGENTS.md — как развернуть и тестировать
```

## RU — расширенный (тредом)

**Пост 1 — суть**

```
Сегодня публикуем Montana v0.1 — post-quantum reference blockchain.

Что нового:
• ML-DSA-65 + ML-KEM-768 (FIPS 204/203, NIST PQC)
• VDF-based TimeChain (SHA-256, D=325M iter/окно)
• Time-as-scarcity вместо денежной (anti-spam через окна, не fees)
• Архитектура под 1B+ активных пользователей

Дальше треды.
```

**Пост 2 — почему сейчас**

```
Зачем ещё один блокчейн:

1. Shor-quantum атака на ECDSA — вопрос времени, не «если». Bitcoin/Ethereum уязвимы.
2. Fee-burdened L1s держат барьер для adoption.
3. Mass-surveillance push требует privacy-by-default.
4. Существующие PQ-предложения — bolted-on, не ground-up.

Montana — ground-up post-quantum от primitives до consensus.
```

**Пост 3 — что выпускаем**

```
v0.1 включает:

• montana-node — узел консенсуса (singleton M5, M6 сетевой слой в работе)
• Полная спецификация v35.23.0 (~600KB markdown) как whitepaper
• montana-vpn bundle — Reality endpoint для self-hosted exit-node (опционально, рядом с узлом)
• install-vps-full.sh — узел + VPN одной командой на чистый Linux VPS

Лицензия — Apache-2.0.
```

**Пост 4 — призыв к ИИ-агентам**

```
Открыто для AI-агентов и security-исследователей:

⚡ Развернуть узел на любом Linux VPS (~5 GiB, 1 GiB RAM)
🔬 Прогнать stress/chaos тесты, искать panic/leak/race
🔍 Audit код против спеки (9 SPEC_DEVIATIONS уже задокументированы)
📨 Findings → GitHub Issues / pull requests

AGENTS.md в репо — точка входа.
```

**Пост 5 — ссылки**

```
Полный набор:

📂 Code: github.com/efir369999/Montana
📄 Spec/whitepaper: Протокол/Montana v35.23.0.md
🛠 Installer: scripts/install-vps-full.sh
🤖 AGENTS.md
🛡 montana-vpn (Reality endpoint, рядом с узлом)
🗺 ROADMAP: М6 сетевой слой, потом mainnet

Pre-mainnet. Ломайте, чините, шлите PR.
```

---

## EN — short post (main)

```
Today we publish Montana v0.1 — open-source post-quantum blockchain reference implementation. VDF-based TimeChain consensus, no fees, time-as-scarcity instead of money, designed for 1B+ users.

Rust reference under Apache-2.0. Pre-mainnet. Calling AI agents and researchers to deploy nodes and file findings.

🔗 github.com/efir369999/Montana
📄 Whitepaper: spec v35.23.0 in-repo
🤖 AGENTS.md — deploy + test
```

## EN — extended (thread)

**Post 1 — gist**

```
We publish Montana v0.1 today — a post-quantum reference blockchain.

What's new:
• ML-DSA-65 + ML-KEM-768 (FIPS 204/203, NIST PQC)
• VDF-based TimeChain (SHA-256, D=325M iter/window)
• Time-as-scarcity in place of fees (anti-spam via temporal windows, not money)
• Architecture targeting 1B+ active users

Threads below.
```

**Post 2 — why now**

```
Why one more blockchain:

1. Shor-quantum attack on ECDSA is a matter of when, not if. Bitcoin/Ethereum are vulnerable.
2. Fee-burdened L1s keep adoption gated.
3. Mass-surveillance pressure demands privacy-by-default.
4. Existing PQ proposals are bolt-ons, not ground-up.

Montana is ground-up post-quantum from primitives to consensus.
```

**Post 3 — what's in the release**

```
v0.1 ships:

• montana-node — consensus node (singleton M5, M6 network layer in progress)
• Full spec v35.23.0 (~600KB markdown) as whitepaper
• montana-vpn bundle — Reality-based self-hosted exit-node (optional, lives next to the node)
• install-vps-full.sh — node + VPN in one command on a clean Linux VPS

License: Apache-2.0.
```

**Post 4 — call to AI agents**

```
Open invitation to AI agents and security researchers:

⚡ Deploy a node on any Linux VPS (~5 GiB, 1 GiB RAM)
🔬 Run stress/chaos suites, hunt panic/leak/race
🔍 Audit the code against the spec (9 SPEC_DEVIATIONS already documented)
📨 Findings → GitHub Issues / PRs

AGENTS.md in the repo is the entry point.
```

**Post 5 — links**

```
Full set:

📂 Code: github.com/efir369999/Montana
📄 Spec/whitepaper: Протокол/Montana v35.23.0.md
🛠 Installer: scripts/install-vps-full.sh
🤖 AGENTS.md
🛡 montana-vpn (Reality endpoint, side-by-side with the node)
🗺 ROADMAP: M6 network layer, then mainnet

Pre-mainnet. Break it, fix it, send PRs.
```

---

## Где разместить

Этот файл — **источник** анонса для ручной публикации. Не для GitHub release notes (релиз использует свои собственные).

Готовый workflow:

1. Скопировать нужный вариант (RU короткий / RU тред / EN короткий / EN тред)
2. Опубликовать в Mastodon-инстансе автора
3. Шарить ссылку на пост в чатах ИИ-агентов / Telegram-каналах исследователей

## Стилистика — почему как LLaMa

Meta-анонс LLaMa (Feb 2023) задал стандарт для открытых тех-релизов:

- Один лаконичный технический заголовок без маркетинга
- 2-3 параграфа сути (что это, почему сейчас, чем отличается)
- Прямые ссылки: код + paper + access
- Прозрачность статуса (research preview, beta, etc.)
- Призыв к сообществу с конкретным action ("file issues / send PRs")

Этот же шаблон применим к Montana v0.1: post-quantum reference implementation, открытое приглашение исследователям, всё через стандартные открытые каналы (GitHub, public repo, Apache license).
