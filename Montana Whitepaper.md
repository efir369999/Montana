# Montana: A Post-Quantum Blockchain with Time as Scarcity

**Alejandro Montana**
[github.com/efir369999/Montana](https://github.com/efir369999/Montana)


## Abstract

A post-quantum cryptocurrency would allow value to be transferred between parties without reliance on classical cryptographic primitives that quantum adversaries can break. Existing chains rely on signatures (ECDSA, EdDSA) whose security collapses under Shor's algorithm and on transaction-fee-based anti-spam mechanisms that price out small users at scale. We propose a blockchain whose consensus signatures and application-layer key encapsulation rest on post-quantum primitives standardized by NIST in 2024 (ML-DSA-65, ML-KEM-768) and whose anti-spam mechanism operates on time rather than money. A sequential delay function over SHA-256 produces a globally ordered chain of windows of approximately 60 seconds each. Each window is sealed by an iterated hash computation that cannot be parallelized and cannot be skipped; verification cost equals computation cost. Operations within a window are rate-limited per identity, by the cumulative chain length of the operating account, and by seniority constraints — three different scarcities derived from time elapsed, not balance held. As long as honest operators run the sequential delay function, the chain extends, regardless of how many actors hold tokens or what fees they would have paid. Network transport is Noise_PQ XX: a three-message handshake with ephemeral ML-KEM-768 key encapsulation on both sides, ML-DSA-65 identity signatures over the transcript, and ChaCha20-Poly1305 AEAD framing on the established session. The transport stack is `TCP → Noise_PQ XX → Yamux`. PeerId is the SHA-256 multihash of each peer's ML-DSA-65 identity public key. Transport confidentiality rests on post-quantum primitives.


## 1. Introduction

Bitcoin and its descendants demonstrate that decentralized monetary consensus is achievable without trusted intermediaries. Two limitations prevent these systems from serving as a general financial substrate at the scale of billions of users.

First, all production cryptocurrencies derive their signature security from elliptic-curve discrete logarithm assumptions. Shor's algorithm [8], when run on a sufficiently large quantum computer, breaks these assumptions in polynomial time. The U.S. National Institute of Standards and Technology standardized post-quantum signature and key encapsulation mechanisms in 2024 (FIPS 203 [2], FIPS 204 [3]); existing major chains have not migrated. The migration is not a trivial parameter change — wire formats, address derivation, multisig schemes, light-client proofs all depend on the underlying primitive.

Second, the anti-spam mechanism in fee-based chains scales poorly under adoption. As block space becomes scarce, small operations are priced out, defeating the original use case of low-friction online payments. Layer-two systems (state channels, rollups) shift the economics rather than remove the underlying scarcity.

We propose Montana, a chain whose consensus signatures rest on post-quantum primitives and whose anti-spam mechanism operates on time rather than money. The chain advances by a sequential delay function over SHA-256, producing globally ordered windows of approximately 60 seconds. Operations are rate-limited by per-identity windows, account chain length, and seniority — three independent scarcities derived from time elapsed.


## 2. Time as a Scarce Resource

In a fee-based chain, the scarce resource is block space; access is allocated by willingness to pay. Spam is deterred by the price of inclusion. Two failure modes follow. Under congestion, ordinary users are excluded by price. Under abundance, spammers re-enter at marginal cost. The mechanism does not converge on a stable point; it oscillates with demand.

We replace block-space scarcity with time scarcity. A sequential delay function over SHA-256 forces a sequential computation that cannot be parallelized: D iterations of SHA-256 must be performed in series, where D is calibrated so that the computation takes approximately 60 seconds on commodity x86_64 hardware. The output of one window is the input to the next. The total length of the chain measures wall-clock time elapsed since genesis, recoverable by anyone who can verify the chain by replaying the iterations.

Time is uniformly available to all participants. An attacker with one hundred times the resources of an honest operator does not get one hundred times more time per chain. The attacker may run more parallel chains, but each chain still advances at the same wall-clock rate. Sybil identities do not produce more time per identity — they produce more identities, each subject to the same per-identity per-window rate limit within the protocol.

Time as scarcity does not require a price feed, an exchange, or a pricing oracle. Its valuation is fixed by the protocol: one window equals one window, regardless of currency value.


## 3. The TimeChain

Let `T_W` denote the chain output at window `W`. The TimeChain advances by

```
T_W = SHA-256^D (T_{W-1})
```

where `T_0` is the genesis seed and `D` is the per-window iteration count. `D` is initialized at 325 000 000 and recalibrated every 20 160 windows (approximately fourteen days) according to a formula tied to median observed wall-clock window times across honest operators. The recalibration is canonical: every honest operator computes the same new `D` from public inputs.

This construction is a sequential delay function: the iteration must be performed in order, and verification requires re-running the same `D` iterations. Verification cost equals computation cost — there is no asymmetric verification shortcut as in verifiable delay functions of Boneh, Bonneau, Bünz, and Fisch [5], Pietrzak [6], or Wesolowski [7]. Those constructions operate over RSA groups or class groups of imaginary quadratic fields and achieve O(log T) or O(1) verification. Montana adopts the simpler primitive for two reasons: (i) the cryptographic surface is minimized, depending only on SHA-256 (FIPS 180-4 [4]) which is already required for hashing, addressing, and Merkle commitments; (ii) verification asymmetry is not strictly required when the verifier is itself an operator running the chain — operators verify by extending the chain, which is the same work they perform for the next window.

A new operator joining the network is required to produce a candidate sequential hash chain of length at least 20 160 windows — the protocol parameter `ssha_entry_windows` equal to one τ₂ epoch, approximately fourteen days of wall-clock time on commodity hardware — before becoming eligible to participate in the lottery. This requirement is the protocol's Sybil entry barrier.

The cost of producing N candidate identities scales linearly. Each candidate chain has the same per-chain wall-clock cost T (approximately fourteen days, i.e. one τ₂ epoch). An attacker with N machines can compute all N chains in parallel at wall-clock T, paying N × T machine-time of computation. With one machine, the attacker pays N × T wall-clock time. There is no quadratic multiplier and no time-non-parallelizability across distinct identities. Sybil cost is therefore linear in hardware and linear in energy expenditure, not super-linear in either.

Sybil resistance within Montana derives from the composition of this entry cost with the in-protocol per-identity rate limits (Section 10) and the seniority gating of the lottery (Section 7). The combined effect is that attacker influence over consensus grows linearly with hardware budget and not at all with token holdings.


## 4. Post-Quantum Primitives

Signatures are produced and verified by ML-DSA-65, the FIPS 204 module-lattice signature scheme [3]. Key encapsulation, where used (operator handshake, encrypted application payloads), is performed by ML-KEM-768, the FIPS 203 module-lattice scheme [2]. Both are members of the NIST PQC standardization output at NIST security level 3 (approximately 192-bit symmetric-equivalent strength). Key sizes: ML-DSA-65 public key 1952 bytes, secret key 4032 bytes, signature 3309 bytes; ML-KEM-768 public key 1184 bytes, ciphertext 1088 bytes, shared secret 32 bytes.

ML-DSA-65 is used in deterministic mode (RND = 0x00 × 32 per FIPS 204 §3.7) to ensure byte-identical signatures for identical (sk, message) pairs. All implementations are required to use constant-time operations to resist timing and power side-channel attacks.

Hashing is SHA-256 (FIPS 180-4 [4]). Grover's algorithm [9] reduces the effective preimage security of SHA-256 from 256 to 128 bits in the quantum model, which remains adequate. Collision resistance against quantum attackers is bounded by 85 bits (BHT algorithm); this affects only protocols that depend on collision resistance for adversarially chosen inputs. Montana's domain-separated hash compositions and signed-input constructions avoid this dependence: the inputs to consensus-critical hashes are either canonical and unpredictable-offline, or signed by honest participants, eliminating the collision attack surface in practice.

Key derivation from a 24-word mnemonic uses PBKDF2-HMAC-SHA-256 with iter = 2^20 to compute a master seed, then HKDF-SHA-256 to derive per-purpose keys (account signing, node signing, encrypted app payloads). The mnemonic wordlist is 256 Russian-language words selected for distinguishability under typing, listening, and transcription. The protocol is alphabet-agnostic; the wordlist is a deployment choice and may be substituted with any 256-word set whose entropy claim per word is identical (8 bits). The wordlist and its selection methodology are maintained in the project repository as `Montana wordlist.txt`.


## 5. Threat Model

Montana's threat model is formulated explicitly to delimit security properties that are claimed from those that are not.

**Attacker classes.** We consider three classes:
- **Profit-seeking adversaries** — adversaries with rational economic motivation, bounded by a budget and seeking positive expected return.
- **Sabotage adversaries** — adversaries with fixed budgets seeking to inflict damage on the network without expectation of monetary return (state-level adversaries, large competitors, disgruntled insiders).
- **Network-level adversaries** — adversaries with control over substantial fractions of network paths, capable of dropping, reordering, or delaying messages between operators.

**Assumed honest majority.** Honest operators control more than 67% of total `active_chain_length` (the sum of cemented operations across all active operator chains). Quorum is weighted by `active_chain_length`, not by headcount, so Sybil identity inflation does not weaken the quorum requirement.

**Hardware-bounded influence.** Attacker advantage in consensus participation scales linearly with hardware budget (parallel SHA-256 compute) and not at all with token holdings. Capital does not buy more time. An adversary with k times the hardware of an honest median operator obtains at most k times the operator share in expectation.

**What Montana defends.** Consensus integrity (no operation is cemented without honest quorum signature); signature unforgeability (post-quantum, ML-DSA-65); Sybil-bounded influence (linear in hardware); chain liveness under honest > 67% (Section 8 and below).

**Out of scope at the protocol layer.** Application-layer metadata anonymity beyond what Anchor encryption provides — the network observes operation timing and counts even when content is encrypted. Fairness of the bootstrap period before the operator population stabilizes is addressed in Section 9. Transport-layer confidentiality against quantum adversaries is in scope and is established by Noise_PQ XX (Section 13).

**Failure conditions.** Safety fails when an attacker controls > 50% of active_chain_length and > 50% of operator SHA-256 compute simultaneously and for a sustained duration. Liveness halts (without safety loss) when fewer than 67% of active operators are responsive within the fallback cascade (Section 8).

**Quantum adversary.** The relevant threat is an adversary who records ciphertexts today and decrypts them once a sufficiently large quantum computer becomes available. Consensus signatures rest on ML-DSA-65 (FIPS 204 [3]), whose security reduces to the hardness of the Module Learning With Errors problem under quantum reductions. Transport handshakes rest on ML-KEM-768 (FIPS 203 [2]) under the same module-lattice assumption. Hashing is SHA-256 (FIPS 180-4 [4]), whose preimage security degrades to 128 bits under Grover [9] and remains adequate. Collision resistance under quantum search degrades to 85 bits (BHT); consensus-critical hash inputs are domain-separated and either canonical-unpredictable or signed by honest participants, eliminating the quantum collision attack surface in practice (invariant [I-8], Section 7).

**Sybil attack.** Producing N candidate operator identities requires N candidate sequential hash chains, each of length `ssha_entry_windows = 20 160` windows (one τ₂ epoch, approximately fourteen days of wall-clock on commodity hardware). With M machines running in parallel, the wall-clock cost of N candidates is `⌈N/M⌉ · τ₂`; the machine-time cost is `N · τ₂`. Scaling is linear in either hardware or wall-clock. There is no quadratic multiplier and no time-non-parallelizability across distinct identities. Sybil resistance therefore composes the entry cost with the in-protocol per-identity rate limits (Section 10) and seniority gating of the lottery (Section 7): adversarial influence over consensus grows linearly with hardware budget and not at all with token holdings.

**Equivocation.** Each account chain is restricted to at most one cemented operation per τ_1 = 1 window. Two operations with the same `prev_hash` from the same sender constitute equivocation; both operations are marked equivocated and neither is cemented. The intra-window ordering surface (a violation source of determinism [I-3] and an additional consensus seed surface [I-8]) is eliminated by construction at N = 1.

**Grinding under hardware asymmetry.** An adversary with k times the SHA-256 throughput of a commodity operator can precompute k hours of TimeChain output in one hour of wall-clock. The lottery seed includes `cemented_bundle_aggregate(W-2)`, a value that depends on ML-DSA-65 signatures from honest operators in window `W-2`. Precomputation against this seed requires forging ML-DSA-65 signatures, which lattice-based EUF-CMA security prevents at NIST level 3 (192-bit quantum-equivalent strength). The grinding horizon collapses to already-cemented windows where attacker-chosen fields are frozen by the registered `node_id` committed at registration (Section 7).

**Eclipse attack.** An adversary controlling adjacent network paths attempts to monopolize a victim operator's peer set. The defense is libp2p outbound diversity: each operator maintains at least 24 outbound connections selected for AS and IPv4-prefix-16 diversity (≥7 distinct autonomous systems). The probability of eclipse at attacker share f = 0.3 of the peer pool is `f^24 < 2⁻⁴⁰`, conforming to the rekey-interval target of standard network cryptographic protocols.

**Long-range rewrite.** An adversary attempting to substitute an alternative chain history from genesis must reproduce all sequential SHA-256 hashes between genesis and the current window — a wall-clock equivalent to the entire chain age. The work is non-parallelizable across windows by construction (each window's input is the previous window's output). At any chain age beyond the entry window, the cost of long-range rewrite exceeds the cost of forward extension by the same adversary by a factor equal to chain age divided by entry threshold (`active_chain_length / ssha_entry_windows`).

**Side-channel resistance.** All ML-DSA-65 implementations used in Montana operate in constant time with respect to secret keys, per FIPS 204 §3.7 implementation guidance. Constant-time discipline is verified by the reference implementation's audit checklist (`Code/docs/security-cards.md`) and is part of the dependency capability requirement [C-5] for the cryptographic library selection in the reference implementation. Memory containing private signing material is allocated through `mlock`-protected pages and cleared via `zeroize` on drop.

**Transport-layer adversary.** The transport handshake is Noise_PQ XX (Section 13), a three-message protocol with ephemeral ML-KEM-768 key encapsulation on both sides, ML-DSA-65 identity signatures over the full transcript, and ChaCha20-Poly1305 AEAD on the established session. An adversary recording transcripts cannot derive the session keys without solving Module-LWE for ML-KEM-768. An active man-in-the-middle cannot substitute identities without forging ML-DSA-65 signatures over a transcript that binds the ephemeral keys, the ML-KEM-768 ciphertexts, and the claimed identity public keys.

This threat model is the basis for the security claims in subsequent sections. Properties beyond this model — including any privacy property beyond explicit content encryption — are out of scope for the protocol layer.


## 6. Operations and the Account Table

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

Total AccountRecord size is 2 059 bytes including all fields. Operations transform state through `apply_proposal(state, proposal) → state'`. The transformation is deterministic, byte-exact reproducible by any node from the same `(state, proposal)` pair. The protocol defines a closed set of operation classes — value transfer, key change, content anchoring, node registration, and account closure — each with a fixed canonical encoding, validation rule, and apply function specified normatively in the protocol specification.

Each account chain is restricted to one operation per τ_1 window. Two operations with the same `prev_hash` from the same sender constitute equivocation; both operations are marked equivocated and neither is cemented. The intra-window ordering problem is eliminated by construction at N = 1, removing both subjective ordering surface (violation of [I-3] determinism) and additional consensus seed surface ([I-8]).

Conservation invariants hold per operation: the sum of balance deltas across all affected records equals the emission delta plus the burn delta. No operation creates or destroys value silently.


## 7. Lottery

The operator who completes the chain for window `W` is selected by a deterministic lottery from the set of registered operators. Each operator submits a `SshaReveal` with the window's chain output and a signature; the lottery winner is

```
winner = argmin_{operator}  ticket(operator, W)
```

where

```
ticket(operator, W) = SHA-256(
    "mt-lottery"                      // domain separator (registry entry mt-lottery)
    || T_W                            // TimeChain canonical value at window W
    || cemented_bundle_aggregate(W-2) // network-bound unpredictability source
    || operator.node_id
    || W                              // u64-LE window index
)
```

The `cemented_bundle_aggregate(W-2)` term is the lottery's network-bound unpredictability source. It incorporates signatures from honest operators in window `W-2` — values that an attacker cannot precompute without the private keys of honest participants. This closes the class of grinding attacks where an adversary with hardware advantage precomputes future chain outputs and grinds attacker-chosen fields against them. The full normative formulation, including the integer-form derivation of the lottery ranking that selects the argmin ticket among cemented `SshaReveal`s weighted by operator chain-length, is in the protocol specification («Lottery» section, domain `mt-lottery` in the Domain separators registry).

The grinding attack proceeds as follows. An adversary with k times the SHA-256 throughput of a commodity operator can precompute k hours of chain output in one hour of wall-clock. If the lottery seed were `H(node_id || T_W || W)` with `T_W` predictable offline, the adversary could generate many candidate keypairs, compute their tickets against precomputed `T_W`, and select the keypair giving the lowest ticket for each future window. This grants disproportionate consensus share for fixed hardware. The `cemented_bundle_aggregate(W-2)` component blocks this attack: the adversary cannot precompute the aggregate without forging honest signatures, which the lattice-based ML-DSA-65 scheme prevents at NIST security level 3 (192-bit quantum-equivalent strength). The grinding horizon collapses to already-cemented windows, where attacker-chosen fields are frozen by the registered `node_id` committed at registration. This is the protocol's invariant [I-8], network-bound unpredictability of consensus seeds.


## 8. Liveness

The chain extends as long as quorum signatures are collected. The mechanism is the fallback cascade, defined in the protocol specification.

The canonical proposer of window `W` is `winner_{W-2}`, the operator whose ticket achieved `argmin(weighted_ticket)` in window `W-2`. If this proposer is offline or submits an invalid proposal, the role passes to `fallback_1 = second_min(weighted_ticket)`, then `fallback_2 = third_min`, and so on. The cascade is canonical — every honest operator computes the same ordering of fallbacks from the cemented set of window `W-2`.

Quorum threshold for `BundledConfirmation` is 67% of `active_chain_length`. With more than 67% honest active_chain_length, quorum is reached at the canonical proposer or one of the early fallbacks, and the proposal is cemented within the window.

The cascade is depth-bounded by `fallback_depth = 255`. If 255 successive fallbacks fail to produce quorum, the protocol halts by liveness — not by safety. Safety is preserved (no invalid proposal is cemented); only progress stops until honest participation exceeds 67% again.

Full mechanism, including BundledConfirmation construction, signature aggregation, and the participation_ratio feedback into D recalibration, is described in the protocol specification.


## 9. Incentive and Bootstrap

The lottery winner of each window receives 13 base units of Ɉ (`13 × 10^9 nɈ`), credited to the winner's operator account at the following window's settlement. There are no transaction fees, no second-tier inflation, no premine, no presale, no founder allocation. Windows are 0-indexed; the genesis window emits nothing, so the first reward is settled at window 1. The total supply grows linearly with the window count; the exact closed-form is defined normatively in the protocol specification.

Storage of accumulated value is not separately incentivized. The protocol does not pay for holding tokens. The single reward path is operating the chain for a window and winning that window's lottery.

For any operator, the expected income per unit time depends on the share of cemented `SshaReveal`s contributed by that operator across windows. With `N` operators of equal computational power running honest sequential hashing, the expected reward per operator per window is `13/N` Ɉ. With unequal power, the share is proportional to the number of valid `SshaReveal`s submitted in time.

**Bootstrap economics.** At small `N`, the per-operator reward is large: a single operator receives all `13` Ɉ per window, two operators split as `6.5` each. As `N` grows, individual share dilutes toward the asymptotic `13/N`. This creates an incentive for early entry while the network is small and a corresponding equilibrium where new entry becomes marginal as `13/N` approaches the operational cost.

The bootstrap dynamic is the per-window reward diluting from `13` toward `13/N` as operators enter; the equilibrium population is set by the point at which `13/N` meets per-operator operating cost.


## 10. Anti-Spam Without Fees

Spam protection is the composition of three time-based mechanisms:

**Per-identity rate.** Operations of class A (Transfer, NicknameBid, etc.) are limited to one per account per window τ_1 = 1 window. An attacker with `N` Sybil identities can perform at most `N` operations per window, but each Sybil identity has its own creation cost (Section 3). The rate is uniform across identities; there is no fast lane.

**Chain-length threshold.** Privileged operations (NodeRegistration, NicknameBid) require the operating account's `account_chain_length` to exceed a threshold `k`. An account must be active for at least `k` windows before issuing such an operation. The threshold cannot be purchased; it can only be earned by elapsed activity.

**Seniority gating.** Lottery weight in the operator selection scales with the operator's `account_chain_length` up to a saturation point. New operators have lower weight; they accrue weight by participating across windows. This dampens flash-mob attacks where many adversarial operators register simultaneously.

These mechanisms together close DoS without monetary barriers. The protocol contains no `fee` field on any operation.


## 11. State Lifecycle and Scaling

Every persistent record in consensus state has either a cost-based barrier, a lifecycle bound, or a hard quota. Accounts whose balance has reached zero and whose `last_active_window` precedes the current window by more than `4 × 20 160` windows (`4 × τ₂`, the protocol parameter `pruning_idle_windows` for AccountRecord pruning) are removed by `apply_candidate_expiry` at the next epoch boundary.

Pruning is not optional; it is part of the canonical state transition. Two honest nodes following the protocol prune identically. The Account Table size is bounded above by

```
|AccountTable(W)| ≤ creation_rate × retention_window
```

which is independent of accumulated wall-clock time, ensuring that long-running chains do not produce unbounded state.

**Scaling to one billion accounts.** AccountRecord is 2 059 bytes. State at 1 × 10^9 active accounts is approximately 2.06 TB, holdable on commodity disks. NodeRecord at 4 034 bytes adds a smaller term proportional to operator count (typically much less than 1% of account count). State growth is dominated by the active account population, bounded by the pruning rule above.

Fast synchronization of new operators against a 2 TB state is supported by snapshot-based sync rooted in the current Merkle commitment, with state delivery in independently verifiable chunks. Synchronization is verify-only: a joining operator validates each chunk against the Merkle root and trusts the source peer for liveness, not for safety.

State size is not unbounded by time. The pruning rule guarantees that state grows with active population, not with chain age.


## 12. Privacy

The protocol exposes balances, transfers, account graphs, and operator identities by default. Application-layer privacy is achieved through Anchor objects: an account commits a 32-byte hash to chain, and the contents (encrypted under the owner's key) are held off-chain by the owner or by a delegated peer. The Anchor mechanism does not give the protocol visibility into the contents.

Privacy is a user choice rather than a protocol-imposed feature. Mass-surveillance through privacy-by-protocol is not within scope; selective privacy through user-managed encryption is. This boundary aligns the protocol with regulatory frameworks (FATF, MiCA) that have rejected protocol-level privacy mixers while accepting end-user encryption of off-chain content.

Beyond content privacy, the protocol does not claim metadata anonymity. The network layer observes operation timing, counts, and operator identity. Operators concerned with metadata exposure run their own nodes and avoid third-party hosting. Tier-1 (self-hosted) and Tier-2 (Noise_PQ tunneled) deployments are differentiated in the Network specification.


## 13. Network and Transport Security

The protocol's wire format and synchronization mechanism are described in [`mt-net`](https://github.com/efir369999/Montana/tree/main/Code/crates/mt-net) and [`mt-net-transport`](https://github.com/efir369999/Montana/tree/main/Code/crates/mt-net-transport) of the reference implementation. Operators discover peers, exchange `SshaReveal` and `BundledConfirmation` messages, and replicate the cemented chain through libp2p over a `TCP → Noise_PQ XX → Yamux` stack.

Transport security is Noise_PQ XX. Each handshake derives ephemeral ML-KEM-768 keypairs on both sides; the responder encapsulates to the initiator's ephemeral public key and the initiator encapsulates to the responder's, producing two FIPS-203 shared secrets. Both sides transmit their ML-DSA-65 identity public keys and sign the transcript with FIPS-204 signatures. Session keys are derived by domain-separated SHA-256 over the concatenation of the two shared secrets and the transcript; the established session is an AEAD-encrypted byte stream under ChaCha20-Poly1305. Consensus signatures (ML-DSA-65) are independent of transport and verified separately.

The Noise_PQ XX handshake is a 3-message protocol with ephemeral ML-KEM-768 keypairs on both sides (so the initiator does not need to know the responder's static KEM public key a priori, a property required for libp2p plug-in into the auth-upgrade slot). Wire format: msg1 1184 bytes (initiator ephemeral KEM pk); msg2 7533 bytes (responder ephemeral KEM pk, KEM ciphertext to initiator, responder ML-DSA-65 identity pk, responder signature over transcript); msg3 6349 bytes (KEM ciphertext to responder, initiator ML-DSA-65 identity pk, initiator signature over transcript). Session keys are derived by SHA-256 with domain separators over the concatenation of the two shared secrets and the transcript; the resulting AEAD-encrypted byte stream is exposed to the application as `mt_noise_pq::stream::NoisePqStream`. PeerId is derived as the SHA-256 multihash of each peer's ML-DSA-65 identity public key (sha2-256 multihash code 0x12, the libp2p / IPFS standard for peer identifiers). The cryptographic identity used in consensus and the routing identity used by libp2p are bound to the same key material. Wire format, KAT vectors, and capability negotiation are normatively specified in the Network specification.

**Security properties of Noise_PQ XX.** Forward secrecy is provided by ephemeral ML-KEM-768 keypairs discarded after the handshake. Identity authenticity is provided by ML-DSA-65 signatures over the full transcript, which binds the ephemeral public keys, the ML-KEM-768 ciphertexts, and the identity public keys of both sides. An active man-in-the-middle who substitutes either side's identity must produce a valid ML-DSA-65 signature over a transcript that includes the substituted identity — equivalent to a EUF-CMA forgery on ML-DSA-65, which the underlying Module-LWE assumption precludes at NIST level 3 (192-bit quantum-equivalent strength). A passive eavesdropper recording the handshake cannot derive the session keys without solving the corresponding Module-LWE instances for both ML-KEM-768 ciphertexts. ChaCha20-Poly1305 (RFC 8439 [10]) provides confidentiality and integrity for the established session under standard pseudo-random-permutation and universal-hash assumptions. Replay of any handshake message is prevented by binding the entire transcript into both identity signatures.

A new node synchronizes by acquiring the current TimeChain head from any honest peer, verifying the chain locally, and replicating the Account Table snapshot rooted in the current Merkle commitment. Synchronization is verify-only; trust in the source peer is required only for liveness, not for safety.


## 14. Calculations

We consider a scenario where an honest operator and an attacker compete to win windows. The probability that the attacker wins a given window is proportional to the attacker's share of total registered operator computational power. With attacker share `p` and honest share `1 − p`, the probability that the attacker wins `k` consecutive windows is `p^k`, decreasing geometrically.

For the lottery to be biased in favor of the attacker, the attacker must control more than half of all registered operator power. With per-operator wall-clock chain advancement being constant (no parallelization within a single chain), Sybil identity multiplication does not increase total power; it only fragments the same power across more identities. The attacker's share is bounded by the number of physical machines they operate, multiplied by the per-machine SHA-256 throughput.

```
P(attacker wins k consecutive windows) = p^k
```

For `p = 0.3` and `k = 10`, P = 5.9 × 10^-6. This is the probability of consecutive single-window wins; it does not correspond directly to chain reorganization probability in the Bitcoin sense, because Montana's cementing rule requires honest quorum signature on `BundledConfirmation` for any operation to take effect (Section 8). A run of consecutive lottery wins by an adversary does not allow the adversary to cement adversarial operations without 67% honest active_chain_length signatures — the lottery selects the proposer, but the proposal still requires quorum.

This is the security argument: monetary capital does not buy more time. The operator economy reduces to a hardware economy in which the unit good (one chain window) is uniformly priced in joules.


## 15. Conclusion

We have proposed a blockchain whose consensus security rests on post-quantum cryptographic primitives and whose anti-spam mechanism operates on time rather than fees. The construction does not require trusted setup, does not require a price feed, and does not impose a monetary barrier on participation. The architecture is designed to scale to billions of active accounts on commodity-disk hardware.

The reference implementation in Rust is available at the cited URL under permissive license (Apache-2.0 / MIT). The multi-node deployment expands as operators self-admit and the population grows from the empty genesis window.


## References

[1] S. Nakamoto, "Bitcoin: A Peer-to-Peer Electronic Cash System," 2008.

[2] National Institute of Standards and Technology, "Module-Lattice-Based Key Encapsulation Mechanism Standard," FIPS 203, 2024.

[3] National Institute of Standards and Technology, "Module-Lattice-Based Digital Signature Standard," FIPS 204, 2024.

[4] National Institute of Standards and Technology, "Secure Hash Standard (SHS)," FIPS 180-4, 2015.

[5] D. Boneh, J. Bonneau, B. Bünz, B. Fisch, "Verifiable Delay Functions," CRYPTO 2018. Cited as related work with O(log T) verification on RSA groups; Montana's sequential delay function adopts a simpler primitive without efficient verification, per invariant [I-7] minimal cryptographic surface.

[6] K. Pietrzak, "Simple Verifiable Delay Functions," ITCS 2019. Related work, class groups of imaginary quadratic fields.

[7] B. Wesolowski, "Efficient verifiable delay functions," EUROCRYPT 2019. Related work, RSA groups with O(1) verification proof.

[8] P. W. Shor, "Polynomial-Time Algorithms for Prime Factorization and Discrete Logarithms on a Quantum Computer," SIAM Journal on Computing, 1997.

[9] L. K. Grover, "A fast quantum mechanical algorithm for database search," STOC 1996.

[10] Y. Nir, A. Langley, "ChaCha20 and Poly1305 for IETF Protocols," IETF RFC 8439, 2018.

[11] T. Perrin, "The Noise Protocol Framework," Revision 34, 2018. Specification of Noise handshake patterns including the XX pattern adopted here with ML-KEM-768 substituted for the Diffie-Hellman component.

[12] E. Heilman, A. Kendler, A. Zohar, S. Goldberg, "Eclipse Attacks on Bitcoin's Peer-to-Peer Network," USENIX Security 2015.

[13] G. Marcus, E. Heilman, S. Goldberg, "Low-Resource Eclipse Attacks on Ethereum's Peer-to-Peer Network," IACR ePrint 2018.

[14] National Institute of Standards and Technology, "Post-Quantum Cryptography Standardization Process: Round 3 Finalists," 2022.


---

Alejandro Montana
