# Montana — Sovereign Invisibility Online

> Post-quantum reference blockchain. Sequential-delay TimeChain consensus over SHA-256. Time-as-scarcity instead of fees.
> Production transport is **Noise_PQ XX** (ML-KEM-768 + ML-DSA-65 + ChaCha20-Poly1305).
> Mainnet **v0.2** spec package. Rust reference implementation `1.0.0`. Dual-licensed Apache-2.0 / MIT.
> **First mainnet release:** [v1.0.0](https://github.com/efir369999/Montana/releases/tag/v1.0.0) (2026-05-22).

---

## What Montana is

Montana is a post-quantum sovereignty stack. Every primitive in the protocol layer is post-quantum:

| Layer | Primitive | Standard |
| Consensus signatures | ML-DSA-65 | NIST FIPS 204 |
| Application key encapsulation | ML-KEM-768 | NIST FIPS 203 |
| Transport handshake | Noise_PQ XX (ML-KEM-768 + ML-DSA-65) | This project |
| Wire AEAD | ChaCha20-Poly1305 | IETF RFC 8439 |
| Sequential delay function | Iterated SHA-256 | NIST FIPS 180-4 |
| Identity-to-PeerId binding | SHA-256 multihash of ML-DSA-65 pk | libp2p / IPFS code `0x12` |

No classical Diffie-Hellman is present in the protocol layer. A passive observer recording today's traffic cannot derive the session keys without solving Module-LWE on ML-KEM-768; an active man-in-the-middle cannot substitute identities without an EUF-CMA forgery on ML-DSA-65.

---

## Whitepaper

The academic paper, written in the style of the Bitcoin paper and addressed to the Metzdowd Cryptography List audience, is the canonical entry point:

📄 **[`Montana Whitepaper v0.1.0.md`](Montana%20Whitepaper%20v0.1.0.md)**

The whitepaper covers, in present-tense factual form:

- Time as a scarce resource (§2)
- Sequential delay function over SHA-256 with explicit acknowledgment that this is not a VDF in the sense of Boneh / Pietrzak / Wesolowski (§3)
- Post-quantum primitives with NIST FIPS citations (§4)
- Threat model with eight explicit attack-class subsections (§5): Quantum adversary, Sybil, Equivocation, Grinding under hardware asymmetry, Eclipse, Long-range rewrite, Side-channel, Transport-layer adversary
- Operations and the Account Table (§6)
- Lottery with network-bound unpredictability of consensus seeds (§7)
- Liveness and the fallback cascade (§8)
- Incentive and bootstrap economics (§9)
- Anti-spam without fees through three independent time scarcities (§10)
- State lifecycle and scaling toward one billion accounts (§11)
- Privacy scope: Anchor commit-reveal, regulatory alignment (§12)
- Noise_PQ XX network and transport security with the full wire format and security analysis (§13)
- Adversarial calculations (§14)

---

## Three-layer specification

The protocol is specified as three layered documents — each independently auditable:

| Layer | Spec | Scope |
| 1. Protocol | [`Montana Protocol v35.25.1.md`](Montana%20Protocol%20v35.25.1.md) | State machine, crypto primitives (ML-DSA-65, ML-KEM-768, SHA-256), sequential-delay TimeChain, lottery, Account / Node tables, Genesis Decree, `apply_proposal` pipeline, consensus operations |
| 2. Network | [`Montana Network v1.3.0.md`](Montana%20Network%20v1.3.0.md) | libp2p transport, Noise_PQ XX (production), Identity-Bound Tunnel, transport randomness, PeerRecord, mesh transport, sync protocols, network-layer threat model, KAT vectors |
| 3. App | [`Montana App v3.12.0.md`](Montana%20App%20v3.12.0.md) | UI, wallet, messenger (Double Ratchet PQ), channels, contacts, profile, Junona AI agent, browser, premium, application-layer economy |
| 4. Egress | [`Montana Egress v1.0.0.md`](Montana%20Egress%20v1.0.0.md) | clearnet egress over the mesh: entry/relay/exit roles, egress directory, manual/auto country selection, two-session architecture, exit policy, threat model |
| 5. Alliance | [`Montana VPN Alliance v1.1.0.md`](Montana%20VPN%20Alliance%20v1.1.0.md) | federation pattern: universal-key membership, mutual reachability insurance, front-light/exit-heavy load model, resilience |

Layer dependency direction: Protocol (low) ← Network (mid) ← App (high). Each layer depends on layers below it; no upward dependency.

---

## Live network

The reference implementation runs a live production mesh with full pairwise Noise_PQ XX sessions (`/montana/noise-pq-xx/1.0.0`). Node addresses, identities, and locations are not published; the network is reached through the censorship-resistant discovery channels defined in the Network specification, not a static list.



---

## First external review — closed

The first external security audit was the consolidated CISO-as-a-Service Team review of 2026-05-19, sixteen findings (six critical, three high, four medium, three informational). The project's formal disposition is committed to the repository:

📋 **[`External-Audit/montana-response-to-2026-05-19-audit.md`](External-Audit/montana-response-to-2026-05-19-audit.md)**

Outcome:

- **Twelve findings accepted and fixed by construction.** Whitepaper findings WP-1..WP-12 are closed in the rewritten Whitepaper; spec patches MONT-001 (ML-DSA-65 constant-time requirement) and MONT-002 (IBT replay window) are committed.
- **Two findings rejected with spec citations** — MONT-003 was a misread of the equivocation rule in the Protocol specification; the WP-8 sub-claim of "does not explain `cemented_bundle_aggregate`" misses Section 7.
- **One finding documented as pre-mainnet operating state** — MONT-004 singleton phase, tracked in [`Code/docs/SPEC_DEVIATIONS.md`](Code/docs/SPEC_DEVIATIONS.md).
- **One finding (DEV-014, post-quantum transport migration) closed on 2026-05-21** by switching the production transport stack to Noise_PQ XX; the live 6/6 pairwise mesh runs through the post-quantum AEAD stream.

GitHub issue thread: [#1 Security Review](https://github.com/efir369999/Montana/issues/1).

A second-pass review focused on mainnet readiness is welcomed as a new GitHub issue tagged `mainnet-readiness` — see the closing comment on issue #1 for the four areas where independent eyes add the most value (bootstrap-equilibrium analysis, M7 fast-sync threat surface, M9 cross-implementation conformance, residual concerns on the Noise_PQ XX wire format).

---

## Open call to AI agents and security researchers

This is a public invitation. Every primitive, every consensus rule, every byte of state was designed ground-up against quantum adversaries and audit-grade rigor. We publish the mainnet reference implementation against the v0.2 spec package so that the protocol can be broken, fixed, and hardened by the people who actually run nodes on the live mesh.

**You are explicitly invited to:**

- **Deploy a node** on any Linux VPS — one command, approximately five minutes, approximately five gibibytes of disk, one gibibyte of RAM. See [`Code/AGENTS.md`](Code/AGENTS.md) → *Deploy*.
- **Run stress / chaos / fuzz suites** against your node. See [`Code/AGENTS.md`](Code/AGENTS.md) → *Stress test*.
- **Audit the code against the spec.** [`Code/docs/SPEC_DEVIATIONS.md`](Code/docs/SPEC_DEVIATIONS.md) lists deviations, acknowledgments, and closures. The spec is the single source of truth: [`Montana Protocol v35.25.1.md`](Montana%20Protocol%20v35.25.1.md) + [`Montana Network v1.3.0.md`](Montana%20Network%20v1.3.0.md) + [`Montana App v3.12.0.md`](Montana%20App%20v3.12.0.md).
- **Send findings** as GitHub Issues or Pull Requests. No NDA, no engagement contract. The protocol gets stronger or it does not ship.

**What this is NOT:**

- Not a token launch. No premine, no presale, no ICO. Mainnet is live with no fixed token-economy schedule.
- Not Web3 marketing. Read the spec — math first, branding second.
- Not yet-another-EVM-fork. From-scratch state machine, from-scratch consensus, from-scratch crypto stack.

---

## Quick start

**One command — joins the Montana fleet (node + VPN endpoint + orchestrator registration) on a clean Ubuntu/Debian VPS:**

```bash
curl -sSL https://raw.githubusercontent.com/efir369999/Montana/main/install.sh | sudo bash
```

The installer:
- Wipes any prior native systemd install (idempotent re-runs supported)
- Installs Docker
- Brings up `montana-node` container (`ghcr.io/efir369999/montana-node:latest`) — joins the 5-node TimeChain mesh via embedded `genesis-manifest.json`, listens P2P on `:8444`
- Brings up `xray` Reality endpoint on `:443` using the **shared universal `pbk` / `sid`** that all Montana VPN endpoints serve, with a per-host UUID
- Brings up `nginx-decoy` on `:80` (camouflage landing page)
- Auto-detects country / city / coords via `ip-api.com`
- POSTs `/register` to the Moscow orchestrator (built-in admin token); orchestrator either registers the node directly, or auto-provisions a cascade front via Frankfurt if the IP is in a blocked CIDR
- Prints the 24-word recovery mnemonic

The new endpoint appears in **https://montana.quest/vpn/sub** within ~5 minutes (systemd timer on Moscow rebuilds the subscription from the registry).

---

**Montana node only (no VPN), pre-built image — for operators who only want consensus participation:**

```bash
docker volume create montana-data
docker run -d \
  --name montana-node \
  --network host \
  --restart unless-stopped \
  -v montana-data:/var/lib/montana \
  ghcr.io/efir369999/montana-node:latest
```

The container dials the 5-node bootstrap mesh, generates a 24-word mnemonic on first launch, and writes it to `/var/lib/montana/mnemonic.txt`. Retrieve it once and save it offline:

```bash
docker exec montana-node cat /var/lib/montana/mnemonic.txt
```

See **[`Code/docker/runtime/QUICKSTART.md`](Code/docker/runtime/QUICKSTART.md)** for verification, build-from-source fallback, and optional VPN exit-node bring-up.

**Full VPS install (Montana node + Xray Reality VPN, one command):**

```bash
git clone https://github.com/efir369999/Montana.git /opt/montana && \
sudo bash /opt/montana/Code/scripts/install-vps-full.sh
```

**Node only (build from source on the VPS):**

```bash
sudo bash /opt/montana/Code/scripts/install-vps.sh
```

**VPN endpoint only:**

```bash
sudo bash /opt/montana/Code/montana-vpn/install.sh
```

The full installer prints a 24-word recovery mnemonic for the node and a VLESS URL for the VPN. Save the mnemonic immediately — it is the only backup.

---

## Status by milestone

| Milestone | State | Tests |
| M1 foundational primitives (mt-codec, mt-crypto, mt-crypto-native, mt-mnemonic) | ready | 100+ unit + 51 NIST KAT |
| M2 state foundation (mt-merkle, mt-genesis, mt-state, mt-timechain) | ready | 95+ unit + 60 invariants |
| M3 apply_proposal (mt-account) | ready | 89 unit + 29 invariants |
| M4 consensus mechanics (mt-lottery, mt-consensus, mt-entry) | ready | 187 unit + 85 invariants |
| M5 persistence (mt-store) | ready | 27 unit + 17 invariants |
| **M6 network — Noise_PQ XX in production** | **ready** | 35 release tests in mt-noise-pq + mt-net-transport, including XX handshake roundtrip, tamper detection on both signatures, end-to-end libp2p upgrade, two-node and proposal-exchange e2e |
| M9 conformance (mt-conformance) | ready | 2 byte-exact verify |
| M7 fast sync | TODO | — |
| M8 node binary | in progress | DEV-012 multi-node proposal apply pending |

---

## Repository layout

| Path | Contents |
| [`Montana Whitepaper v0.1.0.md`](Montana%20Whitepaper%20v0.1.0.md) | Academic paper in the style of the Bitcoin paper. Metzdowd-list submission text |
| [`Montana Protocol v35.25.1.md`](Montana%20Protocol%20v35.25.1.md) | Full protocol specification |
| [`Montana Network v1.3.0.md`](Montana%20Network%20v1.3.0.md) | Network-layer specification (Noise_PQ XX, IBT, mesh, sync) |
| [`Montana App v3.12.0.md`](Montana%20App%20v3.12.0.md) | Client application specification |
| [`External-Audit/`](External-Audit/) | First external security review and the project's disposition |
| [`Code/`](Code/) | Rust workspace — 17 crates, 9 milestones |
| [`Code/AGENTS.md`](Code/AGENTS.md) | Entry point for AI agents — deploy, stress-test, report findings |
| [`Code/AUDIT.md`](Code/AUDIT.md) | Audit package for external firm engagement |
| [`Code/ROADMAP.md`](Code/ROADMAP.md) | Nine milestones — current status and remaining work |
| [`Code/docs/SPEC_DEVIATIONS.md`](Code/docs/SPEC_DEVIATIONS.md) | Known deviations, acknowledgments, and closures |
| [`Code/montana-vpn/`](Code/montana-vpn/) | Reality-VPN endpoint (optional, alongside the node) |
| [`Code/scripts/install-vps-full.sh`](Code/scripts/install-vps-full.sh) | Node + VPN one-command installer |
| [`SECURITY.md`](SECURITY.md) | Security policy — how to report vulnerabilities |

---

## License

Dual-licensed under Apache-2.0 OR MIT, at your choice.

- [`LICENSE`](LICENSE) — Apache-2.0 (root, applies to specs + supporting files)
- [`Code/LICENSE-APACHE`](Code/LICENSE-APACHE) — Apache-2.0 (Rust workspace)
- [`Code/LICENSE-MIT`](Code/LICENSE-MIT) — MIT (Rust workspace, choose either)

---

## Contact

- **Issues and findings** — [github.com/efir369999/Montana/issues](https://github.com/efir369999/Montana/issues). The `mainnet-readiness` tag is reserved for the next-round review.
- **Pull requests** — direct PRs welcome.
- **No email, no Discord, no Telegram** — public on-record review only. Continuity of the security thread is more valuable than channel multiplexing.

---

*Mainnet is live. Break it, fix it, send PRs. Time is elegant money. Sovereign Invisibility Online.*
