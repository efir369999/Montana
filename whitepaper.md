# Montana: A Post-Quantum Blockchain with Time as Scarcity

**Alejandro Montana**
[github.com/efir369999/Montana](https://github.com/efir369999/Montana)


## Abstract

A post-quantum cryptocurrency would allow value to be transferred between parties without reliance on classical cryptographic primitives that quantum adversaries can break. Existing chains rely on signatures (ECDSA, EdDSA) whose security collapses under Shor's algorithm and on transaction-fee-based anti-spam mechanisms that price out small users at scale. We propose a blockchain whose security rests entirely on post-quantum primitives standardized by NIST in 2024 (ML-DSA-65, ML-KEM-768) and on hashing (SHA-256), and whose anti-spam mechanism operates on time rather than money. A verifiable delay function over SHA-256 produces a globally ordered chain of windows of approximately 60 seconds each. Each window is sealed by a sequential proof of work whose computation cannot be parallelized and cannot be skipped. Operations within a window are rate-limited per identity, by the cumulative chain length of the operating account, and by seniority constraints — three different scarcities derived from time elapsed, not balance held. As long as honest operators run the verifiable delay function, the chain extends, regardless of how many actors hold tokens or what fees they would have paid.


## 1. Introduction

Bitcoin and its descendants demonstrate that decentralized monetary consensus is achievable without trusted intermediaries. Two limitations prevent these systems from serving as a general financial substrate at the scale of a billion users.

First, all production cryptocurrencies derive their signature security from elliptic-curve discrete logarithm assumptions. Shor's algorithm [8], when run on a sufficiently large quantum computer, breaks these assumptions in polynomial time. The U.S. National Institute of Standards and Technology standardized post-quantum signature and key encapsulation mechanisms in 2024 (FIPS 203 [2], 204 [3], 205); existing major chains have not migrated. The migration is not a trivial parameter change — wire formats, address derivation, multisig schemes, light-client proofs all depend on the underlying primitive.

Second, the anti-spam mechanism in fee-based chains scales poorly under adoption. As block space becomes scarce, small operations are priced out, defeating the original use case of low-friction online payments. Layer-two systems (state channels, rollups) shift the economics rather than remove the underlying scarcity.

We propose Montana, a chain whose security rests on post-quantum primitives only and whose anti-spam mechanism operates on time rather than money. The chain advances by a verifiable delay function over SHA-256, producing globally ordered windows of approximately 60 seconds. Operations are rate-limited by per-identity windows, account chain length, and seniority — three independent scarcities derived from time elapsed.


## 2. Time as a Scarce Resource

In a fee-based chain, the scarce resource is block space; access is allocated by willingness to pay. Spam is deterred by the price of inclusion. Two failure modes follow. Under congestion, ordinary users are excluded by price. Under abundance, spammers re-enter at marginal cost. The mechanism does not converge on a stable point; it oscillates with demand.

We replace block-space scarcity with time scarcity. The verifiable delay function (VDF) [5,6,7] over SHA-256 forces a sequential computation that cannot be parallelized: D iterations of SHA-256 must be performed in series, where D is calibrated so that the computation takes approximately 60 seconds on commodity x86_64 hardware. The output of one window is the input to the next. The total length of the chain measures wall-clock time elapsed since genesis, recoverable by anyone who can verify the VDF output.

Time is uniformly available to all participants. An attacker with one hundred times the resources of an honest operator does not get one hundred times more time. The attacker may run more parallel chains, but each chain still advances at the same wall-clock rate. Sybil identities do not produce more time per identity — they produce more identities, each subject to the same per-identity per-window rate limit.

Time as scarcity does not require a price feed, an exchange, a pricing oracle. Its valuation is fixed by the protocol: one window equals one window, regardless of currency value.


## 3. The TimeChain

Let `T_r` denote the VDF output at window `r`. The TimeChain advances by

```
T_r = SHA-256^D (T_{r-1})
```

where `T_0` is the genesis seed and `D` is the per-window iteration count. `D` is initialized at 325 000 000 and recalibrated every 20 160 windows (approximately fourteen days) according to a formula tied to median observed wall-clock window times across honest operators. The recalibration is canonical: every honest operator computes the same new `D` from public inputs.

The verifiability of the VDF allows any node to confirm `T_r` from `T_{r-1}` by performing the same `D` iterations. There is no trusted setup; the output is a public function of the input and the parameter.

A new operator joining the network is required to produce a candidate VDF chain of length at least 20 160 windows (approximately ten hours of wall-clock time on commodity hardware). This requirement is the protocol's Sybil defense: producing N false identities requires N candidate chains, each consuming N times the wall-clock time. There is no shortcut.


## 4. Post-Quantum Primitives

Signatures are produced and verified by ML-DSA-65, the FIPS 204 module-lattice signature scheme. Key encapsulation, where used (operator handshake, encrypted application payloads), is performed by ML-KEM-768, the FIPS 203 module-lattice scheme. Both are members of the NIST PQC standardization output. Key sizes are: public key 1952 bytes (ML-DSA-65), secret key 4032 bytes, signature 3309 bytes; ML-KEM-768 public key 1184 bytes, ciphertext 1088 bytes, shared secret 32 bytes.

Hashing is SHA-256 (FIPS 180-4 [4]). Grover's algorithm [9] reduces the effective preimage security of SHA-256 from 256 to 128 bits in the quantum model, which remains adequate.

Key derivation from a 24-word mnemonic uses PBKDF2-HMAC-SHA-256 with iter=2^20 to compute a master seed, then HKDF-SHA-256 to derive per-purpose keys (account signing, node signing, encrypted app payloads). The mnemonic wordlist is 256 Russian-language words selected for distinguishability under typing, listening, and transcription. The protocol is alphabet-agnostic; the wordlist is a deployment choice and may be substituted with any 256-word set whose entropy claim per word is identical (8 bits).


## 5. Operations and the Account Table

The state is a single Account Table mapping account identifiers to records:

```
AccountRecord {
  account_id            32 bytes (SHA-256 of account public key)
  public_key            1952 bytes (ML-DSA-65)
  balance               16 bytes (u128, denominated in nɈ; 1 Ɉ = 10^9 nɈ)
  account_chain_length  8 bytes (u64, count of cemented operations from this account)
  last_active_window    8 bytes (u64, window index of most recent operation)
  is_node_operator      1 byte (boolean flag)
  ...
}
```

Operations transform state through `apply_proposal(state, proposal) → state'`. The transformation is deterministic, byte-exact reproducible by any node from the same `(state, proposal)` pair. The set of operation classes is closed: Transfer, OpenAccount, ChangeKey, NodeRegistration, Anchor, NicknameBid, TransferActivation, CloseAccount. Each operation has a fixed canonical encoding, a fixed validation rule, and a fixed apply function.

Conservation invariants hold per operation: the sum of balance deltas across all affected records equals the emission delta plus the burn delta. No operation creates or destroys value silently.


## 6. Lottery

The operator who completes the VDF for window `r` is selected by a deterministic lottery from the set of registered operators. Each operator submits a `VdfReveal` with the window's VDF output and a signature; the lottery winner is

```
winner = argmin_{operator}  ticket(operator, r)
```

where

```
ticket(operator, r) = SHA-256(operator.node_id || cemented_bundle_aggregate(r-2) || r)
```

The `cemented_bundle_aggregate(r-2)` term is the lottery's network-bound unpredictability source: it incorporates signatures from honest operators in window `r-2`, which an attacker cannot precompute without privkeys held by honest participants. This closes the class of attacks where an adversary with hardware advantage precomputes future windows and grinds attacker-chosen fields against them.


## 7. Incentive

The lottery winner of window `r` receives 13 base units of Ɉ (`13 × 10^9 nɈ`), credited to the operator account. There are no transaction fees. There is no second-tier inflation. There is no premine, no presale, no founder allocation. The total emission at window `r` is exactly `13 × r` units, a closed-form function of window count.

Storage of accumulated value is not separately incentivized. The protocol does not pay for holding tokens. The single reward path is operating the VDF for a window and winning that window's lottery.

For any operator, the expected income per unit time depends on the share of cemented `VdfReveal`s contributed by that operator across windows. With `N` operators of equal computational power running honest VDF, the expected reward per operator per window is `13/N` Ɉ. With unequal power, the share is proportional to the number of valid `VdfReveal`s submitted in time.


## 8. Anti-Spam Without Fees

Spam protection is the composition of three time-based mechanisms:

**Per-identity rate.** Operations of class A (Transfer, NicknameBid, etc.) are limited to one per account per window τ_1 = 1 window. An attacker with N Sybil identities can perform at most N operations per window, but each Sybil identity has its own creation cost (see below). The rate is uniform across identities; there is no fast lane.

**Chain-length threshold.** Privileged operations (e.g. NodeRegistration, NicknameBid) require the operating account's `account_chain_length` to exceed a threshold k. An account must be active for at least k windows before issuing such an operation. The threshold cannot be purchased; it can only be earned by elapsed activity.

**Seniority gating.** Lottery weight in the operator selection scales with the operator's `account_chain_length` up to a saturation point. New operators have lower weight; they accrue weight by participating across windows. This dampens flash-mob attacks where many adversarial operators register simultaneously.

These mechanisms together close DoS without monetary barriers. The protocol contains no `fee` field on any operation.


## 9. State Lifecycle and Pruning

Every persistent record in consensus state has either a cost-based barrier, a lifecycle bound, or a hard quota. Account creation requires the creator to submit an opening operation whose validation includes a chain-length precondition. Accounts whose balance falls below `MIN_ACCOUNT_BALANCE = 1 nɈ` and whose `last_active_window` precedes the current window by more than `8 × 20 160` windows are pruned by `apply_candidate_expiry` at the next epoch boundary.

Pruning is not optional; it is part of the canonical state transition. Two honest nodes following the protocol prune identically. The Account Table size is bounded above by

```
|AccountTable(W)| ≤ creation_rate × retention_window
```

which is independent of accumulated wall-clock time, ensuring that long-running chains do not produce unbounded state.


## 10. Privacy

The protocol exposes balances, transfers, account graphs, and operator identities by default. Application-layer privacy is achieved through Anchor objects: an account commits a 32-byte hash to chain, and the contents (encrypted under the owner's key) are held off-chain by the owner or by a delegated peer. The Anchor mechanism does not give the protocol visibility into the contents.

Privacy is a user choice rather than a protocol-imposed feature. Mass-surveillance through privacy-by-protocol is not within scope; selective privacy through user-managed encryption is. This boundary aligns the protocol with regulatory frameworks (FATF, MiCA) that have rejected protocol-level privacy mixers while accepting end-user encryption of off-chain content.


## 11. Network and Synchronization

The protocol's wire format and synchronization mechanism are described in [`mt-net`](https://github.com/efir369999/Montana/tree/main/Код/crates/mt-net) and [`mt-net-transport`](https://github.com/efir369999/Montana/tree/main/Код/crates/mt-net-transport) of the reference implementation. Operators discover peers, exchange `VdfReveal` and `BundledConfirmation` messages, and replicate the cemented chain via libp2p over TCP+TLS.

A new node synchronizes by acquiring the current TimeChain head from any honest peer, verifying the VDF chain locally, and replicating the Account Table snapshot rooted in the current Merkle commitment. Synchronization is verify-only; no trust in the source peer is required beyond the TLS connection.


## 12. Calculations

We consider a scenario where an honest operator and an attacker compete to win windows. The probability that the attacker wins a given window is proportional to the attacker's share of total VDF computational power. With attacker share `p` and honest share `1 − p`, the probability that the attacker wins `k` consecutive windows is `p^k`, decreasing geometrically.

For the lottery to be biased in favor of the attacker, the attacker must control more than half of all registered operator power. With per-operator wall-clock VDF being constant (no parallelization), Sybil identity multiplication does not increase total power; it only fragments the same power across more identities. The attacker's share is bounded by the number of physical machines they operate, not by capital.

This is the security argument: monetary capital does not buy more time. The operator economy reduces to a hardware economy in which the unit good (one VDF window) is uniformly priced in joules.

```
P(attacker wins k consecutive windows) = p^k
```

For `p = 0.3` and `k = 10`, P = 5.9 × 10^-6, comparable to the Bitcoin probability of an attacker reorganizing a chain after 10 confirmations under analogous attacker share.


## 13. Conclusion

We have proposed a blockchain whose security rests on post-quantum cryptographic primitives and whose anti-spam mechanism operates on time rather than fees. The construction does not require trusted setup, does not require a price feed, and does not impose a monetary barrier on participation. The mechanism scales to a billion active accounts as a baseline architectural target.

The reference implementation in Rust is available at the cited URL under permissive license (Apache-2.0 / MIT). Further work includes the network-layer integration into the node binary (M6 multi-node deployment), the snapshot-based fast synchronization (M7), and the conformance suite expansion to second implementations in independent languages (M9).


## References

[1] S. Nakamoto, "Bitcoin: A Peer-to-Peer Electronic Cash System," 2008.

[2] National Institute of Standards and Technology, "Module-Lattice-Based Key Encapsulation Mechanism Standard," FIPS 203, 2024.

[3] National Institute of Standards and Technology, "Module-Lattice-Based Digital Signature Standard," FIPS 204, 2024.

[4] National Institute of Standards and Technology, "Secure Hash Standard (SHS)," FIPS 180-4, 2015.

[5] D. Boneh, J. Bonneau, B. Bünz, B. Fisch, "Verifiable Delay Functions," CRYPTO 2018.

[6] K. Pietrzak, "Simple Verifiable Delay Functions," ITCS 2019.

[7] B. Wesolowski, "Efficient verifiable delay functions," EUROCRYPT 2019.

[8] P. W. Shor, "Polynomial-Time Algorithms for Prime Factorization and Discrete Logarithms on a Quantum Computer," SIAM Journal on Computing, 1997.

[9] L. K. Grover, "A fast quantum mechanical algorithm for database search," STOC 1996.


---

Alejandro Montana
