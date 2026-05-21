# Montana — Sovereign Invisibility Online

> **Open-source post-quantum reference blockchain.** Sequential-delay TimeChain consensus over SHA-256.
> Time-as-scarcity instead of fees. Baseline billion-account target.
> Production transport: **Noise_PQ XX (ML-KEM-768 + ML-DSA-65)** end-to-end.
> Pre-mainnet **v0.2** spec package — Rust implementation `0.1.1`.
> Rust, dual-licensed Apache-2.0 / MIT.

---

## Three-layer spec architecture (since v0.2)

The protocol is specified as three layered documents — each independently auditable:

| Layer | Spec | Lines | Scope |
|-------|------|-------|-------|
| 1. Protocol | [`Montana Protocol v35.25.1.md`](Montana%20Protocol%20v35.25.1.md) | 4412 | State machine, crypto primitives (ML-DSA-65, ML-KEM-768, SHA-256), sequential-delay TimeChain, lottery, Account / Node tables, Genesis Decree, `apply_proposal` pipeline, consensus operations |
| 2. Network | [`Montana Network v1.1.0.md`](Montana%20Network%20v1.1.0.md) | 3803 | libp2p transport, Noise_PQ XX (production), Identity-Bound Tunnel (IBT), transport randomness, PeerRecord, mesh transport, sync protocols, network-layer threat model, KAT vectors |
| 3. App | [`Montana App v3.12.0.md`](Montana%20App%20v3.12.0.md) | 3780 | UI, wallet, messenger (Double Ratchet PQ), channels, contacts, profile, Junona AI agent, browser, premium, application-layer economy |

Layer dependency direction: Protocol (low) ← Network (mid) ← App (high). Each layer depends on layers below it; no upward dependency.

---

> **Live network status:** [efir.org/explorer/](https://efir.org/explorer/) — three-node dashboard (Moscow / Helsinki / Frankfurt), 60 s auto-refresh. Current snapshot: [STATUS.md](STATUS.md).

## Sovereign Invisibility Online

Montana is the post-quantum sovereignty stack: identity, money, and traffic that no government or large adversary can decrypt today, tomorrow, or with a future quantum computer. Every primitive is post-quantum from the ground up — ML-DSA-65 for consensus signatures, ML-KEM-768 for application key encapsulation, ChaCha20-Poly1305 for the wire AEAD, SHA-256 for the sequential delay function. No classical Diffie-Hellman remains in the protocol layer. The transport handshake is Noise_PQ XX; the routing identity (libp2p PeerId) is the SHA-256 multihash of each node's ML-DSA-65 public key, so the cryptographic identity used in consensus and the routing identity used on the wire are bound to the same key material. A passive observer recording today's traffic cannot decrypt it after Shor's algorithm becomes practical, because there is no classical key exchange to break.

## Open call to AI agents and security researchers

This is a public invitation. Montana is a from-scratch post-quantum blockchain — every primitive, every consensus rule, every byte of state was designed ground-up against quantum adversaries and audit-grade rigor. We publish this pre-mainnet reference implementation against the v0.2 spec package **before** mainnet so that the protocol can be broken, fixed, and hardened by the people who will actually use it.

**You are explicitly invited to:**

- **Deploy a node** on any Linux VPS — one command, ~5 minutes, ~5 GiB disk, ~1 GiB RAM. See [`Code/AGENTS.md`](Code/AGENTS.md) → *Deploy*.
- **Run stress / chaos / fuzz suites** against your node. We have not. Find panic, leak, race, OOM, byte-divergence between runs. See [`Code/AGENTS.md`](Code/AGENTS.md) → *Stress test*.
- **Audit the code against the spec.** [`Code/docs/SPEC_DEVIATIONS.md`](Code/docs/SPEC_DEVIATIONS.md) lists known deviations, acknowledgments, and closures. The spec is the single source of truth: [`Montana Protocol v35.25.1.md`](Montana%20Protocol%20v35.25.1.md) + [`Montana Network v1.1.0.md`](Montana%20Network%20v1.1.0.md) + [`Montana App v3.12.0.md`](Montana%20App%20v3.12.0.md).
- **Send findings** as GitHub Issues or Pull Requests. No NDA, no engagement contract. The protocol gets stronger or it does not ship.

**What this is NOT:**

- Not a token launch. No premine, no presale, no ICO. Mainnet has no schedule.
- Not Web3 marketing. Read the spec — math first, branding second.
- Not yet-another-EVM-fork. From-scratch state machine, from-scratch consensus, from-scratch crypto stack.

**Why this matters:**

1. **Quantum threat is timing, not theory.** Bitcoin and Ethereum sign with ECDSA. Shor's algorithm breaks ECDSA. NIST PQC finalized ML-KEM and ML-DSA in 2024 (FIPS 203/204). Montana uses ML-DSA-65 for consensus signatures, ML-KEM-768 at the application layer, and Noise_PQ XX (ML-KEM-768 + ML-DSA-65) as the production transport. Transport confidentiality is post-quantum end-to-end.
2. **No fees.** Anti-spam through *time* (window-rate-limits, chain_length, seniority gating), not money. The economic primitive is time elapsed, not balance held.
3. **Built toward billion-account scale.** `AccountRecord` is 2 059 bytes, so 1B active accounts imply about 2.06 TB of state; M7 fast-sync benchmarks are the gate for claiming comfortable onboarding at that scale.

---

## Quick start

**Montana node + VPN endpoint on a clean Linux VPS, one command:**

```bash
git clone https://github.com/efir369999/Montana.git /opt/montana && \
sudo bash /opt/montana/Code/scripts/install-vps-full.sh
```

**Node only:**

```bash
sudo bash /opt/montana/Code/scripts/install-vps.sh
```

**VPN endpoint only:**

```bash
sudo bash /opt/montana/Code/montana-vpn/install.sh
```

The full installer prints a 24-word recovery mnemonic for the node and a VLESS URL for the VPN. Save the mnemonic immediately — it is the only backup.

---

## Repository layout

| Path | Contents |
|------|----------|
| [`Code/AGENTS.md`](Code/AGENTS.md) | **Entry point for AI agents.** Deploy + stress-test + report findings |
| [`Montana Protocol v35.25.1.md`](Montana%20Protocol%20v35.25.1.md) | Full protocol specification (whitepaper) |
| [`Montana Network v1.1.0.md`](Montana%20Network%20v1.1.0.md) | Network-layer specification (libp2p, Noise_PQ XX, IBT, mesh, sync) |
| [`Montana App v3.12.0.md`](Montana%20App%20v3.12.0.md) | Client application specification |
| [`Code/`](Code/) | Rust workspace — 17 crates, 9 milestones |
| [`Code/montana-vpn/`](Code/montana-vpn/) | Reality-VPN endpoint (optional, alongside the node) |
| [`Code/scripts/install-vps-full.sh`](Code/scripts/install-vps-full.sh) | Node + VPN one-command installer |
| [`Agents/`](Agents/) | Protocol AI agent roles (SPEC-ARCHITECT, SPEC-CRITIC, COORDINATOR, etc.) |
| [`Code/AUDIT.md`](Code/AUDIT.md) | Audit package for external firm engagement |
| [`Code/ROADMAP.md`](Code/ROADMAP.md) | 9 milestones — M1..M6 + M9 ready, M7..M8 in progress |
| [`Code/docs/SPEC_DEVIATIONS.md`](Code/docs/SPEC_DEVIATIONS.md) | Known deviations, acknowledgments, and closures (pre-mainnet node phase) |
| [`SECURITY.md`](SECURITY.md) | Security policy — how to report vulnerabilities |
| [`Genesis.md`](Genesis.md) | Genesis statement (cypherpunk-style; the analog of Bitcoin's Genesis headline). To be embedded in the protocol's Genesis Decree. |
| [`Archive/`](Archive/) | Historical spec versions |

## Status

**M1 + M2 + M3 + M4 + M5 + M6 + M9 — ready for external audit firm engagement.**

| Layer | Status | Tests |
|-------|--------|-------|
| M1 foundational primitives | ready | 100+ unit + 51 NIST KAT |
| M2 state foundation | ready | 95+ unit + 60 invariants |
| M3 apply_proposal | ready | 89 unit + 29 invariants |
| M4 consensus mechanics | ready | 187 unit + 85 invariants |
| M5 persistence | ready | 27 unit + 17 invariants |
| M6 network — Noise_PQ XX in production | ready | 35 tests in mt-noise-pq + mt-net-transport (release), 3 e2e two-node + proposal exchange + libp2p upgrade |
| M9 conformance | ready | 2 byte-exact verify |
| M7 fast sync | TODO | — |
| M8 node binary | in progress | partial; DEV-012 multi-node proposal apply remains open |

## License

Dual-licensed under Apache-2.0 OR MIT, at your choice.

- [`LICENSE`](LICENSE) — Apache-2.0 (root, applies to spec + Agents/ + supporting files)
- [`Code/LICENSE-APACHE`](Code/LICENSE-APACHE) — Apache-2.0 (Rust workspace)
- [`Code/LICENSE-MIT`](Code/LICENSE-MIT) — MIT (Rust workspace, choose either)

## Contact

- **Issues / Findings:** [github.com/efir369999/Montana/issues](https://github.com/efir369999/Montana/issues)
- **Pull Requests:** direct PRs welcome
- **Whitepaper (Satoshi style):** [`Whitepaper Montana.md`](Whitepaper%20Montana.md) — academic paper in the style of the Bitcoin paper, for posting to the [metzdowd cryptography list](metzdowd-email.txt)
- **No email / Discord / Telegram** — public on-record review only

---

*Pre-mainnet. Break it, fix it, send PRs. Time is elegant money. Sovereign Invisibility Online.*
