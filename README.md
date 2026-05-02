# Montana — Post-Quantum Blockchain Protocol

> Specification v35.23.0 + Rust implementation. Preparing for external security audit.

## Что это

Montana — постквантовый блокчейн-протокол с **TimeChain** консенсусом на основе VDF (Verifiable Delay Functions). Цель — масштабирование на ≥1 миллиард активных пользователей с криптографической стойкостью к квантовым атакам.

Ключевые свойства:
- **Постквантовая криптография**: ML-DSA-65 (FIPS 204), ML-KEM-768 (FIPS 203), SHA-256, HKDF
- **TimeChain**: VDF-based последовательное доказательство времени (≠ Proof-of-Work, ≠ Proof-of-Stake)
- **Privacy by default**: приватная сеть, пользователь сам выбирает что раскрыть
- **Audit-ready**: 53/53 internal critic findings закрыты (M6 + M9)

## Структура репозитория

| Путь | Что |
|---|---|
| `Montana v35.23.0.md` | **Актуальная спецификация протокола** (700 KB, single source of truth) |
| `Montana App v3.11.0.md` | Спецификация приложения (iOS / macOS) |
| `Код/` | Rust workspace (16 crates) — реализация протокола |
| `Архив/` | История версий спецификации (60+ файлов) |
| `Внешний аудит/` | Отчёты внутренних критиков (Pass 1-17) |
| `CLAUDE.md`, `CRITIC.md` | Роли архитектора и критика |
| `crypto/`, `Montana wordlist.txt` | Вспомогательные материалы |

## Сборка

```bash
cd Код
cargo build --workspace --release
cargo test --workspace
```

Подробная инструкция: [Код/docs/build-from-source.md](Код/docs/build-from-source.md).

## Аудит

- **Чек-лист**: [Код/docs/audit-checklist.md](Код/docs/audit-checklist.md)
- **ROADMAP**: [Код/ROADMAP.md](Код/ROADMAP.md)
- **Reproducible builds**: [Код/docs/build-from-source.md](Код/docs/build-from-source.md)

Текущий статус: M6 (Network Layer) + M9 (Anchor Pipeline) закрыты, готов к external audit firm engagement.

## Лицензия

Apache License 2.0 — см. [LICENSE](LICENSE).

## Безопасность

Уязвимости — см. [SECURITY.md](SECURITY.md). Не открывайте public issues для security-проблем.

---

## English TL;DR

Montana is a post-quantum blockchain protocol using VDF-based TimeChain consensus, targeting 1B+ users with quantum-resistant cryptography (ML-DSA-65, ML-KEM-768). Specification in `Montana v35.23.0.md`, Rust implementation in `Код/`. M6 (network) and M9 (anchor) milestones complete; ready for external security audit. See `Код/docs/build-from-source.md` for reproducible build verification, `Код/docs/audit-checklist.md` for the audit trail. Apache-2.0 licensed.
