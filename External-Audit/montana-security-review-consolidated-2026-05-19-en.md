# Consolidated Security Review Report: Montana Protocol

**Date:** 2026-05-19  
**Created by:** CISO-as-a-Service Team (Adversarial Protocol Design Review + Technical Vulnerability Analysis)  
**Addressed to:** Alejandro Montana (Project Author) and technically proficient reviewers  
**Analyzed Artifacts:** Whitepaper Montana.md, Protocol v35.25.0, Network v1.0.0, App v3.12.0, CRITIC.md, CLAUDE.md, README.md  
**Predecessor Reviews:** Adversarial Protocol Design Review (2026-05-19), Technical Vulnerability Analysis (2026-05-19)

---

## Executive Summary

The Montana Protocol is an ambitious post-quantum blockchain project with an unusually mature specification apparatus -- 4,416 lines of protocol specification, 22 critic passes, 15 architecture gates. The protocol specification itself is exceptionally well thought-out for a pre-mainnet project. The whitepaper, however, is in its current form **not ready for publication** on the Metzdowd Cryptography List.

The review identifies three critical technical errors in the whitepaper (incorrect SSHA terminology, partially incorrect post-quantum claim, mathematically inconsistent Sybil analysis), several high-severity deficiencies (missing threat model, missing liveness proof, unsubstantiated 1B claim, bootstrap economics unanalyzed), as well as four critical and nine high findings at the implementation level. The central problem: The whitepaper does not correctly represent the actual maturity level of the technical specifications -- the specifications are significantly more mature than the project's public face.

---

## 1. Critical Findings

*All findings with severity "critical" or "high", consolidated and deduplicated from both review streams.*

### 1.1 Incorrect SSHA Terminology (Whitepaper)

**Severity:** Critical  
**Source:** Adversarial Protocol Design Review

The whitepaper claims that SHA-256 sequential hashing is a "Verifiable Delay Function" and cites Boneh et al., Pietrzak, and Wesolowski for this. This is technically incorrect.

SSHAs in the sense of the cited literature have the property that verification of an output is substantially faster than computation -- typically O(log T) instead of O(T). Montana has sequential SHA-256 hashing: verification costs O(T), every node must perform all D = 325,000,000 iterations itself. This is sequential proof-of-work, not a SSHA.

Cryptographers on the Metzdowd list will recognize this immediately. This undermines the credibility of the entire paper.

**Recommendation:** Correct the terminology ("sequential hash chain" instead of "SSHA") and remove the incorrect references. The protocol specification itself uses more correct language -- the whitepaper must catch up.

---

### 1.2 Post-Quantum Claim Partially Incorrect (Whitepaper)

**Severity:** Critical  
**Source:** Adversarial Protocol Design Review

The whitepaper claims in the abstract: *"security rests entirely on post-quantum primitives"*. The network specification explicitly refutes this:

> "Quantum store-now-decrypt-later: ❌ TLS handshake X25519 vulnerable" (Tier 1, Privacy table)

TLS 1.3 with X25519 uses classical ECDH -- breakable by Shor's algorithm. A quantum attacker who records network traffic today can decrypt it once a sufficiently large quantum computer becomes available.

The post-quantum claim applies to the consensus layer, not to the network layer. This is a substantial distinction that is not communicated in the whitepaper.

**Recommendation:** Be precise: "consensus layer security rests on post-quantum primitives; network transport layer uses TLS 1.3 (Tier 2 with Noise_PQ available for full PQ protection)".

---

### 1.3 Mathematically Inconsistent Sybil Analysis (Whitepaper)

**Severity:** Critical  
**Source:** Adversarial Protocol Design Review

Quote from the whitepaper: *"producing N false identities requires N candidate chains, each consuming N times the wall-clock time"*

This is wrong. N chains cost N × (one chain), not N × N × (one chain). Additionally: An attacker with N servers can compute N chains in parallel. Wall-clock time remains constant at approximately 10 hours -- only energy expenditure scales linearly. The whitepaper implies quadratic costs; the actual costs are linear in the number of machines.

**Recommendation:** Correct formulation: "producing N false identities requires N sequential hash chains, each consuming approximately 10 hours of wall-clock time on commodity hardware. An attacker with N machines can compute these chains in parallel, so the wall-clock cost is constant but the hardware cost scales linearly with N."

---

### 1.4 Singleton Phase Without Network Consensus (Implementation)

**Severity:** Critical  
**Source:** Technical Vulnerability Analysis (MONT-004)  
**CVSS:3.1:** AV:N/AC:L/PR:N/UI:N/S:C/C:H/I:H/A:H = 10.0

The current implementation runs in M5 singleton mode without real network consensus. All security properties of the protocol -- Byzantine Fault Tolerance, Sybil protection, lottery fairness -- are not active in this mode. The protocol has not yet proven any of its security properties in a multi-node setting.

This is not a design flaw, but the documented pre-mainnet state (9 SPEC_DEVIATIONS in the README). However, it is the most critical finding for any security review: The security promises of the whitepaper have not yet been empirically validated.

**Recommendation:** Clearly communicate in the whitepaper and in public communication that the project is in the pre-mainnet stage and that the security properties have not yet been proven in multi-node operation.

---

## 2. High Findings

### 2.1 Missing Threat Model (Whitepaper)

**Severity:** High  
**Source:** Adversarial Protocol Design Review

For publication on the Metzdowd Cryptography List, an explicit threat model is mandatory. The whitepaper has none. Missing elements:

- Who are the attackers? (Profit-seeking, sabotage, state-sponsored)
- What assumptions are made? (Honest majority? Which majority?)
- What can an attacker with X% of computational power do?
- Under what conditions does the protocol fail?

The calculation P = p^k in Section 12 is insufficient -- it does not explain what "wins k consecutive windows" means for the attacker.

**Recommendation:** Add an explicit threat model as a separate section.

---

### 2.2 Missing Liveness Proof (Whitepaper)

**Severity:** High  
**Source:** Adversarial Protocol Design Review

The whitepaper claims liveness ("the chain extends") without proof. The protocol specification contains complex mechanisms for fallback proposals, participation-ratio adjustment, and adaptive SSHA -- the whitepaper mentions none of these.

Open questions: What is the minimum quorum for progress? What happens during a network split? What happens when D calibration diverges between nodes?

**Recommendation:** Formalize liveness conditions or refer to the protocol specification.

---

### 2.3 1B-User Claim Without Scaling Analysis (Whitepaper)

**Severity:** High  
**Source:** Adversarial Protocol Design Review

The whitepaper names "1B users" as the "baseline architectural target" without any quantitative analysis. A simple calculation that is missing:

- AccountRecord = 2,059 bytes
- 1B accounts × 2,059 bytes = approximately 2 TB of state
- Every full node must hold that
- Fast-sync at 2 TB: how long?
- TPS at 1B active users: how many?

Without these numbers, the 1B claim is not defensible for the Metzdowd list.

**Recommendation:** Either add a quantitative scaling analysis or remove the claim.

---

### 2.4 Bootstrap Economics Not Analyzed (Whitepaper)

**Severity:** High  
**Source:** Adversarial Protocol Design Review

The whitepaper emphasizes "no fees" as a core feature, but does not analyze bootstrap economics: Why should anyone operate a node if the Ɉ price is zero? How does the network achieve critical mass? What is the Nash equilibrium for early operators?

**Recommendation:** Analyze bootstrap economics or explicitly mark it as an open question.

---

### 2.5 ML-DSA-65 Side-Channel Risk (Implementation)

**Severity:** High  
**Source:** Technical Vulnerability Analysis (MONT-001)  
**CVSS:3.1:** AV:L/AC:H/PR:N/UI:N/S:U/C:H/I:H/A:N = 6.7

The protocol specification mandates the deterministic signature mode for ML-DSA-65 (RND = 0x00 × 32, FIPS 204 §3.7), but contains no reference to the obligation of constant-time implementation. A node operator running 24/7 and producing millions of signatures offers a considerable attack surface for differential power analysis and timing attacks.

**Recommendation:** Explicitly add in the protocol specification: "All implementations MUST use constant-time operations for ML-DSA-65 signatures."

---

### 2.6 apply_proposal Race Condition (Implementation)

**Severity:** High  
**Source:** Technical Vulnerability Analysis (MONT-003)  
**CVSS:3.1:** AV:N/AC:L/PR:L/UI:N/S:U/C:N/I:H/A:L = 7.1

The specification describes apply_proposal as deterministic, but does not explicitly specify the order for multiple operations from the same sender within a window. An attacker can submit two valid operations with the same prev_hash (frontier_hash). The specification does not clarify how conflicts within the same proposal are resolved.

**Recommendation:** Include in the specification an explicit conflict-resolution rule for competing operations from the same sender within a window.

---

### 2.7 IBT-Proof Replay Window (Implementation)

**Severity:** Medium (effective priority High due to network position)  
**Source:** Technical Vulnerability Analysis (MONT-002)  
**CVSS:3.1:** AV:N/AC:H/PR:N/UI:N/S:U/C:L/I:L/A:N = 4.8

The online IBT proof is valid for 2 windows (approximately 120 seconds). With an attacker holding a man-in-the-middle position, an intercepted proof can be replayed against the same server_node_id within the window. Domain separation between mt-tunnel-online and mt-tunnel-mesh is correct, but the online window itself is vulnerable without nonce tracking.

**Recommendation:** Implement nonce tracking for online IBT proofs, analogous to the nonce tracking already in place for mesh IBT proofs.

---

### 2.8 Hardware Asymmetry Incompletely Addressed (Whitepaper)

**Severity:** High  
**Source:** Adversarial Protocol Design Review

The whitepaper claims: *"An attacker with one hundred times the resources does not get one hundred times more time."* This holds for wall-clock time -- but not for grinding. An ASIC attacker with ×10 SHA-256 speed can precompute 10 hours of hash chains in one hour of wall-clock time and grind keypairs against precomputed inputs.

The protocol specification has cemented_bundle_aggregate(r-2) as a countermeasure ([I-8]). The whitepaper does not explain this component.

**Recommendation:** Refine hardware-asymmetry analysis in the whitepaper and refer to cemented_bundle_aggregate as a countermeasure.

---

## 3. Design Strengths

The Montana Protocol exhibits several notable strengths that are explicitly acknowledged in the assessment:

**Exceptionally mature specification apparatus.** 4,416 lines of protocol specification, 22 critic passes, 15 architecture gates, three separate specification layers (Protocol, Network, App) -- for a pre-mainnet project this level of maturity is unusually high.

**Correct post-quantum cryptography at the consensus level.** ML-DSA-65 (FIPS 204) and ML-KEM-768 (FIPS 203) are correctly applied. The choice of a uniform NIST Security Level 3 for both primitives is architecturally consistent and aligns with [I-7] (minimal cryptographic surface).

**Well-thought-out Sybil protection through sequential hashing.** The basic mechanism -- sequential SHA-256 hashing as an entry barrier for node registration -- is conceptually sound. The costs are symmetric for all participants and cannot be accelerated through capital (only through hardware speed).

**Lottery mechanism with network-bound unpredictability.** The use of cemented_bundle_aggregate(r-2) as the lottery seed ([I-8]) correctly closes the class of hardware-asymmetry grinding attacks on lottery endpoints. This is a non-trivial design decision that is absent in many comparable protocols.

**State lifecycle management.** The Storage Cards for AccountRecord, NodeTable, Candidate Pool, and Anchor Records show a well-thought-out understanding of state-bloat risks ([I-14]). The combination of time-based barriers, activity-based pruning, and hard quotas is methodologically correct.

**Regulatory compatibility by design.** The explicit decision against privacy mixers, ring signatures, and hidden flows ([I-6]) is right for a regulatorily compatible deployment. The separation between the public financial layer and the encrypted application layer (anchor mechanism) is elegant.

**Deterministic arithmetic framework.** [I-9] with integer specifications, test vectors, and explicit rounding directives for all consensus formulas is a quality feature that prevents cross-implementation forks.

---

## 4. Open Risks by Category

### 4.1 Cryptography

| Risk | Severity | Status |
|------|----------|--------|
| Incorrect SSHA terminology in whitepaper | Critical | Open |
| Post-quantum claim partially incorrect (TLS X25519) | Critical | Open |
| ML-DSA-65 without constant-time requirement in specification | High | Open |
| PBKDF2 iteration count documented without migration path | Medium | Open |
| Grover analysis incomplete (collision resistance SHA-256 at 85 bits) | Medium | Open |
| FIPS reference inconsistency (FIPS 205 cited, not used) | Medium | Open |

### 4.2 Consensus

| Risk | Severity | Status |
|------|----------|--------|
| Singleton phase without network consensus (all BFT properties inactive) | Critical | Pre-mainnet state, documented |
| apply_proposal race condition with competing operations | High | Open |
| Missing liveness proof in whitepaper | High | Open |
| Mathematically inconsistent Sybil analysis in whitepaper | Critical | Open |
| Hardware-asymmetry analysis in whitepaper incomplete | High | Open |

### 4.3 Network

| Risk | Severity | Status |
|------|----------|--------|
| IBT-proof replay window without nonce tracking (online) | Medium | Open |
| Dandelion++ timer leak (timing analysis possible) | Medium | Open |
| Pre-key bundle staleness (outdated bundles not invalidated) | Medium | Open |

### 4.4 Application

| Risk | Severity | Status |
|------|----------|--------|
| 1B-user claim without scaling analysis | High | Open |
| Bootstrap economics not analyzed | High | Open |
| Mnemonic wordlist not publicly formalized | Low | Open |

### 4.5 Economics

| Risk | Severity | Status |
|------|----------|--------|
| Bootstrap economics: Nash equilibrium for early operators not analyzed | High | Open |
| Rational-delay equilibrium not excluded (operators wait for higher network density) | High | Open |
| No analysis of equilibrium between speculation and usage | Medium | Open |

---

## 5. Recommendations

### Prioritized Action List

**Priority 1 -- Before any publication on the Metzdowd list (Blocker)**

1. **Correct SSHA terminology.** "Sequential hash chain" instead of "SSHA", remove references Boneh/Pietrzak/Wesolowski or contextualize them correctly. Effort: low, impact: high.

2. **Precise post-quantum claim.** Explicitly differentiate between consensus layer (fully PQ) and network layer (TLS 1.3 with X25519, Tier 2 with Noise_PQ available). Effort: low, impact: high.

3. **Correct the Sybil analysis.** Mathematically correct formulation of costs (linear in machines, not quadratic). Effort: low, impact: high.

4. **Add a threat model.** Define attacker types, assumptions, and failure conditions explicitly. Effort: medium, impact: high.

**Priority 2 -- Before M7/M8 (High urgency)**

5. **Add constant-time requirement to protocol specification.** ML-DSA-65 implementations must use constant-time operations. Effort: low, impact: medium.

6. **Specify apply_proposal conflict resolution.** Explicit rule for competing operations from the same sender within a window. Effort: medium, impact: high.

7. **Nonce tracking for online IBT proofs.** Analogous to the existing mesh-IBT nonce tracking. Effort: medium, impact: medium.

**Priority 3 -- Before mainnet**

8. **Scaling analysis for the 1B claim.** Quantify state size, fast-sync duration, TPS capacity -- or remove the claim. Effort: high, impact: medium.

9. **Analyze bootstrap economics.** Nash equilibrium for early operators, rule out rational-delay equilibrium. Effort: high, impact: high.

10. **Formalize liveness proof.** Document minimum quorum, network-split behavior, D-calibration divergence. Effort: high, impact: high.

### Strategic Recommendation: Two Paths

**Path A -- Focused whitepaper (recommended for timely publication):**  
Reduce the whitepaper to the core promise: post-quantum consensus with time as a scarcity resource. Remove all unsubstantiated claims (1B, no-fees as unique selling point, SSHA terminology). Add threat model. Formalize liveness conditions. Length: 6-8 pages.

**Path B -- Full academic paper:**  
Expand the whitepaper into a full academic paper with threat model, liveness proof, scaling analysis, security reductions, and comparison with related work. Length: 20-30 pages. Time investment: considerable.

Path A is the more realistic route for timely publication on the Metzdowd list.

---

## 6. Assessment of Publication Readiness

**Verdict: Rework required. The whitepaper is in its current form not suitable for the Metzdowd Cryptography List.**

### Rationale

The Metzdowd Cryptography List is a forum for technically proficient cryptographers who judge protocols on the basis of their mathematical and cryptographic correctness. The three critical errors in the whitepaper (incorrect SSHA terminology, partially incorrect PQ claim, inconsistent Sybil analysis) will be immediately recognized by this audience and undermine the credibility of the entire project -- regardless of how sound the underlying protocol specification is.

### What the whitepaper has

- Clear problem statement (post-quantum gap in existing chains, fee-based anti-spam problems)
- Interesting core mechanism (time as a scarcity resource)
- Reference to a mature protocol specification
- Correct fundamental intuition for Sybil protection

### What is missing

- Correct terminology for the central mechanism
- Honest representation of PQ coverage (consensus vs. network)
- Explicit threat model
- Quantitative scaling analysis
- Liveness proof or reference
- Clear distinction between protocol promise and current implementation state

### Potential

The project has the potential for a strong publication. The protocol specification is significantly more mature than the whitepaper. With Path A (focused whitepaper, 6-8 pages, correct terminology, explicit threat model), a publication-ready version is achievable in a manageable time investment.

---

## Appendix: Finding Overview

| ID | Title | Severity | Source | Status |
|----|-------|----------|--------|--------|
| WP-1 | Incorrect SSHA terminology | Critical | Whitepaper review | Open |
| WP-2 | Post-quantum claim partially incorrect | Critical | Whitepaper review | Open |
| WP-3 | Inconsistent Sybil analysis | Critical | Whitepaper review | Open |
| WP-4 | Missing threat model | High | Whitepaper review | Open |
| WP-5 | Missing liveness proof | High | Whitepaper review | Open |
| WP-6 | 1B claim without scaling analysis | High | Whitepaper review | Open |
| WP-7 | Bootstrap economics not analyzed | High | Whitepaper review | Open |
| WP-8 | Hardware asymmetry incomplete | High | Whitepaper review | Open |
| WP-9 | Misleading comparative calculation | Medium | Whitepaper review | Open |
| WP-10 | Grover analysis incomplete | Medium | Whitepaper review | Open |
| WP-11 | FIPS reference inconsistency | Medium | Whitepaper review | Open |
| WP-12 | Mnemonic formalization missing | Low | Whitepaper review | Open |
| MONT-001 | ML-DSA-65 side-channel risk | High | Vulnerability analysis | Open |
| MONT-002 | IBT-proof replay window | Medium | Vulnerability analysis | Open |
| MONT-003 | apply_proposal race condition | High | Vulnerability analysis | Open |
| MONT-004 | Singleton phase without network consensus | Critical | Vulnerability analysis | Pre-mainnet state |

*Data basis: Adversarial Protocol Design Review (2026-05-19) + Technical Vulnerability Analysis (2026-05-19). Additional findings of the vulnerability analysis (MONT-005 to MONT-023, severity medium to low) are documented in the full vulnerability analysis.*

---

*CISO-as-a-Service Team, 2026-05-19*
