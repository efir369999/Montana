# Reproducible Build from Source — Audit Reproduction Guide

Полная инструкция для external auditor / independent reviewer для byte-identical reproduction Montana reference implementation от source.

См. также: `AUDIT.md` (audit package overview), `docs/audit-checklist.md` (per-layer self-attestation), `docs/security-cards.md` (crypto primitives).

---

## 1. Prerequisites

### Toolchain

| Component | Version | Source | Verification |
|-----------|---------|--------|--------------|
| Rust toolchain | stable, ≥ 1.70 (pinned via `rust-toolchain.toml`) | rustup.rs | `rustc --version` |
| Cargo | bundled | bundled | `cargo --version` |
| Git | ≥ 2.30 | system | `git --version` |
| OpenSSL 3.5 LTS | =3.5.5 (pinned via `openssl-src` workspace dep) | vendored через `openssl-src` crate | autobuilt by Cargo |
| C compiler | clang ≥ 13 либо gcc ≥ 11 | system | for openssl-src vendored build |

### Hardware reference (для timing benchmark verification)

- Genesis hardware reference (per spec [I-18]): Apple iMac 24-inch M1 2021, 8 GB unified memory, macOS Sequoia 15.7.3, Rust 1.92.0 stable, sha2 crate 0.10.9 + ARM SHA-2 hardware extensions
- D₀ benchmark expected: median single-thread SHA-256 rate 5.097 MH/s
- Other hardware: D₀ value remains 325 000 000 (Genesis Decree authoritative); только SSHA wall-clock varies

---

## 2. Clone & checkout

```bash
git clone <repo-url> montana
cd montana/Протокол/Code

# Verify HEAD matches expected commit (audit signature confirms specific revision)
git rev-parse HEAD
# Expected for the v35.25.1 audit cycle: use the audited commit hash or a later forward-compatible revision.
```

---

## 3. First build

```bash
# Single-core/single-process per .cargo/config.toml (anti-overheat policy для PBKDF2 tests)
cargo build --workspace --release
```

Expected duration:
- First build: 5-15 минут (libp2p ~120 transitive deps)
- Subsequent builds: 30-60 секунд (incremental)

---

## 4. Mandatory checks (4 green requirement)

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --workspace
cargo build --workspace --release
```

All four must exit with code 0.

---

## 5. Conformance verification

```bash
# M9 standalone test vectors
cargo test -p mt-conformance
# Expected: 2 tests pass (envelope_vectors_byte_exact + pow_target_byte_exact)

# M6 network layer
cargo test -p mt-net --features testing
# Expected: 96 unit + 14 integration = 110 tests pass

# M6 transport layer
cargo test -p mt-net-transport --features testing
# Expected: 11 unit + 3 e2e = 14 tests pass (включая two-node handshake +
#           proposal exchange + 512 KiB boundary)
```

---

## 6. NIST KAT verification (M1 cryptography)

```bash
cargo test -p mt-crypto --features testing
# Expected: NIST FIPS 204 ML-DSA-65 + FIPS 203 ML-KEM-768 byte-exact против
# ACVP-Server published vectors (50+ KAT cases pass)
```

---

## 7. Manual Validation Gate (interactive)

См. `ROADMAP.md` секцию «Локальный shakedown — Manual Validation Gate».

Сценарии 0-7 — interactive verification каждого механизма через example
binaries в `crates/mt-examples/`. Полное прохождение требует ~2-3 часов
ручного operator time.

```bash
cargo run --release --example m1_mnemonic recovery-fingerprint
cargo run --release --example m1_mnemonic keypair
cargo run --release --example m1_crypto keypair
# ... остальные scenarios см. ROADMAP
```

---

## 8. Reproducibility verification

Two independent builds на different machines должны дать byte-identical
binaries:

```bash
# Build 1
cargo build --release -p montana-node
sha256sum target/release/montana-node > /tmp/build1.sha256

# Build 2 (другая machine, same toolchain)
cargo build --release -p montana-node
sha256sum target/release/montana-node > /tmp/build2.sha256

# Сравнить
diff /tmp/build1.sha256 /tmp/build2.sha256
# Expected: empty output (byte-identical)
```

Note: на момент M6 closure, `montana-node` находится в M8 SPEC_DEVIATIONS rewrite
phase (см. `docs/SPEC_DEVIATIONS.md` DEV-001..009). Для full reproducibility
verification защищать через CI matrix builds.

---

## 9. Audit firm engagement

Recommended firms (см. AUDIT.md «Audit firm engagement» section):

- **NCC Group** — strong PQ crypto + iOS wallet experience
- **Trail of Bits** — blockchain wallet specialty (Slither, Echidna)
- **Cure53** — Berlin, mobile + crypto + browser
- **Quarkslab** — French, hardware + iOS
- **Cryspen** — formal verification (HACL\* contributors), для PQ crypto bottom layer

Estimated cost: $50k-$250k за 4-8 недель полный scope audit M1+M2+M3+M4+M5+M6+M9.

---

## 10. Contact / questions

- Spec issues: `Протокол/CRITIC.md` (роль критика спеки)
- Code issues: `Code/CRITIC.md` (роль критика реализации)
- Audit findings: open issue в репозитории либо direct contact автора (Alejandro Montana)
