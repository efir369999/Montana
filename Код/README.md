# Montana — Reference Implementation (Rust)

Reference implementation протокола Montana на Rust. Byte-for-byte соответствие спецификации.

- **Спецификация:** Montana — текущая версия и путь к файлу зафиксированы в `VERSION.md`
- **Версия реализации:** см. `VERSION.md`
- **План разработки:** см. `ROADMAP.md` — 17 crates, 9 milestones, текущий статус
- **Audit package:** см. `AUDIT.md` — pre-audit self-attestation для external firm engagement
- **Audit checklist:** `docs/audit-checklist.md`
- **Security cards:** `docs/security-cards.md`
- **Spec deviations:** `docs/SPEC_DEVIATIONS.md`
- **Build instructions:** `docs/build-from-source.md`
- **Роль архитектора кода:** `CLAUDE.md`
- **Роль критика реализации:** `CRITIC.md`

## Статус

**M1 + M2 + M3 + M4 + M5 + M6 + M9 — READY для external audit firm engagement.**

| Layer | Status | Crates | LOC | Tests |
|-------|--------|--------|-----|-------|
| M1 foundational | ✅ ready | mt-codec, mt-crypto, mt-crypto-native, mt-mnemonic | ~2000 | 100+ unit + 51 NIST KAT |
| M2 state foundation | ✅ ready | mt-merkle, mt-genesis, mt-state, mt-timechain | 1821 | 95+ unit + 60 invariants |
| M3 apply_proposal | ✅ ready | mt-account | 2556 | 89 unit + 29 invariants |
| M4 consensus mechanics | ✅ ready | mt-lottery, mt-consensus, mt-entry | 3858 | 187 unit + 85 invariants |
| M5 persistence | ✅ ready | mt-store | 955 | 27 unit + 17 invariants |
| **M6 network** | ✅ ready | **mt-net, mt-net-transport** | ~3300 | **110 + 14** (вкл. 3 e2e two-node) |
| **M9 conformance** | ✅ ready | **mt-conformance** | ~150 | **2 byte-exact verify** |
| M7 fast sync | ⏳ TODO | mt-sync | — | — |
| M8 node binary | 🔄 in progress | montana-node | ~600 | partial |

## Требования

- Rust stable, минимум 1.70 (закреплено в `rust-toolchain.toml`)
- `cargo`, `git`
- macOS / Linux (Windows partial via libp2p Windows support)

## Сборка и проверка

Четыре команды должны быть зелёными перед любым commit:

```bash
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test --all
cargo build --all --release
```

## Conformance verification

Cross-implementation byte-exact verification против Rust reference:

```bash
cargo test -p mt-conformance
cargo test -p mt-net-transport --features testing
cargo test -p mt-net --features testing
```

## Структура

```
.
├── Cargo.toml             workspace root, shared deps и profiles
├── rust-toolchain.toml    pinned Rust channel
├── rustfmt.toml           настройки форматтера
├── clippy.toml            настройки линтера
├── VERSION.md             pin на версию спеки
├── AUDIT.md               audit package для external firm engagement
├── ROADMAP.md             план разработки 9 milestones
├── CLAUDE.md              роль архитектора реализации
├── CRITIC.md              роль критика реализации
├── docs/
│   ├── audit-checklist.md  pre-audit self-attestation
│   ├── security-cards.md   crypto primitives security analysis
│   ├── SPEC_DEVIATIONS.md  known deviations from spec
│   └── build-from-source.md reproducible build instructions
├── crates/
│   ├── mt-codec, mt-crypto, mt-crypto-native, mt-mnemonic    M1 layer
│   ├── mt-merkle, mt-genesis, mt-state, mt-timechain         M2 layer
│   ├── mt-account                                            M3 layer
│   ├── mt-lottery, mt-consensus, mt-entry                    M4 layer
│   ├── mt-store                                              M5 layer
│   ├── mt-net, mt-net-transport                              M6 layer
│   ├── mt-conformance                                        M9 layer
│   ├── montana-node                                          M8 (in progress)
│   └── mt-examples                                           manual validation
└── bench/                 VDF benchmark (standalone, не в workspace)
```

## Лицензия

MIT OR Apache-2.0 (стандарт Rust-экосистемы).
