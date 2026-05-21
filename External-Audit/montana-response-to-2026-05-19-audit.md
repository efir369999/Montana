# Montana Response to CISO-as-a-Service Consolidated Security Review (2026-05-19)

**Date:** 2026-05-20
**Author:** Alejandro Montana
**Responding to:** `montana-security-review-consolidated-2026-05-19-en.md` (CISO-as-a-Service Team)
**Spec basis at time of response:** Montana Protocol v35.25.0, Network v1.0.0, App v3.12.0, Whitepaper Montana.md

This document is the Montana project's formal response to the 19 May 2026 consolidated security review. Findings are addressed by category: accepted with fix, defended with spec citation, documented pre-mainnet state, and open scope for the academic Path B.

---

## Summary of disposition

| ID | Audit severity | Verified severity | Disposition |
|----|----------------|-------------------|-------------|
| WP-1 VDF terminology | Critical | Critical | Accept and fix |
| WP-2 PQ claim partially incorrect | Critical | Critical | Accept and fix |
| WP-3 Sybil math | Critical | Critical | Accept and fix |
| WP-4 Missing threat model | High | High | Accept and fix |
| WP-5 Missing liveness proof | High | Medium (paper gap only) | Accept and fix |
| WP-6 1B without scaling | High | High | Accept and fix |
| WP-7 Bootstrap economics | High | High | Accept and fix |
| WP-8 Hardware asymmetry | High | Medium | Partial — defend "does not explain" claim, expand depth |
| WP-9 Misleading comparative calculation | Medium | Medium | Accept and fix |
| WP-10 Grover analysis incomplete | Medium | Low | Accept and fix |
| WP-11 FIPS reference inconsistency | Medium | Medium | Accept and fix |
| WP-12 Mnemonic formalization | Low | Low | Accept and fix |
| MONT-001 ML-DSA-65 side-channel | High | Low | Accept and fix (spec patch) |
| MONT-002 IBT replay window | Medium | Low | Accept and fix (spec patch) |
| MONT-003 apply_proposal race | High | **Rejected — false finding** | Defend with spec citations |
| MONT-004 Singleton phase | Critical | Informational | Documented pre-mainnet state |

---

## 1. Accepted critical findings (whitepaper rewrite)

The following four critical findings are accepted in full. They are addressed in the whitepaper revision committed alongside this response.

### 1.1 WP-1 — VDF terminology

**Accepted.** Cited works (Boneh et al. CRYPTO 2018; Pietrzak ITCS 2019; Wesolowski EUROCRYPT 2019) define verifiable delay functions as having sublinear verification — O(log T) or O(1) — typically via RSA groups or class groups of imaginary quadratic fields. Montana's primitive is iterated SHA-256 with verification cost O(D) per window, equal to computation cost. This is a sequential delay function, not a VDF in the sense of the cited literature.

Whitepaper change:
- Terminology shifted to "sequential delay function over SHA-256" with explicit acknowledgment that verification cost equals computation cost.
- References to Boneh/Pietrzak/Wesolowski reframed as related work with sublinear verification, with explicit reasoning under invariant [I-7] (minimal cryptographic surface) for why Montana adopts the simpler primitive at the cost of asymmetric verification.

### 1.2 WP-2 — Post-quantum claim partially incorrect

**Accepted and closed.** Abstract phrasing "security rests entirely on post-quantum primitives" had overstated coverage. At the time of the review, the transport layer used TLS 1.3 with classical ECDHE (X25519), which is broken by Shor's algorithm and was vulnerable to store-now-decrypt-later attacks on transport confidentiality (consensus integrity was not affected because consensus signatures already used ML-DSA-65).

Closure (2026-05-21):
- Production transport switched from `(libp2p::tls::Config + libp2p::noise::Config)` to **Noise_PQ XX** (ML-KEM-768 ephemeral KEM on both sides of the handshake, ML-DSA-65 identity signatures over the transcript, ChaCha20-Poly1305 AEAD on the established session).
- Transport stack is now `TCP → Noise_PQ XX → Yamux`. The classical TLS 1.3 + Noise XK chain has been removed.
- PeerId is now derived from each peer's ML-DSA-65 identity public key via SHA-256 multihash (libp2p / IPFS sha2-256 multihash code 0x12); the cryptographic identity used in consensus and the routing identity used by libp2p are bound to the same key material.
- Deployed on the three-node Genesis cohort (Moscow, Frankfurt, Helsinki); the full 6/6 pairwise mesh negotiates `/montana/noise-pq-xx/1.0.0` and exchanges heartbeats over the post-quantum AEAD stream.
- Code: `crates/mt-noise-pq/src/xx_handshake.rs` (handshake state machine), `crates/mt-net-transport/src/xx_noise_pq_upgrade.rs` (`NoisePqXxConfig` implementing both `InboundConnectionUpgrade` and `OutboundConnectionUpgrade`), `crates/mt-net-transport/src/transport.rs` (production wire-up). Tracker: `Code/docs/SPEC_DEVIATIONS.md` DEV-014 (closed).

Whitepaper change:
- Abstract rewritten to state Noise_PQ XX as the production transport.
- Section 13 (Network and Transport Security) rewritten in factual present tense; classical TLS 1.3 + Noise XK described as historical only.

### 1.3 WP-3 — Sybil analysis math

**Accepted.** The sentence "producing N false identities requires N candidate chains, each consuming N times the wall-clock time" is arithmetically incorrect. Each candidate VDF chain has a fixed wall-clock cost T regardless of N. Total cost scales linearly in hardware (N machines compute N chains in parallel at wall-clock T with N × T machine-hours) or linearly in time (one machine computes N chains sequentially at N × T wall-clock). The product N × N does not arise.

Whitepaper change:
- Section 3 rewritten to state the correct linear scaling and to clarify that Sybil resistance derives from hardware cost combined with per-identity protocol rate limits, not from time-non-parallelizability across identities.

### 1.4 WP-4 — Missing threat model

**Accepted.** An explicit threat model section is added to the whitepaper.

Content:
- Attacker classes: profit-seeking, sabotage, state-level adversary.
- Honest-majority assumption: > 67% of total active_chain_length controlled by honest operators (quorum-weighted, not headcount).
- Hardware-bounded influence: attacker advantage scales linearly with hardware budget, not with token holdings.
- Failure conditions: when adversary controls > 50% persistent VDF compute and ChainLength-weighted operator share simultaneously.
- Out of scope: application-layer metadata anonymity beyond Anchor encryption (the network observes operation timing and counts even when content is encrypted). Transport-layer confidentiality against quantum adversaries is in scope and is closed by Noise_PQ XX as the production handshake.

---

## 2. Accepted high and medium findings

### 2.1 WP-5 — Missing liveness proof in whitepaper

**Accepted as paper gap.** The whitepaper does not currently surface the liveness mechanism. The protocol specification has full machinery:
- Fallback cascade `fallback_1 = second_min(weighted_ticket)`, `fallback_2 = third_min`, ..., up to `fallback_depth = 255`.
- Halt-by-liveness condition: `fallback_depth = 255` without quorum is documented as a liveness halt, not a safety failure (spec line 1981).
- Quorum requirement: > 67% active_chain_length signatures on BundledConfirmation.

Whitepaper change:
- Section added referencing the fallback cascade, quorum threshold, and halt-by-liveness condition with explicit pointer to the protocol specification for the full mechanism.

Severity assessment: this is a paper-side gap, not a design gap. The mechanism exists and is well-specified.

### 2.2 WP-6 — 1B-user claim without scaling analysis

**Accepted.** AccountRecord is 2 059 bytes (Storage Card, spec line 349). State at 1 billion active accounts is approximately 2.06 TB, holdable on commodity disks but requiring fast-sync via Merkle snapshot (milestone M7).

Whitepaper change:
- Section 9 expanded with a scaling table: AccountRecord 2 059 B, state at 1 × 10^9 active ≈ 2.06 TB, fast-sync via snapshot scheduled for M7 with measurable benchmarks for completion criteria.
- "Baseline architectural target" framing retained but qualified by quantitative bounds.

### 2.3 WP-7 — Bootstrap economics not analyzed

**Accepted in part.** Whitepaper currently states reward per operator per window as 13 / N Ɉ without analysis of bootstrap incentives at small N or rational-delay equilibria.

Whitepaper change:
- Section 7 expanded with a bootstrap economics paragraph: initial operators receive full per-window emission of 13 Ɉ when N is small; reward dilution at growth approaches the asymptotic 13 / N share at large N.
- Open research question explicitly marked: formal Nash equilibrium analysis excluding rational-delay strategies is deferred to milestone M9+ academic paper (Path B).

### 2.4 WP-8 — Hardware asymmetry

**Partial accept.** The audit's recommendation to refine the hardware-asymmetry analysis is reasonable. However, the audit's negative claim that the whitepaper "does not explain this component" (referring to `cemented_bundle_aggregate(r-2)`) is factually incorrect.

Whitepaper Section 6 (lines 90-93) reads:

> ticket(operator, r) = SHA-256(operator.node_id || cemented_bundle_aggregate(r-2) || r)
>
> The cemented_bundle_aggregate(r-2) term is the lottery's network-bound unpredictability source: it incorporates signatures from honest operators in window r-2, which an attacker cannot precompute without privkeys held by honest participants. This closes the class of attacks where an adversary with hardware advantage precomputes future windows and grinds attacker-chosen fields against them.

The component is named, the formula given, and the closure rationale explicit. The audit observation that the depth of analysis is shallow is fair; the observation that the mechanism is unexplained is not.

Whitepaper change:
- Section 6 expanded by approximately half a paragraph to give the grinding precomputation attack explicitly and the cemented_bundle_aggregate closure with NIST level 3 security target.
- Audit claim "does not explain" formally rejected via citation.

### 2.5 MONT-001 — ML-DSA-65 constant-time requirement

**Accepted.** The protocol specification at line 3834 contains the generic requirement "audited libraries with constant-time guarantees and published test vectors". This generic requirement is not explicitly bound to ML-DSA-65 in the row of the cryptographic primitives table at line 3839. The audit recommendation to make the requirement explicit at the primitive row is correct.

Spec change:
- Crypto primitives table row for ML-DSA-65 amended to require constant-time implementation explicitly per FIPS 140-3 side-channel guidance.

### 2.6 MONT-002 — IBT proof replay window for online

**Accepted as defense-in-depth.** Network specification line 70 establishes replay protection through two mechanisms simultaneously: server_node_id binding restricts replay to a specific recipient, and the 2-window slot bounds the replay horizon to approximately 120 seconds. The audit recommendation to add per-nonce tracking analogous to the mesh IBT proof tracking is a defense-in-depth improvement, not a closure of a structural weakness.

Spec change:
- Online IBT handshake specification amended to add nonce tracking with pruning at 2-window horizon, mirroring the existing mesh IBT nonce tracking design.

### 2.7 WP-9 to WP-12 — minor findings

**Accepted.** Each addressed in the whitepaper revision: comparative calculation in Section 12 rephrased to avoid implying analogy with Bitcoin's PoW security; Grover analysis expanded to address collision resistance separately from preimage resistance; FIPS 205 reference removed from references list as SLH-DSA-SHA2 is not currently used; mnemonic wordlist formalization committed to milestone M9 (`Montana wordlist.txt` file is already maintained in the repository).

---

## 3. Rejected finding (defended with spec citation)

### 3.1 MONT-003 — apply_proposal race condition

**Rejected.** Citations below are translated from Russian (the protocol specification is currently maintained in Russian; English translation is in progress). Line numbers refer to `Montana Protocol v35.25.1.md`.

The audit asserts:

> "The specification describes apply_proposal as deterministic, but does not explicitly specify the order for multiple operations from the same sender within a window. An attacker can submit two valid operations with the same prev_hash (frontier_hash). The specification does not clarify how conflicts within the same proposal are resolved."

This claim is contradicted by explicit specification text. Citations from Montana Protocol v35.25.0:

**Line 1141 (translated from Russian):**
> Each account has one chain. Two operations with the same prev_hash = equivocation.

**Line 1147 (translated from Russian):**
> A node receives operation X with prev_hash = H. The node has already seen operation Y with prev_hash = H, Y ≠ X. A fork is detected. Both operations are marked as equivocated.

**Line 1186 (translated from Russian):**
> Each account chain: 1 operation per τ₁.

**Line 1194 (translated from Russian):**
> Dependency rule. The operation of an account in a window references frontier_hash from the settled state of the previous window. N > 1 operations of one account in one window would require intra-window ordering — either subjective (mempool-dependent, violation of [I-3]), or canonical hash composition (expansion of [I-8] surface). With N = 1, the problem is absent: the order of the operation is unique.

**Line 1198 (translated from Russian):**
> Binary resolution of double-spend. The rule "67% active_chain_length for one operation per one prev_hash" works because the conflict is binary: either A, or B. N > 1 operations per window makes the conflict multi-way and requires an additional mechanism for choosing between three or more branches per window — a liveness blocker and new attack surface.

The specification therefore:
1. Limits each sender to exactly one operation per τ_1 by design.
2. Defines two operations with the same prev_hash as equivocation, not as concurrent valid operations.
3. Provides explicit conflict resolution: equivocation marks both operations as equivocated; the binary double-spend resolution rule applies.
4. Justifies the N = 1 design as required by global invariant [I-3] (deterministic consensus state) and [I-8] (network-bound unpredictability of consensus seeds).

The asserted race condition does not exist within the specification's actual rule set. The finding is rejected.

### 3.2 Sub-claim of WP-8

**Rejected.** As cited above (Section 2.4), the whitepaper does explain `cemented_bundle_aggregate(r-2)` directly in Section 6 lines 90-93. The audit's claim that the component is unexplained is factually incorrect.

The depth of the explanation is acknowledged as shallow and is being expanded in the whitepaper revision (Section 2.4 above). The negative claim itself is rejected.

---

## 4. Documented pre-mainnet state

### 4.1 MONT-004 — Singleton phase without network consensus

**Documented, severity downgraded to Informational.** The reference implementation operates in milestone M5 (singleton mode) at the time of the audit. This is the documented pre-mainnet state, published in the project README and tracked under nine SPEC_DEVIATIONS entries (DEV-001 to DEV-009 in `docs/SPEC_DEVIATIONS.md`).

The CVSS:3.1 score of 10.0 assigned by the audit is inflated. CVSS scores presuppose deployed systems where the security properties are claimed to be active. Montana's M5 singleton is a development phase with explicitly disclaimed BFT/Sybil/lottery security properties — these properties activate at milestone M6 when the network layer is integrated. Assigning CVSS 10.0 to a system that does not claim those properties at this milestone is methodologically incorrect.

Pre-mainnet status is documented at:
- Project README — "Status: pre-mainnet, M5 singleton phase".
- `docs/SPEC_DEVIATIONS.md` — full enumeration of nine known deviations.
- Whitepaper Section 13 — "M6 multi-node deployment, M7 snapshot-based fast synchronization, M9 conformance suite to second implementations".

No further action beyond clearer surface-level communication in the whitepaper revision is taken.

---

## 5. Open scope for Path B (academic paper, M9+)

The audit recommends two paths: Path A (focused 6-8 page whitepaper) and Path B (full academic paper 20-30 pages). Montana selects Path A for the present Metzdowd-readiness rewrite and reserves Path B for the M9 milestone when the conformance suite to second implementations is available.

Items deferred to Path B:
- Empirical scaling analysis at 1 billion active accounts with benchmarks for fast-sync duration, TPS capacity, and merkle proof verification times.
- Formal liveness proof in the style of distributed-systems publications (PODC, DISC, OPODIS).
- Bootstrap Nash equilibrium analysis with simulation and exclusion of rational-delay strategies.
- Security reduction for the lottery ticket function to a grinding-resistance model under hardware asymmetry assumptions.
- Comparison table with Algorand, Tendermint, Cosmos, Solana on the dimensions of BFT properties, post-quantum coverage, anti-spam mechanism, scaling profile, and state lifecycle.

These items are research-grade work appropriate for a journal or conference submission, not for a list publication.

---

## 6. Acknowledgment

The Montana project thanks the CISO-as-a-Service Team for the thorough review. The critical findings on terminology and abstract precision (WP-1, WP-2, WP-3) are valuable and would have torpedoed publication if left unaddressed. The methodology of separating Adversarial Protocol Design Review from Technical Vulnerability Analysis is sound and recommended for future external reviews of the project.

The two false claims (MONT-003 race condition, WP-8 sub-claim "does not explain") are noted as procedural reminders that audit reviewers should verify negative claims through reading the relevant specification sections, not through keyword grep. The project's own internal architect role (`Montana-Protocol/CLAUDE.md` (Russian) Gate −1, step 5) formalizes this discipline; the same discipline applies to external review verification.

Path A whitepaper revision is committed alongside this response. Spec patches MONT-001 (constant-time) and MONT-002 (online IBT nonce tracking) are committed separately. DEV-014 (post-quantum transport migration) was closed on 2026-05-21 by switching the production transport stack to Noise_PQ XX; see the WP-2 disposition above for details and code locations. The project is ready for Metzdowd Cryptography List submission and welcomes a follow-up review focused on mainnet readiness.

Alejandro Montana
