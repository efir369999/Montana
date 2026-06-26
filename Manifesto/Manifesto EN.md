# The Montana Manifesto

**Version:** 2.0.4
**Date:** 2026-06-25
**Author:** Alejandro Montana
**Repository:** [github.com/efir369999/Montana](https://github.com/efir369999/Montana)

> *"Who controls the past controls the future. Who controls the present controls the past."*
> — Orwell, *1984*

**An architecture that closes the failures of both fiat and cryptocurrency on a single foundation.**

## I. Two Worlds That Failed

Money is supposed to be three things at once: a way to save, a way to pay, and a way to settle accounts between strangers without trusting either side. For most people on Earth, money is none of these three. Two worlds have tried to provide it. Both have failed.

**The fiat world.** Savings are erased by inflation faster than they can be earned. Accounts are frozen by banks, censored by governments, surveilled by payment processors. A migrant sending wages home pays thirty percent to a remittance corridor. A transfer of seven cents is uneconomical because the fee consumes it. A transfer across borders takes days and costs more than the contents of an average wallet. Wages are paid in a currency people did not choose, on a rail they cannot leave, and the rail extracts at every step. Generations are priced out of housing because monetary expansion went into asset prices, not into wages. Sanctions hit ordinary people first and hardest. Central-bank digital currencies arrive as programmable surveillance with smarter-than-cash backends. The 20th-century financial system was built for an industrial economy with a small number of large institutions; it was never re-designed for the billions of people, the trillions of small payments, the autonomous software agents, and the speed at which value moves now.

**The crypto world.** Bitcoin's whitepaper was titled *A Peer-to-Peer Electronic Cash System*. The cryptographic answer is famous. But Bitcoin became digital gold, not cash: a seven-cent payment is uneconomical because the fee consumes it, settlement is ten minutes at best and unbounded under congestion, the merchant cannot let the customer leave, the anti-spam mechanism is denominated in the very money the system creates and oscillates with demand. Ethereum took the substrate role under a name that literally means *ether* — the medium that fills space, the substrate on which everything moves — but the implementation charges fees denominated in its own asset, runs on classical elliptic-curve signatures that Shor's algorithm breaks, and exposes an extractive ordering layer (MEV) that turns neutrality into a marketing claim. Stablecoins moved billions in volume by tying their value to the very banking system they were supposed to escape. DeFi yields were built on Ponzi-like recycling of one chain's emission through another. Exchanges keep failing on top of the chains they were built to obviate.

Bitcoin missed *cash*. Ethereum missed *ether*. The rest never tried to solve the problem for the people who needed it solved. Every existing system fails on the same underlying ground: the rail's reliability was coupled to the price of its native asset, the rail's neutrality was contingent on whoever produced the next block, and the rail's primitive set was the one Shor's algorithm breaks.

Two worlds, the same people, the same failure: the institutions that issue money will not solve this, the institutions that move it will not solve this, the crypto industry that promised to solve this turned into a casino.

Montana proposes a foundation that addresses both failures at their shared cause.

## II. The Insight: Time, Not Money

The scarcity that money is supposed to defend against — abuse, spam, inflation, extraction — must come from somewhere outside money itself. Bitcoin denominated it in money and the mechanism oscillated. Ethereum denominated it in money and got priced out under congestion. Every fee-based chain has the same problem: when the rail's reliability is denominated in the rail's own currency, the currency's volatility is the rail's failure mode.

The only scarcity available to all participants on equal terms, that no party can buy or sell, is **time**.

An attacker with one hundred times the resources of an honest operator does not get one hundred times more time per chain. A Sybil identity does not produce more time per identity. Capital cannot retroactively purchase past participation. Time as scarcity does not require a price feed, an oracle, or an exchange to measure: one window is one window, regardless of any currency's value.

Build a rail whose anti-abuse scarcity is time, and the rail's reliability is decoupled from the price of any asset. Build a money on top of that rail whose emission is a closed-form function of time, and the money is predictable for decades. Build presence-weighted consensus on the rail, and no one can be debanked, frozen, censored, or eclipsed by capital. The pain that 20th-century money systems caused — and that 21st-century crypto did not relieve — is closed by replacing the monetary scarcity at the foundation with a temporal one.

This is the economics of time. Montana is what it looks like in production.

## III. Montana in Plain Terms

Montana is a peer-to-peer rail for moving and recording value, owned by no one, freezable by no one, censorable by no one.

- **You hold the seed phrase. You hold the money.** Twenty-four words derive your keys. No bank, no government, no chain operator can freeze, seize or revoke your balance. A balance, once credited, belongs to its key forever. Dormant non-zero accounts are never touched, no matter how long they sit.
- **You can run a node and be paid.** Commodity hardware (one CPU core), 24/7 uptime, a network connection. The reward is `13 Ɉ` for sealing each window — closed-form, predictable, paid by the protocol to whoever does the work. No premine, no presale, no founder allocation, no halving, no supply cap.
- **You can use the rail without running anything.** A key on a phone. Send and receive Montana; commit hashes for documents and messages via `Anchor`. No fees. No gas. Settlement within one window of the canonical order.
- **No founder, no DAO, no governance, no veto.** The author is removed from the protocol by construction. Advisory councils may exist outside; none have binding force inside. No state, no corporation, no individual — including the author — can stop the network from running or rewrite a finalized event.
- **Autonomous agents are first-class participants.** Software acting on behalf of a human is a first-class operator and user of the rail, by construction. The same key, the same account chain, the same protocol — no separate plumbing for agents.
- **Post-quantum from the first day.** Signatures, key encapsulation, transport — all built on primitives that survive a sufficiently large quantum computer (FIPS 203, FIPS 204), at NIST security level 3 (≈ 192-bit quantum-equivalent strength).

Montana does not ask permission, does not ask to be regulated, does not ask to be adopted by a bank. It runs on the time of the people who run it, and they are paid for the time they spend running it.

## IV. The Ecosystem

Montana is more than a protocol. The protocol is the foundation; on top of it lives a full system of applications that no existing player can build, because every existing player is trapped in either fiat or crypto.

- **The Rail.** The Montana protocol — canonical order, post-quantum signatures, presence-weighted consensus, no fees, no founder. The substrate everything else stands on.
- **The Money** (`Ɉ`). Paid by the protocol to whoever runs a node and seals a window. Closed-form linear emission. Owned by you the moment you earn it or receive it.
- **The Wallet.** Twenty-four words on any device — phone, hardware, paper. The seed phrase is the account; the account is the user; the user owns their money. iOS, Android, desktop, web, CLI — every client uses the same primitives.
- **The Messenger** (Montana Messenger). A reference messenger client is in development as an end-to-end encrypted application federated through Montana nodes, with no central server and no Telegram-style operator able to read or block messages. The seed-derived identity that holds the user's balance is the identity that receives the messages.
- **The Data Layer** (`Anchor`). Any document, message or fact committed to the canonical order forever as a 32-byte hash. The contents stay with you, encrypted under your key. The protocol records existence; you control content.
- **The Agent Infrastructure** (Junona). Autonomous AI agents are first-class participants of the protocol by construction — they hold accounts, run nodes, transact, build chains of presence. Junona is defined as the reference agent acting on the user's behalf through delegated keys and a bounded permission model; the reference implementation is in active development.
- **The Sovereign Infrastructure** (Pluton). Sovereign physical infrastructure designed to run nodes, host applications, and provide the hardware layer under operator ownership rather than hyperscaler rental. First Pluton sites are at the design stage.
- **The Insurance Layer** (Vera Montana). A sovereign-grade insurance and reserve layer designed to sit on top of the protocol, denominated in the user's choice of unit, settling on the Montana rail. Currently at the design stage.
- **The Hub** (`hub.montana.quest`). The network's own code-hosting infrastructure — public for the public network, not rented from a hyperscaler.

This is a substrate no incumbent can occupy. Banks are bound to extraction; cryptocurrency firms are bound to fees and governance tokens; states are bound to surveillance and control; hyperscalers are bound to rental. Each existing actor has a structural incentive against the substrate Montana provides. The substrate exists only where no actor can build it from inside the model that funds them.

Montana occupies that space. Every layer that touches money, communication, data, identity and agency at the scale of one billion participants is built on a single sovereign substrate, with no rent, no founder, no veto, no master switch.

## V. Canonical Order, Not Wall-Clock Time

Each Montana node performs a **sequential delay computation** — an iterated SHA-256 hash chain `T_W = SHA-256^D (T_{W-1})` with `D = 325 000 000` iterations per window. `D` is fixed in the Genesis Decree from a single quartz measurement on the genesis hardware (Apple iMac M1 2021, idle, single-thread, 5.097 MH/s SHA-256 single-thread); after Genesis the protocol consults no clock ([I-18]). At the genesis-hardware rate, `D / 5.097 × 10⁶ ≈ 63.8 seconds` per window — the wall-clock duration of a window is therefore an emergent property of each node's hardware and is not part of consensus state.

This is **not** a verifiable delay function in the sense of Boneh-Bonneau-Bünz-Fisch [CRYPTO 2018], Pietrzak [ITCS 2019] or Wesolowski [EUROCRYPT 2019]. Those constructions provide succinct verification of order `O(log T)` or `O(1)`, but they operate over RSA groups or class groups of imaginary quadratic fields — assumptions broken by Shor's algorithm. Post-quantum succinct SSHA constructions remain at research-grade status; none has published security audits or standardization at the level of FIPS 203 / FIPS 204. Montana takes the simpler primitive: an iterated SHA-256 chain. Verification cost equals computation cost; a verifier re-runs the same iterations the prover ran. SHA-256 is already required for addressing, hashing and Merkle commitments — no new assumption is added. The cryptographic surface is minimized to one primitive ([I-7]).

The output is the **TimeChain**: a canonical, monotonic, unambiguous, independently verifiable sequence of windows. Montana does not measure physical duration. Mapping a window number to a calendar is the observer's task, not the protocol's.

## VI. The Hierarchy of Truth

Every layer is impossible without the one below.

1. **Canonical order** (`TimeChain`) — irreversible sequential computation. The base layer.
2. **Presence** (`NodeChain`) — a node's chain length, accumulated one window at a time as the node is canonically cemented into the order. Weight in consensus is presence, not capital. Capital cannot retroactively purchase past participation.
3. **The rail's reward unit** (`Account`, `Ɉ`) — the protocol-level emission paid to the operator that seals a window: `EMISSION_moneta = 13 × 10⁹ moneta = 13 Ɉ`. Supply is closed-form and grows linearly with the window count; the exact formula is defined in the protocol specification. `Ɉ` is the rail's bookkeeping; it is not a promise of stable purchasing power, and not the unit of account a stable currency would use.
4. **History** (`Anchor`) — a 32-byte hash bound to a window for the lifetime of the network. Rewriting it requires recomputing every iteration of the chain from the Genesis Decree. Mathematically impossible.

`1 Ɉ = 10⁹ moneta`. The international ticker is `MONT`.

## VII. Post-Quantum from the First Day

- **Consensus signatures:** ML-DSA-65 (FIPS 204).
- **Transport key encapsulation:** ML-KEM-768 (FIPS 203).
- **Hashing:** SHA-256 (FIPS 180-4).
- **Transport handshake:** Noise_PQ XX — ephemeral ML-KEM-768 on both sides, an ML-DSA-65 signature binding the full transcript, ChaCha20-Poly1305 AEAD framing (RFC 8439) on the established session.
- **PeerId:** the SHA-256 multihash of each peer's ML-DSA-65 identity public key.

No ECDSA. No EdDSA. No classical Diffie-Hellman. No assumption that Shor's algorithm will be late.

## VIII. Properties of the Rail

What makes Montana a neutral settlement and ordering rail are not features layered on a chain — they are the chain.

- **Zero fees.** The protocol contains no `fee` field on any operation. A seven-cent transfer settles.
- **Asynchronous finality without extractable ordering.** Transfers do not wait for blocks. They are cemented through a P2P quorum of signatures from active operators within a single window of the canonical order (one window of `D` sequential SHA-256 iterations, ≈ 64 s on the genesis hardware per §V). Proposer discretion over inclusion is zero; operation ordering inside a window is fixed by canonical τ₁-rate rules, not by the proposer's local mempool view. There is no extractable position over the order of operations.
- **Constant monotonic emission as bookkeeping.** `13 Ɉ` per window, fixed by the Genesis Decree, closed-form. `Ɉ` is what the rail pays its operators — bookkeeping for the work of sealing a window, not a stable unit of account.
- **No plutocracy by construction.** Whoever holds a billion `Ɉ` has no more power in consensus than the operator of a Mac Mini. A node's weight is its chain length. The lottery seed incorporates `cemented_bundle_aggregate(W-2)`, signatures from honest operators two windows back, which closes the grinding attack class under hardware asymmetry without depending on rational-cost arguments.
- **Two-thirds honest chain length.** Safety holds while honest operators control more than two-thirds of `active_chain_length`. Capital does not enter the threshold.

## IX. The Economics of Time

Anti-abuse is done by time, not by money — three independent scarcities, each derived from time elapsed.

- **Per-identity rate per window.** One operation per account per window τ₁. An attacker with N Sybil identities gets at most N operations per window, but each identity has its own creation cost.
- **`account_chain_length` thresholds.** Privileged operations require the operating account to have been active for at least `k` windows. The threshold cannot be purchased.
- **Sequential entry barrier for node operators.** Node registration requires producing a sequential SHA-256 chain of length `ssha_chain_length × D` iterations. The protocol parameter `ssha_entry_windows = 20 160 windows` (one τ₂ epoch) sets the threshold; at `D = 325 000 000` iterations per window, the total cost is `ssha_entry_windows × D = 6.552 × 10¹² SHA-256 hashes`, which is fourteen days of wall-clock at the genesis-hardware calibration (one window ≈ 64 s emergent per §V). An attacker with `M` parallel machines produces `M` identities at the same wall-clock cost, not faster.

Together these three close DoS without monetary barriers. Time as scarcity does not require a price feed, an oracle or an exchange to measure.

## X. The Ladder of Sovereignty, Scale, and Removal of the Author

- **Two roles, one chain.** *Account user*: a key on a phone or hardware wallet, no protocol-layer earnings, barrier is a first incoming Transfer (the AccountRecord is created atomically with crediting the amount). *Node operator*: commodity hardware, 24/7 uptime, sequential SHA-256 entry barrier at registration, earnings through the per-window node lottery. The seed phrase and the account chain belong to the user, not to the node; the user moves up the ladder when they choose to.
- **Calibrated for one billion users.** AccountRecord is 2 059 bytes; state at 10⁹ accounts is approximately 2.06 TB, holdable on commodity disks. Pruning is canonical: state size is bounded by active population, not by chain age. Mechanisms that do not scale to 10⁹ are discarded without discussion.
- **Privacy as a choice.** Balances and account graphs are public by default ([I-2]). Application-level privacy is achieved through `Anchor`: a 32-byte hash is committed to the chain; encrypted content is held off-chain by its owner. The protocol has no visibility into the contents.
- **No governance in state.** No DAO, no treasury, no founder veto. Advisory councils may exist outside; none have binding force inside. The author is removed from the protocol by construction. Montana launches as a peer-to-peer network with no founder-controlled bootstrap quorum.

## XI. What Montana Is

Not the currency Bitcoin's title promised — that is a stable unit of account, and it lives one floor up. Not digital gold. Not yield. Not governance. Not a brand. Not a privacy mixer. Not an L2. Not a blockchain with a timestamping feature. Not the next crypto project. Not a faster version of what failed.

Montana is an architecture that closes the failures of both fiat and cryptocurrency on a single foundation: a settlement and ordering rail whose anti-abuse scarcity is time rather than money, whose consensus weight is presence rather than capital, whose primitive set survives quantum adversaries, and whose author is removed from the protocol by construction.

A rail no actor can remove a user from. A money no institution can freeze. A unit that compensates the operator for the work of sealing a window. A settlement layer with no extractable position over the order of operations. A communication layer that does not rent messages back to their sender. A data layer that does not sell its contents. A protocol with no founder, no DAO, no veto, no central control. A network whose security derives from the time of its operators.

The medium that fills the space between participants, with no charge for being present and no extractable position over the order of operations. The ether the name promised.

---

**Reference implementation:** Rust, Apache-2.0 / MIT. Twenty-three crates including `mt-timechain`, `mt-consensus`, `mt-lottery`, `mt-crypto`, `mt-net`, `mt-noise-pq`. Specification: [Montana Whitepaper.md](../Montana%20Whitepaper.md) and [Montana Protocol v35.25.1](../Montana%20Protocol%20v35.25.1.md).

**Symbol:** **Ɉ** — Montana, the rail's protocol-level reward unit. `moneta` — the smallest indivisible unit (`1 Ɉ = 10⁹ moneta`). **Ticker:** `MONT`.

Alejandro Montana
*Ничто_Nothing_无_金元Ɉ*
2026-05-29
