# The Montana Manifesto

**Version:** 1.0.0
**Date:** 2026-05-28
**Author:** Alejandro Montana
**Repository:** [github.com/efir369999/Montana](https://github.com/efir369999/Montana)

> *"He who controls the past controls the future. He who controls the present controls the past."*
> — George Orwell, *1984*

## I. The Question

Bitcoin answered one question: **Whom do we trust with money?** *No one. Trust mathematics.*

Bitcoin removed trust from money but left trust in time. Its difficulty adjusts to the wall-clocks of its miners; its block heights are measured against the watches of the world outside.

Montana answers a deeper question: **Whom do we trust with time?**

Money is a derivative of time, not the other way around. Today the infrastructure of time (NTP), of position (GPS), of communication (messaging servers) and of history (centralized databases) demands unconditional trust in a third party. One point of failure is one point of control. To control this infrastructure is to control the present. To control the databases is to rewrite the past.

Montana makes *1984* technically impossible.

## II. Time as Computation

In Montana, a Verifiable Delay Function is not a clock that *displays* time. The VDF *is* time, written into the work of a sequential SHA-256 hash chain (FIPS 180-4). Each window is a sequential computation of `D ≈ 325 000 000` iterations on commodity x86_64 hardware. It cannot be parallelized; it cannot be faked; it cannot be hurried beyond the physics of the processor.

Montana does not consume external time. Montana **produces** it. The output is an unbreakable cryptographic arrow of time — the **TimeChain**.

We chose a sequential SHA-256 delay function over the efficiently-verifiable constructions of Boneh, Bonneau, Bünz and Fisch [CRYPTO 2018], Pietrzak [ITCS 2019] and Wesolowski [EUROCRYPT 2019] deliberately. Verification cost equals computation cost. The minimal cryptographic surface is its own audit. SHA-256 is already required for hashing, addressing and Merkle commitments; no new assumption is added.

## III. The Hierarchy of Truth

Montana is built on a strict dependency. Every layer is impossible without the one below.

1. **Time** (`TimeChain`) — irreversible computation. The base layer of physics. Every operator ticks independently; together they form one global oscillator.
2. **Presence** (`NodeChain`) — proof that a specific identity accompanied this stream of time. Weight in the network is measured by proven time of presence, not by capital. Capital does not buy more time.
3. **Money** (`Account`, `TimeCoin`) — the quantitative derivative of proven presence. The unit `Ɉ` is not a reward for solving meaningless puzzles; it is the recording of a passed second in the network's ledger. Emission is closed-form: `supply(W) = 13 × (W + 1) Ɉ`. No premine. No presale. No founder allocation.
4. **History** (`Anchor`) — the binding of any external fact (document, message, transaction) to this protected timeline. A hash is sealed in the TimeChain. To rewrite it is to recompute every iteration of the VDF from genesis. Mathematically impossible.

*Money without proven presence is a phantom. Presence without verifiable time is a claim. Time without irreversible computation is trust.*

## IV. Post-Quantum from the First Day

All consensus signatures are **ML-DSA-65** (FIPS 204). All transport key encapsulation is **ML-KEM-768** (FIPS 203). Hashing is **SHA-256** (FIPS 180-4). The transport handshake is **Noise_PQ XX**: ephemeral ML-KEM-768 on both sides, an ML-DSA-65 signature binding the full handshake transcript, and ChaCha20-Poly1305 AEAD framing on the established session (RFC 8439).

No ECDSA. No EdDSA. No classical Diffie-Hellman. No assumption that Shor's algorithm will be late.

PeerId is the SHA-256 multihash of each peer's ML-DSA-65 identity public key. Routing identity and consensus identity are bound to the same key material.

## V. Architecture Without Compromise

- **Zero fees.** Anti-spam is operated by time, not by money: per-identity rate per window, `account_chain_length` thresholds, seniority gating. The protocol contains no `fee` field on any operation.
- **Asynchronous finality.** Transfers do not wait for blocks. They are cemented through a P2P quorum of signatures from active operators in approximately 300 milliseconds.
- **No plutocracy.** Whoever holds one billion `Ɉ` has no more power in consensus than the operator of a Mac Mini. Emission (chronometric) and consensus (Proof of Time) are mathematically separated. The lottery seed incorporates `cemented_bundle_aggregate(W-2)` — a value an attacker cannot precompute without forging the signatures of honest participants.
- **No governance in state.** There is no DAO, no treasury, no founder veto. Advisory councils may exist outside the protocol; none of them have binding force inside it. The author is removed from the protocol.
- **No genesis nodes.** Montana launches as a peer-to-peer network in the style of Bitcoin. Any participant joins by running one command in a terminal. There is no founder-controlled bootstrap quorum.
- **67% honest active chain length.** Safety holds while honest operators control more than two-thirds of `active_chain_length`. Capital does not enter this threshold.

## VI. The Scale Baseline

Every decision in Montana is calibrated for **at least one billion active users**. Mechanisms that do not scale to 10⁹ are discarded without discussion. AccountRecord is 2 059 bytes; state at 10⁹ accounts is approximately 2.06 TB, holdable on commodity disks. The pruning rule is canonical: state size is bounded by active population, not by chain age.

## VII. Privacy as a Choice

Balances, transfers and operator identities are public by default. Privacy is achieved through **Anchor** objects: a 32-byte hash is committed to the chain and the encrypted content is held off-chain by its owner. The protocol has no visibility into the contents. Privacy is what the user chooses to keep — not what the protocol imposes nor what the protocol forbids.

## VIII. What Montana Is Not

Montana is not a faster Ethereum. Montana is not an L2. Montana is not a privacy mixer. Montana is not yield. Montana is not governance. Montana is not a brand.

Montana is the digital atomic clock for the internet. It is the standard of frequency from which money, presence and history derive.

---

**Reference implementation:** Rust, Apache-2.0 / MIT. Twelve crates including `mt-timechain`, `mt-consensus`, `mt-lottery`, `mt-crypto`, `mt-net`, `mt-noise-pq`. Specification: [Whitepaper Montana.md](../Whitepaper%20Montana.md).

**Symbol:** **Ɉ** — one second of Montana time.

Alejandro Montana
*Ничто_Nothing_无_金元Ɉ*
2026-05-28
