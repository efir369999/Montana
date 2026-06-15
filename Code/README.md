# Montana

> Open-source post-quantum reference blockchain. Sequential-delay TimeChain consensus over SHA-256.
> Time-as-scarcity instead of fees. Baseline billion-account target, with M7 fast-sync benchmarks pending.
> Rust implementation `0.1.1` for the pre-mainnet **v0.2** spec package. Dual-licensed Apache-2.0 / MIT.

---

## 📨 Open call to AI agents and security researchers

This is an open invitation. Montana is a from-scratch post-quantum blockchain — every primitive, every consensus rule, every byte of state was designed ground-up against quantum adversaries and audit-grade rigor. We publish this pre-mainnet reference implementation against the v0.2 spec package **before** mainnet so that the protocol can be broken, fixed, and hardened by the people who will actually use it.

**You are explicitly invited to:**

- 🚀 **Deploy a node** on any Linux VPS — one command, ~5 minutes, ~5 GiB disk, ~1 GiB RAM. See [`AGENTS.md`](AGENTS.md) → *Deploy*.
- 🔬 **Run stress / chaos / fuzz suites** against your node. We have not. Find panic, leak, race, OOM, byte-divergence between runs. See [`AGENTS.md`](AGENTS.md) → *Stress test*.
- 🔍 **Audit the code against the spec.** [`docs/SPEC_DEVIATIONS.md`](docs/SPEC_DEVIATIONS.md) lists known deviations, acknowledgments, and closures. Find more. The spec is the single source of truth: [`../Montana Protocol v35.25.1.md`](../Montana%20Protocol%20v35.25.1.md) (~600 KB, full protocol specification).
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


## ⚡ Quick start

**Deploy node + VPN endpoint on a clean Linux VPS:**

```bash
git clone https://github.com/efir369999/Montana.git /opt/montana && \
sudo bash /opt/montana/Code/scripts/install-vps-full.sh
```

**Or just the node:**

```bash
sudo bash /opt/montana/Code/scripts/install-vps.sh
```

**Or just the VPN endpoint:**

```bash
sudo bash /opt/montana/Code/montana-vpn/install.sh
```

The full installer prints back the 24-word recovery mnemonic for the node and a VLESS URL for the VPN. Save the mnemonic immediately — it is the only backup.

---

## Reference implementation status

**M1 + M2 + M3 + M4 + M5 + M6 + M9 — ready for external audit firm engagement.**

| Layer | Status | Crates | LOC | Tests |
|-------|--------|--------|-----|-------|
| M1 foundational | ✅ ready | mt-codec, mt-crypto, mt-crypto-native, mt-mnemonic | ~2000 | 100+ unit + 51 NIST KAT |
| M2 state foundation | ✅ ready | mt-merkle, mt-genesis, mt-state, mt-timechain | 1821 | 95+ unit + 60 invariants |
| M3 apply_proposal | ✅ ready | mt-account | 2556 | 89 unit + 29 invariants |
| M4 consensus mechanics | ✅ ready | mt-lottery, mt-consensus, mt-entry | 3858 | 187 unit + 85 invariants |
| M5 persistence | ✅ ready | mt-store | 955 | 27 unit + 17 invariants |
| **M6 network** | ✅ ready | **mt-net, mt-net-transport** | ~3300 | **127 tests** (mt-net 112 + mt-net-transport 15, incl. 3 e2e two-node) |
| **M9 conformance** | ✅ ready | **mt-conformance** | ~150 | **2 byte-exact verify** |
| M7 fast sync | ⏳ TODO | mt-sync | — | — |
| M8 node binary | 🔄 in progress | montana-node | ~600 | partial; DEV-012 multi-node proposal apply remains open |

## Documentation map

| File | What |
|------|------|
| [`AGENTS.md`](AGENTS.md) | **Start here.** Deploy + stress-test + report findings |
| [`../Montana Protocol v35.25.1.md`](../Montana%20Protocol%20v35.25.1.md) | Full protocol specification |
| [`VERSION.md`](VERSION.md) | Spec target + impl version, single source of truth |
| [`ROADMAP.md`](ROADMAP.md) | 9 milestones, current status, next |
| [`AUDIT.md`](AUDIT.md) | Pre-audit self-attestation for external firm engagement |
| [`docs/audit-checklist.md`](docs/audit-checklist.md) | What internal audit covered |
| [`docs/security-cards.md`](docs/security-cards.md) | Per-primitive security analysis |
| [`docs/SPEC_DEVIATIONS.md`](docs/SPEC_DEVIATIONS.md) | Known deviations, acknowledgments, and closures |
| [`docs/build-from-source.md`](docs/build-from-source.md) | Reproducible build instructions |
| [`montana-vpn/README.md`](montana-vpn/README.md) | Companion VPN endpoint (xray Reality) |
| [`../Whitepaper Montana v0.1.0.md`](../Whitepaper Montana v0.1.0.md) | Whitepaper in Satoshi style |
| [`CLAUDE.md`](CLAUDE.md) | Architect role for code (process, not normative) |
| [`CRITIC.md`](CRITIC.md) | Critic role for implementation review |

## Build from source

```bash
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test --all
cargo build --all --release
```

All four must be green before any commit. See [`docs/build-from-source.md`](docs/build-from-source.md) for the reproducible-build path.

## Conformance verification

Cross-implementation byte-exact verification against this Rust reference:

```bash
cargo test -p mt-conformance
cargo test -p mt-net-transport --features testing
cargo test -p mt-net --features testing
```

## Repository layout

```
.
├── AGENTS.md                  ⭐ entry point for AI agents
├── README.md                  this file
├── VERSION.md                 spec pin
├── AUDIT.md                   audit package
├── ROADMAP.md                 9 milestones plan
├── LICENSE-APACHE             Apache-2.0 license text
├── LICENSE-MIT                MIT license text
├── Cargo.toml                 workspace root
├── rust-toolchain.toml        pinned Rust channel
├── .cargo/config.toml         single-core build (anti-overheat policy)
├── docs/                      audit-checklist, security-cards, SPEC_DEVIATIONS, build-from-source
├── crates/                    17 mt-* + montana-node + mt-examples
│   ├── mt-codec / mt-crypto / mt-crypto-native / mt-mnemonic    M1
│   ├── mt-merkle / mt-genesis / mt-state / mt-timechain         M2
│   ├── mt-account                                               M3
│   ├── mt-lottery / mt-consensus / mt-entry                     M4
│   ├── mt-store                                                 M5
│   ├── mt-net / mt-net-transport                                M6
│   ├── mt-conformance                                           M9
│   ├── montana-node                                             M8 (in progress)
│   └── mt-examples                                              manual validation harness
├── scripts/
│   ├── install-vps.sh         node-only installer (Linux VPS)
│   ├── install-vps-full.sh    node + VPN, one command
│   ├── install-local-mac.sh   node-only installer (macOS launchd)
│   └── pre-commit.sh          local pre-commit hook
├── montana-vpn/               companion VPN endpoint (xray Reality)
│   ├── README.md
│   ├── install.sh
│   ├── config-template/
│   └── docs/
└── bench/                     VDF benchmark (standalone)
```

## License

Dual-licensed under Apache-2.0 OR MIT, at your choice.

- [LICENSE-APACHE](LICENSE-APACHE)
- [LICENSE-MIT](LICENSE-MIT)

You may use, copy, modify, and redistribute under either license.

---

*Pre-mainnet. Break it, fix it, send PRs.*
