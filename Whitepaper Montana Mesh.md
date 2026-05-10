# Montana: A Sovereign Mesh of Cities

**Alejandro Montana**
[github.com/efir369999/Montana](https://github.com/efir369999/Montana) · [montana.quest](https://montana.quest)

---

## Abstract

A network in which value moves between parties without trusted intermediaries and without dependence on classical cryptography must simultaneously solve three problems: global consensus on event ordering, transport-level protection against passive observers and active censors, and infrastructural resilience against the failure or capture of individual nodes. Existing systems solve one of the three. Bitcoin gives consensus but not transport privacy. Tor gives transport privacy but not consensus. Tailscale and WireGuard give peer-to-peer connectivity but not infrastructure sovereignty. Montana attempts to combine all three in a single system. It consists of two coupled layers: **TimeChain**, a post-quantum blockchain whose scarcity is time (not block space and not fees), and **Mesh VPN**, a federation of city-nodes, each of which exposes its region of the internet and underwrites its neighbors when they fail. The invariant metaphor: each node is a city on a map; the network of VPN-cities is the internet.

---

## 1. Introduction

Bitcoin [1] demonstrated that decentralized monetary consensus is achievable without intermediaries. Tor [11] demonstrated that anonymous traffic routing is achievable on a public network. WireGuard [12] and its descendants showed that simple, fast peer-to-peer VPN is possible on top of modern cryptography. None of these systems combine the three properties Montana targets — a sovereign internet at the scale of ≥10⁹ users: global ordering, transport privacy, and self-sovereign infrastructure.

Bitcoin and its descendants further leave two unresolved vulnerabilities. First, all production blockchains derive signature security from elliptic-curve discrete-logarithm assumptions. Shor's algorithm [8], on a sufficiently large quantum computer, breaks these assumptions in polynomial time. NIST standardized post-quantum signatures and key-encapsulation mechanisms in 2024 (FIPS 203 [2], 204 [3], 205); major chains have not migrated. Second, fee-based anti-spam scales poorly under adoption: under congestion small operations are priced out, under abundance spam returns at marginal cost. Layer-2 systems (state channels, rollups) shift the economics rather than remove the underlying scarcity.

Montana proposes a chain whose signature security rests entirely on post-quantum primitives, and whose anti-spam mechanism operates on time rather than money [13]. The chain advances by a verifiable delay function (VDF) [5,6,7] over SHA-256, producing globally ordered windows of approximately 60 seconds. Operations within a window are rate-limited by three independent time-derived scarcities: per-identity, account chain length, and seniority. On top of this layer, the protocol deploys a second layer — mesh VPN — using Reality (xray) [14] to mask traffic as a regular TLS handshake to a legitimate public destination.

---

## 2. Architecture: Two Layers, One Network

Montana consists of two layers physically realized by the same set of nodes:

**Layer 1 — TimeChain.** Global clock + identity registry + state. Responsible for: event ordering, node-operator registration, emission accounting, state preservation. Detailed exposition is in [Whitepaper Montana](Whitepaper%20Montana.md); section 4 here is a summary.

**Layer 2 — Mesh VPN.** Routing of user traffic through city-nodes. Responsible for: transport privacy, censorship circumvention, failover among nodes. Sections 5–7 elaborate.

The layers are not independent: an operator who wishes to participate fully must run both a TimeChain node (`montana-node`) and a VPN server (`xray`) with Reality. TimeChain provides self-sovereign identity and proof-of-uptime; VPN provides bandwidth for users. Without TimeChain a VPN node is an ordinary VPS; without VPN a TimeChain node is an observer with no useful payload. Only together do they make a sovereign network node.

---

## 3. The City Metaphor

A Montana node = a city on the map. As of 2026-05-10, the network consists of three cities:

- **Moscow** (55.7558° N, 37.6173° E) — TimeChain Active validator, window emitter;
- **Frankfurt** (50.1109° N, 8.6821° E) — TimeChain candidate, VPN origin;
- **Helsinki** (60.1699° N, 24.9384° E) — TimeChain candidate, VPN front for Frankfurt.

The metaphor is not decorative. It enforces three implementation invariants:

(a) **Each city opens its region.** A user choosing "Helsinki" reaches the public internet as Helsinki sees it: from Helsinki's ASN, with Helsinki's DNS resolution, with Helsinki's reachability to resources blocked elsewhere.

(b) **Cities underwrite each other.** When a node fails or is captured, the remaining cities accept its clients via the federation mechanism described in section 6.4. There is no central failover point.

(c) **The network of cities is the internet.** The end user does not distinguish between "using the internet" and "using Montana" — Montana becomes the internet for those who join. This is the limit goal; section 9 describes the partial state of realization at present.

---

## 4. Layer 1 — TimeChain (summary)

The full description is in `Montana Protocol v35.25.0` and in the TimeChain whitepaper [13]. Only the part relevant to Layer 2 is given here.

**Window.** Let `T_r` denote the VDF output at window `r`. The chain advances by `T_r = SHA-256^D (T_{r-1})`, where `T_0` is the genesis seed and `D` the per-window iteration count. At the current epoch, `D = 325 000 000`, calibrating the window to ≈60 seconds on commodity x86_64. `D` is recalibrated every `τ₂ = 20 160` windows (≈14 days) by a canonical formula.

**Emission.** Each window mints exactly `13 Ɉ = 13·10⁹ nɈ`. Supply is given by the closed form `supply(W) = 13·(W+1) Ɉ`. At the time of writing `W = 34 922`, supply ≈ 454 000 Ɉ.

**Node registration.** A new operator must build a candidate VDF chain of length `τ₂` windows (≈10 hours wall-clock on M-class Mac). This is the Sybil defense: N identities require N candidate chains. After completing the candidate VDF, the node submits `NodeRegistration` and at the next selection event (every 336 windows) is admitted into the `NodeTable` as Active.

**Current network state** (live snapshot 2026-05-10):

| City | Phase | window | NodeTable | balance |
|---|---|---|---|---|
| Moscow | Active | 34922 | 1 | 453 388 Ɉ |
| Frankfurt | CandidateVdf 42% | 34920 | 1 | 0 |
| Helsinki | CandidateVdf 4% | 34901 | 1 | 0 |
| Mac (candidate) | CandidateVdf 0.5% | 100 | 0 | 0 |

A single Active validator at present — Moscow. Frankfurt and Helsinki finish candidate-VDF and register within hours of wall-clock; after that NodeTable grows to three.

---

## 5. Layer 2 — Mesh VPN

### 5.1. Transport: xray Reality

Each VPN node runs [xray](https://github.com/XTLS/Xray-core) with a VLESS inbound over Reality [14]. Reality is a modification of TLS 1.3 in which the client initiates a handshake to a real public destination (e.g. `www.googletagmanager.com`) but receives the first response from the proxy server; to a DPI observer the entire handshake is indistinguishable from ordinary TLS to that public site. The `xtls-rprx-vision` flow further reduces signatures in the content stream.

Baseline single-inbound config on a node:

```json
{
  "port": 443,
  "protocol": "vless",
  "streamSettings": {
    "network": "tcp",
    "security": "reality",
    "realitySettings": {
      "dest": "www.googletagmanager.com:443",
      "serverNames": ["www.googletagmanager.com"],
      "shortIds": ["<8 hex bytes per node>"],
      "privateKey": "<X25519 private — per node>"
    }
  },
  "settings": {
    "clients": [{ "id": "<UUID>", "flow": "xtls-rprx-vision" }],
    "decryption": "none"
  }
}
```

Each node holds **its own** keypair and **its own** UUID client list. Inter-node coordination is at the federation level only (section 6.3), not at the keymaterial level.

### 5.2. Client

The end user installs a compatible client (Happ for iOS, the v2rayNG-derived `Монтана.apk` for Android, Hiddify or v2box for desktop) and subscribes to the single sub URL `https://montana.quest/vpn/sub`. The sub serves a base64-encoded concatenation of every node's `vless://` URL, refreshed every five minutes (section 6.3). The client switches automatically between nodes on failure.

### 5.3. Per-city sub

In addition to the federated pool, each active VPN-city has its own endpoint:

- `GET /vpn/city/fra` — vless URL of Frankfurt;
- `GET /vpn/city/fin` — vless URL of Helsinki;
- `GET /vpn/city/msk` — currently 404 (Moscow is in `node only` mode at present, see section 9).

The per-city sub is the entry point for interfaces in which the user selects a city explicitly (e.g. the city map on `montana.quest/net`).

---

## 6. Federation Among Cities

### 6.1. Principle: Locally True, Globally Aggregated

Each node knows the truth only about itself. The query "what is the VPN config for city X?" is answered by what **node X publishes**. There is no centralized database; the aggregator only collects truths from nodes.

### 6.2. Node Source of Truth: `my-vpn.json`

Each node publishes locally `/var/lib/montana-net/my-vpn.json`:

```json
{
  "node": "frankfurt",
  "primary": false,
  "vless": "vless://...@<host>:443?...#Montana%20FRA"
}
```

The file is accessible only via SSH from the trusted aggregator node (Moscow) presenting the dedicated public key `vpn_stats2`, restricted to `forced-command cat /var/lib/montana-net/my-vpn.json`. No public HTTP exposure.

### 6.3. Aggregator: `montana-sub.timer`

Moscow runs the systemd timer `montana-sub.timer` (every 5 minutes) invoking `/opt/montana-net/sub-aggregator.sh`. The script:

1. Pulls `my-vpn.json` from each known peer via SSH (forced-command).
2. Collects the `vless://` list, sorts it (primary first).
3. Concatenates with `\n`, base64-encodes.
4. Writes to `/var/www/montana_quest/vpn/sub`.

A companion collector `/opt/montana-net/aggregator.sh` collects `peers.json` from the same nodes and publishes `/var/www/montana_quest/vpn/network.json` — the federation health view.

### 6.4. Failover Graph

Current fronting topology:

- **Helsinki fronts Frankfurt.** The Helsinki vless URL in the federation pool points to `cdn.montana.quest:443`, which is proxied to Helsinki as the primary entry point. If Helsinki fails, clients reach Frankfurt directly via `89.19.208.158:443` (the secondary URL in the same sub). This is spelled out in `cities.json` via the `vpn.fronts` and `vpn.fronted_by` fields.
- **Moscow is not yet a VPN.** When VPN is brought up on Moscow (Roadmap, section 9), it joins the pool as a third entry point, peering with Frankfurt and Helsinki.

### 6.5. Health Probe

`peer-health.py` (Moscow, same timer) performs a TLS handshake to each VPN endpoint with the target SNI. The result is written to `/var/www/montana_quest/vpn/health.json`. The aggregator does not evict a node from sub on transient failure — the client switches itself. Health data is for explorer display (`montana.quest/net`).

---

## 7. Privacy by Default

### 7.1. At the TimeChain Layer

Account ID is `SHA-256(public_key)`. The chain itself does not require KYC metadata. Balances are public (as in Bitcoin), but nicknames and inter-account links are not exposed by default — the user chooses what to reveal. See the dedicated document "Privacy by default".

### 7.2. At the VPN Layer

Reality masks the handshake as ordinary TLS to a legitimate public destination. A DPI observer of the handshake cannot distinguish Montana-VPN from a visit to `www.googletagmanager.com`. SNI and certificate match the public destination. Payload encryption is TLS 1.3 (over Reality) plus VLESS encapsulation; the key is negotiated per session.

### 7.3. At the Explorer Layer

The public dashboard (`montana.quest/net`) **does not expose node IPs** in JSON or HTML. Node coordinates are at city granularity (Moscow, Frankfurt, Helsinki), not at the data-center level. Hosting-provider names are not published. This reduces the surface for targeted DDoS and social attacks.

---

## 8. Scale

The baseline target is supporting ≥10⁹ active users. Every architectural decision in Montana is checked against this baseline; mechanisms that do not scale are dropped without discussion. See the dedicated document "Scale baseline 1B+".

Layer estimates:

**TimeChain.** AccountTable grows with each new registration. At 10⁹ accounts and an average record size of ≈2 KB, the table is on the order of 2 TB. This does not fit in RAM but does fit on a single node's SSD, provided the node maintains only an active-set index. Emission of 13 Ɉ/window × 525 600 windows/year ≈ 6.83M Ɉ/year — acceptable inflation against the announced supply.

**VPN mesh.** At a typical load of 5 Mbps per user, a node with 10 Gbps uplink serves ≈2 000 concurrent active sessions. 10⁹ users at 1% concurrent activity assumption ≈ 10⁷ active streams, requiring ≈5 000 nodes. This is the federation target: a network of thousands of cities. The current three are the starting point.

---

## 9. Current State and Roadmap

### 9.1. As of 2026-05-10

- **TimeChain**: 3 nodes (Moscow Active, Frankfurt+Helsinki in candidate-VDF), 1 candidate (Mac). Genesis 2026-01-09. Window ≈ 35 000.
- **VPN**: 3 active points (Moscow :2053, Frankfurt :443, Helsinki :443). Helsinki fronts Frankfurt; Moscow is a standalone third origin. Federated `/vpn/sub` aggregates all three.
- **Explorer**: `montana.quest/net` — live dashboard for 4 nodes, mobile-adapted, no IP exposure.
- **City map**: backend `/net/cities.json` ready, `/vpn/city/{msk,fra,fin}` serves per-city URLs. All three cities are marked as VPN nodes. The visual map is the next step.

### 9.2. Near-term

- Frankfurt and Helsinki finish candidate-VDF and register as Active validators. AccountTable / supply divergence between nodes collapses to zero.
- Visual city map on `montana.quest/net` — a separate frontend iteration.

### 9.3. Mid-term

- Federation expansion: new city-nodes on demand. Onboarding — bring up `montana-node` + xray Reality + publish `my-vpn.json`. The aggregator picks them up automatically.
- Mobile distribution: `Монтана.apk` is already built (rebrand of v2rayNG, keystore = genesis secret). The iOS analog is the Happ deeplink via `/vpn/sub`.

### 9.4. Long-term

- Tens to hundreds of cities. The federated sub-pool shards by region. Health probe becomes part of consensus (a node not responding for >7 days is ejected from `NodeTable` via a dedicated operation).
- Each city — its own ML-DSA-65 identity, its own operator account, its own VPN keypair.

---

## 10. References

[1] Nakamoto S. *Bitcoin: A Peer-to-Peer Electronic Cash System*. 2008.
[2] NIST FIPS 203. *Module-Lattice-Based Key-Encapsulation Mechanism Standard*. 2024.
[3] NIST FIPS 204. *Module-Lattice-Based Digital Signature Standard*. 2024.
[4] NIST FIPS 180-4. *Secure Hash Standard*. 2015.
[5] Boneh D., Bonneau J., Bünz B., Fisch B. *Verifiable Delay Functions*. CRYPTO 2018.
[6] Wesolowski B. *Efficient Verifiable Delay Functions*. EUROCRYPT 2019.
[7] Pietrzak K. *Simple Verifiable Delay Functions*. ITCS 2019.
[8] Shor P. *Polynomial-Time Algorithms for Prime Factorization and Discrete Logarithms on a Quantum Computer*. SIAM J. Comput., 1997.
[9] Grover L. *A Fast Quantum Mechanical Algorithm for Database Search*. STOC 1996.
[10] *Montana Protocol v35.25.0*. Montana spec, 2026.
[11] Dingledine R., Mathewson N., Syverson P. *Tor: The Second-Generation Onion Router*. USENIX Security 2004.
[12] Donenfeld J. *WireGuard: Next Generation Kernel Network Tunnel*. NDSS 2017.
[13] *Whitepaper Montana* (TimeChain layer) — `Montana/Montana-Protocol/Whitepaper Montana.md`.
[14] *XTLS Reality* — `github.com/XTLS/Xray-core/discussions/1295`.

---

*This document is published in three languages: Russian (`Whitepaper Montana Mesh RU.md`), English (this file), and Chinese (`Whitepaper Montana Mesh ZH.md`). All three are content-identical; in case of discrepancy, the canonical version is the Russian one.*
