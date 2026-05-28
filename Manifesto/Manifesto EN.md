# The Montana Manifesto

**Version:** 1.3.0
**Date:** 2026-05-29
**Author:** Alejandro Montana
**Repository:** [github.com/efir369999/Montana](https://github.com/efir369999/Montana)

> *"Who controls the past controls the future. Who controls the present controls the past."*
> — Orwell, *1984*

**The neutral rail Bitcoin's title conflated with the currency. The substrate Ethereum's name claimed and did not build. The economics of time the digital-money tradition has not yet written.**

## I. Two Jobs Conflated, and Two Names That Did Not Deliver

Bitcoin's title was *A Peer-to-Peer Electronic Cash System*. The phrase put two distinct jobs under one name.

- **Being a stable unit of account.** A currency that does not lose purchasing power as a function of speculative demand. This needs an accountable issuer with a reserve and a buy-back capability — the kind of accountability a government, a central bank or a major institution provides with a balance sheet.
- **Being a neutral settlement and ordering layer.** A rail that clears payments, orders events and provides the substrate on which units of account move. It cannot be the accountable issuer, and it should not pretend to be.

Bitcoin tried to do both at once. Its anti-spam mechanism, denominated in its own asset, tied the rail's reliability to the price of that asset; the asset's volatility put unit-of-account stability out of reach; the two jobs interfered. The title promised electronic cash; what Bitcoin became was *digital gold* — neither a stable usable currency nor a fee-free rail.

Ethereum took the second job under a different name. *Ether* in its title is the medium that fills space — the substrate on which everything moves. The proposition was correct: what the world needed was a settlement layer, not another currency. But the implementation charges fees denominated in its own asset, runs on classical elliptic-curve signatures that Shor's algorithm breaks, and exposes an extractive ordering layer (MEV) that turns neutrality into a marketing claim rather than a property. The name promised the ether. The implementation did not deliver it.

Two famous names, two missed roles. Bitcoin missed *cash*; Ethereum missed *ether*. Both missed for the same underlying reason: the rail's reliability was coupled to the price of its native asset, the rail's neutrality was contingent on whoever produced the next block, and the rail's primitive set was the one Shor breaks. The two jobs could not be done at once, and the rail half was not done at all.

Montana picks **one** of the two jobs on purpose: the neutral, fee-free, post-quantum settlement and ordering rail. The stable usable currency that consumers see in daily life lives one floor up — issued by accountable parties with their own reserves and buy-back logic, denominated in whatever units those issuers choose. Montana provides the substrate on which any such currency can run.

The rail does not need to know the unit of account it carries. The currency does not have to be the rail.

What is needed for a real neutral rail — and what neither Bitcoin's rail layer nor Ethereum's implementation delivered — is the following:

- A non-monetary anti-abuse scarcity, so the rail's reliability is not coupled to the price of its native asset.
- Asynchronous finality fast enough that the rail behaves like a settlement layer, not a queue of blocks, and without an extractable position over the order of operations.
- A post-quantum primitive set, because long-lived rails carrying value across decades cannot rest on assumptions Shor's algorithm breaks.

Montana takes all three.

Montana addresses, at the same level, three places where trust must be removed from the rail itself:

- **Trust in time.** The protocol produces a canonical order of events with no external source.
- **Trust in storage.** A user's data lives on the user's node, not in a corporation's database.
- **Trust in communication.** Messages flow between users through their nodes, with no intermediary.

The solution to the first problem is the foundation of the other two — and the carrier of the time-economics that makes the rail fee-free.

## II. Canonical Order, Not Wall-Clock Time

Each Montana node performs a **sequential delay computation** — an iterated SHA-256 hash chain `T_W = SHA-256^D (T_{W-1})` with `D = 325 000 000` iterations per window. `D` is fixed in the Genesis Decree from a single quartz measurement on the genesis hardware (Apple iMac M1 2021, idle, single-thread); after Genesis the protocol consults no clock ([I-18]). The wall-clock duration of a window is an emergent property of each node's hardware and is not part of consensus state.

This is **not** a verifiable delay function in the sense of Boneh-Bonneau-Bünz-Fisch [CRYPTO 2018], Pietrzak [ITCS 2019] or Wesolowski [EUROCRYPT 2019]. Those constructions provide succinct verification of order `O(log T)` or `O(1)`, but they operate over RSA groups or class groups of imaginary quadratic fields — assumptions broken by Shor's algorithm. A production-grade post-quantum succinct VDF does not yet exist. Montana takes the simpler primitive: an iterated SHA-256 chain. Verification cost equals computation cost; a verifier re-runs the same iterations the prover ran. SHA-256 is already required for addressing, hashing and Merkle commitments — no new assumption is added. The cryptographic surface is minimized to one primitive ([I-7]).

The output is the **TimeChain**: a canonical, monotonic, unambiguous, independently verifiable sequence of windows. Montana does not measure physical duration. Mapping a window number to a calendar is the observer's task, not the protocol's.

## III. The Hierarchy of Truth

Every layer is impossible without the one below.

1. **Canonical order** (`TimeChain`) — irreversible sequential computation. The base layer.
2. **Presence** (`NodeChain`) — a node's chain length, accumulated one window at a time as the node is canonically cemented into the order. Weight in consensus is presence, not capital. Capital cannot retroactively purchase past participation.
3. **The rail's reward unit** (`Account`, `Ɉ`) — the protocol-level emission paid to the operator that seals a window: `EMISSION_moneta = 13 × 10⁹ moneta = 13 Ɉ`. Supply is closed-form: `supply_moneta(W) = EMISSION_moneta × (W + 1)`. No premine, no presale, no founder allocation, no halving, no supply cap, no discretionary issuance. `Ɉ` is the rail's bookkeeping; it is not a promise of stable purchasing power and not the unit of account a stable currency would use.
4. **History** (`Anchor`) — a 32-byte hash bound to a window for the lifetime of the network. Rewriting it requires recomputing every iteration of the chain from the Genesis Decree. Mathematically impossible.

Dormant accounts with a non-zero balance are never touched, no matter how long they sit. The only state cleanup the protocol performs is the removal of empty `AccountRecord` entries (`balance == 0`) idle for more than `4 × τ₂` — garbage collection of records that hold nothing. A balance, once credited, belongs to its key forever.

`1 Ɉ = 10⁹ moneta`. The international ticker is `MONT`.

## IV. Post-Quantum from the First Day

- **Consensus signatures:** ML-DSA-65 (FIPS 204).
- **Transport key encapsulation:** ML-KEM-768 (FIPS 203).
- **Hashing:** SHA-256 (FIPS 180-4).
- **Transport handshake:** Noise_PQ XX — ephemeral ML-KEM-768 on both sides, an ML-DSA-65 signature binding the full transcript, ChaCha20-Poly1305 AEAD framing (RFC 8439) on the established session.
- **PeerId:** the SHA-256 multihash of each peer's ML-DSA-65 identity public key.

No ECDSA. No EdDSA. No classical Diffie-Hellman. No assumption that Shor's algorithm will be late.

## V. Properties of the Rail

What makes Montana a neutral settlement and ordering rail are not features layered on a chain — they are the chain.

- **Zero fees.** The protocol contains no `fee` field on any operation. A seven-cent transfer settles. A high-volume settlement application is not priced out by a congestion auction.
- **Asynchronous finality without extractable ordering.** Transfers do not wait for blocks. They are cemented through a P2P quorum of signatures from active operators within a single window of the canonical order (approximately one minute of wall-clock at the genesis-hardware calibration; the wall-clock duration is emergent, not part of consensus state). The proposer's discretion over which operations to include or omit is zero — only cemented BundledConfirmations enter the chain, and operation ordering inside a window is fixed by τ₁-rate canonical rules, not by the proposer's local mempool view. There is no extractable position over the order of operations.
- **Constant monotonic emission as bookkeeping.** `13 Ɉ` per window, fixed by the Genesis Decree, closed-form. No halving, no supply cap, no discretionary issuance. `Ɉ` is what the rail pays its operators — bookkeeping for the work of sealing a window, not a stable unit of account. Currencies that need to be stable live one floor up, where parties with balance sheets can issue them.
- **No plutocracy by construction.** Whoever holds a billion `Ɉ` has no more power in consensus than the operator of a Mac Mini. A node's weight is its chain length — its history of cemented presence. The lottery seed incorporates `cemented_bundle_aggregate(W-2)`, signatures from honest operators two windows back, which closes the grinding attack class under hardware asymmetry without depending on rational-cost arguments.
- **Two-thirds honest chain length.** Safety holds while honest operators control more than two-thirds of `active_chain_length`. Capital does not enter the threshold.

## VI. The Economics of Time

Anti-abuse is done by time, not by money — three independent scarcities, each derived from time elapsed. This is what decouples the rail's reliability from the price of its asset.

- **Per-identity rate per window.** One operation per account per window τ₁. An attacker with N Sybil identities gets at most N operations per window, but each identity has its own creation cost.
- **`account_chain_length` thresholds.** Privileged operations require the operating account to have been active for at least `k` windows. The threshold cannot be purchased.
- **Sequential entry barrier for node operators.** Node registration requires producing a sequential SHA-256 chain of length `vdf_chain_length × D` iterations — approximately fourteen days of wall-clock on a commodity x86_64 core. Sequential time is non-acquirable; an attacker with `M` parallel machines produces `M` identities at the same wall-clock cost, not faster.

Together these three close DoS without monetary barriers. Time as scarcity does not require a price feed, an oracle or an exchange to measure. Its valuation is fixed by the protocol: one window is one window, regardless of `Ɉ` price, regardless of the prices of any currency riding on top.

## VII. The Ladder of Sovereignty

Two roles, one chain.

- **Account user.** A key in a smartphone or hardware wallet. Sends and receives Montana; commits 32-byte hashes via `Anchor`; runs applications on top of someone else's node. No protocol-layer earnings. Barrier: a first incoming Transfer (the AccountRecord is created atomically together with crediting the amount).
- **Node operator.** Commodity hardware (one CPU core), 24/7 uptime, a network connection, and the sequential SHA-256 entry barrier at registration. Full participation in consensus. Earnings through the per-window node lottery.

The seed phrase and the account chain belong to the user, not to the node. The user moves up the ladder when they choose to.

## VIII. The Scale Baseline, Privacy, and Removal of the Author

- **Scale.** Every decision is calibrated for at least one billion active users. Mechanisms that do not scale to 10⁹ are discarded without discussion. AccountRecord is 2 059 bytes; state at 10⁹ accounts is approximately 2.06 TB, holdable on commodity disks. Pruning is canonical: state size is bounded by active population, not by chain age.
- **Privacy.** Balances and account graphs are public by default ([I-2]). Application-level privacy is achieved through `Anchor`: a 32-byte hash is committed to the chain; encrypted content is held off-chain by its owner. The protocol has no visibility into the contents. Privacy is what the user chooses to keep — not what the protocol imposes nor what the protocol forbids.
- **No governance in state.** No DAO, no treasury, no founder veto. Advisory councils may exist outside the protocol; none of them have binding force inside it. The author is removed from the protocol by construction. Montana launches as a peer-to-peer network with no founder-controlled bootstrap quorum.

## IX. What Montana Is

Not the currency Bitcoin's title promised. Not the substrate Ethereum's name claimed and did not build. Not digital gold. Not yield. Not governance. Not a brand. Not a privacy mixer. Not an L2. Not a blockchain with a timestamping feature.

Montana is **the neutral, fee-free, post-quantum settlement and ordering rail on which a usable currency can run** — not the currency itself.

The economics of time is what makes that rail possible: a non-monetary scarcity that decouples the rail's reliability from the price of its asset, so the rail does not have to do the currency's job in order to function.

A time frame of reference with a value-transfer feature. The standard of frequency from which the parties capable of doing the currency job — governments, central banks, accountable institutions, autonomous agents with reserve logic — can build the currencies people actually use. The medium that fills the space between participants, with no charge for being present and no extractable position over the order of operations.

The ether the name promised.

---

**Reference implementation:** Rust, Apache-2.0 / MIT. Twenty-three crates including `mt-timechain`, `mt-consensus`, `mt-lottery`, `mt-crypto`, `mt-net`, `mt-noise-pq`. Specification: [Whitepaper Montana.md](../Whitepaper%20Montana.md) and [Montana Protocol v35.25.1](../Montana%20Protocol%20v35.25.1.md).

**Symbol:** **Ɉ** — Montana, the rail's protocol-level reward unit. `moneta` — the smallest indivisible unit (`1 Ɉ = 10⁹ moneta`). **Ticker:** `MONT`.

Alejandro Montana
*Ничто_Nothing_无_金元Ɉ*
2026-05-29
