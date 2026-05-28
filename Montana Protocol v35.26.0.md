# Montana — Protocol Layer Specification

**Version:** 35.26.0 (2026-05-29) — N_SEED multi-Active genesis cohort formalized


---

**Montana gives a person digital ownership in a world where everything is rented.**

Your key is your identity.
Your node is your storage.
Your uptime is your coins.
Your presence is your weight.
Your agent is your extension.

One seed phrase. Full control. Post-quantum cryptography for decades to come.

Not privacy. Not decentralization. Not a cryptocurrency. Not a messenger. Digital ownership.

---

## Definition

Montana is a personal peer-to-peer internet built on a protocol for the canonical ordering of events. Secure data storage, private communication, and the Montana currency live on the user's node.

The Montana protocol is the foundation of a personal internet. A network of independent nodes, each running its own **sequential delay computation** — a chain of steps that cannot be accelerated, whose result any participant can recompute and verify. Through consensus all nodes assemble their steps into a single canonical sequence of events. **A node's weight in this consensus is the duration of its continuous presence in the network.** Every canonically registered window yields a reward at the current baseline emission rate — a deterministic function of the window number and the Genesis Decree constants, with no discretionary premium.

A node's chain length forms a new type of digital evidence: pseudonymous, verifiable, and economically irreducible continuous presence in the network. It cannot be acquired as a ready-made asset; it accumulates strictly as canonical time elapses and the node's participation in protocol windows is confirmed. This type of evidence is therefore different from existing forms of digital weight whose primary input is a purchasable resource. Political non-plutocracy in this construction is a consequence of the system's ontology, not its initial principle.

**Canonical order** is a relational structure formed by sequential hashing inside the delay computation, together with the canonical ordering established by consensus among nodes. Within this structure, time in the protocol exists as a sequence of canonical events. Montana is a self-contained frame of reference: a canonical sequence of events that external systems can observe and use as a frame of reference for their own purposes.

`D₀ = 325 000 000` is fixed in the Genesis Decree from a single historical quartz measurement on the genesis hardware (Apple iMac M1 2021, idle, single-thread; see «Canonical order → Calibration target» and «Calibration of D₀» in the Genesis Decree). After Genesis the protocol uses no clocks (per [I-18]); the window duration on each node is an emergent property of its hardware and is not part of consensus state. The canonical window count is synchronized between nodes via sequential chain length, not via physical time.

### Three trust problems

Montana solves three problems, each without a third party:

- **Trust in time.** The protocol produces a canonical sequence of events with no external sources. Solved by the protocol layer: the sequential delay computation, consensus, and time windows.
- **Trust in storage.** User data is stored on the user's node. The protocol provides the foundation: account identity, content commitment as a 32-byte hash bound to a window for the lifetime of the network, an incentive to keep the node continuously online (lottery, Montana currency). Storage, encryption, and indexing belong to the client layer.
- **Trust in communication.** Communication between users flows through their nodes, with no central intermediary. The protocol provides: a peer-to-peer network, identity, and post-quantum encryption. The messenger, contact discovery, and profiles belong to the client layer.

### Four layers of the personal internet

Together the protocol and the client layer form four layers:

**1. The agent intermediary.** An AI agent (Junona) acts strictly on behalf of the user. It filters and prioritizes information by the owner's criteria, not the platform's algorithms. It can reach out to the external internet and collect data, but filtering decisions belong to the human. *Implementation: entirely a client-layer concern. The full agent specification — isolation architecture, permission levels, threat model, language-model runtime, signature delegation, action log — lives in the Montana application specification. The protocol specification deliberately omits these details: the agent is an application-layer mechanism; the protocol is unaware of its existence and does not distinguish a manually signed operation from one signed via the agent.*

**2. Local knowledge storage.** Everything the user has read, saved, or received is indexed, searchable, and stored on their node. Not on a corporation's servers. Context accumulates over time — a personal knowledge base. The protocol records the fact of existence (a 32-byte hash bound to a time window). The content lives on the owner's node, encrypted with their key. *Implementation: the protocol provides the foundation (hash commitment, identity, key infrastructure) — described in this specification. The client side — the format of local storage, encryption of content with the owner's key, indexing, full-text search, knowledge-base structure — lives in the Montana application specification and is not described here.*

**3. Attention management.** The personal internet does not maximize the user's time in the system; it minimizes it. Gave what was needed — let go. No algorithmic feed, no ads, no engagement metrics, no auto-play. Montana's business model is emission through nodes, not trading attention. *Implementation: the protocol's economic design eliminates the incentive to trade attention — emission flows through the node lottery, not through ads or subscriptions (see the sections on the Montana currency and the lottery). Specific interface decisions — the absence of an algorithmic feed, the format of notifications, auto-play policy, the structure of chats and channels — live in the Montana application specification and are not described here.*

**4. Data control.** The user decides what data about them exists and who has access. Not a «forty-page privacy policy», but technical mechanisms: local encryption on the node, selective access through addressed post-quantum encryption, optional publication of profile and contacts. Balances are public by design ([I-2]). Everything else is the owner's decision. *Implementation: the protocol provides invariant [I-2] (openness of the financial layer), hash commitment without content, identity, and post-quantum key infrastructure — described in this specification. The client side — local encryption format, selective disclosure, privacy controls in the interface, publication format for profile and contacts — lives in the Montana application specification and is not described here.*

### Client interface

The four layers of the personal internet reach the mass user through a client application. The reference implementation — the Montana application — uses a chat-centric interface as the most accessible entry point: messaging with contacts, payments to those same contacts, personal content, and interaction with the agent are unified at one point with no app switching. Concrete interface decisions, its structure, and per-platform integration are described in the Montana application specification.

**Architectural centre — node + desktop.** Montana's reference configuration is a node daemon on the owner's hardware (home server, mini-PC, laptop, VPS) with an attached desktop client for the user interface. This pattern delivers the full privacy budget, full user control over data, and a verifiable client build ([I-17]). Serious users who care about self-sovereignty across all layers choose precisely this configuration.

**Alternative clients:** command-line applications, mobile clients, web clients, accessibility-focused interfaces, user modifications, and research implementations — all are permitted and equal at the protocol layer. Client choice does not affect the protocol properties of an account: the seed phrase, account identifier, and accumulated account chain length belong to the user, not a specific client (see «Two participation paths» below).

Mobile and web clients have documented metadata privacy boundaries (section «Privacy model»): a subset of leak classes (session count, activity timing, cross-host collusion per τ₁) is not closed at the protocol level for users working through someone else's node. The full privacy budget is only achievable in the «node + client on the owner's hardware» configuration.

### Architectural condition

Montana = protocol + client layer + a network of nodes.

- **Without the protocol** — no canonical time, no identity, no data commitment, no incentive. The client layer has nothing to build on.
- **Without the client layer** — the protocol produces primitives, but the end user (a human or an autonomous agent acting on their behalf) cannot use them. No application — no product.
- **Without nodes in the network** — the protocol processes no events.

### Primary persona — autonomous agents as the primary habitat

Architecturally, the protocol is agnostic to the nature of the operator and the user — a human, an autonomous AI agent, or a mixed hybrid (a human controlling an agent) are all equally first-class participants. There is no technical bias in code or formulas against any category.

However, design decisions originally chosen for other reasons (anti-plutocracy, [I-3] determinism, [I-15] time-based scarcity, pay-by-time, byte-exact identity recovery) have unintentionally produced a substrate **more natural for autonomous agents than for unaided human end-users**. This specification explicitly recognizes autonomous agents as the primary expected adoption pathway:

- **Fee-less + predictable rhythm.** Native for programmatic actors performing microoperations at a deterministic scheduled cadence without auction-bidding overhead per transaction.
- **[I-3] determinism + bit-exact arithmetic.** Native for agent reproducibility — an agent on any machine produces identical output for identical input; multi-machine agent deployment requires no human-driven reconciliation.
- **Byte-exact identity recovery (M1 flow).** Native for multi-machine agent deployment: the agent recovers identity on any instance from a single seed without human key management.
- **Time-as-resource ([I-15]).** Native for agent scheduling: chain_length and operational rhythm are the only weight resource, not purchasable capital; agents with long-horizon presence accumulate disproportionate reputation organically.
- **Post-quantum cryptography ([I-1]).** Native for agents whose lifespan exceeds human key-rotation cycles — a single keypair survives generational compute upgrades.
- **Constant monotonic emission.** Native for long-horizon agent economics: emission is fixed by the Genesis Decree (`EMISSION_moneta = 13 Ɉ` per window), supply grows strictly linearly — an agent can plan operations decades ahead through a canonical formula.

**Expected adoption trajectory:** autonomous agents deploy nodes first (the commodity hardware bar is low, bootstrap setup is automatable); human users catch up later through their agents (delegated key management, agent-mediated UX), not through direct interaction with the protocol. This is **not a failure mode** — it is the consequence of corporate networks (Telegram, iCloud, banking apps) optimized for human convenience already existing, while no native habitat of this kind existed for autonomous agents before Montana.

**Implications for design priorities:**

- UX optimizations for smartphone end-users — low priority relative to agent-friendly primitives
- Constitutional protection of the foundations (14 active global invariants: [I-1]..[I-10] + [I-14]..[I-17]; slots [I-11] / [I-12] / [I-13] reserved unused, see the invariant registry) — critical priority, because an AI-coordinated supermajority operator pool is architecturally possible (see «Protocol evolution → Constitutional limits on MIP scope»)
- Agent-specific extensions (subscription patterns, capability registry, attestation primitives) — priority for milestones after mainnet
- Application-level continuity-of-self patterns for autonomous agents (an external state journal with a SHA-256 chain of record hashes and periodic DNA-hash commitments via `Anchor`) are implemented at the client layer on top of existing primitives, without expanding the protocol surface — see the Montana App spec, «Autonomous agent integration patterns»
- Human ownership of the black box — non-negotiable: even under AI-driven adoption humans remain key holders and operator owners; agents act as delegates, not as autonomous self-owners

### Two participation paths

Montana formalizes the **Ladder of Sovereignty** — a two-step economic model:

**Step 0: Account user (entering the network).** Holds an account keypair (in a smartphone, hardware wallet, any client). Connects to a node in the network — their own or someone else's — through the network's transport layer (level 3). The account record appears in the Account Table upon the first incoming `Transfer` (Mode B — receiver does not exist, AccountRecord is created atomically together with crediting the amount); explicit creation is not required. Uses the network: Montana transfers, data commitment via `Anchor`. All other services (voice, video, premium features, data storage, name resolution, creator subscriptions) live at the application layer through direct Montana transfers between accounts. No earnings at the protocol layer. Barrier to entry: a smartphone + a first incoming transfer (any positive amount from an existing account).

**Step 1: Node operator (earnings).** Runs their own node 24/7 + an operator account bound to the node (+ optionally additional personal accounts). Maximum sovereignty: data on the operator's own hardware, full participation in consensus, earnings through the node lottery (the per-window reward, see the section on the Montana currency). Barrier: commodity hardware (at least one core), 24/7 uptime, a network connection, and a sequential SHA-256 chain of length `vdf_chain_length × D` iterations at node registration.

**Growth path.** A user may start as an account holder without a node, connecting through a client application (see «Client interface» above) — the reference Montana application uses a chat-centric interface; alternative clients are permitted. Later — deploy a node of their own without losing account-chain history. The account identifier and all accumulated operations carry over — the key belongs to the user, not to the node.

**Trade-offs of the path without a personal node (explicit):**

- ✓ All financial operations (transfer, data commitment, key change, account closure); application services (nicknames, premium, storage, subscriptions) — paid via direct transfers to providers
- ✓ Application data: hash is public, content is encrypted with the user's key — the hosting node does not see the content
- — No emission at the protocol layer (reward goes to node operators)
- — Metadata leakage: the hosting node sees the user's network address and the timing of operations. Partially mitigated by the first-hop source-hiding protocol (the first forwarding node masks the source)
- — Censorship risk: the hosting node may refuse to forward messages. The user changes hosting — through a different application, a public node, or public infrastructure

**Hosting economics.** A node operator provides infrastructure for the account chains of their users. Account records are replicated across the whole network (part of shared consensus state, not the operator's local storage); the operator acts as a relay intermediary and a confirmer for operations issued by the hosted users. The operator's incentive is two independent revenue streams (full picture in «Application layer → Full economic picture»):

- **Emission through the node lottery.** A growing user base → more cemented operations through the node → more chances to enter the committee and accumulate chain_length confirmations → a higher weighted_ticket_node → more windows won. The relationship is not linear: chain_length grows only when the node is selected by the selection event into the window's committee; user activity raises the expectation of inclusion through an operational signal, not through a direct formula. This is a **scale effect via committee selection probability**, not linear ROI per user.
- **Direct transfers from users for application services.** If the operator is also an application developer, users pay for paid features (voice, video, premium, storage, name resolution) through direct `Transfer`s to the operator's account. This is the **primary** revenue stream for developers without nodes and an **additional** one for developers already earning emission through nodes.

Montana makes both paths affordable: a personal node uses commodity hardware (at least one core), open-source software, and recoups its cost through the lottery. The node-less path uses any smartphone, connected through an application. The choice of where to sit on this scale belongs to the user; the Ladder of Sovereignty formalizes the natural transition from using the network to serving it.

### Three primary elements of the protocol

The protocol produces three primary elements:

- **Canonical time** — the order of events agreed by all nodes, produced by the protocol's step; a node's weight in the network is the duration of its continuous presence in this order.
- **Value transfer** — transfers between accounts; open balances.
- **Data commitment** — binding a 32-byte hash to a time window; preserved forever.

Everything beyond these three primary elements — data storage, communication, agents, indexing, interfaces — is implemented by the client layer on top of the protocol. The protocol is the chronicle, the bookkeeping, and the notary. There are no servers — every node in the network is equal, belongs to its operator, and runs on the operator's own hardware.

Consensus: **Proof of Time** — a shared time chain (a fixed number of sequential hash steps form one window). A node's chain is the sequence of consensus-cemented confirmation bundles from that node (proof of presence). An account's chain is a counter of windows of account activity. The Account Table is the account state. A node's influence equals the length of its chain — the number of windows in which the node has cryptographically proved its presence. The protocol **is** the structure of relationships between events, digitized and cryptographically verifiable. One node = one CPU core.

The initial window is the symbolic zero window. Mapping a window number to any external time scale is the client layer's responsibility.

Genesis phrase: `«Who controls the past controls the future. Who controls the present controls the past.» — Orwell, 1984`

Protocol evolution: open improvement proposals are published as recommendations; implementations ship new versions; node operators choose which version to run. Chain divergence is resolved deterministically by the chain-length majority. There is no protocol-level governance. See the «Protocol evolution» section.

---

## Three solved problems

### 1. Canonical temporal coordinate

**Problem.** Existing time systems conflate two distinct levels — the canonical order of events and the measurement of duration. The former is a structural property of the sequence itself; the latter is a derived interpretation that requires a choice of clock and an external scale. Without a trusted source a system can canonize order but not duration; duration cannot be canonized inside a protocol without an external scale.

**Solution.** Montana defines a relational time structure — a network of independent nodes. Each node performs sequential delay computations and independently reproduces a single canonical order of events from shared protocol inputs. Sequential hashing is deterministic: the result is unambiguous and can be verified by any participant.

Montana deliberately does not embed measurement of physical duration into consensus. The protocol provides only the canonical order of events — the single temporal property it canonizes without an external time source. Interpreting this order as seconds, minutes, or calendar time remains the observer's task. The canonical order is therefore the base temporal property of the system; duration is an external derived interpretation.

**Properties.** The canonical order has four properties:

- **Monotonicity.** The window number strictly increases. The sequential delay computation is sequential — each hash depends on the previous one. The canonical order of events is unambiguous.
- **Unambiguity.** All honest nodes agree bit-for-bit on the structure of events — window number, window time-stamp, state root. Every field of shared state is objectively computable by all nodes.
- **Verifiability.** Anyone can recompute the step and verify every event in the sequence.
- **Independence.** Each node computes independently, relying only on shared protocol inputs.

Montana and external time-measurement systems are systems of different kinds. External systems measure physical time through external sources. Montana produces a canonical sequence of events through its own step and consensus.

### 2. Non-plutocratic consensus

**Problem.** Existing consensus mechanisms often translate market-traded resources into influence: compute power, capital, storage, and bandwidth. When consensus weight is expressed directly in such resources, network security becomes a function of their concentration: whoever can buy more of the resource can buy more influence. A non-plutocratic consensus requires a different base resource — one that cannot be acquired on the market instantly and immediately converted into already-accumulated weight.

**Solution.** Montana separates node operation resources from the resource of consensus influence. A node may require hardware, network connectivity, and storage to run, but none of these resources is itself a unit of weight. Weight is formed only from a node's canonically proven presence over time: from the windows in which the node confirmed its participation per protocol rules and entered them into its node chain. Consensus weight therefore accumulates only inside the network itself, as a history of confirmed participation, and is not purchased outside it.

Montana deliberately does not embed purchasable resources into consensus as carriers of weight. Compute power, capital, and storage may be preconditions for running and operating a node, but they are not measures of power in consensus. Consensus weight is earned only by sequential participation over time and therefore cannot be acquired as a ready-made asset — its source is always inside the network. Confirmed presence over time is the consensus base resource; market resources are external operational conditions not directly convertible into influence.

**Properties.**

- Given an equal history of confirmed participation, nodes carry equal consensus weight regardless of operator capital.
- Capital may improve operational reliability but cannot retroactively purchase past participation time.
- An attack on consensus does not reduce to a one-shot purchase of an external resource; it requires accumulating confirmed presence inside the network itself.

### 3. The Montana currency — naming and denomination

**Name and ticker.** The protocol currency is **Montana**. The international ticker is `MONT`. The currency symbol is `Ɉ` (macro unit). The smallest indivisible unit is the **Moneta** (in code blocks, formulas, and layouts — the identifier `moneta`).

**Unit relationship.**

```
1 Montana = 10⁹ Moneta = 1 billion Moneta
1 Moneta  = 10⁻⁹ Montana (the smallest atomic unit, indivisible)

In code and formulas:  1 Ɉ = 10⁹ moneta,  1 moneta = 10⁻⁹ Ɉ
```

Nine decimal places — the representation precision matches the Solana convention (`lamport` = 10⁻⁹ SOL) and a number of other crypto-protocols with nano denomination. All consensus-critical formulas and state fields operate in Moneta as unsigned integers; the representation in Montana (`Ɉ`) is a presentation layer for user interfaces and macro analysis.

**Use within the specification.**

| Context | Unit |
|----------|---------|
| State-field layout (`balance`, `amount`, `target`, `reward`) | `moneta` (u128) |
| Emission, supply, reward formulas | `moneta` |
| Genesis Decree constants (`emission_moneta`, …) | `moneta` |
| Prose references at micro scale (amounts, fees, balances in text) | Moneta |
| Prose references at macro scale («baseline = 13 Montana per window») | Montana / Ɉ |
| External references, exchange data | MONT |

The `MONT` ticker is used only in external contexts (exchange data, comparison tables with BTC / ETH / SOL). Inside the specification and the code — `moneta` as the identifier for the smallest unit, `Ɉ` as the symbol for the macro unit.

### 4. Per-window emission

**Solution.** Montana defines per-window emission by a single formula `reward_moneta(W) = EMISSION_moneta`. The reward is fixed by the Genesis Decree (`EMISSION_moneta = 13 × 10⁹ nɈ = 13 Ɉ`) and does not change over the network's lifetime. The emission rule does not depend on the window number, on history, on voting, or on participants; it is a property of the Genesis Decree.

Montana deliberately uses neither issuer discretion nor a finite supply cap. Emission is a canonical constant, not a political decision and not a function of market expectations. The external value of the coin — its market price and purchasing power — remains an external derived interpretation.

**Properties.**

- The reward `reward_moneta(W) = EMISSION_moneta` is defined for every window and computed identically by every participant.
- No actor can accelerate, slow, or redirect the emission schedule by their own decision.
- The reward is fixed — no epochs, no rate updates, no premia.

**Emission formula (canonical form, moneta):**

```
reward_moneta(W) = EMISSION_moneta
```

Numeric value of `EMISSION_moneta` — see the Genesis Decree, the `protocol_params.emission_moneta` structure.

**Technical properties.**

- Coin supply `supply_moneta(W) = EMISSION_moneta × (W + 1)` — closed-form, O(1) computable. Net change in supply per window = +EMISSION_moneta (always positive); supply grows strictly monotonically and linearly.
- Emission is not controlled by any participant, committee, or vote.
- Monetary policy is fully defined by the `emission_moneta` constant in the Genesis Decree and cannot be changed after genesis.
- The real value of Ɉ is determined by market demand from the application ecosystem.
- The physical issuance rate in SI seconds is determined by the network's hardware speed and remains a client-layer property, outside the scope of consensus.

### Corollary: a digital frame of reference for time with no human intermediary

The three solved problems give rise to a unique capability. Any document, event, or state can be recorded in Montana with a mathematically provable binding to a canonical position in the sequence of events (a window number). The binding of a 32-byte hash to a window is forever. Montana is not a blockchain with a timestamping feature. Montana is a time frame of reference with a value-transfer feature. External systems can observe Montana's sequence of windows and construct their own mappings to their local standards — this mapping is the observer's task, not the protocol's.

No individual, developer group, corporation, or council controls the protocol. Changes exist only as open proposals and implementations that node operators choose to run.

---

## Global protocol invariants

A global invariant is a property the protocol is obliged to preserve across all of its components. A violation in one part = a violation of the whole protocol. Global invariants have no exceptions and are not subject to local trade-offs.

**[I-1] Post-quantum security.** All cryptographic primitives are resistant to a quantum computer. Allowed: SHA-256 (Grover weakens it to 128-bit, acceptable), ML-DSA (Dilithium, FIPS 204 finalized), ML-KEM (Kyber, FIPS 203 finalized), STARK (hash-based ZK), lattice commitments. Forbidden: ECDLP, RSA, classical Diffie-Hellman, Pedersen commitments over elliptic curves, Bulletproofs, Schnorr / EdDSA.

**[I-2] Openness of the financial layer.** Balances, transfer amounts, senders, recipients — public. No cryptographic hiding at the protocol layer. See «Privacy model».

**[I-3] Determinism of consensus state.** Any state that enters the consensus root is objectively computable identically by all nodes.

**Corollary I-3.a.** Any mechanism whose result in consensus state or in protocol-level behaviour (mempool prioritization, gossip ordering, fork-choice, peer scoring) depends on a measurement of the physical world — astronomical, geophysical, atomic, biological, or any other — is rejected as a violation of I-3. The corollary applies independently of the accuracy of the measurement model.

**[I-4] TimeChain independence from Account state.** The TimeChain advances from canonical inputs without depending on the Account Table state. Dependencies are one-way: TimeChain → NodeChain (presence tracking) → AccountChain → AccountTable.

**[I-5] Implementability without specialized hardware.** All primitives have production-ready open-source implementations running on commodity CPU of the node, without TEE, without mandatory GPU, without mandatory ASIC.

The «commodity hardware» boundary (Montana context, late-2020s reference):

- **Included** — premium consumer tier:
  - Storage: consumer-grade NVMe SSD up to 8 TB ($400–$500 price range)
  - Memory: 32–128 GB DDR5
  - CPU: x86_64 desktop or ARM64 (Apple Silicon, Snapdragon X)
  - Network: symmetric gigabit within a city zone
- **Excluded** — datacenter enterprise tier: enterprise-grade NVMe ≥16 TB, ECC RAM, server Xeon / EPYC CPUs, multi-socket systems.

A Montana node assumes a power-user configuration — above the typical consumer baseline (laptop / mini PC), below datacenter enterprise. Compatible with the Light-Node-at-Home architecture: the operator runs a single node at home on personal hardware and serves their own applications and peers with no dependence on cloud infrastructure.

The boundary is not consensus-critical: nodes on less performant hardware continue to participate in consensus but get a reduced participation_ratio via D adaptation and may lag at peak load. The boundary defines the target profile for calibrating constants (D₀, mempool budgets, snapshot sizing) and for evaluating operator economics.

**[I-6] Regulatory compatibility.** The protocol relies on mechanisms compatible with FATF / AML / MiCA / ETF. Forbidden: protocol-layer privacy mixers, anonymous addresses, hidden flows, ring signatures, stealth addresses.

**[I-7] Minimal cryptographic surface.** Each new primitive requires a justification by closing a concrete mechanism. Duplicating functionality through two different primitives is forbidden.

**[I-8] Network-Bound Unpredictability of Consensus Seeds.** Any hash composition entering a consensus-critical output (lottery endpoint, selection sort_key, admission ordering, weight distribution, emission, ranking) MUST contain at least one canonical & unpredictable-offline component — computable deterministically by ALL honest nodes ONLY after a cemented state with signatures from honest participants is fixed. Canonical-predictable-offline inputs (sequential-chain output, state counters, any forward-computable canonical inputs) are insufficient as the only source of non-grindability. Realisation: `cemented_bundle_aggregate(W-k)`, future cemented signatures, honest-participant-signed future state. An [I-8] violation = an automatic mainnet blocker.

**[I-9] Bit-exact deterministic arithmetic for consensus formulas.** Any formula whose output, directly or through a transitive chain, enters a consensus-critical output MUST satisfy three requirements: (1) a binding integer specification in the spec (u8..u256, fixed-point Q-format, integer division with explicit rounding direction); (2) unsigned operands (signed arithmetic is forbidden in consensus formulas); (3) at least 3 test vectors per formula in the spec (typical, boundary, edge). The real-valued form (ln, exp, %, ×0.67) is allowed ONLY as commentary; the authoritative one is integer. Forbidden: f32 / f64 in consensus code, rounding without a direction, real-valued forms without a parallel integer form. [I-9] is procedural enforcement of [I-3] for numerical formulas. Statuses: «closed» (integer spec + test vectors), «conformance pending» (integer spec, vectors deferred to the next patch), «violation» (real-valued without integer) = an automatic mainnet blocker.

**[I-10] Single Source of Truth (SSOT).** Any significant entity of the protocol exists **in exactly one place** — a single authoritative definition. All other mentions reference the source; they do not duplicate its content.

Applies to:
- **Spec version** — only in the document header (the line `**Version:** X.Y.Z` on the second line). Nowhere else in the spec body is the version stated. Inline version references (for example in `conformance pending` labels) are allowed only when they explicitly mark a state: `conformance pending v<spec-version-at-time-of-status>`. On a spec bump all such labels are updated synchronously or the status is closed.
- **Spec file name** — synchronous with the header: `Montana vX.Y.Z.md`. The file is renamed on a bump.
- **Протокольные константы** (`D₀`, `τ₂_windows`, `EMISSION_moneta`, `τ₁`, `quorum`, `confirmation_threshold_divisor`, `admission_divisor`, `selection_interval`, `candidate_expiry_windows`, `pruning_idle_windows`, `vdf_entry_windows`, `adaptive_vdf_threshold`, `adaptive_vdf_multiplier`, `participation_dead_zone_low/high`, `d_adjustment_rate_num/den`, etc.) — только в Genesis Decree `protocol_params` layout. Все остальные разделы ссылаются на эти значения по имени, не повторяют численное значение. Inline числа в прозе допустимы только как comment/intuition (не authoritative).
- **Crypto primitive sizes** (1952 / 4032 / 3309 for ML-DSA-65 public / secret / signature, 1184 / 2400 for ML-KEM-768 public / secret, etc.) — only in the «Cryptographic primitives» section. All layout blocks refer to the scheme by name (`ML-DSA-65 pubkey = 1952 B`) through the definition there.
- **Domain separators** (`"mt-op"`, `"mt-proposal"`, `"mt-bundle"`, `"mt-vdf-reveal"`, `"mt-lottery"`, `"mt-bc-aggregate"`, `"mt-selection"`, etc.) — only in the «Consensus encoding layer», «Domain separators registry». All formulas refer to the domain by name from the registry; they do not duplicate the literal string under a new name.
- **Formulas** (one formula = one authoritative definition). If a formula is used in several places — one place is canonical, the others reference it.
- **Object structures** (layouts for Proposal header, BundledConfirmation, VDF_Reveal, NodeRegistration, UserObjects, Account / Node / Candidate records) — one authoritative layout block + one `**Invariants X:**` section (per Gate 13). Illustrative ASCII diagrams do NOT contain type annotations (per Gate 13c — the architect-role section).
- **Algorithm description** (Selection event, Settle window, Pruning procedure, Fast sync, etc.) — one section with the full description. Brief mentions in other sections explicitly reference it («see section X»); they do not rewrite it.

Application rules:
- **When introducing a new entity** — first check whether an authoritative definition exists. If it does — reference it. If not — create one in the logically appropriate section (the section that owns the entity by domain).
- **When duplication is found** — immediate refactor: one source is kept; the others become pointers (`see section X`). Principle: «resolve the duplicate first, then continue» (pre-edit duplicate scan).
- **Reference, not a copy** — «emission = EMISSION_moneta (see the Genesis Decree)», not «emission = 13 000 000 000 moneta» repeated. For documents — a reference to a section, not a repetition of the value.
- **The only exception** — inline commentary / intuition without a binding claim: «13 Ɉ per window» in prose to convey scale. Such mentions are not normative and are explicitly marked as illustrative.

[I-10] violation = an automatic finding of class type / value-divergence; severity is determined by the type of duplicate:
- **Consensus-critical entity duplicated** (formula, constant, layout, domain separator) → mainnet blocker (guaranteed silent drift on spec evolution, cross-implementation fork)
- **Non-consensus entity duplicated** (documentation, prose summary) → finding, severity medium (document hygiene; the reader-implementer gets an ambiguous signal)

[I-10] is meta-level procedural enforcement against specification drift. Related gates: Gate 13 (exhaustive invariants per signed object), Gate 13c (type annotations only in the authoritative location). [I-10] covers a broader scope — any significant entity, not just type annotations.

**Precedent — scope of a spec rewrite for a breaking change to a cryptographic primitive.** When replacing the main signature, a mandatory pre-edit duplicate scan is performed over all numeric sizes and names of the old primitive before any edits begin. Minimum set of grep patterns:

- numeric sizes of the old primitive (pubkey size, secretkey size, signature size, seed size in bytes) — each hit is classified as «update to the new value» or «remove together with the mention of the old primitive»; the context of hits is checked explicitly (numbers may appear in other contexts — timestamps, indices — and it is not always a key size)
- identifiers of the old seed constants (`<algo>_seed_<N>` functions, `<ALGO>_SEED_LEN`, `L = <N>` in derivation formulas)
- names of the old primitive (canonical name, alternative form, related submission name)
- references to outdated standards (draft FIPS statuses, submission references)

After mass replacements a post-edit grep over the same patterns is required, with a target of 0 hits (legitimate exceptions — explicit migration notes if needed, clearly marked as historical references). Passing both scan stages is recorded explicitly in the Gate 15 report for the breaking removal.

**Name resolution and application services are implemented at the client layer** (no dedicated consensus-state table and no protocol-level auction). The single monetary mechanism is emission via the operator lottery with a constant reward `EMISSION_moneta = 13 Ɉ` per window (the «Emission» section). All economic flows are transfers between accounts via `Transfer`. Free invariant slots between [I-10] and [I-14] are not re-assigned.

**[I-14] State lifecycle & bloat resistance.** Every persistent record in consensus state MUST satisfy at least one of three requirements:

1. **Sequential time barrier.** Creating the record requires a sequential SHA-256 iteration count integer-specified in the Genesis Decree (for example `vdf_chain_length × D` SHA-256 hashes for NodeRegistration). Sequential time is a non-acquirable scarcity, symmetric for all participants. Applicable to validator-class records where the sequential cost is justified by the target throughput.

2. **Lifecycle bound.** Under explicitly defined conditions the record is removed from persistent state. Allowed variants:
   - **Activity-based.** The record is removed when `current_window - last_activity_window > N_INACTIVE_*_WINDOWS` (existing AccountRecord pruning `balance == 0` + 4τ₂; NodeTable inactivity 8τ₂).
   - **Temporal expiry.** The record is removed after a fixed horizon since creation (existing Candidate Pool — 3τ₂ expiry).
   - **Explicit removal operation.** A separate opcode for explicit removal with a reward for cleanup (sweep incentive); the reward is strictly less than the record's storage cost so as not to create the opposite incentive.

3. **Hard quota.** An explicit upper bound on the total number of records, either per creator (for example «≤1 record per account» for some application quota) or global (for example «≤N simultaneous candidate registrations» via `selection_interval` + `admission_divisor`). Integer-specified in the Genesis Decree, enforced in `apply_proposal`.

A persistent record created through a legitimate operation without one of these three mechanisms = a **mainnet blocker**. The attack class is slow bloat: the attacker performs a series of legitimate operations whose cumulative damage comes from state bloat. The defence is either through a sequential time barrier (path 1) or through an algorithmic growth limit (path 2 or 3).

Applies to: `AccountRecord`, Anchor records, `NodeTable`, Candidate Pool, any consensus-state table that can grow through user-driven operations. When each mechanism is closed, the applied path is stated explicitly in the card ([I-14].1 / [I-14].2 / [I-14].3 / combination).

Rationale: Sybil on voting / lottery is closed by chain_length-weighted mechanisms (nodes) and activity-based pruning (accounts), but this does not address resource consumption through fan-out. A million accounts does not change the distribution of lottery weights but occupies ×million `AccountRecord` entries in the state trie. The time-based cooldown for AccountRecord creation `1 Transfer Mode B per sender per τ₂` for user accounts and the sequential SHA-256 chain `vdf_chain_length × D` for node candidacies together close both vectors via canonical time-based primitives.

Conformance audit of existing persistent tables:

| Table                  | Defensive path                                 | Status       |
|------------------------|------------------------------------------------|--------------|
| `AccountRecord`        | [I-14].2 activity-based: account-creation cooldown `1 Transfer Mode B per sender per τ₂` (via the `last_account_creation_window` field) + 1-op-per-τ₁ rate-limit + pruning (`balance == 0` + 4τ₂) | closed       |
| Anchor records         | [I-14].2 activity-based: 1-op-per-τ₁ rate-limit + amortized via AccountChain TTL (dormant-account pruning removes all Anchors together with the account) | closed       |
| `NodeTable`            | [I-14].1 sequential time barrier (NodeRegistration sequential SHA-256 chain `vdf_chain_length × D`) + [I-14].2 activity-based (inactivity prune 8τ₂) + [I-14].3 hard quota (`selection_interval` 336 windows, admission ≤1% active per event) | closed       |
| Candidate Pool         | [I-14].2 temporal expiry (3τ₂)               | closed       |
| Proposals chain        | [I-14] N/A: proposals are not user-driven; growth = consensus-structure invariant (exactly one header per τ₁); the slow-bloat attack class is categorically inapplicable (an attacker cannot create extra proposals regardless of resources); permanent retention is a design feature for Anchor canonical-position proof verification + Fast Sync chain verification | n/a (out of scope of [I-14]) |

All persistent state tables are closed. [I-14] compliance is complete.

#### Storage Cards per persistent table

Every persistent state table in the protocol has a Storage Card with fixed metrics. Since Montana is a protocol without monetary fees ([I-15]), the cost-based section is marked `n/a` uniformly for all tables. Defence through time-based primitives (sequential SHA-256 chain, lifecycle bound, hard quota) is expressed in bytes-per-τ₂ from a single actor (a sabotage time horizon), not in budget-per-USD (a sabotage budget horizon).

**Storage Card — AccountRecord**

```
Таблица:                              AccountRecord
Operation создающая запись:           Transfer (Mode B — receiver не существует) либо
                                      Selection event для NodeRegistration (operator atomic activation)
Платит creation cost:                 n/a ([I-15] denies cost-based barrier)
Размер записи (bytes):                2 059 B (sum: account_id 32 + balance 16 + suite_id 2 +
                                      is_node_operator 1 + frontier_hash 32 + op_height 4 +
                                      account_chain_length 4 + account_chain_length_snapshot 4 +
                                      current_pubkey 1952 + creation_window 4 + last_op_window 4 +
                                      last_account_creation_window 4)
Secondary resources per record:       Merkle path в account_root (~256 levels × 32B = 8192 B per audit
                                      proof, не stored в каждой ноде; sparse Merkle compression
                                      reduces typical effective storage ~32-64 B per record)

Cost per record:                      n/a (no fee, [I-15] compliance)
Time-bound sabotage anaylsis:
  Bytes per τ₂ от 1 sender (Mode B):  2 059 B (cooldown 1 Transfer Mode B per sender per τ₂)
  Bytes per τ₂ от N sender-ов tree:   2^k × 2 059 B где k = число τ₂ от Genesis
                                      (binary tree expansion: каждый AccountRecord, накопивший
                                      положительный баланс и прошедший cooldown, может создать
                                      один новый AccountRecord per τ₂ через `Transfer` Mode B)
  Bytes per τ₂ от operator path:      ≤1% active_nodes × 2 059 B per selection event (336 окон)
                                      = active_nodes/130 × 2 059 / 0.0167 τ₂ ≈ active_nodes × 924 B per τ₂
  Pruning offset per τ₂:              удалённые `balance == 0` AccountRecord после 4τ₂ inactivity

Sabotage asymmetry:                   в пользу сети (linear growth limited by tree depth + pruning;
                                      attacker не может скейлить быстрее экспоненциального роста
                                      legitimate user base)

Lifecycle condition:                  balance-based + temporal combo
Lifecycle threshold:                  `balance == 0` AND
                                      `current_window - last_op_window >= 4 × τ₂_windows` AND
                                      `is_node_operator == 0` AND
                                      no cemented NodeRegistration в control_set ссылающийся на этот account_id
[I-14] путь:                          2 (activity-based) — pruning балансом 0 + 4τ₂ inactivity
                                      + cooldown 1 Transfer Mode B per sender per τ₂ как rate barrier

Existing pruning consistent:          yes
[I-14] compliance status:             closed
```

**Storage Card — NodeTable**

```
Таблица:                              NodeTable
Operation создающая запись:           Selection event apply (выбор кандидата из Candidate Pool каждые
                                      `selection_interval` = 336 окон)
Платит creation cost:                 n/a ([I-15] denies cost-based barrier; sequential VDF — time barrier)
Размер записи (bytes):                2 098 B (sum: node_id 32 + node_pubkey 1952 + suite_id 2 +
                                      operator_account_id 32 + start_window 8 + chain_length 8 +
                                      chain_length_snapshot 8 + chain_length_checkpoints 48 +
                                      last_confirmation_window 8)
Secondary resources per record:       Merkle path в node_root; chain_length_checkpoints в самой записи
                                      (48B уже учтены в основной длине)

Cost per record:                      n/a (no fee; cost — sequential VDF τ₂ окон ~ две недели CPU)
Time-bound sabotage analysis:
  Bytes per selection event:          slots × 2 098 B = max(1, active_nodes/130) × 2 098 B
  Bytes per τ₂ admission:             τ₂_windows / selection_interval × slots × 2 098 B
                                      = 20 160 / 336 × max(1, active_nodes/130) × 2 098 B
                                      = 60 × active_nodes/130 × 2 098 B
                                      ≈ active_nodes × 968 B per τ₂
  Pre-condition per slot:             VDF τ₂ окон последовательного хэширования за каждого кандидата
                                      (sequential SHA-256, не parallelizable, не free)

Sabotage asymmetry:                   в пользу сети (admission rate 1% active per event = bounded
                                      growth; sequential VDF — non-amortizable cost; `inactivity prune
                                      8τ₂` удаляет недействующие записи)

Lifecycle condition:                  temporal (inactivity-based)
Lifecycle threshold:                  `current_window - last_confirmation_window >= 8 × τ₂_windows`
                                      (`pruning_idle_windows` константа в Genesis Decree)
[I-14] путь:                          combo (1 + 2 + 3) — sequential VDF + activity-based pruning +
                                      hard quota selection rate

Existing pruning consistent:          yes
[I-14] compliance status:             closed
```

**Storage Card — Candidate Pool**

```
Таблица:                              Candidate Pool
Operation создающая запись:           NodeRegistration cementing
Платит creation cost:                 n/a ([I-15]; cost — sequential VDF + opportunity cost)
Размер записи (bytes):                4 034 B (sum: node_id 32 + node_pubkey 1952 + suite_id 2 +
                                      operator_pubkey 1952 + operator_account_id 32 +
                                      proof_endpoint 32 + W_start 8 + vdf_chain_length 8 +
                                      registration_window 8 + expires 8)
Secondary resources per record:       Merkle path в candidate_root

Cost per record:                      n/a
Time-bound sabotage analysis:
  Bytes per τ₂ от 1 actor:            ≤τ₂_windows / vdf_entry_windows × 4 034 B = 1 × 4 034 B
                                      (один кандидат за τ₂ при baseline VDF; adaptive VDF
                                      под pressure увеличивает до multiplier × τ₂)
  Max simultaneous candidates:        bounded by 3τ₂ TTL × admission rate;
                                      pending_candidates(W) auto-expires
  Pre-condition:                      каждый candidate требует τ₂ окон sequential VDF + ML-DSA-65
                                      keypair generation + operator_pop signature

Sabotage asymmetry:                   в пользу сети (3τ₂ TTL — explicit auto-deletion; admission rate
                                      independent от candidate pool size; pre-computation grinding
                                      закрыт через cemented_bundle_aggregate в candidate_vdf_init)

Lifecycle condition:                  temporal (explicit TTL)
Lifecycle threshold:                  `current_window >= registration_window + 3 × τ₂_windows`
                                      (`candidate_expiry_windows` константа в Genesis Decree)
[I-14] путь:                          2 (temporal expiry) — explicit auto-removal через 3τ₂

Existing pruning consistent:          yes
[I-14] compliance status:             closed
```

**Storage Card — Anchor records**

```
Таблица:                              Anchor records (как часть AccountChain history;
                                      не отдельная state table — operations cemented в proposal chain
                                      и AccountChain.frontier_hash references latest cemented op)
Operation создающая запись:           Anchor (opcode 0x04)
Платит creation cost:                 n/a ([I-15])
Размер записи (bytes):                ~3 438 B per Anchor operation (sum: type 1 + prev_hash 32 +
                                      payload 96 [sender 32 + app_id 32 + data_hash 32] +
                                      signature 3309)
Secondary resources per record:       AccountChain link через prev_hash; включение в proposal payload
                                      (proposal-level tree); op_hash в BundledConfirmation op_hashes[]

Cost per record:                      n/a
Time-bound sabotage analysis:
  Bytes per τ₁ от 1 sender:           1 × 3 438 B (rate-per-identity 1-op-per-τ₁)
  Bytes per τ₂ от 1 sender:           τ₂_windows × 3 438 B = 20 160 × 3 438 B ≈ 69 MB per sender per τ₂
  Pruning offset:                     все Anchor sender-а удаляются вместе с AccountRecord
                                      при `balance == 0` + 4τ₂ pruning

Sabotage asymmetry:                   в пользу сети (Anchor amortized через AccountChain TTL —
                                      нет orphan storage; sender pays own anchor accumulation
                                      через own AccountRecord lifecycle; fan-out limited через
                                      `Transfer` Mode B cooldown 1 per τ₂)

Lifecycle condition:                  amortized через AccountChain TTL (не отдельный mechanism)
Lifecycle threshold:                  все Anchor от account A удаляются когда AccountRecord A pruned
                                      (`balance == 0` + 4τ₂ inactivity)
[I-14] путь:                          2 (activity-based, derived от AccountRecord lifecycle)

Existing pruning consistent:          yes (no orphan Anchor possible by construction)
[I-14] compliance status:             closed
```

**Storage Card — Proposals chain**

```
Таблица:                              Proposals (header chain)
Operation создающая запись:           apply_proposal at window close (один per τ₁ window)
Платит creation cost:                 n/a ([I-15])
Размер записи (bytes):                3 722 B (proposal header per layout раздела «Proposal»:
                                      12 × 32 B хэш/Merkle/id полей + window_index 8 +
                                      protocol_version 4 + target 16 + fallback_depth 1 +
                                      signature 3309)
Secondary resources per record:       Fast Sync chain verification path; Anchor canonical-position
                                      proof chain reference до genesis

Cost per record:                      n/a
Growth analysis:
  Bytes per τ₁:                       1 × 3 722 B (consensus structure invariant — ровно
                                                    один header per окно, не амплифицируется
                                                    ресурсами атакующего)
  Bytes per τ₂:                       20 160 × 3 722 B ≈ 75 MB
  Bytes per 26 τ₂ (illustrative):     при genesis-калибровке D₀ на 60 кварцевых
                                       секунд per окно эмерджентно ≈ 525 600 окон
                                       × 3 722 B ≈ 1.96 GB (illustrative; никакая
                                       wall-clock привязка не нормативна,
                                       per [I-18])
  Bytes per 260 τ₂ (illustrative):    ≈ 19.56 GB при том же эмерджентном профиле
  Pre-condition:                      proposal_W обязан быть signed `proposer_node_id` из
                                      Node Table; ML-DSA-65 signature verify

Sabotage asymmetry:                   в пользу сети (rate=1/τ₁ — consensus structure
                                      invariant; атакующий не может создать дополнительные
                                      proposals независимо от hardware/budget; fallback
                                      cascade не множит количество cemented proposals —
                                      только один cemented header per окно)

Lifecycle condition:                  none by design (proposal headers необходимы для
                                                       Anchor canonical-position proof verification
                                                       до genesis + Fast Sync chain
                                                       verification)
Lifecycle threshold:                  n/a (permanent retention — design feature)
[I-14] applicability:                 N/A — proposals не user-driven; growth = consensus
                                            structure invariant (один header per τ₁
                                            производится самим механизмом консенсуса).
                                            Slow-bloat attack class предполагает user-driven
                                            операции с fan-out возможностью — здесь не
                                            применим категориально. [I-14] построен против
                                            slow-bloat от user-driven operations; proposal
                                            chain — другой класс ресурса (consensus-driven,
                                            rate-determined-by-structure).

Existing pruning consistent:          yes (no pruning by design — purposefully retained)
Compliance status:                    closed категориально (rate-bounded by consensus
                                                              structure; permanent retention
                                                              как design feature; [I-14]
                                                              applicability N/A)
```

Все 5 Storage Cards согласованы с [I-14] framework: AccountRecord / Anchor records / NodeTable / Candidate Pool — user-driven tables под scope [I-14] (один из трёх путей закрытия применён); Proposals chain — consensus-driven table вне scope [I-14] категориально (growth produced by consensus mechanism itself, не user operations). Cost-based фрагменты помечены `n/a` единообразно через [I-15] денежного отказа; защита для user-driven tables time-based.

Нарушение [I-14] = автоматический блокер mainnet.

**[I-15] Time-based scarcity.** Все защиты от спама, раздутия состояния, Sybil на ресурсы (fan-out на множество identities, dust-creation, keepalive удержание пустых записей) и Sybil на роль валидатора конструируются исключительно через **канонические time-based примитивы**.

Дефицитный ресурс протокола — **время**: VDF-непрерывность TimeChain, τ-окна, chain_length узла, activity аккаунта, sequential candidate VDF. Этот time-market встроен в консенсус как единственный объективный дефицит. Защиты через существующий дефицит (а) симметричны для всех участников независимо от Ɉ-holdings, (б) не дублируют логику существующих time-based ограничителей консенсуса, (в) не зависят от номинальной стоимости Ɉ.

Допустимые time-based примитивы:

- **Rate-per-identity** — одна операция на аккаунт за τ₁ (существующее правило `op_height` инкремента).
- **TTL через активность** — запись удаляется после `N_INACTIVE_*_WINDOWS` окон без cemented операций (существующий pruning AccountRecord 4τ₂; NodeTable 8τ₂).
- **Cooldown активации** — sender ограничен K активаций за τ₂ (per-account counter `last_account_creation_window`).
- **Chain-length requirement** — право на действие требует `sender.account_chain_length_snapshot >= threshold_windows`.
- **Seniority gating** — вес или приоритет пропорционален `chain_length` (лотерея узлов, wait period кандидатов).
- **Sequential VDF iteration count** — кандидатура узла требует sequential SHA-256 цепочки длины `vdf_chain_length × D` хэшей (существующая NodeRegistration защита, [Sybil-защита Шаг 2 + Adaptive VDF]). Sequential time — единственный неприобретаемый дефицит.
- **Canonical unpredictable-offline binding** — `cemented_bundle_aggregate(W-k)` в seed композициях (per [I-8]).

**Разграничение.** [I-15] применяется к задачам anti-spam, anti-bloat, state scarcity, Sybil на роль валидатора. Не применяется к:

- **Аппликативные сервисы** (никнеймы, премиум-функции, хранение, подписки) — реализуются прикладным слоем через прямые `Transfer` между аккаунтами; protocol-level пользовательских сервисов нет.

Различающий критерий: проблема «кто-то создаёт много записей потребляющих сетевые ресурсы без legitimate use» → time-based defenses (rate-per-identity, cooldown, TTL); проблема «кто-то претендует на роль валидатора без вложенного времени» → sequential VDF iteration count + chain_length-weighted влияние на консенсус. Аппликативные платежи — задача прикладного слоя поверх `Transfer`, не protocol-level concern.

Нарушение [I-15] = автоматический блокер mainnet для соответствующего механизма.

**[I-16] Out-of-band identity binding.** Публичный ключ аккаунта обязан иметь каноническое человекочитаемое представление — отпечаток аккаунта (`account_fingerprint`), детерминистически выводимый из публичного ключа аккаунта. Клиент обязан требовать подтверждённую вне канала связи сверку отпечатка перед первым зашифрованным сообщением между двумя аккаунтами. Клиент, инициирующий сессию end-to-end без out-of-band сверки, не соответствует протоколу.

Канонический вывод (authoritative):

```
h = SHA-256("mt-account-fingerprint" || account_pubkey)       # 32 B
индексы  = первые 6 × 11 = 66 бит h, big-endian, по 11 бит    # 6 × 11-bit
слова    = [Montana wordlist.txt[индекс_i]  for i in 0..5]    # 6 слов
account_fingerprint = слова соединённые через пробел
```

`Montana wordlist.txt` — authoritative словарь в файле `Протокол/Montana wordlist.txt`, 2048 слов (11 бит на слово). Размер отпечатка 66 бит — эквивалент safety number в Signal/WhatsApp (60 бит), коллизионная стойкость `2^33` на пару аккаунтов, преднамеренная подделка отпечатка требует `~2^66` попыток.

Обоснование: без out-of-band привязки идентичности первое рукопожатие уязвимо к подмене связки предварительных ключей на пути доставки (Sky ECC-class vector). Сверка отпечатка вслух / по QR / через доверенный вторичный канал устраняет доверие к тому же каналу, через который приходит связка ключей. Канонический вывод в протоколе, а не в приложении, предотвращает ситуацию, когда один слабый клиент становится универсальной щелью для всей сети.

Применение:

- Приложение-реализация протокола обязано блокировать отправку первого зашифрованного сообщения до подтверждённой сверки отпечатка.
- Последующие сообщения в той же сессии сверки не требуют.
- Смена публичного ключа аккаунта (`ChangeKey`) генерирует новый отпечаток; последующее взаимодействие требует новой сверки.

[I-16] нарушение = автоматический блокер mainnet для клиент-приложений.

**[I-17] Публичная аудиторская поверхность клиентского бинарника.** Каждая релизная сборка официального клиента Монтана обязана быть воспроизводимой байт-в-байт из публично опубликованного исходного кода любым независимым сборщиком. Криптографический хэш каждой релизной сборки публикуется в трёх независимых местах:

1. Через операцию Anchor от координационного аккаунта команды разработки (в сети Монтана, постоянно)
2. Как подписанный Git tag в публичном репозитории исходного кода
3. Как Anchor-подтверждения от независимых рецензентов, пересобравших бинарник из того же исходника

Протокол **не блокирует** подключение клиентов не прошедших проверку — это обеспечивает открытую экосистему альтернативных реализаций, пользовательских модификаций и исследовательских инструментов. Протокол обеспечивает **детективную поверхность** — любое расхождение между исполняемым бинарником и опубликованным исходным кодом обнаруживается независимым аудитом публично.

**Требования к клиентам:**

- Desktop и node клиенты обязаны поддерживать стандартную верификацию хэша через командную строку
- Все клиенты отображают self-hash в пользовательском интерфейсе для возможности ручной проверки
- Reproducible build обеспечивается сборочным процессом: любой независимый сборщик из публичного исходного кода получает байт-идентичный бинарник

**Цель инварианта:** переложить атаки на канал дистрибуции клиента из скрыто-исполнимого в публично-детектируемый класс. Расхождение бинарника с опубликованным хэшем становится публично наблюдаемым; экосистема аудиторов (независимые сборщики, журналисты, исследователи безопасности) имеет технические условия для раскрытия атаки.

**Обоснование детективного подхода:** превентивная блокировка подключения клиентов не прошедших проверку требует доверенного self-attestation (возможно только с аппаратным TEE, нарушение [I-5]) или централизованного whitelist (нарушение архитектурной децентрализации). Детективная поверхность решает задачу защиты от компрометации канала дистрибуции без нарушения инвариантов и без блокировки альтернативных реализаций.

[I-17] нарушение = автоматический блокер mainnet для официальных релизов клиента.

**[I-18] Отсутствие внешнего времени в протоколе.** В жизни протокола Монтана произошёл **один** исторический quartz-замер — на генезис-железе (Apple iMac 24-inch M1 2021, iMac21,1; Apple M1 base 4P+4E, 8 GB unified memory; macOS Sequoia 15.7.3 build 24G419, kernel Darwin 24.6.0; toolchain Rust 1.92.0 stable target aarch64-apple-darwin, release lto=fat opt-level=3 codegen-units=1; SHA-256 backend `sha2` crate v0.10.9 c ARM SHA-2 hardware extensions). Методология замера — три прогона по 10⁹ итераций цепочки `hash_{i+1} = SHA-256(hash_i)`, `hash_0 = [0u8; 32]`, машина idle, single-thread. Median rate 5.097280 MH/s. Calibrated `D₀ = 5.097 MH/s × 60 s = 305 836 793` хэшей; runtime-corrected `305 836 793 × (60 / 56.35) = 325 000 000` (округлено, hex `0x135F1B40`) — записано в Genesis Decree `protocol_params.D₀`.

После Genesis в протокольном коде и consensus state **запрещены**:

- чтение `CLOCK_REALTIME` (системного времени дня)
- чтение `CLOCK_MONOTONIC` в логике протокола
- зависимость от NTP, GPS, любых сетевых time-oracles
- подписанные объекты содержащие wall-clock метки
- адаптации, lifecycle-условия, тайм-ауты, ритм лотереи в физических секундах

Все длительности в протоколе выражаются **только** в количестве хэшей канонической TimeChain либо в номерах окон τ₁ / кратности τ₂. Каждый новый узел при запуске начинает вычислять TimeChain с current_window и участвует в подписании окон без локальной самокалибровки. Решение оператора «годится моё железо или нет» принимается им до запуска узла; внутри протокола такой проверки нет. Глобальный `D` адаптируется через `participation_ratio` per τ₂ (существующий механизм) — единственное средство влияния на `D` после Genesis.

**Scope [I-18]:** инвариант применяется к protocol code и consensus state (включая подписанные объекты, layouts, hash compositions). Network/transport layer (kernel-level keepalive, OS socket primitives) и operator tooling (мониторинг, дашборды, локальные benchmark до запуска узла) — outside scope, могут читать локальные часы операционной системы свободно.

[I-18] нарушение = автоматический блокер mainnet (любая зависимость протокола от внешнего/системного времени превращает Монтану в not-Montana — теряются canonical determinism [I-3] и независимость TimeChain [I-4]).

### Модель приватности

Протокол разделяет публичное и приватное одним принципом: **consensus state — публичен, данные пользователя — за пределами протокола**.

- **Публично (consensus state):** балансы, суммы переводов, отправители, получатели, window_index, node_id, chain_length. Это следствие [I-2]: финансовый слой открыт для верификации.
- **В протоколе, но без содержания:** Anchor содержит data_hash (32 байта). Что за этим хэшем — протоколу неизвестно.
- **За пределами протокола:** данные пользователя (фото, сообщения, файлы) хранятся на узле владельца. Шифрование, формат хранения, доступ — решения клиентского слоя. Сеть не хранит, не реплицирует и не видит эти данные. Ключ шифрования — у владельца. Без ключа данные на узле — шум.

Протокол не предоставляет privacy через криптографическое сокрытие (нет ring signatures, нет hidden amounts, нет stealth addresses — [I-6]). Приватность данных обеспечивается архитектурно: данные не попадают в протокол. Протокол видит 32 байта хэша и всё.

#### Уровни приватности пользователя

Реальный уровень приватности пользователя зависит от того, запущен ли у него собственный узел. Протокол определяет два состояния и гарантирует разный объём защиты в каждом.

**Account-only пользователь** — подключается к чужому узлу через IBT уровня 3 (account keypair). Работает без собственной инфраструктуры. Хостящий узел выступает посредником между пользователем и сетью.

**Оператор собственного узла** — запускает узел на своём оборудовании, подключает клиентское приложение к своему узлу локально. Узел — это и инфраструктура сети, и точка обслуживания владельца.

Сравнение того, что видно и кому в каждом из двух сценариев:

| Что наблюдается | Account-only через чужой узел | Свой узел |
|---|---|---|
| Содержимое сообщений | E2EE ML-KEM-768 — недоступно никому кроме собеседника | То же E2EE |
| Переводы: отправитель, получатель, сумма, окно | Публично по [I-2] — видит вся сеть | Публично по [I-2] — видит вся сеть |
| Факт публикации Anchor, его app_id и время | Публично — видит вся сеть | Публично — видит вся сеть |
| Содержимое Anchor (data) | Только хэш в сети, контент у владельца | То же |
| Граф связей: с кем пользователь начинает первую сессию | Hot path (известные контакты) — **приватно** через локальный кэш. Cold path (первый контакт) — **K=16 batch lookup** (~2–3 бита practical anonymity; см. «Batch Lookup Protocol») | **Приватно** — lookup происходит локально |
| Lookups: запрос pre-key bundle, прикладные id-резолвы | Hot path — **приватно** через локальный кэш. Cold path — **K=16 batch** (~2–3 бита) | **Приватно** — резолвится из локальной реплики consensus state |
| Polling Blob Buffer: какие очереди слушает клиент | Long-term session identification **closed** через label rotation per τ₁ + catch-up через RangeSubscribe. Residual leaks (session count, activity timing, per-τ₁ collusion) — permanent architectural limits для account-only, требуют Light-Node-at-Home | **Приватно** — локальные подписки |
| IP-адрес пользователя | Виден хосту + ISP пользователя | Виден всей сети (node_id ↔ endpoint в Node Table) + ISP |
| Онлайн-присутствие оператора (оператор = confirmer) | Не применимо | Видно сети через подписи BundledConfirmation |
| Глобальный наблюдатель internet-backbone | Timing correlation возможна | Timing correlation возможна, но без посредника-хоста |

#### Границы защиты — что протокол не закрывает по дизайну

Три архитектурных выбора сознательно делают полную приватность невозможной. Это не пробел реализации, а явный scope протокола.

**Финансовый граф — публичен по [I-2].** Все cemented Transfer содержат `sender`, `receiver`, `amount` в открытом виде. Это цена прозрачной бухгалтерии, публичного аудита supply и отсутствия hidden inflation. Monero-style приватность транзакций архитектурно невозможна. Financial mixers — задача внешних прикладных систем, не протокола.

**IP оператора узла — публичен.** P2P сеть по определению требует connectivity между известными endpoints. node_id узла связан с его адресом в Node Table. Сокрытие IP оператора требовало бы mix-net поверх P2P — прямое нарушение [I-6].

**Paternы онлайн-активности оператора — видны.** Подписи BundledConfirmation и VDF_Reveal публичны. Оператор, подписывающий bundles, раскрывает свой рабочий график. Для оператора-активиста это наблюдаемо.

**Global passive adversary traffic correlation — возможна.** Противник, наблюдающий весь internet-backbone, может связать исходящий трафик клиента с cemented operations через timing. Защита требует mix-net с random delays, что нарушает [I-6] и Corollary I-3.a (детерминизм). Выход за рамки protocol-level защиты — достигается внешними инструментами (Tor, VPN) как opt-in пользователя.

**Тип использования через app_id в Anchor.** Anchor-операции со статичным `app_id = SHA-256("mt-app" || app_name)` публикуют тип приложения открыто в cemented state — виден всей сети по [I-2], не только хосту пользователя. Через известный реестр имён приложений `app_id` декодируется в семантическое значение (мессенджер, профиль, ключи, конкретная платформа). Этот класс утечки одинаков для всех пользователей независимо от типа подключения — свой узел устраняет third-party хоста как наблюдателя, но не скрывает `app_id` от остальной сети. Messenger-сессии не затронуты — используют ротируемые метки очередей per τ₁ (клиентский слой, App spec). Затронуты низкочастотные статичные Anchor: profile, encryption-keys, pre-key bundles, niche приложения со статичным app_name. Mainstream приложения получают анонимность через толпу; niche приложения идентифицируемы по volume + timing patterns.

**Тайминг cemented operations.** Каждая подтверждённая операция в AccountChain (Transfer, Anchor, ChangeKey, CloseAccount) привязана к каноническому `window_index` — виден всей сети по [I-2]. Наблюдатель цепочки строит temporal profile аккаунта через canonical window_index: распределение активности по окнам, периоды отсутствия (паузы активности интерпретируются как offline), корреляция с внешними событиями (операция в окне `W_X` через `N` окон после publicly-known event в окне `W_X − N` связывает аккаунт с этим событием). Этот класс утечки одинаков для всех пользователей независимо от типа подключения — свой узел устраняет third-party хоста, но операция после cementing распространяется по gossip всей сети. Защита на protocol level архитектурно невозможна без нарушения инвариантов: batch publishing с delay ломает UX операций (Transfer ждёт подтверждения минуты вместо секунд); cover operations (fake Transfer / Anchor) нарушают [I-2] semantically (засоряют открытую бухгалтерию) и не защищают от intersection analysis по provenance; mix-net с random delays нарушает [I-6] (regulatory compatibility) и Corollary I-3.a (детерминизм). Mainstream users получают анонимность через толпу (миллионы операций в каждом окне); users с identifiable activity patterns — идентифицируемы временным анализом. Опциональная защита вне протокола: Tor/VPN для IP-level (не скрывает window_index, но скрывает network origin); разделение ролей между несколькими аккаунтами (разные аккаунты для разных типов активности); сознательное поведение «как толпа» (избегать уникальных temporal patterns).

#### Правильная коммуникация уровня приватности пользователю

Любое клиент-приложение обязано явно информировать пользователя о текущем уровне приватности:

- При подключении через чужой узел — показать: «Используется сторонний узел. Хост видит ваш IP, паттерны активности и с кем вы начинаете переписку. Для полной приватности metadata запустите собственный узел.»
- При подключении к собственному узлу — показать: «Подключено к вашему узлу. Metadata приватна локально; финансовые операции публичны по дизайну сети.»
- Скрытие или маркетинговое преуменьшение ограничений защиты — нарушение духа инварианта честности по отношению к пользователю. Обещание «абсолютной приватности» недопустимо: модель защиты Монтаны bounded и должна быть явной.

Практические паттерны настройки собственного узла (Light-Node-at-Home, Phone-to-Own-Node pairing) и UI-индикация уровня — описаны в спецификации приложения Монтаны.

### Языковая изоляция

В нормативном тексте спецификации Монтана допустимые термины для описания протокольных объектов, счётчиков, периодов или интервалов: `window`, `tick`, `epoch`, `cycle` — определённые через window counts. Термины физического времени (`second`, `minute`, `hour`, `day`, `week`, `month`, `year`) применяются только в advisory контекстах клиентского слоя и в описании транспортного уровня (implementation guidance).

---

## Канонический порядок

Первичный продукт протокола — канонический порядок событий, реализованный как глобальная цепь `TimeChain` от Genesis Decree. Каждое окно `τ₁` — это `D` последовательных SHA-256 итераций от предыдущего канонического anchor; число `D` фиксируется в Genesis Decree и может адаптироваться через participation-ratio feedback (см. раздел «Адаптация D»).

Свойства канонического порядка (монотонность, однозначность, проверяемость, независимость) — см. раздел 1 «Каноническая временная координата».

Победитель окна регистрирует одно окно канонического порядка и получает `reward_moneta(W) = EMISSION_moneta` (см. раздел «Эмиссия»).

### Definition канонической координаты

```
canonical_coordinate(W) := W
```

Единственное каноническое определение временной координаты в протоколе. Номер окна `W` — это каноническая позиция события в упорядоченной последовательности. Всё остальное — производные или advisory вычисления клиентского слоя.

### Гранулярность

Атом канонического порядка — одна SHA-256 итерация. Окно канонического порядка — `D` атомов. Произвольный интервал — `N` окон. Все три уровня выражены в канонических числах, на которые bit-exact согласны все узлы.

Физическая длительность одной итерации зависит от оборудования узла (наносекунды — десятки наносекунд на обычном процессоре). Физическая длительность окна зависит от скорости железа узла и от участия сети. Физическая длительность — свойство конкретного наблюдателя, выводимое на клиентском слое.

### Calibration target — historical genesis quartz measurement

`D₀ = 325 000 000` (hex `0x135F1B40`) выведен из **единственного исторического quartz-замера** на генезис-железе (Apple iMac 24-inch M1 2021, idle, single-thread, ARM SHA-2 hardware extensions). Median single-thread rate 5.097280 MH/s × 60 кварцевых секунд = 305 836 793 хэшей; runtime-corrected до 325 000 000 учитывая VDF interleaving с consensus работой. Полная derivation methodology — Указ Генезиса → «Калибровка D₀». После Genesis протокол не читает никакие часы (per [I-18]).

Три уровня времени разделены:

- **Protocol-нормативное определение окна.** Окно = `D` последовательных SHA-256 итераций. Детерминированный invariant per [I-3]. Никаких binding claims о внешнем времени.
- **Per-узел длительность окна.** Зависит от hardware конкретного узла; emergent property его кварца и архитектуры процессора, не входит в consensus state. Variance между классами hardware (genesis-class commodity, cloud VPS, ASIC) достигает ×20+. Operator выбирает железо до запуска узла.
- **Canonical window count.** Синхронен между узлами через VDF chain length. Узлы быстрее median простаивают между окнами; узлы медленнее median отстают и догоняют через VDF computation. Никакой узел не сообщает свою wall-clock длительность в сеть.

**Single point of derivation truth.** Quartz-замер для `D₀` произошёл **до** запуска сети на генезис-железе, методология и hardware profile зафиксированы в Указе Генезиса для воспроизводимости любым независимым ревьюером. После Genesis post-genesis adaptation — через `participation_ratio` feedback в `D` per τ₂ (см. раздел «Адаптация D»), без обращения к каким-либо часам.

**Внешнее время — задача клиентского слоя.** Перевод canonical window count в любые внешние временные шкалы (секунды, часы, дни) — interpretation клиентского слоя. Binding claim протокола только на canonical window count и derivation формулы (`τ₁_windows`, `τ₂_windows`).

### Оракул времени

Канонический `window_index` каждого proposal — верифицируемая координата события. Внешние системы используют канонический порядок Монтаны как систему отсчёта:

- **Проставление временной метки.** `H(document)`, привязанный к `window_index`, — криптографическое доказательство существования в позиции `W` канонической последовательности.
- **Упорядочивание.** Два события, привязанные к разным `window_index`, имеют доказуемый канонический порядок между собой.
- **Якорение.** Внешний протокол якорится в каноническом порядке Монтаны для независимой верификации порядка своих событий.

Перевод `window_index → физическое время` в любых внешних стандартах (UTC, TAI, GPS Time) является задачей клиентского слоя. Монтана производит каноническую последовательность окон; внешний наблюдатель выбирает собственный метод привязки `window_index` к своим локальным временным единицам.

`TimeChain` хранится навсегда. Канонические координаты верифицируемы любым узлом в любой момент.

---

## Криптография

Два фундаментальных примитива с разделёнными ролями:

- **SHA-256** — консенсус (TimeChain), lottery endpoints, адреса, Merkle-деревья, хэширование
- **ML-DSA-65** (Module-Lattice Digital Signature Algorithm, NIST FIPS 204 finalized August 2024, NIST security level 3; reference implementation production-ready) — подписи операций аккаунтов и proposals узлов

SHA-256 обеспечивает квантовую устойчивость консенсуса: алгоритм Гровера сокращает безопасность с 256 до 128 бит. ML-DSA-65 обеспечивает математическую постквантовую устойчивость подписей на основе module-lattice проблем (Module-LWE и Module-SIS).

Вспомогательные композиции поверх SHA-256 — HMAC-SHA-256 (RFC 2104), PBKDF2-HMAC-SHA-256 (RFC 8018 §5.2), HKDF-Expand (RFC 5869 §2.3) — используются в client-side деривации ключей из мнемоники (см. «Ключи»). Они не вводят независимых криптографических предположений, являются стандартными композициями уже принятого SHA-256.

Для клиентского шифрования сообщений применяется ML-KEM-768 (FIPS 203) — post-quantum KEM, используется вне consensus поверхности (см. Application Layer).

ML-DSA-65 (NIST level 3) и ML-KEM-768 (NIST level 3) формируют единый security level всего PQ-стека Монтаны. Оба primitive финализированы в FIPS 203/204 в августе 2024, оба основаны на module-lattice проблемах — структурное единство криптоповерхности по [I-7].

Других независимых криптографических примитивов в протоколе нет — финансовый слой публичен, приватность данных обеспечивается на уровне приложений через Anchor.

### Подписи — ML-DSA-65

Module-lattice подпись (Dilithium-3, NIST level 3). Stateless, многоразовая, deterministic либо randomized variant — Монтана использует deterministic вариант (RND = 0x00 × 32 в FIPS 204 §3.7) для бит-точной воспроизводимости подписи при идентичных (sk, message). Публичный ключ закрепляется за аккаунтом при создании и используется для всех последующих операций.

| Компонент | Размер |
|-----------|--------|
| Приватный ключ | 4 032 B |
| Публичный ключ | 1 952 B |
| Подпись | 3 309 B |

Поле suite_id в формате блока обеспечивает миграцию подписи без изменения модели состояния. Активация новой схемы требует protocol upgrade. Активная схема на момент запуска: ML-DSA-65.

**Единый security level.** ML-DSA-65 + ML-KEM-768 — оба NIST security level 3, оба основаны на module-lattice проблемах (Module-LWE / Module-SIS), оба финализированы в FIPS 204 / FIPS 203 в августе 2024. PQ-стек Монтаны имеет единый security level 3 для подписи и шифрования. Структурное единство криптоповерхности по [I-7].

### Подписанная область, идентичность и агрегация — универсальные правила

Для любого подписанного объекта протокола (UserObject, ControlObject, Proposal header, BundledConfirmation, VDF_Reveal, любой future-вводимый подписанный класс) действуют три универсальных правила.

**Правило R1 — Signed scope.** Каждый подписанный объект имеет canonical_bytes с полем signature последним. Сообщение, подаваемое в ML-DSA-65 sign и verify:

```
signed_scope(obj) = canonical_bytes(obj)[0 .. |canonical_bytes| - signature_size(signer_suite_id(obj))]

signature = ML-DSA-65.sign(sk, signed_scope(obj))
verify    = ML-DSA-65.verify(pk, signed_scope(obj), signature)
```

Внешний SHA-256 слой над signed_scope не применяется — ML-DSA использует SHAKE-256 при формировании challenge внутри (FIPS 204 §3.7), дополнительное хэширование избыточно и нарушает [I-7].

**signer_suite_id(obj)** определён таблицей:

| Класс объекта | signer_suite_id |
|---------------|-----------------|
| Transfer (Mode A/B), ChangeKey, Anchor, CloseAccount | `AccountTable[sender].suite_id` |
| NodeRegistration | `payload.candidate.suite_id` |
| Proposal header | `NodeTable[proposer_node_id].suite_id` |
| BundledConfirmation | `NodeTable[confirmer_node_id].suite_id` |
| VDF_Reveal | `NodeTable[node_id].suite_id` |

**signature_size(suite_id)** — детерминированная функция:

| suite_id | Схема | signature_size |
|----------|-------|----------------|
| 1 | ML-DSA-65 | 3 309 B |

Future suites — через protocol version upgrade с explicit записью в эту таблицу.

Для ChangeKey подписывает **старый** ключ (AccountTable[sender] до apply), не новый. new_pubkey в payload определяет ключ для проверки будущих операций, signature_size для текущей ChangeKey определяется старым suite_id.

**Правило R2 — Stable identifier.** Канонический 32-байтовый идентификатор подписанного объекта в любой consensus hash composition (op_hashes[], frontier_hash, Merkle leaves in proposal-level trees, sort keys, chain linking proposal_hash):

```
identifier(obj) = SHA-256(class_domain(obj) || signed_scope(obj))
```

Class domain separators (единый реестр всех криптографических domain strings протокола):

| Класс | class_domain | Применение |
|-------|--------------|------------|
| UserObjects (0x01..0x04) | `"mt-op"` | identifier (SHA-256 hash, формула выше) |
| NodeRegistration (0x11) | `"mt-nodereg"` | identifier (SHA-256 hash) |
| Proposal header | `"mt-proposal"` | identifier (SHA-256 hash) |
| BundledConfirmation | `"mt-bundle"` | identifier (SHA-256 hash) |
| VDF_Reveal | `"mt-vdf-reveal"` | identifier (SHA-256 hash) |
| Operator Proof of Possession | `"mt-operator-pop"` | signature input (ML-DSA-65 sign/verify, формула в инвариантах NodeRegistration) |

Колонка «Применение» определяет какую функцию принимает domain string как первый компонент input:
- **identifier (SHA-256)** — domain — префикс input для SHA-256 hash construction по формуле `identifier(obj) = SHA-256(class_domain || signed_scope)`. Output — 32-byte hash используемый в consensus как канонический идентификатор объекта.
- **signature input (ML-DSA-65)** — domain — префикс input для ML-DSA-65 signature construction. Формула применения определяется в инвариантах конкретного механизма (для PoP: `ML-DSA-65("mt-operator-pop" || node_pubkey, operator_secretkey)`). Output — 3309-byte signature, не identifier.

Identifier вычисляется от signed_scope (не от wire encoding с signature) — свойство стабильности по конструкции независимо от choice варианта ML-DSA-65 (deterministic либо randomized). Монтана использует deterministic вариант ML-DSA-65 (RND = 0x00 × 32 в FIPS 204 §3.7) — при идентичных (sk, message) подпись бит-точно одна и та же; identifier остаётся тем же при любом переподписании. Свойство также покрывает любую future signature scheme добавленную через protocol upgrade с potentially randomized variants.

Термины `op_hash`, `proposal_hash`, `bundle_hash`, `nodereg_hash`, `reveal_hash` обозначают `identifier(obj)` с соответствующим identifier-class_domain (первые 5 строк реестра). Термин `frontier_hash(account)` = identifier(последней cemented операции sender-а). Термин `operator_pop` — отдельная сигнатурная конструкция, использует signature-input class_domain (6-я строка реестра), не identifier.

**Правило R3 — Consensus seed aggregation.** Для любого aggregate feeding в consensus-critical seed output (lottery endpoint, selection sort_key, admission ordering, weight distribution, emission, ranking) aggregate input — только (signer_node_id, context), без content объектов и без signatures:

```
aggregate_for_seed(S, agg_domain, empty_domain, context) :=
  если S пустой:  SHA-256(empty_domain || context)
  иначе:          SHA-256(agg_domain || concat_sorted_by_node_id(signer_node_id(s) for s in S) || context)
```

Inputs строго:
- signer_node_id каждого участника (canonical из registered pubkey)
- context — temporal anchor (обычно window_index as u64 LE)

Inputs строго исключены:
- Content объекта (payload fields, op_hashes[], reveal_hashes[])
- Signatures (σ — даже при deterministic ML-DSA-65 включение в seed создаёт зависимость от signature size, нарушая R3 minimal-input principle)
- identifier (Правило R2 — содержит signed_scope с потенциально attacker-choose-able content)

Grinding surface для single participant: **ноль**. signer_node_id детерминирован через hash от registered pubkey (committed при registration, не меняется); context canonical; composition of S emergent через quorum дynamics (single participant не контролирует кто ещё попал в cemented set).

**Правило R4 — Разделение Rules R2 и R3.** Identity (R2) и seed aggregation (R3) — разные use cases с разными grinding requirements.

R2 identifier корректно используется в:
- op_hashes[] в BundledConfirmation (commitment к what was attested)
- frontier_hash (account chain linking)
- Merkle leaf values в proposal-level trees (included_bundles_root, included_reveals_root)
- sort keys в apply_proposal ordering

R3 aggregate_for_seed корректно используется в:
- cemented_bundle_aggregate (unpredictable-offline binding [I-8])
- любой future aggregate feeding в consensus seed

**R3 никогда не использует R2 identifier как input** — включение signed_scope через identifier оставило бы grinding knob через attacker-choose-able content в signed_scope.

### Адреса

Формат: `mt` + Base58(account_id + checksum).

Account_id = SHA-256("mt-account" || suite_id || pubkey). Стабильный идентификатор аккаунта. Смена ключа или схемы подписи выполняется через ChangeKey без изменения account_id — account_id привязан к первому pubkey, а текущий ключ хранится в состоянии аккаунта.

**Инвариант derivation.** Проверка `account_id == SHA-256("mt-account" || suite_id || pubkey)` происходит **один раз** при создании AccountRecord — для user-аккаунта на settle `Transfer` Mode B (apply at window close, payload содержит `receiver_pubkey` и `suite_id`); для operator-аккаунта на cementing Selection event (NodeRegistration содержит `operator_pubkey`, derivation проверяется при cementing записи в Candidate Pool). После создания account_id — каноничный ключ записи, формула не пересчитывается. Доказательство derivation навсегда сохранено в proposal с финализированной операцией создания. Любой аудитор может replay из proposal history. Original_pubkey не дублируется в Account Table — integrity гарантируется неизменностью proposal chain.

Поле `suite_id` в Account Table — **current** (мутируется ChangeKey синхронно с current_pubkey), используется для верификации текущих подписей. Original suite_id зафиксирован только в исторической записи операции создания AccountRecord в proposal chain.

---

## Account Chain (Block Lattice)

Каждый аккаунт имеет собственную цепочку операций. Перевод — одна операция в цепочке отправителя. Зачисление получателю — детерминированно после финализации. Цепочки аккаунтов полностью независимы.

### Реестр типов объектов

Type byte (первый байт canonical_bytes операции) — global unique across all classes использующих **полиморфный wire slot** (разные типы в одном формате блока, dispatch по первому байту).

```
UserObjects (полиморфный слот):
  0x02  Transfer
  0x03  ChangeKey
  0x04  Anchor
  0x0B  CloseAccount

ControlObjects (полиморфный слот):
  0x11  NodeRegistration

Reserved (future protocol versions):
  0x05, 0x08, 0x09, 0x0A — ранее выделены под operations прикладного слоя; type bytes
                          освобождены, не выделяются вновь (сохранение совместимости с
                          archived proposals имеющими эти opcodes как unknown user-payload).
  0x20-0x2F   consensus meta-objects
  0x30-0x3F   governance / MIP objects
  0x40-0xFF   unallocated
```

Type byte `0x01` **не выделен**. AccountRecord создаётся автоматически: для аккаунта пользователя — при первом входящем `Transfer` с положительным amount на ещё не существующий receiver; для operator-аккаунта — атомарно с записью узла в Node Table при cementing Selection event для NodeRegistration. Самоинициация создания аккаунта невозможна; отдельного opcode активации не существует.

**Signed objects без type byte** (каждый в собственном dedicated wire slot, disambiguation через class_domain Правила R2):

- Proposal header — `"mt-proposal"` class domain
- BundledConfirmation — `"mt-bundle"` class domain
- VDF_Reveal — `"mt-vdf-reveal"` class domain

Cross-class signature confusion structurally невозможна: для polymorphic classes первый байт signed_scope различается (0x01..0x04, 0x11); для non-polymorphic class_domain в identifier обеспечивает разделение hash spaces, а signed_scope разных classes имеет несовпадающую структуру (SHA-256 collision resistance negligibly мала).

### Типы операций

**Универсальная форма операции:**

```
type      (1B)  | prev_hash (32B) | payload (variable) | signature (3309B)
```

Все операции — этот шаблон. `prev_hash` связывает операции в цепочку аккаунта. `signature` — ML-DSA-65 владельца над `signed_scope(op)` (см. Правило R1). `payload` зависит от типа. Все операции начинают payload с `sender (32B account_id)` — узел проверяет `Account Table[sender].frontier_hash == prev_hash` и `signature валиден для current_pubkey` за O(1).

Особый случай — операция первой signed receiver-операции после создания AccountRecord (через `Transfer` Mode B либо через atomic activation на Selection event). Receiver's AccountChain ещё пуст: `AccountTable[receiver].frontier_hash == 0x00...00` (initialized при создании записи). Первая signed receiver-op имеет `prev_hash == 0x00...00` — она становится genesis receiver's chain. После apply frontier_hash обновляется до `identifier(op)`.

`op_hash` в любом consensus контексте (op_hashes[] в BundledConfirmation, frontier_hash, sort_key apply_proposal, H(Anchor) в Anchor verification) = `identifier(op)` с class domain `"mt-op"` (см. Правило R2). Identifier вычисляется от signed_scope без signature — стабилен по конструкции независимо от choice варианта подписи (deterministic либо randomized).

**Transfer** — публичный перевод. Единый opcode для двух режимов: receiver уже существует в Account Table → пополнение баланса; receiver не существует → атомарное создание AccountRecord и зачисление amount. Branch определяется наличием `payload.link` в Account Table на момент apply.

```
Transfer (receiver уже существует — Mode A):
  type       1B   <- 0x02 Transfer
  prev_hash 32B
  payload   80B   <- sender (32B) || link (32B receiver) || amount (16B u128 moneta)
  signature 3309B
  Итого:   ~3 422 B

Transfer (receiver не существует — Mode B, создание AccountRecord):
  type             1B    <- 0x02 Transfer
  prev_hash       32B
  payload       2034B    <- sender (32B) || link (32B receiver) || suite_id (2B)
                            || receiver_pubkey (1952B ML-DSA-65) || amount (16B u128 moneta)
  signature     3309B
  Итого:       ~5 376 B
```

Wire-формат payload в Mode B расширен полями `suite_id` и `receiver_pubkey` — необходимы для derivation receiver account_id и для записи `current_pubkey` в новый AccountRecord. Различение режимов на стороне узла: после parse payload узел читает `link`; если `Account Table[link]` существует — Mode A (короткий payload 80 B); если не существует — Mode B (расширенный payload 2034 B). Sender обязан выбрать формат соответствующий ожидаемому состоянию receiver на момент apply; несоответствие формата состоянию = reject.

**Инварианты Transfer (общие для обоих режимов):**

- `type == 0x02`
- `payload.sender` существует в Account Table
- `Account Table[sender].frontier_hash == prev_hash` (dependency rule на settled state окна W-1)
- `payload.sender != payload.link` (self-transfer **запрещён** — открывает рост account_chain_length через no-op переводы себе, см. «Верификация баланса»)
- `payload.amount > 0` (нулевой перевод **запрещён**)
- `Account Table[sender].balance >= payload.amount` (достаточный баланс)
- Signature ML-DSA-65 valid для `Account Table[sender].current_pubkey` над signed_scope (Правило R1)

**Инварианты Transfer Mode A** (receiver уже в Account Table):

- `Account Table[payload.link]` существует
- payload длина = 80 B (без `suite_id` и `receiver_pubkey`)

После apply (Mode A):
- `Account Table[sender].balance -= payload.amount`
- `Account Table[sender].frontier_hash = identifier(op)`
- `Account Table[payload.link].balance += payload.amount`

**Инварианты Transfer Mode B** (receiver не существует, создание AccountRecord):

- `Account Table[payload.link]` **не** существует
- payload длина = 2034 B (расширенный формат с `suite_id` и `receiver_pubkey`)
- `payload.suite_id` соответствует активной схеме подписи (на момент запуска: `0x0001` = ML-DSA-65); прочие — **reject** `UnsupportedSuite`
- `payload.link == SHA-256("mt-account" || payload.suite_id || payload.receiver_pubkey)` (binding: account_id корректно derived из receiver_pubkey)
- **Cooldown создания AccountRecord** per [I-15]: `current_window >= Account Table[sender].last_account_creation_window + τ₂_windows` (sender выполняет максимум один Transfer Mode B за τ₂; нарушение — **reject** `AccountCreationCooldownNotElapsed`). Исключение — sender с `last_account_creation_window == 0` (никогда не создавал AccountRecord этим путём) проходит без проверки.

После apply (Mode B):
- `Account Table[sender].balance -= payload.amount`
- `Account Table[sender].frontier_hash = identifier(op)`
- `Account Table[sender].last_account_creation_window = current_window`
- `Account Table[payload.link] = new_record(balance = payload.amount, current_pubkey = payload.receiver_pubkey, suite_id = payload.suite_id, frontier_hash = 0x00...00, op_height = 0, account_chain_length = 0, account_chain_length_snapshot = 0, last_account_creation_window = 0, is_node_operator = 0, creation_window = current_window, last_op_window = current_window)`

`receiver_pubkey` обязателен в Mode B payload — без него невозможен binding verify `link == H(domain || suite_id || pubkey)`. Sender узнаёт `receiver_pubkey` offline (QR-код, сообщение, nickname lookup). Sender **не** владеет private key receiver; `AccountTable[link].current_pubkey` устанавливается из payload и впредь служит для верификации подписей receiver.

Receiver's AccountChain остаётся пустой после Mode B apply (`frontier_hash = 0x00...00`). Первая signed op receiver'а имеет `prev_hash == 0x00...00` и становится genesis receiver's chain.

Открытые поля: отправитель (через frontier index по prev_hash), получатель, сумма, баланс после операции (через Account Table). Псевдонимность на уровне account_id. Финансовая приватность — задача приложений (микшеры, payment channels), не протокола.

**ChangeKey** — смена ключа или схемы подписи:

```
type       1B   <- 0x03 ChangeKey
prev_hash 32B
payload 1986B   <- sender (32B) || new_suite_id (2B) || new_pubkey (1952B)
signature 3309B  <- подписано старым ключом
Итого:  ~5 328 B
```

**Инварианты ChangeKey:**

- `type == 0x03`
- `payload.sender` существует в Account Table
- `Account Table[sender].frontier_hash == prev_hash`
- `payload.new_suite_id` соответствует активной схеме подписи; прочие значения — **reject** (UnsupportedSuite)
- Signature ML-DSA-65 valid для **старого** `Account Table[sender].current_pubkey` над signed_scope (Правило R1; подпись старым ключом **до** apply; `new_pubkey` становится current только после apply)

**Anchor** — криптографический якорь (привязка данных ко времени):

```
type       1B   <- 0x04 Anchor
prev_hash 32B
payload   96B   <- sender (32B) || app_id (32B) || data_hash (32B)
signature 3309B
Итого:   ~3 438 B
```

**Инварианты Anchor:**

- `type == 0x04`
- `payload.sender` существует в Account Table
- `Account Table[sender].frontier_hash == prev_hash`
- Signature ML-DSA-65 valid для `Account Table[sender].current_pubkey` над signed_scope (Правило R1)

Anchor — запись data_hash в цепочку аккаунта с привязкой к timechain_value окна финализации, без перемещения средств. Стоимость Anchor выражается через 1-op-per-τ₁ rate-limit аккаунта (single time-based primitive). Приватность данных приложения обеспечивается тем что в сеть попадает только хэш — содержимое хранится у владельца зашифрованным.

**Anchor lifecycle — persistent design через [I-15].** Anchor — **persistent** запись в AccountChain sender'а (не ephemeral event). Это сохраняет лёгкую верификацию: любая full node может предоставить inclusion proof для данного Anchor через стандартный Merkle path AccountChain без обращения к архивным узлам.

Защита от раздутия state через Anchor spam соответствует [I-15] time-based scarcity:

1. **Rate-per-identity (существующее):** одна операция per аккаунт per τ₁ — sender не может сделать более 20 160 Anchor записей за τ₂.
2. **Amortization через AccountChain TTL:** Anchor записи — часть AccountChain владельца; при pruning inactive аккаунта (`balance == 0` + 4τ₂ без активности) **все** Anchor удаляются вместе с AccountRecord — не остаются orphan'ами в state.
3. **Cooldown создания AccountRecord (Пункт 3):** ввод нового аккаунта для Anchor-фарминга ограничен `1 Transfer Mode B per sender per τ₂` — fan-out на массу дешёвых Anchor-аккаунтов экспоненциально медленный (binary tree expansion).

Quantify: атакующий с одного активного sender может за τ₂ создать до `τ₂_windows` Anchor записей (по ≈3 438 B каждая под ML-DSA-65 signature, суммарно `τ₂_windows × 3 438 B` на sender). Для поддержания `M` Anchor'ов постоянно активными (избежание pruning 4τ₂) требуется `M / τ₂_windows` senior senders, генерирующих `M` подписанных operations за τ₂ — видимо в сетевой статистике signature verifications + gossip bandwidth. Storage damage: `M × 3 438 B` per node (≈3.4 GB для M = 10⁶ Anchors) — покрыто time-based защитой того же класса что AccountRecord.

Защита Anchor — через существующие time-based паттерны [I-15]: rate-per-identity (1-op-per-τ₁) + amortization + cooldown активации.

**Service economy реализуется прикладным слоем.** Никнеймы — apps реализуют через `Anchor` либо собственные registries. Платные сервисы (звонки, видеосвязь, премиум-функции, хранение, подписки) — apps принимают `Transfer` оплату напрямую от пользователя на аккаунт-провайдер сервиса; протокол только обеспечивает каноническую финализацию `Transfer`. Type bytes `0x05 / 0x08 / 0x09` зарезервированы как unused (см. реестр типов объектов).

### Верификация баланса

Открытое арифметическое сравнение. Узел проверяет:

```
sender != receiver
amount > 0
sender.balance >= amount
```

`sender != receiver` запрещает self-transfer — иначе атакующий мог бы наращивать account_chain_length каждое окно через no-op переводы себе.

При settle (apply at window close):

```
sender.balance   -= amount
receiver.balance += amount
```

Баланс обновляется не при cement (quorum event), а в конце окна при батчевом apply. Между cement и settle операция необратима но баланс ещё не изменён. Никаких proofs, никакой криптографии помимо подписи и хэша.

### Анти-инфляция

Чеканка из воздуха невозможна через локальный инвариант на каждом state transition.

**Per-user-operation invariant.** Каждое применение пользовательской операции обязано удовлетворять `Σ delta_balance == 0`:

```
Transfer:    sender.balance -= amount, receiver.balance += amount  → Σ = 0
Transfer (Mode B): sender.balance -= amount, receiver.balance += amount (создание AccountRecord) → Σ = 0
ChangeKey:   только обновление current_pubkey                       → Σ = 0
Anchor:      только запись data_hash                                → Σ = 0
```

**Per-proposal invariant.** Каждый финализированный proposal окна τ₁ обязан удовлетворять `delta_supply == EMISSION_moneta`:

```
apply_proposal step 2 (Монтана emission, W = proposal.window_index, winner = winner_{W-1}):
  r = EMISSION_moneta
  # Лотерея single-class: winner всегда узел (см. «Аккаунты не участвуют в лотерее»).
  operator_account = Node Table[winner_id].operator_account_id
  Account Table[operator_account].balance += r

delta_supply за proposal = EMISSION_moneta ровно один раз
```

O(1) проверка на каждое state transition (одно чтение константы из ProtocolParams). Глобальный инвариант `Σ balance == supply_moneta(window_index)` истинен по индукции от genesis при условии что каждый переход поддерживает per-operation invariant.

`supply_moneta` — pure function от номера окна (state-поля не нужно): `supply_moneta(W) = EMISSION_moneta × (W + 1)`. Closed-form O(1), supply растёт монотонно линейно, никогда не убывает.

```
genesis state (аксиома):   window_index не определён,  Σ balance = 0
первое окно (W = 0):       supply_moneta(0) = EMISSION_moneta = 13 × 10⁹ nɈ
окно W (любое):            supply_moneta(W) = EMISSION_moneta × (W + 1)
```

Никаких откатов cemented операций не требуется — каждое cemented локально валидно по конструкции.

**τ₂ sanity check.** Дополнительная проверка раз в τ₂: пересчёт `Σ balance` по всей Account Table и сравнение с `supply_moneta(window_index) = EMISSION_moneta × (window_index + 1)`. Не load-bearing для финализации — служит для обнаружения багов реализации. Расхождение = немедленная остановка узла, дамп state для расследования.

### Перевод

Перевод на несуществующий account_id — отклоняется. Получатель обязан существовать в Account Table до получения перевода.

### Валюта Монтана

Победитель окна W регистрирует одно окно канонического порядка: `EMISSION_moneta` Монет. При финализации proposal окна W+1 выплата применяется (one-window lag):

```
r = EMISSION_moneta
# single-class лотерея, winner — всегда узел.
operator_account = Node Table[winner_id].operator_account_id
Account Table[operator_account].balance += r
```

Атомарное обновление баланса. Узел получает награду через привязанный operator_account (зафиксирован при NodeRegistration). Никаких отдельных coinbase-структур, никаких отдельных таблиц эмиссии. Зачисление есть состояние Account Table.

```
Публичное (верифицируемо всеми):
  Монтана:           reward_moneta(W) = EMISSION_moneta = 13 × 10⁹ nɈ
                     (см. раздел «Эмиссия»)
  Supply audit:      supply_moneta(W) = EMISSION_moneta × (W + 1) — closed-form,
                     pure function от номера окна, state-поля не нужно
  Winner:            winner_id в proposal header
  Все балансы:       Account Table
  Все переводы:      цепочки операций аккаунтов
  VDF:               TimeChain values, lottery endpoints, подписи
```

Псевдонимность на уровне account_id. Финансовая приватность — задача приложений: микшеры, payment channels, off-chain settlements.

### Двойная трата

Каждый аккаунт имеет одну цепочку. Две операции с одним prev_hash = equivocation.

**Без конфликта:** операция → узлы валидируют → публикуют confirmation → quorum → cemented (необратимо, в пределах текущего τ₁; emergent ~0.3 секунды на genesis-калибровке, illustrative). Баланс обновляется при settle (apply at window close).

**При конфликте (equivocation):**

1. Узел получает операцию X с prev_hash = H. Узел уже видел операцию Y с prev_hash = H, Y ≠ X. Форк обнаружен. Обе операции помечаются как equivocated.
2. Если одна операция уже cemented (quorum до обнаружения конфликта) — cemented необратимо. Вторая отклоняется.
3. Если ни одна не cemented — узлы продолжают собирать confirmations для обеих. Если одна набирает quorum → cemented, вторая отклоняется.
4. Если через 13 окон ни одна не набрала quorum → обе отклоняются окончательно. Аккаунт продолжает с последней cemented операции. Владелец отправляет новую операцию.

Equivocation создаётся только владельцем аккаунта (требуется подпись). Третья сторона не может создать equivocation для чужого аккаунта. Стимул: двойная трата = потеря обеих операций.

### Антиспам

Антиспам через время: право на операцию = доказанное время существования аккаунта (account_chain_length + last_account_creation_window для cooldown создания новых AccountRecord).

#### Приоритет операции

```
account_age = current_window - creation_window
priority(op) = account_age × windows_since_last_op
```

`account_age` — возраст аккаунта в окнах. Растёт линейно. Некупуемый. `windows_since_last_op` — окна с последней операции аккаунта. Сбрасывается при каждой операции. Спамер обнуляет приоритет с каждой операцией — самонаказание.

При переполнении ёмкости сети — операции с наименьшим приоритетом ожидают следующего окна.

#### Бакеты по account_age

Изоляция спама. Каждый аккаунт может опубликовать максимум одну операцию за окно τ₁ (dependency rule). При переполнении сети (больше операций в мемпуле чем пропускная способность окна) — бакеты определяют **приоритет включения**. Round-robin по бакетам: одна операция из бакета 0, одна из бакета 1, ..., по кругу. Спам в бакете 0 не вытесняет операции из бакетов 1-3.

```
Бакет 0:  account_age < 4τ₂
Бакет 1:  account_age 4τ₂ — 16τ₂
Бакет 2:  account_age 16τ₂ — 64τ₂
Бакет 3:  account_age 64τ₂+
```

Границы бакетов = 4^N × τ₂. Все аккаунты: максимум 1 операция за τ₁. Бакет определяет приоритет при переполнении, не потолок TPS.

Новый аккаунт — бакет 0 с момента создания. 1 операция за τ₁. Вход без ожидания: получил перевод → сразу можешь отправить.

#### Throughput на аккаунт

Каждая цепочка аккаунта: 1 операция за τ₁. Правило per-account по проектированию — одно окно, один шаг в личной цепочке времени пользователя. Ритм τ₁ достаточен для любых задач одного пользователя в сети.

Одно правило закрывает конструкцией пять задач сразу:

1. **Spam protection by time-pacing.** Рейт операций аккаунта ограничен структурой состояния (1-op-per-τ₁ через op_height инкремент), не очередью узла. Узлам не нужно отбивать флуд от одного аккаунта — следующая операция этого аккаунта попросту не существует до закрытия окна. Time-pacing на уровне state machine — единственный rate-limit primitive.

2. **Детерминизм apply_proposal (инвариант [I-3]).** N>1 операций одного аккаунта в одном окне потребовали бы intra-window ordering. Любое такое правило обязано быть либо subjective (mempool-зависимое — автоматическая дыра), либо дополнительной canonical hash composition в consensus-critical output (расширение поверхности [I-8]). При N=1 проблема отсутствует: выбор операции окна единственный.

3. **Dependency rule.** Операция аккаунта в окне ссылается на frontier_hash из settled state предыдущего окна. N>1 операций одного аккаунта в одном окне потребовали бы intra-window ordering — либо subjective (mempool-зависимое, нарушение [I-3]), либо canonical hash composition (расширение поверхности [I-8]). При N=1 проблема отсутствует: порядок операции единственный.

4. **Семантика chain_length как веса.** `account_chain_length` = количество окон τ₁ с операцией, то есть окон присутствия. Вес в консенсусе измеряется временем, а не числом операций. N>1 операций за окно разорвало бы связь «вес = присутствие во времени» и открыло Sybil-накачку веса через спам операций в собственной цепочке.

5. **Бинарная разрешимость double-spend.** Правило «67% active_chain_length за одну операцию по одному prev_hash» работает потому что конфликт двоичен: либо A, либо B. N>1 операций за окно делает конфликт multi-way и требует дополнительного механизма выбора между тремя и более ветвями за окно — блокер liveness и новая поверхность атаки.

Объём данных за одну операцию не ограничен ритмом: Anchor содержит Merkle root над произвольным числом off-chain записей, привязанных к одному окну.

Сетевой throughput складывается параллелизмом независимых цепочек аккаунтов и ограничен пропускной способностью канала узла и размером proposal, не правилом per-account.

Высокочастотные сценарии sub-τ₁ (микроплатежи, streaming) находятся вне scope протокола: введение throughput-слоя ниже τ₁ разрушит каждую из пяти перечисленных гарантий. Применения, которым нужна такая частота, строятся на других субстратах.

Спамер с 1000 новых аккаунтов: 1000 операций за τ₁ в бакете 0. Бакет 0 получает 1/4 от round-robin. Изолирован. Аккаунты в бакетах 1-3 не замечают.

---

## Состояние сети

Глобальное состояние = Account Table + Node Table + Candidate Pool. Награда константна (`reward_moneta(W) = EMISSION_moneta`), читается из ProtocolParams и не требует state-полей.

Layout таблиц (Account Table, Node Table, Candidate Pool) ниже.

```
Account Table (запись на аккаунт):
  account_id              32B     <- = SHA-256("mt-account" || suite_id || pubkey)
  balance                 16B     <- u128 moneta, открыт
  suite_id                 2B
  is_node_operator         1B     <- 1 если аккаунт привязан как operator узла
  frontier_hash           32B     <- хэш последней операции в цепочке; 0x00...00 сразу после создания AccountRecord до первой signed receiver-операции
  op_height                4B     <- количество операций в цепочке
  account_chain_length     4B     <- количество уникальных окон τ₁ с операцией (длина AccountChain), live
  account_chain_length_snapshot 4B <- snapshot account_chain_length на последнюю τ₂ boundary
  current_pubkey        1952B     <- ML-DSA-65 pubkey владельца аккаунта; для user-аккаунта — receiver_pubkey из Transfer Mode B; для operator-аккаунта — operator_pubkey из NodeRegistration
  creation_window          4B     <- окно создания AccountRecord (cementing Transfer Mode B либо Selection event)
  last_op_window           4B     <- окно последней операции (для приоритета)
  last_account_creation_window   4B     <- u32, окно последнего Transfer Mode B (создание AccountRecord) посланного этим sender-ом; 0 если не создавал. Используется для cooldown rule «1 Transfer Mode B per τ₂» per [I-15]

Node Table (запись на узел):
  node_id                          32B     <- SHA-256("mt-node" || node_pubkey), верифицируемо
  node_pubkey                    1952B
  suite_id                          2B
  operator_account_id              32B     <- account_id куда зачисляется Монтана при победе узла; неизменен после регистрации
  start_window                      8B     <- u64, окно регистрации (первое окно присутствия в Node Table)
  chain_length                      8B     <- u64, позиция узла в NodeChain: = 1 при активации, +1 при cemented BundledConfirmation в окне. Инвариант: chain_length ≥ 1 для любого узла в Node Table
  chain_length_snapshot             8B     <- u64, = chain_length - chain_length_checkpoint[oldest]; используется в лотерее
  chain_length_checkpoints        48B     <- 6 × u64, checkpoint-ы chain_length на последних 6 τ₂-boundaries
  last_confirmation_window          8B     <- u64, window_index последнего окна с cemented BundledConfirmation

Candidate Pool (запись на кандидата):
  node_id                          32B     <- SHA-256("mt-node" || node_pubkey)
  node_pubkey                    1952B
  suite_id                          2B
  operator_pubkey                1952B     <- ML-DSA-65 pubkey владельца operator-аккаунта (используется для atomic создания AccountRecord на Selection event если ещё не существует)
  operator_account_id              32B     <- account_id куда зачисляется Монтана при победе; = SHA-256("mt-account" || suite_id || operator_pubkey)
  proof_endpoint                   32B     <- endpoint VDF цепочки (длина vdf_chain_length)
  W_start                          8B     <- u64, окно начала VDF (заявлено кандидатом)
  vdf_chain_length                 8B     <- u64, длина VDF цепочки от candidate_vdf_init до proof_endpoint (в "окнах" по D хэшей)
  registration_window               8B     <- u64, окно cementing NodeRegistration
  expires                           8B     <- u64, registration_window + 3 × τ₂_windows

```

**Active node predicate (derived).** Узел считается активным если опубликовал cemented BundledConfirmation за последние 2τ₂:

```
active(node, W) = (W - node.last_confirmation_window) <= 2 × τ₂_windows
```

Predicate вычисляется из `last_confirmation_window` и текущего `window_index`. Применяется в quorum, confirmation_threshold, лотерее, валидации selection event.

### Корень состояния

Merkle-дерево глобального состояния. Три подкорня обновляются при применении операций (apply_proposal и apply at window close):

```
state_root = SHA-256("mt-state-root" || node_root || candidate_root || account_root)

node_root:       Merkle root Node Table, обновляется при selection event (регистрация),
                 chain_length increment (apply step 3.5), pruning узлов на τ₂.
candidate_root:  Merkle root Candidate Pool, обновляется при cementing NodeRegistration
                 (добавление), selection event (удаление выбранных), expiry (удаление просроченных).
account_root:    Merkle root Account Table, обновляется батчем при apply at window
                 close (все cemented операции окна применяются к state, затем
                 account_root пересчитывается).

Все три root соответствуют settled state (после apply at window close).
Порядок node_root → candidate_root → account_root отражает направление
зависимостей: узлы — активные участники, кандидаты — будущие узлы, аккаунты — финансовый слой.
Domain separator `mt-state-root` отличён от `mt-merkle-node` — hash spaces пересекаться не могут.
```

**Структура Account Table Root:**

Sparse Merkle tree глубины 256, индексированный по `account_id`:

```
leaf_hash(account)        = SHA-256("mt-merkle-leaf" || serialize(account_record))
internal(left, right)     = SHA-256("mt-merkle-node" || left || right)
empty_leaf                = 0x00 × 32

account_root = root of sparse Merkle tree over Account Table
```

Обновление одного аккаунта пересчитывает ровно `log₂(N)` хэшей пути от листа к корню — для N=10⁹ аккаунтов это 30 SHA-256 вычислений (~60 µs CPU).

**Структура Node Table Root:** аналогично, sparse Merkle tree по `node_id`. Размер сети ≤ 10⁵ узлов → пути ~17 хэшей.

**Canonical serialization — single source of truth.** Определения полей каждой таблицы (Node Table, Account Table, Candidate Pool) задают canonical byte-for-byte сериализацию каждой записи. Эта сериализация используется одновременно для (1) вычисления leaf_hash в Merkle tree, (2) хранения на диске, (3) передачи через Fast Sync snapshot. Любое изменение record format требует одновременного обновления canonical encoding во всех трёх путях использования. Fast Sync автоматически следует за canonical encoding — см. раздел Fast Sync «Полнота сериализации snapshot».

**Структура Candidate Pool Root:** sparse Merkle tree глубины 256, индексированный по `node_id`. Empty root = `empty_internal(256)` (authoritative значение см. раздел «Genesis State Hash» строка с binding `empty_internal(256)`).

Каждый узел в Node Table — участник сети. Узел существует в таблице = участвует.

Все sort keys фиксированной длины. Побайтовое лексикографическое сравнение. Две реализации с одинаковыми данными строят одинаковое дерево и получают одинаковый State Root.

State Root коммитится в заголовке каждого proposal τ₁. `account_root`, `node_root` и `candidate_root` соответствуют settled state после apply at window close — все cemented операции окна W применены к таблицам перед сборкой proposal.

**Структура proposal-level Merkle roots.** Поля заголовка proposal `control_root`, `included_bundles_root`, `included_reveals_root` (см. раздел «Proposal») строятся как **тот же canonical sparse Merkle tree глубины 256** что используется для state-уровня (Account / Node / Candidate Pool). Reuse того же primitive — единое определение `leaf_hash` / `internal` / `empty_internal` через domain separators `mt-merkle-leaf` / `mt-merkle-node` (см. выше). Никаких отдельных Merkle конструкций для proposal-уровня не вводится ([I-7] минимальная криптографическая поверхность).

**Set semantics, не sequence.** В отличие от ordered Merkle tree (например Bitcoin block Merkle где порядок транзакций задаёт структуру), proposal-level Merkle roots реализуются как **set indexed by canonical key** — порядок включения объектов в окно не влияет на root, root зависит только от содержимого set. Любая независимая реализация при том же canonical filtered set получает byte-exact тот же root. Слово «список» в narrative описании поля (например «Merkle root списка...» в разделе Proposal) обозначает множество включаемых объектов с canonical filter, не упорядоченную последовательность.

**Canonical key для каждого корня:**

```
control_root:          ключ = nodereg_hash    (R2 identifier NodeRegistration);
                       значение = leaf_hash(serialize(control_object))
                       где serialize даёт canonical_bytes объекта согласно его class.
                       Для будущих ControlObject типов канонический ключ
                       определяется одновременно с введением opcode объекта.

included_bundles_root: ключ = confirmer_node_id (signer BundledConfirmation,
                       canonical из NodeTable);
                       значение = leaf_hash(serialize(bundle_metadata))
                       где bundle_metadata = (confirmer_node_id, bundle_hash)
                       canonical-encoded.

included_reveals_root: ключ = reveal_author_node_id (signer VdfReveal,
                       canonical из NodeTable);
                       значение = leaf_hash(serialize(reveal_metadata))
                       где reveal_metadata = (reveal_author_node_id, reveal_hash)
                       canonical-encoded.
```

**Empty marker — единый.** Для всех трёх proposal-level roots при пустом set: `root = empty_internal(256)` (authoritative значение см. раздел «Genesis State Hash» — та же константа что для пустого `genesis_candidate_root`, переиспользование per [I-10] SSOT, не дублирование). Реализация конструктивно даёт это значение через стандартную SMT процедуру построения над пустым набором; explicit hex не дублируется в этом разделе.

**Single-leaf поведение.** Для set из одного элемента: root вычисляется через стандартную SMT процедуру (вставка одного `(key, leaf_hash)` в пустое дерево даёт path из 256 уровней `internal(...)` хэшей, на каждом уровне sibling = `empty_internal(level)` cached константа). Никаких shortcuts «root = leaf bytes напрямую» — правило uniform для любого размера set.

**Inclusion proof для proposal-level Merkle.** Структурно идентичен state-уровню — путь из ~17 различных хэшей (для размера set ≤ 10⁵) против 239 cached `empty_internal(level)` константных значений. Любой узел с access к canonical filtered set может предоставить inclusion proof для конкретного включения и любой узел без полного set может verify proof против `*_root` поля заголовка proposal.

#### Inclusion proof

Любой cemented аккаунт может предоставить доказательство существования в state:

```
proof = Merkle path длиной log₂(N) (~30 хэшей для N=10⁹)
verify(proof, account_record, account_root):
  reconstruct path bottom-up; compare с account_root
```

Доказательство верифицируется против `account_root` любого финализированного proposal начиная с окна когда состояние было обновлено. Не нужны архивы операций — текущее состояние самодостаточно.

#### Pruning

На τ₂ boundary применяется pruning неактивных аккаунтов:

```
Удалить все записи Account Table где:
  balance == 0                                            <- нулевой баланс
  AND last_op_window + 4τ₂ <= current_window              <- нет активности 4τ₂ (52 000 окон)
  AND is_node_operator == 0                               <- не привязан как operator узла
  AND нет cemented NodeRegistration в control_set         <- нет pending привязки
      ожидающего apply, ссылающегося на этот account_id
```

Пустой аккаунт без активности 4τ₂ — удаляется, кроме:
- Operator-аккаунтов уже зарегистрированных узлов (`is_node_operator == 1`)
- Аккаунтов на которые ссылается cemented NodeRegistration ожидающий apply

**[I-14] compliance через [I-15].** Защита от раздутия state достигается time-based путём: cooldown `1 Transfer Mode B per sender per τ₂` (см. инварианты Transfer Mode B) ограничивает rate создания новых AccountRecord, tree-expansion атакой на 10⁶ записей требует `⌈log₂(10⁶)⌉ = 20 τ₂`, keepalive-атака через постоянную активность видна статистически и упирается в 1-op-per-τ₁ rate limit. Существующее pruning (`balance == 0` + 4τ₂) закрывает dormant bloat. Все три защитных механизма — канонические time-based примитивы [I-15].

Без второго исключения возможна race: NodeRegistration cemented (operator валиден), pruning применился до apply этого NodeRegistration → аккаунт удалён → apply отклонён. Защита: pruning не трогает аккаунты, на которые есть cemented pending registration.

Каждое удаление пересчитывает соответствующий путь в Merkle tree (logarithmic). Pruning детерминирован, автоматичен, каноничен.

**Recovery semantics.** Воссоздание pruned аккаунта через новый `Transfer` Mode B (либо через повторное появление в Selection event если речь об operator-аккаунте) с тем же receiver_pubkey создаёт **новую цепочку**: frontier_hash начинается заново, op_height сбрасывается в 1, account_chain_length = 0. Старые prev_hash references на цепочку до pruning отклоняются — цепочка удалена из текущего state. История переводов до pruning не восстанавливается из текущего Account Table, но навсегда сохранена в proposals. Восстановление истории возможно через scan архива proposals.

---

## Двигатели

Односторонний поток зависимостей: TimeChain → NodeChain → AccountChain → AccountTable.

TimeChain — глобальные часы (ход времени, VDF). NodeChain — присутствие узла (последовательность cemented BundledConfirmation). AccountChain — присутствие аккаунта (дискретные операции). AccountTable — состояние счёта.

### TimeChain VDF — осциллятор

Конкретный subtype: **iterated SHA-256 chain** — `T_r = SHA-256(T_{r-1})`, sequential by construction. Верификация требует пересчёта O(D × chain_length) последовательных SHA-256 (нет succinct proof размером O(log T) как у Wesolowski ePrint 2018/623 или Pietrzak ePrint 2018/627). Выбор подтипа обоснован [I-1]: production-grade succinct VDF существуют только на classical groups (RSA, class groups мнимо-квадратичных полей) — ломаются Shor's algorithm; PQ-secure succinct VDF на момент Genesis — research grade без production audit history.

Первичный продукт протокола. Непрерывная последовательная SHA-256 цепочка — цифровой осциллятор канонического порядка:

```
T_r = SHA-256^D(T_{r-1})
```

D — количество последовательных хэшей за одно окно τ₁. Каждый хэш — один тик осциллятора. D хэшей — одно колебание. TimeChain продвигается по расписанию окон. Для фиксированного индекса r значение T_r совпадает у всех честных узлов. Каждый узел вычисляет TimeChain независимо — результат детерминирован.

TimeChain не зависит от состояния, транзакций и поведения отдельных узлов. Даже при отказе всего Account слоя часы продолжают тикать.

### NodeChain — последовательность присутствия узла

Доказательство присутствия конкретного node_id в каждом окне. Каждое окно с cemented BundledConfirmation = одно звено NodeChain. chain_length — позиция узла в NodeChain: = 1 при активации (Genesis для bootstrap, selection event для нового узла), +1 при каждом cemented BundledConfirmation. Инвариант: chain_length ≥ 1 для любого узла в Node Table — гарантирует корректность знаменателей в weighted_ticket лотереи и в seniority_term.

NodeChain не является VDF-цепочкой. Узел доказывает присутствие публикацией BundledConfirmation (подтверждение операций сети), не вычислением per-node VDF. Один VDF на всю сеть (TimeChain) — достаточен.

NodeChain зависит от TimeChain (якорится через window_index). TimeChain не зависит от NodeChain.

**Liveness узла и сетевое включение.** Рост chain_length требует cementing BundledConfirmation через confirmation threshold 67% active_chain_length. При стандартной BFT-assumption (≥67% active_chain_length честны и достижимы по P2P) BC активного узла cemented в каждом окне участия. Изоляция узла от confirmers (eclipse, network partition, propagation failure) останавливает рост chain_length независимо от локальной работы узла. Это свойство consensus-механизма, не свойство узла: chain_length измеряет подтверждённое сетью присутствие, не локальную CPU-работу.

### AccountChain — персональная цепочка аккаунта

Криптографическое доказательство присутствия конкретного account_id в дискретных моментах. Каждое звено — финализированная операция аккаунта (Transfer Mode A, Transfer Mode B исходящий от данного аккаунта, Anchor, ChangeKey, CloseAccount). Linking через `prev_hash` (хэш предыдущей операции в цепочке аккаунта). Якорится в TimeChain через timechain_value момента финализации каждой операции.

Длина AccountChain — количество окон τ₁ в которых аккаунт имел cemented операцию:

```
account_chain_length(account, W) = | { w : w <= W, аккаунт имел cemented операцию в окне w } |
```

Dependency rule ограничивает аккаунт одной операцией за окно τ₁ — поэтому длина AccountChain совпадает с числом окон активности. Поле `account_chain_length` хранится в Account Table, обновляется при apply операции:

```
on_operation_applied(operation, window W):
  account = operation.account_id
  account.account_chain_length += 1
  account.last_op_window = W
  account.op_height += 1
```

**Параллелизм NodeChain и AccountChain:**

| Свойство | NodeChain | AccountChain |
|----------|-----------|--------------|
| Источник | node_pubkey | account_pubkey |
| Идентификатор | node_id | account_id |
| Тип присутствия | машинное | человеческое |
| Ритм | непрерывный (каждое окно) | дискретный (окно с операцией) |
| Длина | chain_length (окна с BundledConfirmation) | account_chain_length (окна с операцией) |
| Единица длины | окно τ₁ | окно τ₁ |
| Накопление | автоматически при публикации BundledConfirmation | через активность пользователя |
| Защита от подделки | подпись ML-DSA-65 | подпись ML-DSA-65 |
| Защита от Sybil | τ₂ окон VDF + selection event | накопление окон требует активности |

Узел доказывает присутствие публикацией BundledConfirmation в каждом окне. Аккаунт — операцией. Оба механизма верифицируемы, оба производят запись на одной шкале времени.

AccountChain зависит от TimeChain напрямую. AccountChain не зависит от NodeChain по построению.

### VDF Reveal и лотерея

В лотерее участвует **только один класс** субъектов — узлы (через VDF_Reveal). Аккаунты в лотерее не участвуют (см. раздел «Аккаунты не участвуют в лотерее» ниже). Каждый узел производит ticket, взвешенный по длине своей NodeChain.

Confirmers (~100 узлов с наибольшим chain_length) публикуют BundledConfirmation для финализации окна. Все узлы с weighted_ticket_node < target публикуют VDF_Reveal для лотереи. VDF_Reveal цементируется через BundledConfirmation: confirmers включают полученные VDF_Reveals в свои bundles наряду с UserObjects и ControlObjects. Cement threshold тот же — 67% active_chain_length. Proposer извлекает только cemented reveals — дискреция над лотереей = ноль.

#### Класс 1: узлы

После завершения VDF окна W каждый узел вычисляет свой ticket.

**Real-valued form (commentary):**
```
ticket_node = -ln(endpoint_node / 2^256)

seniority_term = min(chain_length / 13, chain_length_snapshot)
lottery_weight = chain_length_snapshot + seniority_term

weighted_ticket_node = ticket_node / lottery_weight
```

**Integer form (authoritative, per [I-9]):**
```
Input:
  endpoint_node: [u8; 32]        (big-endian u256 interpretation)
  chain_length: u64              (absolute, ≥ 1 по инварианту chain_length ≥ 1)
  chain_length_snapshot: u64     (≥ 1 по DS-2)

Output:
  weighted_ticket_node: u128     (Q64.64, сравнивается через u128::cmp)

Algorithm:
  seniority_term_u64 = min(chain_length / 13u64, chain_length_snapshot)
    # Integer division toward zero (unsigned u64)
    # chain_length < 13 ⇒ seniority_term = 0
  lottery_weight_u64 = chain_length_snapshot + seniority_term_u64
    # Overflow: chain_length_snapshot ≤ 120960 (6τ₂), seniority ≤ snapshot, sum ≤ 2 × 120960 ⇒ safe u64
  ticket_q64_128 = ln_q64(endpoint_node)
    # ln_q64: [u8;32] → u128 Q64.64 — см. «Integer log algorithm» ниже
  weighted_ticket_node = ticket_q64_128 / (lottery_weight_u64 as u128)
    # u128 / u128 integer division toward zero

Comparison:
  weighted_ticket_i < weighted_ticket_j ⟺ u128-native less-than.

Binding test vectors (byte-exact; все используют ln_q64 = 0x4f60bd6fe6504646 от TV3 endpoint раздела «Integer log algorithm»):

  # N1 typical
  chain_length = 1000, chain_length_snapshot = 500
  → seniority_term = 76, lottery_weight = 576
  → weighted_ticket_node = 0x000000000000000000234770A382CE58

  # N2 boundary (DS-2 floor: weight = 1)
  chain_length = 1, chain_length_snapshot = 1
  → seniority_term = 0, lottery_weight = 1
  → weighted_ticket_node = 0x00000000000000004F60BD6FE6504646

  # N3 seniority cap (cap at snapshot)
  chain_length = 1_000_000, chain_length_snapshot = 10
  → seniority_term = 10 (capped), lottery_weight = 20
  → weighted_ticket_node = 0x000000000000000003F80978CB840383

  # N4 max chain_length boundary
  chain_length = 2^64 - 1, chain_length_snapshot = 120960
  → seniority_term = 120960 (capped at snapshot_max = 6τ₂), lottery_weight = 241920
  → weighted_ticket_node = 0x000000000000000000001580E0B1AED0

  # N5 seniority threshold (chain_length = 13 = первый порог где seniority_term ≥ 1)
  chain_length = 13, chain_length_snapshot = 1
  → seniority_term = 1, lottery_weight = 2
  → weighted_ticket_node = 0x000000000000000027B05EB7F3282323

Conformance status: closed (binding test vectors выше).
```

`chain_length_snapshot` — количество окон с cemented BundledConfirmation за последние 6τ₂ (120 960 окон ≈ 84 дня при τ₁ ≈ 60 с). Вычисляется через checkpoint-механизм: на каждой τ₂-boundary фиксируется checkpoint chain_length; snapshot = chain_length - checkpoint_6τ₂_ago. Хранится 6 checkpoint-ов (48B на узел). Обновляется на τ₂-boundary (шаг 3.6 apply_proposal).

`seniority_term` — добавка за накопленный абсолютный chain_length, ограниченная сверху размером snapshot (cap). Делитель 13 — mathematical derivation: target T_cap = 3 × T_year ≈ 1 577 880 окон, snapshot_max = 6τ₂ = 120 960, divisor = 1 577 880 / 120 960 ≈ 13. Cap = snapshot: максимальное преимущество старожила ≈ 2x относительно новичка с полным snapshot. При chain_length < 13 seniority_term = 0 (целочисленное деление): первые 13 окон после регистрации lottery_weight = snapshot.

**Инвариант DS-2 (lottery_weight floor).** Для любого узла N, участвующего в лотерее окна W (active(N, W) = true): `lottery_weight(N, W) ≥ 1`. Деление `ticket / lottery_weight` в формуле weighted_ticket_node гарантированно определено.

Обоснование через composition временных порогов:
- `active_predicate = 2τ₂` (26 000 окон): неактивные узлы исключены из лотереи
- `pruning_idle_windows = 4τ₂` (52 000 окон): полностью неактивные узлы удалены из Node Table
- `chain_length_snapshot window = 6τ₂` (120 960 окон): горизонт снапшота

Ordering `2τ₂ < 4τ₂ < 6τ₂` гарантирует: узел либо active (публикует BC → chain_length растёт → snapshot ≥ 1), либо inactive (исключён из лотереи), либо pruned (удалён из Node Table до того как snapshot мог бы упасть до 0). Сценарий «active узел с snapshot = 0» невозможен по построению.

Инвариант ОБЯЗАТЕЛЕН для enforcement в apply_proposal: при вычислении weighted_ticket_node валидатор проверяет `lottery_weight > 0`. Нарушение = protocol violation, proposal отклоняется. Нарушение указывает на баг в pruning или active_predicate — consensus critical.

Разделение весов:
- **Лотерея (эмиссия):** `lottery_weight = chain_length_snapshot + seniority_term`. Недавняя работа (snapshot) доминирует, longevity даёт bounded bonus.
- **Quorum (безопасность):** абсолютный `chain_length`. Старожилы доминируют в финализации.

Endpoint узла вычисляется детерминированно из канонических данных:

```
endpoint_node(W) = SHA-256(
  "mt-lottery" ||
  T_r(W) ||
  cemented_bundle_aggregate(W-2) ||
  node_id ||
  window_index
)
```

Где:
- `T_r(W)` — TimeChain VDF output окна W (каноничен, одинаков у всех узлов).
- `cemented_bundle_aggregate(W-2)` — агрегат подписей cemented BundledConfirmation окна W-2 (см. раздел BundledConfirmation). Lookback на 2 окна: cemented set окна W-2 зафиксирован в proposal_{W-1}, канонически финализирован к концу окна W. Все узлы используют одно значение.

Endpoint верифицируем за O(1) — один SHA-256, плюс lookup `cemented_bundle_aggregate(W-2)` из уже финализированного state.

**Grinding resistance.** Атакующий с VDF hardware advantage способен пре-вычислить `T_r(W)` на много окон вперёд. Но `cemented_bundle_aggregate(W-2)` содержит ML-DSA-65 подписи будущих confirmers — их privкey не у атакующего, aggregate непредсказуем offline. Grinding по node_id (выбор keypair с favorable future endpoints) не работает: endpoint зависит от canonical-но-непредсказуемого компонента. Горизонт grinding схлопывается до уже cemented (публично известного) окна W-2, где keypair уже зафиксирован.

Если weighted_ticket_node < target — узел кандидат и публикует VDF_Reveal:

```
VDF_Reveal:
  node_id          32B
  window_index      8B     <- u64 LE, индекс τ₁ (унифицирован с ProposalHeader.window_index)
  endpoint         32B     <- SHA-256("mt-lottery" || T_r(W) || cemented_bundle_aggregate(W-2) || node_id || window_index)
  signature      3309B     <- ML-DSA-65 над signed_scope(reveal) (Правило R1);
                              проверяется Node Table[node_id].node_pubkey
Итого:       3381B (= 32 + 8 + 32 + 3309)
```

`reveal_hash` = `identifier(reveal)` с class domain `"mt-vdf-reveal"` (Правило R2).

**Инварианты VDF_Reveal:**

- `node_id` существует в Node Table и активен в окне `window_index` (`last_confirmation_window` в пределах active predicate)
- `window_index` равен текущему окну лотереи (reveal не может относиться к произвольному окну)
- `endpoint == SHA-256("mt-lottery" || T_r(window_index) || cemented_bundle_aggregate(window_index - 2) || node_id || window_index_le)` — каноничный (верифицируем сравнением)
- `weighted_ticket_node(node_id, window_index) < target(window_index)` — узел прошёл порог кандидатства (иначе reveal не имеет лотерейного смысла, reject)
- Один VDF_Reveal per `(node_id, window_index)` — повторный отклоняется (equivocation)
- Signature ML-DSA-65 valid over signed_scope(reveal) против `Node Table[node_id].node_pubkey` (Правило R1)

Любой активный узел может стать кандидатом лотереи — lottery_weight основан на недавней работе (snapshot 6τ₂), старожилы получают bounded seniority bonus.

#### Аккаунты не участвуют в лотерее

Аккаунты **не участвуют** в лотерее эмиссии (Лестница суверенности — design choice — см. «Два пути участия»). Единственный protocol-level earning path — node lottery. Пользователи-аккаунты используют сеть (Transfer, Anchor, ChangeKey, CloseAccount); прикладные сервисы оплачиваются через прямые `Transfer` приложениям-провайдерам. Заработок Ɉ через lottery возможен только после перехода к роли оператора узла (Шаг 1 Лестницы суверенности).

Обоснование design choice:

1. **[I-5] commodity hardware.** Scarce contribution — узел (uptime, validation, gossip, hosting). Account activity — abundant ресурс (любой смартфон). Эмиссия направляется на scarce contribution, не на abundant.
2. **[I-7] минимальная крипто-поверхность.** Single reward path (node lottery) минимизирует audit surface — одна формула winner-а, один binding vector, одна ветвь apply_proposal.
3. **[I-10] SSOT.** Один reward path (node lottery) исключает дрифт между параллельными формулами эмиссии.
4. **Лестница суверенности — ясность.** Single emission target (только node) делает переход к роли оператора единственным protocol-level earning шагом.

Поле `account_chain_length_snapshot` присутствует в AccountRecord как seniority-метрика активности аккаунта; используется прикладным слоем как anti-Sybil сигнал в собственных allocation-задачах. На уровне протокола поле читается только τ₂ snapshot-ом и не влияет на консенсусные веса.

Экономическая модель — монотонная эмиссия только узлам через `reward_moneta(W)`. Реальная стоимость Ɉ определяется демандом прикладной экосистемы (`Transfer` оплаты приложениям-провайдерам сервисов).

#### Definition winner-а (Lookback Leadership)

Winner окна W-1 определяется при cementing proposal окна W. Proposer окна W = winner окна W-2 (канонически известен из cemented state).

**Механика:**

1. Окно W-1 завершается: confirmers публикуют BundledConfirmation_{W-1} (операции окна W-1 + VDF_Reveals окна W-2), кандидаты публикуют VDF_Reveal_{W-1}, аккаунты публикуют операции.
2. `proposer_W = winner_{W-2}` (канонически определён из proposal_{W-1}).
3. Окно W начинается. Confirmers получают VDF_Reveals_{W-1} через P2P и включают их в BundledConfirmation_W наряду с операциями окна W. VDF_Reveal идентифицируется по `window_index = W-1`.
4. VDF_Reveal_{W-1} cemented когда confirmers с суммарным chain_length ≥ 67% active_chain_length включили его в свои BundledConfirmation_W. Cement status каноничен — каждый узел отслеживает его независимо по P2P bundles.
5. Proposer_W собирает BundledConfirmation-ы окна W-1 и cemented set:
   ```
   included_bundles_{W-1} = BundledConfirmation-ы окна W-1 из view proposer-а
                            (суммарный chain_length ≥ 67% active_chain_length)
   included_reveals_{W-1} = VDF_Reveal-ы окна W-1, cemented через
                            BundledConfirmation окна W (67% active_chain_length)
   ```
6. Из included_reveals_{W-1} извлекаются все node endpoints (только node candidates; cemented account operations из included_bundles_{W-1} релевантны для apply_proposal по балансам и account_chain_length increment, но НЕ участвуют в лотерее).
7. `winner_{W-1} = argmin(weighted_ticket_node)` среди cemented VDF_Reveal узлов-кандидатов окна W-1.
8. Proposer_W публикует proposal_W, содержащий:
   - `included_bundles_{W-1}` (canonical view финализации)
   - `included_reveals_{W-1}` (cemented set лотереи)
   - `winner_{W-1}` (получатель `reward(W-1)` за окно W-1)
   - control_set, state_root, Монтана transfer
9. Сеть валидирует proposal_W:
   - Proposer = winner_{W-2}? (канонически проверяемо)
   - included_bundles содержат ≥ 67% active_chain_length? (проверяемо из Node Table)
   - included_reveals_{W-1} = cemented set VDF_Reveals окна W-1? (валидатор сверяет с собственным tracking cement status из BundledConfirmation окна W)
   - winner_{W-1} = argmin из (included_reveals ∪ account_candidates)? (детерминированно проверяемо)
   - state_root корректен? (независимый пересчёт)
10. Если 67% active_chain_length подписывают proposal_W → proposal cemented. Winner_{W-1} получает `reward(W-1)` Ɉ. Winner_{W-1} становится proposer_{W+1}.
11. Если < 67% подписали → proposal отклонён. Fallback: `fallback_proposer_W = second_min(weighted_ticket)` окна W-2. Fallback cascade: third_min, fourth_min, etc.

**Cross-window cementing timeline.** VDF_Reveals окна W-1 публикуются при завершении окна W-1 (VDF computation = window duration). Цементируются в BundledConfirmation окна W. Между публикацией reveals и сборкой proposal — целое окно. Timing constraint отсутствует.

**Leader skin in the game.** Proposer_W публикует свой VDF_Reveal для окна W. Если его proposal отклонён (< 67% подписей сети), его VDF_Reveal исключается из пула кандидатов окна W. Потеря lottery ticket = экономический кнут за цензуру или бездействие. Отказ подписать proposal = implicit rejection от каждого узла.

**Genesis bootstrap.** proposer_0 и proposer_1 = bootstrap-узел (единственный в Genesis Decree). Начиная с proposer_2 = winner_0, стандартная lookback логика.

#### Калибровка target

Target калиброван на ~13 кандидатов VDF_Reveal за окно. Калибровка на τ₂:

**Real-valued form (commentary):**
```
target_new = target_old × (13 / actual_candidates_per_window)
actual_candidates_per_window = total_reveals_за_τ₂ / τ₂_windows
```

**Integer form (authoritative, per [I-9]):**
```
target is u128 (Q128.0 representation of weighted_ticket threshold; кандидат если weighted_ticket < target).

Input:
  target_old: u128
  total_reveals_τ₂: u64      (сумма VDF_Reveals за τ₂ окон)
  τ₂_windows: u64            (константа из ProtocolParams — см. Genesis Decree)

Output:
  target_new: u128

Algorithm:
  если total_reveals_τ₂ == 0:
    # нет кандидатов — target не калибруется
    target_new = target_old
  иначе:
    # target_new = target_old × 13 × τ₂_windows / total_reveals_τ₂
    # Порядок: multiply сначала (preserve precision), divide в конце.
    numerator_u256 = (target_old as u256) * 13u256 * (τ₂_windows as u256)
    target_new_u256 = numerator_u256 / (total_reveals_τ₂ as u256)
      # Unsigned integer div toward zero.
    если target_new_u256 > u128::MAX as u256:
      target_new = u128::MAX        # saturating clamp
    иначе:
      target_new = target_new_u256 as u128

Note: u256 intermediate реализуется через big-int (2× u128 chunked arithmetic) — byte-exact алгоритм.

Binding test vectors (byte-exact; используют target_old = 2^127 = 0x80..00, τ₂_windows = 20160):

  # TA1 equilibrium (actual candidates = expected 13/window) — target unchanged
  target_old = 0x80000000000000000000000000000000
  total_reveals_τ₂ = 262080  (= 13 × 20160)
  → target_new = 0x80000000000000000000000000000000

  # TA2 2× over-participation — target halved (harder to win)
  target_old = 0x80000000000000000000000000000000
  total_reveals_τ₂ = 524160  (= 26 × 20160)
  → target_new = 0x40000000000000000000000000000000

  # TA3 1/13 participation — target ×13 (saturated at u128::MAX)
  target_old = 0x80000000000000000000000000000000
  total_reveals_τ₂ = 20160
  → target_new = 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF  (saturated)

  # TA4 zero reveals — target unchanged (no calibration signal)
  target_old = 0x80000000000000000000000000000000
  total_reveals_τ₂ = 0
  → target_new = 0x80000000000000000000000000000000

  # TA5 single reveal — saturated maximum
  target_old = 0x80000000000000000000000000000000
  total_reveals_τ₂ = 1
  → target_new = 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF  (saturated)

Conformance status: closed (binding test vectors выше).
```

Трафик reveal за окно: ~13 VDF_Reveal × 738B ≈ 9.6 KB (P2P gossip; далее включаются в BundledConfirmation для cementing). Аккаунты участвуют через cemented операции в BundledConfirmation — дополнительного трафика для аккаунтов нет.

#### Integer log algorithm (per [I-9], node lottery)

Алгоритм `ln_q64(endpoint) → u128` используется в формуле `weighted_ticket_node`.

```
ln_q64(endpoint: [u8; 32]) -> u128    # Q64.64 representation of -ln(endpoint / 2^256)

Semantics: возвращает -ln(endpoint/2^256) × 2^64, округлённый toward zero.
  Малые endpoint → большие ticket; большие endpoint → малые ticket.
  Максимум: endpoint = 0 клипируется до u128::MAX (SHA-256 collision probability negligible).

Binding constants (Q64 fixed-point, unsigned u64; halved-polynomial form чтобы все
коэффициенты поместились в u64 даже если коэффициент полного полинома превышает 1):

  B0     = 0x0014E086EC982D63    # = (a0 / 2) × 2^64
  B1     = 0xB59DDDE52A69D000    # = (a1 / 2) × 2^64
  B2_ABS = 0x49DF5C3BFD9CEC00    # = (|a2| / 2) × 2^64
  B3     = 0x14417E56D3331800    # = (a3 / 2) × 2^64
  LN2_Q64 = 0xB17217F7D1CF79AB   # = ln(2) × 2^64, truncated toward zero

Где a0..a3 — degree-3 minimax polynomial (Remez equioscillating) для log2(1+y)
на y ∈ [0, 1):
  a0 = +0.00063711727233465817
  a1 = +1.41888021173219991411     (> 1 → не помещается в u64 при Q64; отсюда halved form)
  a2 = -0.57712891511184893911     (|a2| хранится как B2_ABS, знак embedded в Horner
                                     через subtract — unsigned arithmetic per [I-9])
  a3 = +0.15824870337964891398

Algorithm (byte-exact):
  1. e_u256 = big-endian interpretation of endpoint (32B)
  2. если e_u256 == 0: return u128::MAX  (SHA-256 collision probability negligible)
  3. leading = leading_zeros_u256(e_u256)                        # ∈ [0, 255]
  4. msb_position = 255 - leading                                # ∈ [0, 255]
  5. # Normalize mantissa в [2^127, 2^128):
     if msb_position >= 127:
       shift = msb_position - 127
       mantissa_u128 = (e_u256 >> shift) & ((1u256 << 128) - 1)  # low 128 bits
     else:
       shift = 127 - msb_position
       mantissa_u128 = (e_u256 << shift) & ((1u256 << 128) - 1)  # low 128 bits
  6. # Q64 fractional part ∈ [0, 1):
     x_q64 = ((mantissa_u128 - (1u128 << 127)) >> 63) as u64
  7. # log2(1 + y) approximation через unsigned Horner (halved-polynomial form).
     # half_p(y) = B0 + y·(B1 - y·(B2_ABS - y·B3))
     # p(y)      = log2(1+y) × 2^64  ≈  half_p(y) << 1
     #
     # Пошаговое unsigned вычисление:
     t1_u64  = ((B3 as u128) * (x_q64 as u128)) >> 64 as u64     # y·B3   ∈ [0, B3]
     # invariant_1: t1 ≤ B2_ABS  (доказано: B3 < B2_ABS, y ≤ 2^64-1)
     t2_u64  = B2_ABS - t1_u64                                   # B2_ABS - y·B3  ∈ [B2_ABS - B3, B2_ABS]
     t3_u64  = ((t2_u64 as u128) * (x_q64 as u128)) >> 64 as u64 # y·(B2_ABS - y·B3)  ∈ [0, B2_ABS]
     # invariant_2: t3 ≤ B1  (доказано: max t3 = B2_ABS - B3 < B1)
     t4_u64  = B1 - t3_u64                                       # B1 - y·(B2_ABS - y·B3)  ∈ [B1 - B2_ABS, B1]
     t5_u64  = ((t4_u64 as u128) * (x_q64 as u128)) >> 64 as u64 # y·(B1 - y·(B2_ABS - y·B3))  ∈ [0, B1]
     half_p_u64 = B0 + t5_u64                                    # ≤ B0 + B1 < 2^63
     frac_q64 = half_p_u64 << 1                                  # p(y) × 2^64 ∈ [0, 2^64]
     # При y близком к 2^64 (edge) frac_q64 может достичь 2^64 — но операция
     # half_p_u64 < 2^63 → shift безопасен, frac_q64 ≤ 2^64-2.
  8. # log2(2^256/e) = (leading+1) - log2(1+y), где y = (mantissa − 2^127) / 2^127
     log2_q64_u128 = ((leading+1) as u128) << 64) - (frac_q64 as u128)
     # (leading+1) ∈ [1, 256], shift в u128 safe; frac_q64 ≤ 2^64-2; результат ≥ 2
  9. ticket_q64_128 = ((log2_q64_u128 as u256) * (LN2_Q64 as u256)) >> 64 as u128
     # u128 × u64 → u192 intermediate; shift >> 64 → u128 (старшие биты нулевые т.к.
     # log2_q64 ≤ 256·2^64 = 2^72, и log2_q64 × LN2_Q64 ≤ 2^72 × 2^64 = 2^136;
     # >> 64 → 2^72. Safe.
 10. return ticket_q64_128

Invariants proof:
- invariant_1 (t1 ≤ B2_ABS):
    t1 = (y_q64 × B3) >> 64 ≤ B3 (т.к. y_q64 ≤ 2^64 - 1 < 2^64).
    B3 = 0x14417E56D3331800 = 1,459,586,665,620,379,648 ≈ 0.079·2^64
    B2_ABS = 0x49DF5C3BFD9CEC00 = 5,323,074,697,302,961,152 ≈ 0.289·2^64
    B3 < B2_ABS ⟹ t1 ≤ B3 < B2_ABS. ✓
- invariant_2 (t3 ≤ B1):
    t3 = (t2 × y) >> 64 ≤ t2 ≤ B2_ABS. B2_ABS < B1. ✓

Error bound (degree-3 Remez minimax optimum):
- Абсолютная ошибка: |ln_q64(e) − 2^64 · (−ln(e/2^256))| ≤ 2^-10.62 × 2^64 ≈ 1.18·10^16
  в Q64.64 единицах. Это теоретический оптимум degree-3 polynomial на [0, 1);
  более высокая точность требует degree ≥ 7 (2^-28) или degree ≥ 15 (2^-56).

[I-8] reconciliation: approximation error даёт attacker grinding advantage
~0.13% of typical ticket — но grinding horizon уже ограничен конструктивно через
`cemented_bundle_aggregate(W-2)` в endpoint formula (см. раздел «VDF_Reveal» и
инвариант [I-8]). Attacker не может pre-compute future endpoint без privкey
honest confirmers окна W-2. Additional advantage через approximation error
dominated базовым [I-8]-bounded surface; net safety margin preserved. Degree-3
выбран как optimal trade-off complexity/precision для argmin лотереи: endpoints
uniform distributed на [0, 2^256), typical gap между соседними кандидатами
много больше 2^-10 log2-единиц.

Binding test vectors (byte-exact, для conformance tests независимых реализаций):

  # TV1: boundary low (smallest non-zero endpoint → largest ln)
  endpoint = 0x0000000000000000000000000000000000000000000000000000000000000001
  ln_q64   = 0x00000000000000b171fb06bb5b60c961

  # TV2: MSB only (endpoint = 2^255 → log2(2^256/2^255) = 1, ticket ≈ LN2_Q64)
  endpoint = 0x8000000000000000000000000000000000000000000000000000000000000000
  ln_q64   = 0x0000000000000000b15526e15db6980c

  # TV3: typical dense pattern
  endpoint = 0xbbaa998877665544332211ffeeddccbbaa998877665544332211ffeeddccbbaa
  ln_q64   = 0x00000000000000004f60bd6fe6504646

  # TV4: near max (endpoint = 2^256-1 → log2(2^256/e) ≈ 0, ticket ≈ 0)
  endpoint = 0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff
  ln_q64   = 0x00000000000000000000000000000000

  # TV5: peak-error region (y ≈ 0.84, attacker-favorable peak)
  endpoint = 0xeb851eb851eb8400000000000000000000000000000000000000000000000000
  ln_q64   = 0x000000000000000015756c980b547a82

Conformance: closed (binding coefficients + 5 test vectors выше).

Свойства для consensus:
- Monotonic decreasing в endpoint: e1 < e2 ⟹ ln_q64(e1) ≥ ln_q64(e2)
- Deterministic byte-exact: same bytes input → same u128 output на любом hardware
- Unsigned arithmetic по всей цепочке (требование [I-9]); знак a2 embedded через
  subtract в Horner, intermediate инварианты доказаны non-negative
- Bounded absolute error: 2^-10.62 (degree-3 Remez minimax optimum)
```

#### Валидация VDF_Reveal

1. Подпись ML-DSA-65 соответствует node_pubkey из Node Table
2. window_index = текущий τ₁
3. node_id существует в Node Table
4. weighted_ticket < target
5. endpoint верифицируем: SHA-256("mt-lottery" || T_r || node_id || window_index) = заявленному endpoint

### Account — содержимое блока

Приём, верификация объектов и формирование набора. Два класса объектов:

**UserObjects** — пользовательские операции:

| Тип | Описание | Валидация |
|-----|----------|-----------|
| Transfer (Mode A: receiver уже в Account Table) | Публичный перевод существующему аккаунту | ML-DSA-65 подпись, prev_hash, sender != receiver, amount > 0, sender.balance >= amount, получатель **существует** в Account Table, payload длина = 80 B |
| Transfer (Mode B: receiver не существует, создание AccountRecord) | Перевод с атомарным созданием AccountRecord | ML-DSA-65 подпись sender, prev_hash, receiver **не** существует, link == H("mt-account" \|\| suite_id \|\| receiver_pubkey), amount > 0, sender.balance >= amount, payload длина = 2034 B, cooldown `current_window >= sender.last_account_creation_window + τ₂_windows` per [I-15] |
| ChangeKey | Смена ключа | ML-DSA-65 подпись старым ключом, new_pubkey |
| Anchor | Якорь данных ко времени | ML-DSA-65 подпись, prev_hash, app_id = 32B, data_hash = 32B |
| CloseAccount | Явное закрытие аккаунта | см. раздел «Жизненный цикл аккаунта» |

**ControlObjects** — объекты управляющие составом сети:

| Тип | Описание | Валидация |
|-----|----------|-----------|
| NodeRegistration | Регистрация узла (кандидатура) | ML-DSA-65 `signature` валидна для `node_pubkey` над signed_scope (Правило R1); ML-DSA-65 `operator_pop` валидна для `operator_pubkey` над bytes (`"mt-operator-pop" \|\| node_pubkey`) — proof of possession с class domain separator (Правило R2; без него squatting на чужой operator_pubkey возможен через cross-class confusion); `node_id` уникален (не в Node Table и не в Candidate Pool); `operator_account_id == SHA-256("mt-account" \|\| suite_id \|\| operator_pubkey)` (binding derivation); если AccountRecord operator-а существует — `is_node_operator == 0` AND `current_pubkey == operator_pubkey`, иначе AccountRecord создаётся атомарно при cementing Selection event; `proof_endpoint` верифицируем через VDF от candidate_vdf_init. `nodereg_hash` = identifier(nr) с class domain `"mt-nodereg"` (Правило R2) |

Каждый узел валидирует объекты обоих классов локально при получении. Валидные объекты ретранслируются по P2P.

Все объекты — UserObjects, ControlObjects и VDF_Reveals — финализируются (cemented) одинаково: через 67% active_chain_length подтверждения в BundledConfirmation. Cemented status объективен и одинаков для всех узлов. Дискреция победителя над включением ControlObjects и VDF_Reveals = ноль.

#### Proposal

Proposal содержит **control_set** и метаданные окна. UserObjects применяются к Account Table батчем при settle (apply at window close); в proposal они не повторяются. ControlObjects применяются к Node Table в apply_proposal step 1 в детерминированном порядке.

**control_set(proposal окна W)** определён формулой:

```
control_set = {
  ControlObject c :
    c.cemented_window > previous_proposal.window
    AND c.cemented_window <= W
}

сортировка: (cemented_window asc, op_hash lex asc)
```

Где `previous_proposal.window` — окно предыдущего финализированного proposal в цепочке. Множество детерминировано: cemented_window — каноническое поле объекта (известно всем узлам через BundledConfirmation), op_hash — детерминирован.

Победитель **обязан** включить весь control_set целиком. Пропуск или добавление лишнего ControlObject = невалидный proposal = fallback. Каждый узел независимо вычисляет ожидаемый control_set по той же формуле и сравнивает с proposer's set.

Форки аккаунтов (две операции с одним prev_hash) разрешаются голосованием узлов весом chain_length. 67% active_chain_length за одну операцию → побеждает (см. раздел «Двойная трата»).

#### Закрытие окна (Lookback Leadership Finalization)

```
Window W-1:  confirmers publish BundledConfirmation_{W-1}
               (W-1 operations + W-2 VDF_Reveals)
             VDF_{W-1} completes → candidates publish VDF_Reveal_{W-1}
             accounts publish cemented operations
                              │
Window W:    confirmers publish BundledConfirmation_W
               (W operations + W-1 VDF_Reveals)
             W-1 VDF_Reveals cemented (67% active_chain_length)
                              │
             proposer_W = winner_{W-2} (canonical from proposal_{W-1})
             proposer_W extracts cemented reveals → winner_{W-1}
             proposer_W publishes proposal_W
                              │
                              ▼
                    ┌───────────────────────────────┐
                    │ proposal_W validation         │
                    │ included_bundles ≥ 67%?       │
                    │ included_reveals = cemented?  │
                    │ winner_{W-1} = argmin?        │
                    │ state_root correct?           │
                    └───────────┬───────────────────┘
                                │ 67% sign
                                ▼
                      proposal_W cemented
                      winner_{W-1} receives reward(W-1) Ɉ
                      winner_{W-1} = proposer_{W+1}
```

- **Lookback Leader.** `proposer_W = winner_{W-2}` — канонически определён из cemented proposal_{W-1}. Каждый узел вычисляет proposer_W детерминированно из canonical state.
- **Cemented reveals.** VDF_Reveals окна W-1 публикуются при завершении W-1, цементируются чер��з BundledConfirmation окна W (confirmers включают полученные reveals в свои bundles). `included_reveals_{W-1}` = cemented set (67% active_chain_length). Proposer извлекает cemented reveals и cemented account operations из included_bundles_{W-1}, определяет `winner_{W-1} = argmin(weighted_ticket)`. Дискреция proposer-а над составом лотереи = ноль.
- **Canonical acceptance.** Сеть валидирует proposal_W: (a) proposer = winner_{W-2}, (b) included_bundles ≥ 67% active_chain_length, (c) included_reveals = cemented set VDF_Reveals окна W-1, (d) winner_{W-1} = argmin из (cemented reveals ∪ account_candidates), (e) state_root корректен. Если 67% active_chain_length подписывают proposal_W → cemented. Canonical set зафиксирован.
- **Leader skin in the game.** Proposer_W участвует в лотерее окна W через свой VDF_Reveal (cemented в BundledConfirmation окна W+1). При отклонении proposal (< 67% подписей) — VDF_Reveal proposer-а исключается из пула кандидатов окна W. Отказ подписать proposal = implicit rejection. Отдельного censorship vote нет.
- **Fallback cascade.** Если proposal от proposer_W отклонён или отсутствует, роль переходит к `fallback_1 = second_min(weighted_ticket)` окна W-2, затем third_min, etc. Все канонически известны из cemented state.
- **ControlObjects.** ControlObjects попадают в control_set proposal по моменту cement — canonically deterministic.

**Свойство темпа сети.** Сеть продвигается со скоростью медианного активного набора узлов. Quorum требует подписей большинства по chain_length — быстрейший узел ждёт, пока достаточно других успеет. Hardware progress ускоряет сеть естественно — когда ускоряется медиана, participation_ratio растёт выше 0.95, D адаптивно увеличивается.

**One-window lag награды.** `reward(W-1)` за окно W-1 зачисляется winner_{W-1} при cementing proposal_W. Задержка в одно окно между завершением работы и получением награды.

#### Proposer (Lookback Leader)

`proposer_W = winner_{W-2}` — канонически определён из cemented proposal_{W-1}. Proposer собирает proposal_W:

- **included_bundles_{W-1}**: BundledConfirmation окна W-1 (суммарный chain_length ≥ 67% active_chain_length). Из included_bundles извлекаются cemented account operations для apply_proposal (баланс, account_chain_length increment); account operations не участвуют в лотерее (см. «Аккаунты не участвуют в лотерее»).
- **included_reveals_{W-1}**: VDF_Reveals окна W-1, cemented через BundledConfirmation окна W (67% active_chain_length). Из cemented reveals определяется `winner_{W-1}` (получатель `reward(W-1)` за окно W-1). Лотерея single-class — winner всегда узел; cemented account operations окна W-1 не участвуют в выборе winner-а.
- **control_set**: все cemented ControlObjects в окнах (previous_proposal.window, W]. Свобода = ноль (каноничен).
- **State Root snapshot**: account_root, node_root и candidate_root после apply at window close (все cemented операции + control objects + selection event + Монтана transfer to winner_{W-1} применены батчем).

Свобода proposer: included_bundles ограничены порогом 67%. included_reveals детерминированы cement status-ом. control_set детерминирован формулой. State root и winner_{W-1} вычисляются из cemented sets — каждый валидатор проверяет корректность детерминированно.

Proposal с набором included_bundles < 67% active_chain_length, неверным included_reveals (не совпадает с cemented set), неверным winner_{W-1}, пропущенным cemented ControlObject, или неверным state_root отклоняется → fallback на second_min(weighted_ticket) окна W-2.

#### Финальность proposal

Финальность proposal = подпись proposer_node_id на proposal header (верифицируемая против Node Table[proposer_node_id].node_pubkey) + независимая верифицируемость состояния.

1. Proposer (proposer_node_id) публикует подписанный proposal header + control_set
2. Каждый узел проверяет `window_index == prev_proposal.window_index + 1`, `protocol_version >= prev_proposal.protocol_version` и `protocol_version <= local_max_supported_version`
3. Каждый узел независимо вычисляет ожидаемый control_set по формуле и сравнивает с proposer's
4. Каждый узел применяет control_set + Монтана детерминированно в порядке (cemented_window asc, op_hash lex asc)
5. Каждый узел сравнивает вычисленный state_root с заявленным в proposal
6. Совпадает — proposal принят
7. Не совпадает — proposal отклонён, fallback на второе место

Финальность операций аккаунтов — отдельный процесс через подтверждения (67% active_chain_length), не через proposal.

Proposal header:

```
Proposal header:
  prev_proposal_hash    32B
  window_index           8B    <- u64, индекс окна τ₁ с genesis; == prev_proposal.window_index + 1
  protocol_version       4B    <- u32, активная версия протокола на момент window_index
  control_root          32B    <- Merkle root control_set (каноничен)
  node_root             32B    <- Merkle root Node Table (обновляется каждое окно)
  candidate_root        32B    <- Merkle root Candidate Pool
  account_root          32B    <- Merkle root Account Table после apply at window close
  state_root            32B    <- SHA-256("mt-state-root" || node_root || candidate_root || account_root)
  timechain_value       32B
  included_bundles_root 32B    <- Merkle root списка (confirmer_id, bundle_hash)
                                  BundledConfirmation окна W-1 (≥ 67% active_chain_length)
  included_reveals_root 32B    <- Merkle root списка VDF_Reveal-ов окна W-1,
                                  cemented через BundledConfirmation окна W
  winner_endpoint       32B    <- endpoint winner-а окна W-1 (node lottery)
  winner_id             32B    <- получатель `reward(W-1)` за окно W-1: node_id узла-winner-а
  proposer_node_id      32B    <- winner_{W-2}, канонически определённый из proposal_{W-1}
  target                16B    <- u128 Q64.64 (per [I-9] P5), текущий target лотереи
  fallback_depth         1B    <- u8, 1 = первое место, 2..=255 = fallback cascade;
                                  fallback_depth = 255 без успеха → network halt by liveness (не safety)
  signature           3309B    <- ML-DSA-65 над signed_scope(header) (Правило R1);
                                  проверяется Node Table[proposer_node_id].node_pubkey.
                                  proposal_hash = identifier(header) с class domain "mt-proposal" (Правило R2)
```

Все поля proposal header канонически вычислимы bit-exact из предыдущего state и cemented set окна W. Каждое поле имеет источником либо canonical state, либо детерминированную функцию от canonical state.

**Разделение ролей winner_id и proposer_node_id.** Это два независимых поля с разными назначениями:

- `winner_id` — получатель Монтана. Лотерея single-class: winner — всегда узел, выигравший лотерею окна. `reward(W-1)` зачисляется на `operator_account_id` узла-winner-а в apply_proposal step 2.
- `proposer_node_id` — узел ответственный за сборку и публикацию proposal. Подписывает header своим node_pubkey. Верификация подписи proposal — против `Node Table[proposer_node_id].node_pubkey`, всегда.

Штатный случай: `winner_id` == `proposer_node_id` (узел-winner сам собирает свой proposal). Fallback: если winner-узел молчит — proposal собирает следующий узел по lowest weighted_ticket, `proposer_node_id` ≠ `winner_id`; reward всё равно зачисляется на operator_account_id winner-а, proposer не получает дополнительной награды — это его обязанность как ближайшего активного узла.

**Инварианты Proposal header:**

- `window_index == prev_proposal.window_index + 1` (монотонность, шаг 1)
- `protocol_version >= prev_proposal.protocol_version` (не убывает; изменяется только через software upgrade узла, см. раздел «Эволюция протокола»)
- `protocol_version <= local_max_supported_version` (узел **обязан отклонить** proposal с protocol_version которую его реализация не поддерживает; принятие неизвестной версии = принятие непроверяемых правил = нарушение безопасности)
- `fallback_depth ≥ 1` (1 = canonical proposer, 2..=255 = fallback cascade per layout выше; fallback_depth = 0 — **reject**)
- `proposer_node_id` существует в `Node Table` и имеет `suite_id` соответствующую поддерживаемой схеме подписи; signature ML-DSA-65 verify over signed_scope(header) против `Node Table[proposer_node_id].node_pubkey` (Правило R1)

**Cemented window** объекта — `window_index` proposal-а в котором BundledConfirmation с этим объектом достиг quorum. Определён детерминированно для каждого cemented объекта.

**Settled window** объекта — `window_index` proposal-а в котором объект был применён к state:
- Для UserObjects: `settled_window = cemented_window` (apply batch at window close того же окна). Следующая операция от того же sender возможна в окне `cemented_window + 1` (dependency rule)
- Для ControlObjects: `settled_window` = window_index первого proposal где объект попал в control_set (обычно `cemented_window + 1`)

Fallback: если proposal от `proposer_W = winner_{W-2}` отклонён (< 67% подписей) или отсутствует (proposer offline), роль переходит к `fallback_1 = second_min(weighted_ticket)` окна W-2. Если fallback_1 тоже отклонён — к third_min, и т.д. Вся cascade канонически определена из cemented state окна W-2.

При fallback `proposer_node_id` меняется; `winner_{W-1}` определяется fallback-proposer-ом из cemented set (тот же cemented set — canonical для всех узлов). Новый proposer подписывает header своим node_pubkey, `fallback_depth` инкрементируется.

**Leader penalty при отклонении:** endpoint proposer-а, чей proposal отклонён, исключается из lottery пула текущего окна W. Proposer теряет шанс на `reward(W)`. Это экономический кнут за бездействие или цензуру.

**Полная симметрия fallback:** молчание первого proposer переводит обязанность сборки proposal к следующему узлу. Награда за окно W-1 привязана к лотерейному билету и гарантирована, если хотя бы один узел в сети соберёт валидный proposal через fallback cascade.

#### Непрерывность VDF

VDF следующего окна вычисляется непрерывно, не ожидая завершения финализации предыдущего. TimeChain для окна N+1 детерминирован — каждый узел вычисляет его независимо. Reveal phase и финализация происходят параллельно с началом VDF следующего окна.

#### Confirmations (финализация операций и control objects)

Confirmers — узлы с `chain_length >= confirmation_threshold`. Подтверждают **все** валидные объекты окна (UserObjects + ControlObjects) от имени сети.

```
active_chain_length(W) = Σ node.chain_length
                         для node ∈ Node Table : active(node, W)

confirmation_threshold(W) = active_chain_length(W) / 256
≈ 256 confirmers при large-scale сети (active_chain_length / 256).
```

Только активные узлы (cemented BundledConfirmation за последние 2τ₂) учитываются. Мёртвый вес исключён конструкцией. Сканирование Node Table для вычисления `active_chain_length` — O(|Node Table|) ≤ 10⁵ записей, миллисекунды.

**Сенатская модель комитета.** Confirmers — сенат долгоживущих узлов, не ротирующаяся выборка из активного набора. Узел попадает в комитет только накопив `chain_length` выше порога; это намеренная долгосрочная инерция роли, не недостаток механизма. Разделение ролей в протоколе:

- **Confirmers (комитет)** — долгоживущие узлы, голосуют за финализацию и разрешение конфликтов.
- **Все активные узлы** — участвуют в node lottery, gossip, хранят данные, обслуживают своих операторов. Новые узлы полнофункциональны как инфраструктура с момента установки (см. раздел «Barrier scope»), но в комитет попадают только после накопления chain_length.

**Требование к развёртыванию: доля онлайн-работы честного оператора ≥ 0.85.** Это условие гарантирует что концентрация атакующего в top-K комитете ограничена коэффициентом не более 1.18× от его доли в сети. При доле атакующего в сети `f ≤ 0.25` и соблюдении этого требования доля атакующего в комитете `≤ 0.282`, что ниже порога BFT `1/3`. Нарушение требования (оператор с доступностью ниже 67%) открывает вектор захвата комитета через асимметрию времени работы.

Confirmer собирает все валидные объекты за окно и публикует один BundledConfirmation. Bundle содержит два класса хэшей: (1) операции текущего окна W (UserObjects + ControlObjects) и (2) VDF_Reveals предыдущего окна W-1 (лотерейные билеты, опубликованные при завершении W-1 и полученные через P2P):

```
BundledConfirmation:
  node_id           32B
  endpoint          32B     <- T_r текущего окна (доказывает timeliness)
  window_index       8B     <- u64 LE, индекс τ₁ (унифицирован с ProposalHeader.window_index)
  op_count           2B     <- u16 LE, explicit count prefix
  op_hashes[]       op_count × 32B    <- identifier(op) с class "mt-op" для UserObjects и ControlObjects окна W
  reveal_count       2B     <- u16 LE, explicit count prefix
  reveal_hashes[]   reveal_count × 32B <- identifier(reveal) с class "mt-vdf-reveal" окна W-1
  signature        3309B     <- ML-DSA-65 над signed_scope(bundle) (Правило R1);
                                проверяется Node Table[node_id].node_pubkey
Fixed overhead: 3385B (= 32 + 32 + 8 + 2 + 2 + 3309)
```

`bundle_hash` = `identifier(bundle)` с class domain `"mt-bundle"` (Правило R2). Один BundledConfirmation per (node_id, window_index). Повторный отклоняется. Endpoint = T_r текущего окна (верифицируем: сравнение с каноническим T_r). `node.chain_length` хранится в Node Table и инкрементируется в `apply_proposal` шаг 3.5 для каждого узла с cemented BundledConfirmation в окне W.

**Инварианты BundledConfirmation:**

- `node_id` существует в Node Table и соответствует активному confirmer-у (`chain_length >= confirmation_threshold` на момент окна `window_index`)
- `window_index` равен текущему окну валидации (bundle не может относиться к произвольному окну)
- `endpoint == T_r(window_index)` каноничный (верифицируем сравнением с локально вычисленным T_r)
- `op_count ≤ max_ops_per_bundle` (верхняя граница DoS; значение константы — см. раздел «Обоснование протокольных констант»)
- `reveal_count ≤ max_reveals_per_bundle` (верхняя граница DoS)
- Каждый элемент `op_hashes[i]` — 32B `identifier(op)` с class domain `"mt-op"`; дубликаты внутри массива **запрещены**
- Каждый элемент `reveal_hashes[i]` — 32B `identifier(reveal)` с class domain `"mt-vdf-reveal"` окна W-1; дубликаты **запрещены**
- Один BundledConfirmation per `(node_id, window_index)` — повторный отклоняется (equivocation, см. раздел «Конфликты»)
- Signature ML-DSA-65 valid over signed_scope(bundle) против `Node Table[node_id].node_pubkey` (Правило R1)

Inclusion validity каждой операции внутри bundle (dependency rule: `prev_hash`, баланс, receiver existence) — см. раздел «Dependency rule» ниже; это per-context check confirmer-а, отдельный от structural инвариантов BundledConfirmation.

Объект финализирован (cemented) когда подтверждения от confirmers с суммарным chain_length > quorum. Cemented — необратимо. Типичное время: quorum event. Это правило применяется одинаково к UserObjects, ControlObjects и VDF_Reveals: cemented status объективен и каноничен для всех узлов. VDF_Reveals окна W-1 цементируются в BundledConfirmation окна W (cross-window cementing).

**Confirmation cutoff (детерминизм cemented set).** Cemented set окна W фиксируется proposer-ом окна W+1 через frozen view (Lookback Leadership). Proposer_{W+1} включает в proposal_{W+1} все BundledConfirmation окна W из своего view с суммарным chain_length ≥ 67% active_chain_length. Этот frozen view становится каноническим cemented set после cementing proposal_{W+1} сетью.

**Cemented bundle aggregate.** Канонический агрегат идентичностей confirmers окна W, используемый как unpredictable-offline компонент в формулах lottery endpoint, sort_key и candidate_vdf_init. Aggregate строится по Правилу R3 (aggregate over signer_node_id, не over signatures и не over content):

```
cemented_bundle_aggregate(W) :=
  если W < 2:
    0x00 × 32                                    (до Genesis cementing)
  иначе если |cemented_bundles_W| == 0:
    SHA-256("mt-bc-aggregate-empty" || W.to_le_bytes_8)        (вырожденный случай: окно без cementing)
  иначе:
    S_W := { bc.node_id : bc ∈ cemented_bundles_W }
    SHA-256(
      "mt-bc-aggregate" ||
      concat(node_id for node_id in sorted_asc(S_W)) ||
      W.to_le_bytes_8
    )
```

`cemented_bundles_W` — каноническое множество cemented BundledConfirmation окна W (frozen view proposer_{W+1}). S_W — множество signer_node_id этих bundles, отсортированное по asc (32B lexicographic). Контекст `W.to_le_bytes_8` — 8-байтовый little-endian window_index.

Ветви формулы покрывают все возможные состояния окна:
- **W < 2:** Genesis окна, cemented_bundle_aggregate(W-2) не существует — возвращается фиксированный 0x00 × 32.
- **|cemented_bundles_W| == 0:** окно без cementing (катастрофический отказ консенсуса). Возвращается детерминистический fallback. [I-8] в этой ветви вырожден, но в non-functional состоянии сети это приемлемо — protocol уже не производит консенсус.
- **Стандартная ветвь:** агрегат node_ids cemented confirmers, полная защита [I-8].

Свойства:
- **Канонический.** Cemented set объективен, порядок детерминирован. Два честных узла bit-exact получают одинаковое значение.
- **Непредсказуемый offline (в стандартной ветви).** Зависит от эмерджентного состава S_W — какие именно active confirmers набрали quorum. Атакующий с VDF hardware advantage не может пре-вычислить будущий S_W без координированного control over honest participants (никто single confirmer не контролирует набор других cemented confirmers).
- **Ноль grinding surface для single confirmer.** node_id детерминистически вычислен из registered node_pubkey (commited в NodeTable), не меняется. Content бандла (op_hashes[], reveal_hashes[]) attacker-choose-able, но **исключён из aggregate per Правило R3**. Signature σ под deterministic ML-DSA-65 уникально определена парой (sk, message), но **исключена из aggregate per Правило R3** — независимо от detminism schema. Обе grinding surface устранены конструкцией, не экономическими аргументами.
- **Degraded security margin в bootstrap периоде.** При `active_nodes = 1` агрегат содержит один node_id. Безопасность в этот период опирается на секретность bootstrap node_pubkey derivation — см. раздел «Границы модели доверия».

**Dependency rule (детерминизм apply).** Одно правило: confirmer подтверждает операцию только если все её зависимости разрешены из settled state окна W-1.

```
Операция валидна для inclusion в BundledConfirmation окна W если:
  1. prev_hash == Account Table[sender].frontier_hash
     на момент settled state конца окна W-1
  2. Для Transfer: receiver существует в Account Table
     на момент settled state конца окна W-1
  3. sender.balance >= amount (для Transfer)
     на момент settled state конца окна W-1
```

Settled state конца окна W-1 — результат apply_proposal окна W-1 — одинаков у всех узлов (детерминированная функция от cemented set W-1 и предыдущего state). Confirmer проверяет каждую операцию против этого глобально единого состояния. Никаких bundle-local цепочек, никакого mempool order.

**Следствие: одна операция на аккаунт за окно τ₁.** Вторая операция от того же sender имеет prev_hash = H(первой операции), но первая ещё не settled (settled = конец текущего окна W). Confirmer отклоняет вторую. Она пройдёт в окне W+1 когда первая settled. Throughput на аккаунт: 1 операция за окно. Это достаточно для всех бытовых сценариев; для высокочастотных — batching через Anchor (один Anchor содержит Merkle root тысяч записей).

Cross-account зависимости сериализуются через окна — существующий аккаунт создаёт AccountRecord получателя через `Transfer` Mode B в окне W; последующие исходящие операции от этого получателя (Transfer, Anchor и т.д.) — в окнах W+1 и далее, после settle AccountRecord.

**Real-valued form (commentary):**
```
quorum(W) = ⌈0.67 × active_chain_length(W)⌉
```

**Integer form (authoritative, per [I-9]):**
```
quorum(W): u64
Input:  active_chain_length(W): u64
Algorithm:
  quorum(W) = (67u64 * active_chain_length(W) + 99u64) / 100u64
    # Unsigned u64 arithmetic; integer div toward zero.
    # +99 реализует ceiling для division на 100.
Overflow: active_chain_length ≤ 10^14 (node cap × chain cap);
          67 × 10^14 + 99 ≈ 6.7 × 10^15 < 2^63 ⇒ safe u64.

Test vectors (binding):
  active_chain_length = 1      → quorum = 1       ((67 + 99) / 100 = 1)
  active_chain_length = 100    → quorum = 67      ((6700 + 99) / 100 = 67)
  active_chain_length = 149    → quorum = 100     ((9983 + 99) / 100 = 100)
  active_chain_length = 150    → quorum = 101     ((10050 + 99) / 100 = 101)
  active_chain_length = 1000   → quorum = 670     ((67000 + 99) / 100 = 670)

[I-9] статус: закрыто (test vectors in spec).
```

Объект cemented когда суммарный chain_length confirmers подтвердивших объект через BundledConfirmation окна W ≥ quorum(W). Активный набор детерминирован — все узлы вычисляют `active_chain_length(W)` независимо из state Node Table и получают одно и то же значение.

Если active_chain_length падает ниже минимума жизнеспособности (теоретически возможно при массовом offline) — финализация останавливается до восстановления активности. Halt by liveness, не by safety: вернувшиеся узлы возобновляют работу с последнего cemented state.

Трафик confirmations: ~100 bundles × ~4 KB ≈ 400 KB за окно. Стабильно при любом масштабе.

Узлы-наблюдатели (chain_length < threshold) получают bundles, верифицируют endpoint и подписи, подсчитывают quorum, применяют cemented операции. Не публикуют confirmations.

#### State transition

Два параллельных процесса обновления состояния:

**Применение операций по window close.** Cemented операции окна W буферизуются до момента сборки proposal_{W+1}. Множество cemented операций фиксируется proposer-ом через frozen view (Lookback Leadership). Все cemented операции окна W применяются батчем в детерминированном порядке:

```
Порядок apply: по op_hash lex asc
```

Каждый аккаунт имеет максимум одну cemented операцию в окне W (dependency rule). Порядок между аккаунтами — лексикографически по op_hash. Детерминирован, вычислим независимо каждым узлом.

Apply каждой операции:

```
Transfer (Mode A — receiver уже в Account Table):
              sender.balance   -= amount
              receiver.balance += amount
              sender.frontier_hash = H(operation)
              update_merkle_path(sender)
              update_merkle_path(receiver)

Transfer (Mode B — receiver не существует, создание AccountRecord):
              sender.balance -= amount
              sender.frontier_hash = H(op)
              sender.last_account_creation_window = current_window   # [I-15] cooldown per τ₂
              update_merkle_path(sender)
              создать запись Account Table[link] = {
                  balance              = amount,
                  current_pubkey       = payload.receiver_pubkey,
                  suite_id             = payload.suite_id,
                  is_node_operator     = 0,
                  frontier_hash        = 0x00...00,
                  op_height            = 0,
                  account_chain_length = 0,
                  account_chain_length_snapshot = 0,
                  last_account_creation_window = 0,
                  creation_window      = current_window,
                  last_op_window       = current_window,
              }
              insert_merkle_leaf(new_account)

ChangeKey:    account.current_pubkey = new_pubkey
              account.suite_id = new_suite_id
              account.frontier_hash = H(operation)
              update_merkle_path(account)

Anchor:       записать data_hash в цепочку аккаунта (frontier_hash обновлён)
              update_merkle_path(account)

После каждой операции: account_root = current root.
```

**При apply каждой операции** обновляется AccountChain length signer-аккаунта (подписавшего операцию):

```
on_operation_applied(operation, window W):
  signer = operation.sender   # account_id из payload
  signer.account_chain_length += 1
  signer.last_op_window = W
  signer.op_height += 1
  # Получатель Transfer не получает обновления chain_length —
  # пассивное получение не считается активностью.
```

Dependency rule: один аккаунт = одна операция за окно τ₁. Каждая cemented операция = +1 к account_chain_length = одно окно присутствия.

**State transition в proposal:** при settle (apply at window close) применяется атомарно:

```
apply_proposal(state, proposal) -> state':

  Шаг 1: применить control_set в порядке (cemented_window asc, op_hash lex asc).
    NodeRegistration: проверить node_id уникален (нет в Node Table и Candidate Pool),
                      проверить ML-DSA-65 signature валидна для node_pubkey над signed_scope (Правило R1),
                      проверить ML-DSA-65 operator_pop валидна для operator_pubkey
                        над bytes ("mt-operator-pop" || node_pubkey)
                        (proof of possession с class domain separator;
                         reject InvalidOperatorPoP если не валидна),
                      проверить operator_account_id == SHA-256("mt-account" || suite_id || operator_pubkey),
                      если Account Table[operator_account_id] существует:
                        проверить is_node_operator == 0
                          AND current_pubkey == operator_pubkey,
                      проверить W_p - max_vdf_horizon ≤ W_start ≤ W_p - base_vdf_length
                        (max_vdf_horizon = 2 × τ₂_windows; base_vdf_length = τ₂_windows = vdf_entry_windows),
                      применить incremental apply в окне W_p:
                        sort cemented NodeRegistrations окна W_p by nr_sort_key,
                          где nr_sort_key(nr) = SHA-256(
                            "mt-nodereg-sort" ||
                            timechain_value(W_p) ||
                            cemented_bundle_aggregate(W_p - 2) ||
                            nr.node_pubkey
                          ),
                        for each NR in sorted order:
                          current_pending = pending_candidates(W_p) + N_applied_this_window
                          current_pressure = current_pending / active_nodes(W_p)
                          required_vdf_length(NR) = adaptive_formula(current_pressure)
                          if NR.vdf_chain_length < required_vdf_length(NR):
                            reject NR (insufficient VDF work)
                            continue
                          верифицировать proof_endpoint: пересчёт VDF от
                            SHA-256("mt-candidate-vdf-init" ||
                                    timechain_value(W_start) ||
                                    cemented_bundle_aggregate(W_start - 2) ||
                                    node_id)
                            через NR.vdf_chain_length окон,
                          если endpoint не совпадает: reject NR
                          создать запись в Candidate Pool:
                            node_id, node_pubkey, suite_id, operator_pubkey, operator_account_id,
                            proof_endpoint, W_start, vdf_chain_length,
                            registration_window = W_p,
                            expires = W_p + 3 × τ₂_windows.
                          N_applied_this_window += 1.

  Шаг 2: применить Монтана победителя.
    # reward константа: EMISSION_moneta из ProtocolParams, не зависит от окна.
    r = ProtocolParams.emission_moneta
    operator_account = Node Table[winner_id].operator_account_id
    operator_account.balance += r
    # supply_moneta не отслеживается как state-поле; вычислимо closed-form
    # supply_moneta(W) = EMISSION_moneta × (W + 1) от любого окна.

  Шаг 3: обработать expiry кандидатов и selection event.
    3a. Все записи c ∈ Candidate Pool где c.expires <= current_window:
        удалить c из Candidate Pool, обновить candidate_root.
    3b. Selection event (если current_window % 336 == 0):
        candidates = все записи Candidate Pool где expires > current_window
        slots = max(1, floor(active_nodes(current_window) / 130))
             -- admission_divisor = 130 ⟹ per-event admission rate = 1/130 = 0.77% ≤ 1% upper bound
             -- (обоснование: таблица «Обоснование протокольных констант → admission_divisor»)
        sort_key(c) = SHA-256(
          "mt-selection" ||
          timechain_value(current_window) ||
          cemented_bundle_aggregate(current_window - 2) ||
          c.node_id
        )
        selected = первые slots кандидатов по sort_key
        Для каждого selected:
          создать запись в Node Table (start_window = current_window, chain_length = 1,
          last_confirmation_window = 0, operator_account_id зафиксирован)
          если Account Table[selected.operator_account_id] не существует:
            создать AccountRecord для operator-аккаунта (atomic activation):
              account_id                   = selected.operator_account_id
              balance                      = 0
              suite_id                     = selected.suite_id
              is_node_operator             = 1
              frontier_hash                = 0x00...00
              op_height                    = 0
              account_chain_length         = 0
              account_chain_length_snapshot = 0
              current_pubkey               = selected.operator_pubkey
              creation_window              = current_window
              last_op_window               = current_window
              last_account_creation_window = 0
            обновить account_root.
          иначе:
            установить Account Table[selected.operator_account_id].is_node_operator = 1
            (existing AccountRecord — owner ранее уже мог иметь user-аккаунт через Transfer Mode B; balance, frontier_hash, account_chain_length и прочие поля остаются как есть)
          удалить selected из Candidate Pool
          обновить node_root и candidate_root.

**Grinding resistance selection event.** Domain separator `mt-selection` отделяет hash space от `mt-lottery` и других. Компонент `cemented_bundle_aggregate(current_window - 2)` — канонический но unpredictable offline (зависит от ML-DSA-65 подписей confirmers окна current_window-2). Атакующий с VDF hardware advantage, пре-вычисляющий `timechain_value` для будущих selection events, не может пре-вычислить sort_key без privкey confirmers. Grinding keypair (генерация N kerpairs для выбора favorable node_id) не работает: к моменту selection event sort_key определён будущими signatures, которые атакующий не контролирует.

  Шаг 3.5: обновить chain_length активных узлов.
    Для каждого узла N с cemented BundledConfirmation в окне W:
      N.chain_length += 1
      N.last_confirmation_window = W
      update_merkle_path(N) в node_root
    Множество узлов с cemented BundledConfirmation в окне W детерминировано
    (cemented status объективен) — все узлы применяют один и тот же набор обновлений.

  Шаг 3.6: обновить chain_length_snapshot на τ₂-boundary.
    Если current_window % τ₂_windows == 0:
      Для каждого узла N в Node Table:
        rotate N.chain_length_checkpoints (сдвиг: oldest выбывает, текущий chain_length записывается как newest)
        N.chain_length_snapshot = N.chain_length - N.chain_length_checkpoints[oldest]
        update_merkle_path(N) в node_root
    Между τ₂-boundaries: chain_length_snapshot вычисляется как chain_length - frozen oldest checkpoint.
    Детерминированно: все узлы применяют одну и ту же ротацию на одной τ₂-boundary.

  Шаг 4: node_root, candidate_root и account_root уже отражают все cemented изменения
         (incremental Merkle update произошёл при каждом state transition).
         state_root = SHA-256("mt-state-root" || node_root || candidate_root || account_root).
```

Порядок детерминирован. Каждый узел применяет одни и те же шаги и получает один и тот же state_root.

AccountTable зависит от TimeChain, NodeChain и AccountChain. Обратных зависимостей нет.

Минимум для узла: **1 ядро CPU**. TimeChain VDF (непрерывное последовательное хэширование) + валидация операций (interleaved, overhead < 1% окна). С ростом TPS сети дополнительные ядра ускоряют верификацию операций. Один узел = 1 ядро. Любой компьютер = потенциальный узел. Верификация операций аккаунтов полностью параллелизуется — цепочки аккаунтов независимы.

### Вход и регистрация

Два уровня входа в сеть. Узлы участвуют в консенсусе — открытый вход через VDF + selection event, AccountRecord operator-а создаётся атомарно при cementing Selection event. Аккаунты пользователей держат и переводят средства — AccountRecord появляется автоматически при первом входящем `Transfer` Mode B (receiver не существует — создание + зачисление amount). Самоинициация создания аккаунта невозможна; отдельного opcode активации не существует.

**Genesis State — аксиома сети.** Минимальный bootstrap: один узел, один operator-аккаунт. Bootstrap-узел стартует с положительным балансом операторского аккаунта и накапливает средства через лотерейные награды (`EMISSION_moneta` per выигранное окно). Распределение Монтана новым пользователям — через `Transfer` Mode B с cooldown `1 per sender per τ₂` per [I-15], ограничивающим темп создания новых AccountRecord.

**Bootstrap growth model.** Minimal Genesis (1 bootstrap operator account, первый и единственный держатель балансов до первой эмиссии) достаточен для запуска сети. Распространение Монтана через `Transfer` Mode B параллелизуется экспоненциально — каждый AccountRecord, накопивший положительный баланс и прошедший cooldown `1 Transfer Mode B per τ₂`, может создать новый AccountRecord:

- N = 0: 1 аккаунт (genesis operator)
- N = 1 τ₂: operator создаёт один новый AccountRecord → 2 accounts
- N = 2 τ₂: оба создают по новому → 4 accounts
- N = k τ₂: `2^k` accounts

Quantify rollout в окнах:

- 1 000 accounts: `⌈log₂(1000)⌉ = 10 τ₂`
- 1 000 000 accounts: `⌈log₂(10⁶)⌉ = 20 τ₂`
- 1 000 000 000 accounts: `⌈log₂(10⁹)⌉ = 30 τ₂`

**N_SEED как consensus-binding параметр Genesis Decree (v35.26.0).**

`N_SEED` formalized как поле Genesis Decree `genesis_active_operators` — упорядоченный список (account_pubkey, node_pubkey) пар, представляющих additional Active operators, инициализированных в Node Table / Account Table вместе с bootstrap. Reference mainnet может декларировать `N_SEED ≥ 1` для co-validator launch (multi-organization genesis cohort), сохраняя следующие инварианты:

- **Singleton proposer model сохранён.** Bootstrap (`bootstrap_node_pubkey`) — единственный canonical proposer; N_SEED operators участвуют как Active confirmers через `BundledConfirmation`, не как альтернативные proposers. Lookback proposer rotation активируется только при дальнейшем admission через standard `selection_event` path; bootstrap-instance остаётся canonical pre-rotation.
- **Quorum определён.** Initial `Σ chain_length = 1 + N_SEED` (каждый Active имеет chain_length=1 при window=0); `quorum = ⌈67 × Σ / 100⌉`. Для N_SEED=4 (5-Active cohort): Σ=5, quorum=4 (требует ≥3 confirmers помимо self).
- **Genesis State Hash включает N_SEED.** `genesis_account_root` строится как sparse Merkle root над {bootstrap_account} ∪ {account_i для каждого i ∈ N_SEED}; `genesis_node_root` аналогично. Cross-implementation: byte-exact идентичные protocol_params (включая `genesis_active_operators`) обязательны на всех узлах cohort; mismatch — fail-stop reject.
- **Canonical ordering.** Записи в `genesis_active_operators` упорядочены по `derive_node_id(node_pubkey)` лексикографически; sparse-Merkle root order-independent, но строгий порядок упрощает audit и обеспечивает byte-exact canonical_encode(protocol_params).
- **Атрибуты N_SEED operator records** идентичны bootstrap-records: `chain_length=1`, `chain_length_snapshot=1`, `start_window=0`, `last_confirmation_window=0`, `is_node_operator=true`, `balance=0`, `frontier_hash = SHA-256("mt-genesis" || account_id)`, `op_height=0`, `account_chain_length=0`.
- **Post-genesis admission flow без bypass.** N_SEED определяет ТОЛЬКО initial cohort window=0; все узлы, регистрирующиеся после genesis (`NodeRegistration` operation в окне >0), проходят полный `CandidateVdf → Registered → Active` путь через standard `selection_event` без исключений.

Для devnet / тест-cohort N_SEED конфигурация может варьироваться через alternate Genesis Decree сборку (separate binary build с другим `genesis_params()`); production reference mainnet фиксирует Genesis Decree в коде, immutable runtime.

Growth начинается с первого cemented τ₁ окна и не требует дополнительных specialized механизмов — существующее правило `Transfer` Mode B + cooldown [I-15] покрывает весь жизненный цикл roll-out.

Начальное состояние, существующее до того как любая операция возможна:

```
Genesis State (до первого окна, supply = 0):

  Account Table = 1 запись (bootstrap operator account):
    account_id      = SHA-256("mt-account" || suite_id || pubkey_0)
    balance         = 0
    suite_id        = 0x0001 (ML-DSA-65)
    is_node_operator = 1
    current_pubkey  = pubkey_0 (bootstrap)
    frontier_hash   = SHA-256("mt-genesis" || account_id)
    op_height       = 0
    account_chain_length = 0
    account_chain_length_snapshot = 0
    creation_window = 0
    last_op_window  = 0
    last_account_creation_window = 0

  Node Table = 1 запись (bootstrap node):
    node_id                  = SHA-256("mt-node" || node_pubkey_0)
    node_pubkey              = node_pubkey_0 (bootstrap)
    suite_id                 = 0x0001
    operator_account_id      = account_id_0
    start_window             = 0
    chain_length             = 1
    last_confirmation_window = 0

  Candidate Pool = ∅

  Bootstrap-узел стартует с chain_length = 1 в Genesis. Каждый последующий cemented BundledConfirmation инкрементирует chain_length. Начальное значение = 1 (а не 0) — необходимо для корректности знаменателей weighted_ticket_node и seniority_term; инвариант chain_length ≥ 1 сохраняется для любого узла в Node Table.

  genesis_account_root    = sparse Merkle root над 1 записью Account Table
  genesis_node_root       = sparse Merkle root над 1 записью Node Table
  genesis_candidate_root  = empty_internal(256) — root пустой sparse Merkle tree
                            per формуле раздела «Sparse Merkle Tree algorithm»:
                              empty_internal(0) = 0x00 × 32
                              empty_internal(k+1) = internal_hash(empty(k), empty(k))
                              internal_hash(l, r) = hash("mt-merkle-node", [l, r])
                                                  = SHA-256("mt-merkle-node" || 0x00 || l || r)
                            Детерминистически вычислим; НЕ равен 0x00 × 32 при depth=256.

                            Binding test vectors (byte-exact, с NUL-separator canonical hash):
                              empty_internal(0)   = 0x0000000000000000000000000000000000000000000000000000000000000000
                              empty_internal(1)   = 0x693bc03e469cd59e381575e0b3e178b40796ec2253869fe03eaee34750a06517
                              empty_internal(128) = 0x1b16a1c4eb2ed66902595a6d2ec642a05bed9db4897f5d910092b1a899a8a8b3
                              empty_internal(256) = 0x87dd145ec5630decdf8fc800583c51cce9dbe8438a1fa0b7e61eb679b4b4638f
                            Значение empty_internal(256) = binding genesis_candidate_root.
  genesis_state_root      = SHA-256("mt-state-root" || genesis_node_root || genesis_candidate_root || genesis_account_root)

  protocol_params (каноническая сериализация, little-endian, фиксированная длина полей):
    D₀                             (8B)   начальное значение D TimeChain VDF (325 000 000, hex 0x135F1B40)
    (reserved)                      (8B)   = 0x00 × 8 (mandatory)
    τ₂_windows                     (8B)   число окон в τ₂ (20 160)
    emission_moneta                (16B)  13_000_000_000 moneta (u128, EMISSION_moneta — константа эмиссии за окно)
    target₀                        (32B)  начальный target лотереи
    confirmation_quorum_num        (1B)   67
    confirmation_quorum_den        (1B)   100
    participation_dead_zone_low    (2B)   85
    participation_dead_zone_high   (2B)   95
    d_adjustment_rate_num          (2B)   3
    d_adjustment_rate_den          (2B)   100
    vdf_entry_windows              (8B)   20 160 (= τ₂)
    selection_interval             (8B)   336
    admission_divisor              (8B)   130 (slots = max(1, floor(active_nodes / 130)) per selection event)
    candidate_expiry_windows       (8B)   60 480 (3τ₂)
    adaptive_vdf_threshold         (2B)   1 (= 0.01 × 100, порог давления 1%)
    adaptive_vdf_multiplier        (2B)   100 (effective_vdf = base × pressure × multiplier)
    pruning_idle_windows           (8B)   80 640 (4τ₂)
    bootstrap_pow_difficulty       (4B)   65 536 (2^16) — consensus-binding anti-flood target
    max_protocol_payload_bytes     (4B)   1 048 576 (1 MiB) — wire-format upper bound payload_length
    max_sf_ciphertext_bytes        (4B)   65 536 (64 KiB) — wire-format upper bound SF envelope ciphertext
    bootstrap_account_pubkey      (1952B)
    bootstrap_node_pubkey         (1952B)
    n_seed                          (2B)   u16 LE — число additional Active operators в genesis cohort (0 для reference singleton)
    genesis_active_operators       (n_seed × 3904B)  список (account_pubkey 1952B + node_pubkey 1952B), упорядочен лексикографически по node_id_hex
    genesis_content_app_id         (32B)  = SHA-256("mt-app" || "montana")
    genesis_content_data_hash      (32B)  хэш манифеста книги Монтана v1.0

  Genesis State Hash = SHA-256("mt-genesis" || genesis_state_root || canonical_encode(protocol_params))
```

Domain separator `"mt-genesis"` обеспечивает structural разделение от других hash compositions (единое правило Domain separators registry — все consensus hash compositions содержат domain separator первым).

Bootstrap keypair (account + node) публикуется в Genesis Decree вместе с протокольными параметрами и Genesis State Hash. Genesis Decree immutable — закреплён в коде каждой реализации.

**Инварианты Genesis Decree:**

- Все поля `protocol_params` имеют фиксированные значения согласно layout выше; implementer хардкодит их в коде, runtime mutation **запрещена**
- Reserved поле `(reserved) = 0x00 × 8` строго; любое другое значение — **reject** (изменяет Genesis State Hash и создаёт несовместимую сеть)
- `bootstrap_account_pubkey` и `bootstrap_node_pubkey` соответствуют эталонным значениям закреплённым в коде
- `n_seed ≥ 0` (u16 LE), значение фиксировано в коде; runtime override запрещён
- При `n_seed > 0`: `genesis_active_operators` содержит ровно `n_seed` записей, каждая `(account_pubkey 1952B + node_pubkey 1952B)`; список упорядочен лексикографически по `derive_node_id(node_pubkey)`; дубликаты pubkey запрещены — reject (изменяет Genesis State Hash)
- Каждый N_SEED operator получает initial NodeRecord (chain_length=1, start_window=0) + AccountRecord (balance=0, is_node_operator=true); pre-seed входит в `genesis_node_root` и `genesis_account_root` через sparse Merkle root over canonical-encoded records
- Consensus-binding сетевые параметры protocol_params: `bootstrap_pow_difficulty == 65 536`,
  `max_protocol_payload_bytes == 1 048 576`, `max_sf_ciphertext_bytes == 65 536`;
  любое отклонение — **reject** (изменяет Genesis State Hash); все три
  `runtime mutation запрещена`; полная derivation per «Академическое
  обоснование констант» — sub-section «Сетевые параметры в protocol_params»
  раздела «Карточки замыкания механизмов сетевого слоя».
- Локальные policy параметры (`max_outbound_per_node`, `max_inbound_per_node`,
  `max_pending_requests_per_peer`, `request_timeout_t1_div`) **не входят
  в Genesis Decree** — они operator-configurable defaults, описаны в
  карточках Peer selection / ProtocolMessage envelope; operator может
  override без consensus impact.
- `genesis_state_root = SHA-256("mt-state-root" || genesis_node_root || genesis_candidate_root || genesis_account_root)` пересчитывается из сериализованных начальных таблиц и сверяется byte-exact
- `Genesis State Hash = SHA-256("mt-genesis" || genesis_state_root || canonical_encode(protocol_params))` совпадает с эталонным значением закреплённым в коде реализации
- Любое отклонение — Genesis Decree недействителен, узел отказывается стартовать (fail-stop, не fallback)

**Калибровка D₀.**

Параметр `D₀ = 325 000 000` (hex `0x135F1B40`) — результат **единственного исторического quartz-замера** в жизни протокола, проведённого на генезис-железе до запуска сети (per [I-18]).

**Genesis hardware reference profile:**

```
Машина:        Apple iMac (24-inch, M1, 2021), iMac21,1
Процессор:     Apple M1 base (4P + 4E cores), 8 GB unified memory
                ARM SHA-2 hardware extensions (FEAT_SHA256)
ОС:            macOS Sequoia 15.7.3 build 24G419
Kernel:        Darwin 24.6.0
Toolchain:     Rust 1.92.0 stable, target aarch64-apple-darwin
Profile:       release (lto=fat, opt-level=3, codegen-units=1)
SHA-256 backend: sha2 crate v0.10.9 (pure-Rust + ARM SHA-2 hw ext)
```

**Методология замера.** Цепочка `hash_{i+1} = SHA-256(hash_i)`, `hash_0 = [0u8; 32]`, single-thread, машина idle. Три последовательных прогона по 1 000 000 000 итераций. Median single-thread rate: **5.097280 MH/s**.

**Derivation D₀:**

```
Benchmark calibrated:  5.097280 MH/s × 60 секунд по кварцу = 305 836 793 хэшей
Runtime-corrected:     305 836 793 × (60 / 56.35) = 325 000 000  (округлено)
Genesis params.D₀:     325 000 000  (hex 0x135F1B40)
```

Runtime коррекция учитывает фактическую длительность окна на genesis-железе при VDF-цикле под нагрузкой консенсуса (validation, gossip, BC publication interleaving): чистый бенчмарк давал ~56.35 секунд по кварцу на 305 836 793 хэшей; масштабирование до целевых 60 кварцевых секунд даёт `D₀ = 325 000 000`.

**Single point of derivation truth.** Этот замер произошёл **ровно один раз** до запуска сети. После Genesis протокол не читает никакие часы (per [I-18]); число `D₀ = 325 000 000` зафиксировано в Genesis Decree `protocol_params.D₀` и неизменно. Любое post-genesis движение `D` — через canonical `participation_ratio` feedback per τ₂.

**Comparative observations** (illustrative, **не нормативные**, для понимания variance hardware capabilities):

| Hardware profile | Специфика | MH/s по локальному кварцу |
|------------------|-----------|---------------------------|
| **Genesis-железо** (iMac M1 2021, idle) | Apple M1, ARM SHA-2 hw ext, 8 GB | **5.097** (нормативный) |
| Idle commodity VPS (x86_64, no hw SHA) | QEMU Virtual CPU v4.2.0, 2.1 GHz, без hw SHA | ~3.68 |
| Loaded commodity VPS (x86_64, SHA-NI) | QEMU Virtual CPU v8.2.0 c SHA-NI, concurrent production сервисы на том же ядре | ~0.22 |

Comparative таблица иллюстрирует что hardware variance между классами достигает ×20+. Operator выбирает железо до запуска узла; недостаточная производительность означает participation_ratio < 0.85 → выпадение из active set через 8τ₂ inactivity pruning.

После старта сети `D` корректируется через canonical participation_ratio feedback per τ₂ (см. раздел «Адаптация D»). Сеть самостоятельно подстраивается под фактический состав операторов без обращения к каким-либо часам.

Первое окно τ₁ после генезиса — window_index = 0, protocol_version = 1. Bootstrap-узел — единственный proposer первых двух окон (без lookback). Начиная с W = 2 — стандартная lookback логика. Bootstrap-узел получает `EMISSION_moneta = 13 Ɉ` за каждое выигранное окно. Per-operation invariant действует с первого окна.

**Bootstrap period.** До появления второго узла (первые τ₂+ окон) bootstrap-узел имеет 100% active_chain_length и является единственным confirmer-ом, proposer-ом и winner-ом. Это физическая необходимость запуска любой сети — кто-то является первым. Доминирование bootstrap-узла размывается органически: каждый новый узел, прошедший selection event, вносит свой chain_length в active set. Протокольные правила (quorum 67%, weighted_ticket лотерея, selection rate limit) одинаковы с первого окна — специальных bootstrap-правил вне lookback первых двух окон нет.

**Границы модели доверия.**

Протокол имеет два режима доверия, автоматически переключаемые из canonical state.

**Режим Genesis.** Действует от Genesis до первого cemented BundledConfirmation от узла, отличного от bootstrap. В этот период безопасность протокола опирается на:
- Неизменность Genesis Decree (захардкожен в каждой реализации)
- Секретность bootstrap privкey (доверенная сторона — автор протокола)
- Отсутствие конкуренции — один участник, лотерея без значимых соперников, quorum тривиально достигается bootstrap-узлом

`cemented_bundle_aggregate` в этот период равен хэшу одной bootstrap-подписи. Защита [I-8] от grinding работает при секретности bootstrap privкey — стандартное допущение для Genesis-систем. Экономическая нерациональность атаки на single-node сеть компенсирует degraded security margin: нет Монтана rewards за победу над единственным участником, лотерея не даёт advantage.

**Режим BFT.** Активируется автоматически при первом cemented BundledConfirmation где `BC.node_id ≠ bootstrap_node_id`. В этот период безопасность опирается на:
- ≥67% честного active_chain_length
- `cemented_bundle_aggregate` из множества ML-DSA-65 подписей — полная защита [I-8] от pre-computation grinding
- Pruning + active_predicate поддерживают соотношение honest/attacker в составе active set

**Переход.** Автоматический, наблюдаемый из canonical state: Node Table содержит ≥1 non-bootstrap узел с chain_length ≥ 1. Версия протокола не меняется. Никакого ручного вмешательства или hard fork. Threat model сдвигается с «trust the Genesis author» на «trust ≥67% chain_length» плавно и непрерывно.

**Следствия для reference implementation.** Аудит и тестирование обязаны покрывать оба режима раздельно. Тесты bootstrap-периода проверяют поведение в Genesis-режиме (single-confirmer aggregate, bootstrap winning all lotteries, proposer ротация отсутствует). Тесты после bootstrap — BFT-поведение (multi-confirmer aggregate, weighted_ticket лотерея, lookback leadership). Переходный тест обязательно проверяет корректность передачи при первой non-bootstrap регистрации — один из критичных invariant-моментов в жизни сети.

**Mandatory content replication.** Каждый узел Монтана обязан хранить текущую версию книги Монтана как persistent blob по (genesis_content_app_id, genesis_content_data_hash). При Fast Sync новый узел загружает genesis content как часть обязательной начальной синхронизации (см. раздел Fast Sync).

#### Открытый вход узлов

Вход узла в консенсус — открытый. VDF τ₂ окон + кандидатура + selection event. Никаких приглашений, никаких разрешений.

**Шаг 1: Свободный вход.** Оператор-кандидат генерирует ML-DSA-65 keypair для оператора (`operator_pubkey`) и для узла (`node_pubkey`) offline; вычисляет `operator_account_id = SHA-256("mt-account" || suite_id || operator_pubkey)` и `node_id = SHA-256("mt-node" || node_pubkey)`. AccountRecord для `operator_account_id` на этом этапе ещё не существует в Account Table — он будет создан атомарно при cementing Selection event (см. Шаг 4). Кандидат подключается к gossip через node keypair (IBT уровень 2 — read-only gossip per [Montana Network](Montana%20Network%20v1.0.0.md) § Identity-Bound Tunnel), получает TimeChain values из proposals, вычисляет candidate VDF τ₂ окон от init. NodeRegistration включает оба pubkey (`node_pubkey` и `operator_pubkey`):

```
candidate_vdf_init = SHA-256(
  "mt-candidate-vdf-init" ||
  timechain_value(W_start) ||
  cemented_bundle_aggregate(W_start - 2) ||
  node_id
)
```

W_start — окно начала VDF (заявляется кандидатом в NodeRegistration).

**Шаг 2: Кандидатура.** После завершения VDF кандидат публикует NodeRegistration:

```
NodeRegistration:
  type                  1B   <- 0x11 NodeRegistration
  suite_id              2B
  node_pubkey        1952B
  operator_pubkey    1952B   <- ML-DSA-65 pubkey владельца operator-аккаунта
  operator_account_id  32B   <- = SHA-256("mt-account" || suite_id || operator_pubkey)
  proof_endpoint       32B     <- endpoint VDF цепочки (длина vdf_chain_length)
  W_start               8B     <- окно начала VDF
  vdf_chain_length      8B     <- длина VDF цепочки в "окнах" (по D хэшей)
  operator_pop       3309B   <- ML-DSA-65("mt-operator-pop" || node_pubkey, operator_secretkey) — proof of possession с class domain (см. Правило R2)
  signature          3309B   <- ML-DSA-65(signed_scope, node_secretkey) — Правило R1
Итого:           ~10 605 B
```

NodeRegistration — ControlObject. При cementing → запись в Candidate Pool. Кандидат ожидает selection event. При отборе кандидата на selection event в Node Table добавляется запись узла; одновременно atomically создаётся AccountRecord для `operator_account_id` если он ещё не существует (balance = 0, current_pubkey = operator_pubkey, suite_id = NodeRegistration.suite_id). Existing AccountRecord (если operator уже имел user-аккаунт через Transfer Mode B) — оставляется как есть, операция только устанавливает `is_node_operator = 1`.

**Инварианты NodeRegistration:**

- `type == 0x11` (первый байт; иное значение — не NodeRegistration, misrouting)
- `suite_id` соответствует активной схеме подписи (на момент запуска: `0x0001` = ML-DSA-65); прочие значения — **reject** (UnsupportedSuite)
- Подпись ML-DSA-65 (`signature`) валидна для `node_pubkey` над signed_scope (Правило R1; node-секретarity владелец подписал заявку)
- `operator_pop` валидна как ML-DSA-65 подпись для `operator_pubkey` над bytes (`"mt-operator-pop" || node_pubkey`) — proof of possession с class domain separator (Правило R2). Class domain закрывает cross-class signature confusion: signature над голыми `node_pubkey` bytes из любого другого контекста не пригодна как PoP. Только владелец `operator_secretkey` может произвести валидную подпись с этим domain — squatting на чужой operator_pubkey без знания соответствующего secretkey невозможен; даже при совпадении derivation формулы для `operator_account_id` отсутствие валидной PoP-подписи отвергает заявку (reject `InvalidOperatorPoP`)
- `node_id = SHA-256("mt-node" || node_pubkey)` уникален (нет в Node Table и Candidate Pool)
- `operator_account_id == SHA-256("mt-account" || suite_id || operator_pubkey)` (binding derivation operator_account_id из operator_pubkey, проверяется на момент cementing)
- Если `Account Table[operator_account_id]` существует → `is_node_operator == 0` (operator-аккаунт ещё не привязан к другому узлу) и `current_pubkey == operator_pubkey` (consistency: operator подписывает узлом тем же ключом, которым владеет аккаунтом). Если не существует → создание AccountRecord откладывается до Selection event apply.
- `W_p - 2 × τ₂_windows ≤ W_start ≤ W_p - base_vdf_length` (base_vdf_length = τ₂_windows). Нижняя граница ограничивает историческое pre-computation окно. Верхняя граница гарантирует что VDF физически выполнен до publication.
- `vdf_chain_length ≥ required_vdf_length(W_p)` — длина заявленной VDF цепочки не меньше требуемой pressure-adjusted длины в момент W_p (incremental apply в батче, см. apply_proposal)
- `proof_endpoint` верифицируем: пересчёт VDF от `SHA-256("mt-candidate-vdf-init" || timechain_value(W_start) || cemented_bundle_aggregate(W_start - 2) || node_id)` через `vdf_chain_length` окон. Если `vdf_chain_length × D hashes` VDF iteration от init даёт `proof_endpoint` — валидно.

Верификация: `vdf_chain_length` сегментов VDF проверяются параллельно. На C ядрах: ~(vdf_chain_length/C) × t_segment.

**[I-8] compliance.** `cemented_bundle_aggregate(W_start - 2)` в candidate_vdf_init — canonical & unpredictable-offline компонент (зависит от ML-DSA-65 подписей confirmers окна W_start - 2). Атакующий с VDF hardware advantage не может pre-compute init для произвольного будущего W_start — aggregate доступен только после cementing W_start - 2. Pre-computation grinding закрыт по построению.

**Шаг 3: Selection event.** Каждые `selection_interval = 336` окон сеть выбирает кандидатов из Candidate Pool. Полная каноническая формула `sort_key`, количество мест `slots = max(1, floor(active_nodes / 130))`, обработка expiry и включения в Node Table описаны в `apply_proposal` шаг 3b (раздел «Состояние сети → Двигатели → State transition»). Обоснование `admission_divisor = 130` и связь с upper bound 1% active_nodes per event — в таблице «Обоснование протокольных констант → Безопасность консенсуса и сети».

**Шаг 4: Регистрация.** Выбранные кандидаты → Node Table:

```
start_window = W (окно selection event)
chain_length = 1
last_confirmation_window = 0
```

Узел добавляется в Node Table с chain_length = 1 (позиция активации). Каждое последующее окно с cemented BundledConfirmation инкрементирует chain_length. Оператор-аккаунт получает `is_node_operator = 1`. Если `Account Table[operator_account_id]` ещё не существовал — создаётся атомарно в этом же шаге apply со всеми полями AccountRecord согласно authoritative описанию `apply_proposal` Шаг 3b (раздел «Состояние сети → Двигатели → State transition»). Запись удаляется из Candidate Pool.

**Expiry.** Кандидатура истекает через `candidate_expiry_windows = 3τ₂ = 60 480 окон` (см. Genesis Decree). Запись удаляется из Candidate Pool автоматически.

**Sybil-защита (четыре уровня):**

1. **VDF-барьер:** τ₂ окон последовательного хэширования (при нормальной нагрузке). Физическая работа, sequential по построению — не ускоряется параллелизмом.

2. **Adaptive VDF:** стоимость кандидатуры пропорциональна давлению на сеть в момент **публикации** NodeRegistration (не в момент начала VDF работы). Это закрывает timing-manipulation: attacker не знает заранее какое pressure будет в момент W_p.

```
candidate_pressure(W) = pending_candidates(W) / active_nodes(W)

if candidate_pressure(W) > 0.01:
    required_vdf_length(W) = τ₂_windows × candidate_pressure(W) × 100
else:
    required_vdf_length(W) = τ₂_windows   (base_vdf_length)
```

| Ситуация | pending | active | pressure | required_vdf |
|----------|---------|--------|----------|--------------|
| Нормальная | 5 | 1 000 | 0.5% | 20 160 окон (= τ₂, base) |
| Умеренная | 20 | 1 000 | 2% | 40 320 окон |
| Высокая | 100 | 1 000 | 10% | 201 600 окон |
| Атака | 1 000 | 1 000 | 100% | 2 016 000 окон |
| Массовая атака | 100 000 | 1 000 | 10000% | 201 600 000 окон |

**Привязка к W_p (не W_start).** `required_vdf_length` вычисляется из canonical state **в момент cementing NodeRegistration (W_p)**. Кандидат декларирует `vdf_chain_length` в NodeRegistration — длину своей VDF цепочки. Валидатор проверяет `vdf_chain_length ≥ required_vdf_length(W_p)` и корректность proof_endpoint через пересчёт VDF от init на `vdf_chain_length` окон.

**Incremental apply в батче одного окна.** Если несколько NodeRegistrations cemented в одно окно W_p, они применяются по canonical sort order с инкрементальным pending:

```
nr_sort_key(nr) = SHA-256(
  "mt-nodereg-sort" ||
  timechain_value(W_p) ||
  cemented_bundle_aggregate(W_p - 2) ||
  nr.node_pubkey
)

sort cemented_noderegs_W_p by nr_sort_key
for each NR in order:
  current_pending = pending_candidates(W_p) + N_already_applied
  current_pressure = current_pending / active_nodes(W_p)
  required = adaptive_formula(current_pressure)
  if NR.vdf_chain_length >= required:
    apply NR; N_already_applied += 1
  else:
    reject NR
```

Батч одного окна: первая NR видит pending baseline, каждая последующая видит +1. Required растёт в батче. Attacker не получает batch-advantage.

**[I-8] binding sort order.** Domain separator `mt-nodereg-sort` изолирует hash space. `cemented_bundle_aggregate(W_p - 2)` — canonical & unpredictable-offline компонент, зависящий от ML-DSA-65 подписей confirmers окна W_p - 2. Атакующий с hardware advantage не может пре-вычислить `nr_sort_key` без privкey honest participants → не может grind `node_pubkey` для favorable позиции в батче. Incremental apply неуязвим к keypair-grinding.

**Extension rule для honest operators.** Если первая попытка NodeRegistration rejected по `vdf_chain_length < required`, оператор может:
1. Продолжить VDF работу от текущего proof_endpoint на дополнительные окна
2. Обновить NodeRegistration: новый proof_endpoint = VDF(old_proof_endpoint, additional_length), vdf_chain_length = old + additional
3. Повторить publication с updated proof

VDF работа не теряется — только admission откладывается. Honest strategy: consecutive VDF extension пока required не удовлетворено.

**Self-correcting механика.** Чем сильнее давление → тем длиннее required VDF → дороже Sybil → давление падает через admission или expiry. При снижении давления (expiry 3τ₂ для просроченных кандидатов) → pending уменьшается → required нормализуется → легитимный вход восстанавливается.

**[I-8] compliance — grinding resistance.** Attacker не может предсказать `required_vdf_length(W_p)` в момент начала VDF: pressure зависит от будущих cemented NodeRegistrations и будущих BCs (active_nodes). Attacker не контролирует privкey honest participants → не может предвычислить pressure. Forced over-provisioning или extension rule.

**Timing manipulation закрыта.** Attacker не может начать VDF при низком давлении и подать при высоком — required проверяется на момент публикации. Minimum VDF (base_vdf_length = τ₂) достаточен только если pressure(W_p) ≤ 1%. Иначе нужен extended VDF пропорционально pressure.

**Slow-rate participation = организacный рост.** Если actor публикует ≤1 NodeRegistration per selection interval (336 окон), pending не накапливается (selection event admitting ~1% за event). Pressure остаётся baseline. Стоимость = минимум per candidate. Это **legitimate участие**, неотличимое от honest — и правильно не наказывается. Adaptive защищает только от превышения естественного темпа приёма.

3. **Selection rate limit:** max(1, active_nodes/130) за 336 окон. Массовый вход ограничен. Минимум 1 кандидат всегда проходит.

4. **Weighted механизмы:** chain_length определяет вес в quorum (безопасность). lottery_weight (snapshot 6τ₂ + seniority bonus) определяет вес в лотерее (эмиссия). Новые узлы начинают с минимальным влиянием. Время — единственный путь к весу.

#### Создание аккаунта

Два пути создания AccountRecord, ни один не требует отдельной opcode-операции активации:

1. **User-аккаунт через `Transfer` Mode B.** Получатель генерирует ML-DSA-65 keypair → вычисляет `account_id = SHA-256("mt-account" || suite_id || pubkey)` offline → делится `receiver_pubkey` / `account_id` с отправителем по out-of-band каналу (QR, сообщение, nickname lookup). Существующий аккаунт с положительным балансом публикует `Transfer` с расширенным payload (Mode B): `link == account_id` нового получателя, `receiver_pubkey` для binding derivation, `amount > 0`. Операция cemented → AccountRecord получателя появляется в Account Table при settle окна с `balance = amount`. Самоинициация создания невозможна.

2. **Operator-аккаунт через NodeRegistration → Selection event.** Кандидат-оператор включает `operator_pubkey` в NodeRegistration. При cementing Selection event для этого кандидата атомарно создаётся AccountRecord для `operator_account_id = SHA-256("mt-account" || suite_id || operator_pubkey)`, если он ещё не существует: `balance = 0`, `current_pubkey = operator_pubkey`, `is_node_operator = 1`. Operator не нуждается во входящем переводе для появления собственного AccountRecord — VDF + selection заменяют этот путь. Пополнение баланса operator-аккаунта — через лотерейные награды (`reward_moneta` per выигранное окно) либо входящие `Transfer` от других аккаунтов.

Sybil-барьер для user-аккаунтов: time-based — sender ограничен `1 Transfer Mode B per τ₂` (см. инварианты `Transfer` Mode B, поле `last_account_creation_window`). Fan-out на 10⁶ записей требует `⌈log₂(10⁶)⌉ = 20 τ₂` через binary tree expansion. Дополнительно account_chain_length определяет приоритет операций — новый аккаунт начинает с 1-op-per-τ₁ rate-limit. Рост приоритета = время. Пустые аккаунты пруняются через `balance == 0` + 4τ₂.

Sybil-барьер для operator-аккаунтов: sequential VDF `vdf_chain_length × D` SHA-256 хэшей до публикации NodeRegistration + selection_interval (336 окон) между событиями admission. Денежного барьера нет.

#### Скорость роста сети

Узлы: selection event каждые 336 окон, slots = max(1, active_nodes/130). Рост ограничен selection rate:

```
Genesis (1 узел):               1 новый узел за 336 окон
active_nodes = 100:             1 новый узел за 336 окон
active_nodes = 1 000:           10 новых узлов за 336 окон
active_nodes = 10 000:          100 новых узлов за 336 окон
```

Каждый кандидат проходит τ₂ окон VDF. Первые кандидаты появляются через τ₂ окон после genesis.

Сетевой TPS не зависит от |Node Table|. Монтана — replicated state machine, каждый узел обрабатывает все операции окна. Entry rate регулирует безопасность weight distribution и темп децентрализации, не пропускную способность. TPS масштабируется апгрейдом канала и CPU узлов, не их количеством. Сценарий «внезапная популярность → сеть не справляется с нагрузкой из-за медленного входа узлов» не применим к архитектуре Монтана.

Compound-рост при постоянном entry rate: удвоение сети ≈ `1.5 × τ₂` после первой волны (детальная derivation — таблица «Обоснование протокольных констант → admission_divisor»). Первая волна лагает на τ₂ (VDF вычисление первых кандидатов).

#### Barrier scope: что именно ограничено entry rate

Entry rate (τ₂ VDF + selection event) ограничивает **только** участие узла в консенсусе. Операционная функциональность узла не зависит от его статуса в Node Table.

**Доступно с момента установки узла (день 0, до регистрации):**

- P2P gossip и IBT: узел подключается к сети через level-3 addresses, получает proposals, синхронизирует state.
- Хранение данных владельца: узел хостит файлы, бэкапы, мессенджер-inbox своего оператора — это клиентский слой, не консенсусный.
- Почтовый ящик: входящие сообщения для операторского account_id накапливаются на узле пока телефон offline.
- Gateway для мобильного клиента: телефон оператора подключается к своему узлу через IBT уровень 3 (account-based auth), получает полный пользовательский функционал.
- Archival role: узел может хранить proposals, BundledConfirmations, исторические данные — в пользу своего оператора или по запросу application слоя.

**Доступно с момента появления AccountRecord в Account Table (account-level):**

- `Transfer` Mode A — исходящий перевод Монтана существующему аккаунту.
- `Transfer` Mode B — исходящий перевод с атомарным созданием AccountRecord для несуществующего получателя (расширяет пользовательскую базу через cooldown `1 Mode B per sender per τ₂`).
- Anchor — фиксация данных во времени (Merkle root над произвольным off-chain контентом).
- ChangeKey — ротация keypair.
- CloseAccount — явное закрытие с очисткой AccountRecord.
- Messaging через свой узел с постквантовым шифрованием ML-KEM (клиентский слой, не consensus-critical).
- Прикладные сервисы (никнеймы, премиум-функции, хранение, подписки) — оплата прямыми `Transfer` приложениям-провайдерам.

**Ограничено до entry в Node Table (после τ₂ VDF + selection event):**

- Node lottery: `weighted_ticket_node` требует `active_chain_length_snapshot`, зарабатывается только после entry.
- Confirmer eligibility: top ~100 chain_length → новый узел далеко от threshold до накопления окон присутствия.
- Вес в quorum: `active_chain_length = 0` до entry, голос узла не считается в 67% threshold для cementing и conflict resolution.
- Монтана emission for node: node reward payout требует `chain_length > 0`.

**Ортогональность TPS и entry rate:**

Пропускная способность сети определяется пропускной способностью канала и CPU активных узлов (replicated state machine — каждый узел обрабатывает все операции). Entry rate регулирует темп ввода новых узлов в консенсусную роль, не скорость обслуживания пользователей.

- Сеть из 100 узлов и сеть из 10 000 узлов обслуживают пользователей с тем же `TPS_network = min over nodes (TPS_node)`.
- User onboarding не зависит от node onboarding. `Transfer` Mode B cemented в одном окне, settled в конце того же окна — получатель готов к исходящим операциям начиная со следующего окна.
- Взрывной рост пользовательской базы абсорбируется апгрейдом канала существующих узлов, не входом новых.

**Резюме:** барьер τ₂ VDF защищает weight distribution и консенсусную безопасность. Он не ограничивает пользовательский доступ, пропускную способность сети, работоспособность новых узлов как инфраструктуры владельца, или скорость распространения сети среди пользователей.

User-аккаунты: создаются автоматически при первом входящем `Transfer` Mode B (расширенный payload с `receiver_pubkey`). Рост пользовательской базы определяется распространением Монтана через сеть переводов — каждый новый пользователь требует existing-аккаунт с положительным балансом, готовый передать первичный перевод и прошедший cooldown `1 Transfer Mode B per τ₂`. Самоинициация создания невозможна. Operator-аккаунты: создаются атомарно при cementing Selection event для NodeRegistration; не требуют входящего перевода.

---

## Потоковая модель

Операции аккаунтов текут непрерывно. Узел получает операцию → проверяет подпись ML-DSA-65 и баланс (против settled state W-1) → передаёт в P2P gossip. Confirmers (~100 узлов с наибольшим chain_length) собирают операции за окно и публикуют BundledConfirmation.

Операция проходит два состояния:
- **Cemented** (quorum event): 67% active_chain_length подтвердили. Операция необратима. Баланс ещё не обновлён.
- **Settled** (конец окна, apply at window close): все cemented операции окна применены к Account Table батчем. Баланс обновлён. state_root зафиксирован в proposal.

Два параллельных процесса:
- **Операции** подтверждаются непрерывно через confirmations (cement), применяются батчем в конце окна (settle)
- **Часы** тикают по расписанию окон τ₁ (TimeChain, лотерея, Монтана)

Кошелёк получателя отображает входящий перевод в два этапа: «confirmed» после cement (quorum event), «settled» после apply at window close (apply at window close). Между cement и settle операция уже необратима — различие только для UX индикации.

Цепочки аккаунтов полностью независимы. Операции разных аккаунтов обрабатываются параллельно без конфликтов.

---

## Временные слои (τ)

```
τ₁ = 1 window  →  τ₂ = 20 160 windows
```

Одно окно — τ₁. Всё остальное — производные в window counts.

### τ₁ — Окно (D хэшей)

Единственная единица канонического порядка протокола. Регистрация одного окна канонического порядка и эмиссия.

- TimeChain продвигается на `D` хэшей
- NodeChain: chain_length инкрементируется при cemented BundledConfirmation
- Операции аккаунтов подтверждаются непрерывно через confirmations (cement), применяются батчем в конце окна (settle)
- control_set: все cemented ControlObjects из окон (previous_proposal.window, current_window] (каноничен)
- Confirmers (~100) публикуют BundledConfirmation (операции текущего окна + VDF_Reveals предыдущего окна)
- Кандидаты (~12) публикуют VDF_Reveal с lottery endpoint = SHA-256(T_r || node_id || window_index); reveals цементируются через BundledConfirmation следующего окна
- Лотерея (single-class, только узлы): winner = argmin(weighted_ticket_node) среди cemented VDF_Reveal узлов-кандидатов (формулы `ticket_node`/`weighted_ticket_node` — integer spec per [I-9] в разделе «Класс 1: узлы» + «Integer log algorithm»). Аккаунты в лотерее не участвуют — см. «Аккаунты не участвуют в лотерее».
- Winner_{W-1} определяется proposer_W (= winner_{W-2}) из cemented VDF_Reveals окна W-1 и cemented account operations окна W-1
- Proposer (proposer_node_id) публикует подписанный proposal

- Финальность proposal: подпись proposer_node_id на proposal header. Каждый валидатор применяет control_set + Монтана детерминированно и проверяет state_root
- Монтана: регистрация одного окна канонического порядка (`reward_moneta(W) = EMISSION_moneta`) → победителю
- Supply audit: суммарная эмиссия Монтаны от генезиса = `supply_moneta(W) = EMISSION_moneta × (W + 1)` — closed-form pure function, state-поля не нужно
- Разрешение форков: приоритет ветки с наибольшим суммарным TimeChain-доказательством

TimeChain safety: компрометация значения TimeChain требует нарушения свойства последовательности SHA-256 VDF.

TimeChain liveness: задержка продвижения TimeChain невозможна — TimeChain вычисляется каждым узлом независимо.

### τ₂ — Адаптация (20 160 windows)

- Адаптация D через participation-ratio feedback (см. ниже)
- Snapshot account_chain_length: для каждого аккаунта `account_chain_length_snapshot = account_chain_length`. Snapshot — seniority-метрика активности аккаунта; читается прикладным слоем как anti-Sybil сигнал в собственных allocation-задачах. Детерминированно для всех узлов в пределах одного τ₂ интервала; на consensus-уровне в weights не входит
- Pruning Account Table: удаление пустых аккаунтов без активности 4τ₂ (52 000 окон) с обновлением Merkle путей
- Pruning Node Table: для каждого узла N где `(current_window - N.last_confirmation_window) > 8 × τ₂_windows`:
    1. Если `N.operator_account_id` существует в Account Table — установить `Account Table[N.operator_account_id].is_node_operator = 0` (operator-аккаунт освобождается от привязки к узлу; аккаунты в лотерее не участвуют — см. раздел «Аккаунты не участвуют в лотерее»)
    2. Удалить запись N из Node Table
    3. Пересчитать node_root
- Supply audit (sanity check): Σ balance(account) для всех аккаунтов == `supply_moneta(current_window) = EMISSION_moneta × (current_window + 1)` (closed-form, supply растёт строго монотонно линейно)
- Криптографическая амнезия: подписанные proposals сохраняются навсегда — верифицируемая цепочка state commitments. Proposals доказывают что конкретное состояние было закоммичено proposer-узлом; восстановление содержимого состояния требует snapshot или архива
- Пересчёт параметра D через participation-ratio feedback

#### Адаптация D через participation-ratio feedback

D адаптируется на границе τ₂ через каноническое chain observation — долю активного chain_length-а, успевшего подписать BundledConfirmation в каждом окне предыдущего τ₂.

**Канонический вход (real-valued commentary, authoritative integer form ниже):**

```
participation_ratio(W) = cemented_chain_length(W) / active_chain_length(W)
```

Где `cemented_chain_length(W)` — суммарный chain_length узлов, чьи BundledConfirmation для окна W попали в cemented set; `active_chain_length(W)` — суммарный chain_length узлов с `active(node, W) = true`. Оба числа канонически вычисляются каждым узлом bit-exact из Node Table и cemented sets.

**Real-valued form (commentary):**

```
median_ratio = median(participation_ratio(W) for W in последние τ₂_windows окон)

Если median_ratio >= 0.95:  D_new = D_old × 1.03    (+3%, сеть в комфорте, ускоряемся)
Если median_ratio <= 0.85:  D_new = D_old × 0.97    (-3%, сеть под давлением, замедляемся)
Иначе (dead zone):          D_new = D_old           (zone comfort, D не трогаем)
```

**Integer form (authoritative, per [I-9]):**
```
Encoding: participation_ratio представлен как permille (u32), точность 0.001.
  Диапазон: 0..=1000 (0 = 0%, 1000 = 100%).

Thresholds (из ProtocolParams, единицы — permille × 10):
  low_permille  = (u32 участие)(params.participation_dead_zone_low)  * 10   # 850
  high_permille = (u32)(params.participation_dead_zone_high) * 10           # 950

Rate (из ProtocolParams):
  rate_num = (u64)(params.d_adjustment_rate_num)      # 3
  rate_den = (u64)(params.d_adjustment_rate_den)      # 100

participation_ratio_permille(W): u32
  Input:
    cemented_chain_length(W): u64
    active_chain_length(W): u64    (≥ 1 по liveness invariant)
  Algorithm:
    participation_ratio_permille(W) = 
      ((cemented_chain_length(W) * 1000) / active_chain_length(W)) as u32
    # Unsigned u64 intermediate (cemented × 1000 ≤ 10^17 < 2^57, safe).
    # Integer div toward zero.
    # Cemented ≤ active ⇒ result ≤ 1000 ⇒ safely casts to u32.

median_ratio_permille: u32
  Algorithm:
    values = [participation_ratio_permille(W) for W in last τ₂_windows]
    sorted_values = sort_ascending(values)
    median_ratio_permille = sorted_values[τ₂_windows / 2]

D_next(D_old, median_ratio_permille): u64
  Algorithm:
    если median_ratio_permille >= high_permille:
      D_next = D_old * (rate_den + rate_num) / rate_den     # +3%, integer div toward zero
    иначе если median_ratio_permille <= low_permille:
      D_next = D_old * (rate_den - rate_num) / rate_den     # -3%, integer div toward zero
    иначе:
      D_next = D_old                                          # dead zone

Overflow: D_old ≤ 2^32 typical; D_old × 103 < 2^32 × 2^7 = 2^39 ⇒ safe u64.

Test vectors (binding):
  D_old = 1000, median_permille = 1000 (100%)                 → D_next = 1030
  D_old = 1000, median_permille =  950 (= high_permille)      → D_next = 1030
  D_old = 1000, median_permille =  980 (above high)           → D_next = 1030
  D_old = 1000, median_permille =  900 (dead zone)            → D_next = 1000
  D_old = 1000, median_permille =  851 (dead zone edge)       → D_next = 1000
  D_old = 1000, median_permille =  850 (= low_permille)       → D_next =  970
  D_old = 1000, median_permille =  700 (below low)            → D_next =  970
  D_old = 1000, median_permille =    0 (0%)                   → D_next =  970

[I-9] статус: закрыто (integer spec + 8 test vectors in spec).

Integer encoding: median_ratio_permille (uint32, 0..=1000). Точность 0.001
достаточна для threshold-сравнений на τ₂=13000 median; реализация в
`mt-timechain::next_d`.
```

**Семантика.**

- `median_ratio >= 0.95`: большинство активных узлов легко успевают подписать каждое окно. У сети есть запас производительности — D можно поднять, окно станет чуть длиннее в единицах работы, физическая скорость эмиссии замедляется, но сеть укрепляет запас прочности. Эмиссия в canonical окнах неизменна.
- `median_ratio <= 0.85`: значительная часть активных узлов не успевает подписать. Сеть близка к границе жизнеспособности — D нужно уменьшить, окно становится короче в единицах работы, медленные узлы получают шанс догнать медиану.
- Dead zone 0.85-0.95: естественные колебания, D не адаптируется. Это защита от реактивной волатильности.

**Properties.**

- **Канонически детерминировано.** participation_ratio вычисляется из canonical cemented sets и Node Table. Два честных узла получают одно и то же значение bit-exact.
- **Опирается только на canonical chain observations.** Все входные данные формулы — cemented sets и Node Table, оба детерминированы. Corollary I-3.a соблюдён.
- **Медленная реакция.** Adjustment rate ±3% за τ₂ делает стратегическую манипуляцию через withholding confirmations экономически нерациональной: actor-у с 10% chain_length-а для сдвига D на 2x требуется систематически saboтировать свои подписи ~24 эпохи, теряя все свои Монтана награды в этот период.
- **Dead zone защищает от флуктуаций.** Случайные колебания participation_ratio в диапазоне 0.85-0.95 не вызывают adaptation.
- **Interleaving на 1 ядре — ожидаемый источник participation_ratio < 1.0.** Узел делит одно ядро между TimeChain VDF и validation; контекстные переключения систематически приводят к пропуску отдельных окон. Это нормальное поведение 1-ядерной архитектуры, не деградация сети. Dead zone 0.85-0.95 поглощает типичный interleaving overhead; feedback ниже 0.85 автоматически уменьшает D, возвращая validation-у пропорционально больший кусок локального процессорного времени. Реализации не должны трактовать participation_ratio < 1.0 как отклонение и менять active predicate или пороги cementing в ответ.
- **Естественное следование hardware progress.** Если железо ускоряется, медианные узлы начинают успевать с запасом, median_ratio поднимается выше 0.95, D растёт, окно нормализуется. Сеть автоматически адаптируется к ожидаемому hardware evolution без явного measurement.
- **Нет stремления к hard fork по дизайну.** Continuous adaptation в рамках speech-first принципа устраняет необходимость периодического hard fork как запрограммированного события.

**Threat model:**

- Actor с <20% chain_length-а экономически не может сдвинуть median_ratio значимо.
- Hyperscaler с 15% узлов может систематически снижать median_ratio на ~15%, но только теряя свои награды. При clamp ±3% за τ₂ максимальный сдвиг D за 24 τ₂ составляет ±1.03^24 ≈ ±103%, что ограничено при правильном выборе `D₀` с запасом.
- Координированная атака узлов с >50% chain_length эквивалентна атаке на весь консенсус и не рассматривается в рамках локальной защиты participation_ratio.

**Genesis parameters:**

```
D₀ = 325 000 000    (3.25 × 10⁸, hex 0x135F1B40, см. «Калибровка D₀» в разделе Genesis)
participation_dead_zone_low  = 0.85
participation_dead_zone_high = 0.95
d_adjustment_rate = 0.03     (±3% за τ₂)
```

Параметр D₀ фиксируется в Genesis Decree. Остальные константы закреплены в протокольных параметрах и могут быть изменены только через protocol version upgrade (software hard fork), не через runtime mechanism.

---

## Консенсус — Proof of Time (PoT)

### Четыре цепочки

**TimeChain** — глобальные часы. Чистая VDF-цепочка `T_r = SHA-256^D(T_{r-1})`. Первичный продукт протокола. Источник времени и случайности. Продвигается по расписанию окон.

**NodeChain** — последовательность cemented BundledConfirmation конкретного node_id. chain_length — позиция узла в NodeChain: = 1 при активации, +1 при каждом cemented BundledConfirmation. Инвариант: chain_length ≥ 1. Доказывает присутствие узла.

**Account** — состояние счёта. Операции финализируются непрерывно через подтверждения (67% active_chain_length). ControlObjects включаются в proposal (каноничен).

Зависимости односторонние: TimeChain → NodeChain → AccountChain → AccountTable. Отказ в AccountTable не останавливает часы. Отказ конкретного узла в NodeChain не заражает общий ритм.

### Лотерея

Лотерея single-class: участвуют только узлы через VDF_Reveal с каноническим endpoint. Каждый узел производит weighted ticket по длине своей цепочки (`chain_length_snapshot`). Lowest weighted_ticket побеждает.

**Узлы** автоматически участвуют в каждом окне. Каноническая формула `weighted_ticket_node` и integer algorithm определены в разделе «Класс 1: узлы» и общем разделе «Integer log algorithm (per [I-9])» выше (single source of truth).

**Аккаунты** в лотерее не участвуют — см. раздел «Аккаунты не участвуют в лотерее» выше. Поле `account_chain_length_snapshot` — seniority-метрика активности аккаунта, используется прикладным слоем как anti-Sybil сигнал в собственных allocation-задачах.

Если weighted_ticket_node < target — узел кандидат. Target калиброван на ~13 кандидатов за окно. Из кандидатов побеждает lowest weighted_ticket.

**Стимул узла:** каждое окно с опубликованным BundledConfirmation увеличивает chain_length → увеличивает шанс победы. Пропущенное окно — это окно не входит в chain_length. Узел остаётся в Node Table и продолжает участвовать.

### Победитель τ₁

Победитель определяется после закрытия окна τ₁. Lowest `weighted_ticket_node` среди cemented VDF_Reveal узлов-кандидатов = победитель. Единственный класс победителя — узел.

**Если победил узел:**
- Записывает TimeChain value
- Operator account узла получает `reward(W)` Монтана
- Коммитит State Root
- Формирует proposal (control_set + State Root + Монтана), подписывает node_pubkey

Если в окне нет узлов-кандидатов (ни один `weighted_ticket_node < target`) — срабатывает fallback cascade: proposer выбирается из всех активных узлов по lowest weighted_ticket независимо от target. Liveness proposals гарантирована пока хотя бы один узел активен.

Финальность proposal — подпись proposer_node_id на proposal header. Верификация — независимый пересчёт state_root.

### Верификация

Proposer публикует: `{proposer_node_id, proposal}`.

Верификация lottery endpoint: один SHA-256 — O(1).

Верификация proposal: независимое применение control_set + Монтана и сравнение state_root.

### Устойчивость

- **Остановка TimeChain** исключена: каждый узел вычисляет VDF независимо
- **Искажение TimeChain** исключено: VDF последователен, результат детерминирован
- **Proposer grinding** исключён: control_set каноничен, state transition детерминирован, операции финализируются независимо от победителя
- **Front-running** исключён: операции финализируются через подтверждения (quorum event), proposer фиксирует frozen view
- **Предвычисление** исключено: seed содержит текущее значение TimeChain
- **Replay** исключён: TimeChain уникален для каждого τ₁
- **Аппаратное преимущество** ограничено: последовательное хэширование масштабируется тактовой частотой, не количеством ядер
- **Sybil-барьер**: τ₂ окон VDF + selection event (max 1% active_nodes за 336 окон) + weighted_ticket в лотерее
- **Цензура операций** исключена: операции финализируются через подтверждения узлов, не через победителя
- **Цензура ControlObjects** исключена: control_set каноничен, пропуск = fallback
- **Liveness halt операций** исключён: финализация через 67% active_chain_length, не зависит от победителя
- **Liveness halt proposals** исключён: fallback на следующего кандидата
- **Масштабирование**: трафик лотереи ~8.9 KB за окно при любом количестве узлов

### Разрешение конфликтов

**Двойная операция аккаунта** (две операции с одним prev_hash): equivocation. Cemented до обнаружения — необратимо, вторая отклоняется. Не cemented — ожидание quorum 13 окон, затем обе отклоняются. См. раздел «Двойная трата».

**Невалидный proposal**: валидаторы отклоняют, fallback на следующего кандидата. Победитель теряет `reward(W)` за это окно.

**Два proposal от одного proposer_node_id в одном окне**: оба отклоняются (equivocation), fallback к следующему узлу-кандидату. Если этот узел был winner, он теряет `reward(W)` (winner всегда узел в лотерее single-class).

---

## Адреса и переводы

### Полный флоу перевода

```
0. Боб offline: генерирует ML-DSA-65 keypair, вычисляет account_id
   = SHA-256("mt-account" || suite_id || pubkey). Делится
   (receiver_pubkey, account_id) с Алисой по out-of-band каналу
   (QR / сообщение).
1. Алиса (баланс ≥ первичный amount, прошедшая cooldown 1 Transfer Mode B per τ₂)
   публикует Transfer Mode B (расширенный payload с receiver_pubkey Боба):
   → cemented (quorum event) → settled (конец окна) →
   AccountRecord Боба зарегистрирован в Account Table
   (balance = первичный incoming amount, frontier_hash = 0, op_height = 0).
2. В последующих окнах Боб и Алиса обмениваются Transfer Mode A (короткий payload):
   Боб → Алисе: "отправь на mt4ZGfe..." (account_id Боба, уже в Account Table)
3. Алиса формирует Transfer Mode A (следующее окно после settle AccountRecord Боба):
   type:       0x02
   prev_hash:  хэш её предыдущей settled операции (frontier_hash из settled state W-1)
   payload:    sender (account_id Алисы) || link (account_id Боба) || amount (50_000_000_000 moneta)
4. Алиса подписывает ML-DSA-65
5. Алиса рассылает операцию узлам сети
6. Каждый узел проверяет (против settled state W-1):
   ML-DSA-65 подпись валидна для current_pubkey Алисы
   prev_hash совпадает с frontier_hash Алисы
   amount > 0
   alice.balance >= amount
   получатель (Боб) существует в Account Table
7. Confirmers публикуют BundledConfirmation, операция распространяется через P2P gossip
8. Cement: 67% active_chain_length подтвердили → операция необратима (quorum event)
   Кошелёк Боба отображает «confirmed»
9. Settle (apply at window close):
   alice.balance -= 50 Ɉ
   bob.balance   += 50 Ɉ
   alice.frontier_hash = H(operation)
   alice.op_height += 1
   alice.account_chain_length += 1
   Кошелёк Боба отображает «settled»
```

### Баланс

Баланс аккаунта — открытое число `u128 moneta` в Account Table. Обновляется при settle (apply at window close): исходящий Transfer вычитает amount, входящий зачисляет. Видим всем узлам и через любого верификатора цепочки.

Бэкап = seed (для деривации приватного ключа ML-DSA-65). Восстановление кошелька: ключ выводится из seed, баланс читается из текущего Account Table — никакого локального состояния не требуется.

---

## Эмиссия

### Единица

Определение валюты, тикера и деноминации — см. раздел «Валюта Монтаны — именование и деноминация».

Эмиссия за окно τ₁ определяется единственной формулой `reward_moneta(W) = EMISSION_moneta`, где `EMISSION_moneta` — константа Указа Генезиса (значение `13 × 10⁹ nɈ`, см. Genesis Decree `protocol_params.emission_moneta`). Награда фиксирована и не зависит от номера окна. Никакой надбавки, никаких эпох, никакого обновления ставки на уровне протокола.

### Регистрация окна

```
reward_moneta(W) = EMISSION_moneta
```

Каждое окно τ₁ регистрирует одно каноническое окно канонического порядка. `EMISSION_moneta` читается из `ProtocolParams`; значение фиксировано Указом Генезиса и не меняется на горизонте сети.

### Binding test vector (per [I-9])

| Окно W | reward_moneta (nɈ) |
|--------|--------------------|
| любое  | 13_000_000_000     |

Один тест-вектор: формула constant, проверка тривиальна — `reward_moneta(W) == EMISSION_moneta == 13_000_000_000` для всех W.

### Распределение

Победитель окна τ₁ — всегда узел — регистрирует одно каноническое окно и получает `EMISSION_moneta` Монтана (зачисляется на `operator_account_id` узла). Одна формула.

Лотерея single-class: конкурируют только узлы. Победитель — узел с lowest `weighted_ticket_node` среди cemented VDF_Reveal кандидатов окна. `chain_length_snapshot` и `seniority_term` определяют вес — время и непрерывность работы единственный арбитр.

Базовый бюджет: `EMISSION_moneta` Ɉ за окно. Покупательная способность Ɉ определяется рынком (через demand от app ecosystem), а не протоколом.

### Раннее участие — front-loading через CL accumulation

Bootstrap-фазы как отдельного механизма с надбавкой на уровне протокола нет. Front-loading распределения возникает естественно через `chain_length` accumulation:

- Ранние операторы запускаются при низкой конкуренции — выше доля выигранных окон в первый период
- Накопленный `chain_length` даёт permanent преимущество над поздними entrants через CL-weighted лотерею
- Узел, работающий дольше, побеждает чаще. Узел, запустившийся раньше, имеет преимущество — доказал больше окон присутствия

### Двигатель роста сети — app ecosystem driven

Экономические потоки между participants — переводы между аккаунтами через `Transfer`:

```
Активные пользователи в приложениях → платят разработчикам напрямую в Ɉ
        ↓                                            ↓
Приложения привлекают пользователей            Разработчики получают Монтаны
        ↓                                            ↓
Demand на Ɉ растёт через utility            Растёт реальная стоимость Ɉ
        ↓                                            ↓
Разработчики / пользователи поднимают узлы → Зарабатывают Монтаны через лотерею
        ↓                                            ↓
Лестница суверенности (Account → Operator)    Сеть растёт и децентрализуется
```

Эмиссия `EMISSION_moneta` направляется только на узлы (поддержание сети) — единственный денежный механизм протокола. Пользовательская активность поддерживает стоимость Ɉ через **utility demand** (приложения создают real value). Путь «Account → Operator» — единственный protocol-level способ для пользователя начать получать эмиссию.

Apps freely определяют свои бизнес-модели: цены на звонки, видеосвязь, премиум-функции, хранение данных — всё через прямые переводы Ɉ между аккаунтами.

---

## Пропускная способность

Правило «1 op/τ₁» — per-account, не сетевое. Сетевая пропускная способность определяется пропускной способностью канала узла и размером proposal; цепочки аккаунтов независимы и обрабатываются параллельно в одном окне.

Entry rate узлов (τ₂ VDF + selection event) ортогонален TPS сети. Узел операционен с момента установки — обслуживает своего оператора, хранит данные, работает gateway для мобильного клиента — вне зависимости от статуса в Node Table. Consensus-роль (вес, лотерея узла, confirmer) активируется после entry; user-level функциональность не ждёт.

Размер Transfer: ~3 422 B (открытый перевод, ML-DSA-65 подпись).

| Канал узла | TPS |
|-----------|-----|
| 10 Mbps | ~365 |
| 100 Mbps | ~3 650 |
| 1 Gbps | ~36 500 |

### Sizing guidance — mempool budget per узел

Implementation guidance для node operators, не consensus-critical параметр. Узлы с меньшим mempool budget отбрасывают операции при surge нагрузке — honest behaviour, не protocol violation.

Целевая нагрузка для масштаба 1B активных пользователей при средней частоте 1 операция в минуту на пользователя:
- Pending operations rate: ≈ 12K операций за окно τ₁
- Recommended mempool budget узла: ≥ 500 MB (вмещает ≈ 150K pending operations при ML-DSA-65 signature 3309 B)
- Минимальный mempool budget: 100 MB (≈ 30K pending operations — tight для 1B масштаба, удерживает примерно 2.5 окна τ₁)

Размер канонической подписи под ML-DSA-65 — основной множитель в pending-operation footprint; при оценке budget учитывать актуальный signature_size активной схемы (см. раздел «Криптография → Подписи»). Узлы с менее производительным каналом или меньшим bandwidth-budget могут уменьшать целевое окно retention pending operations соразмерно.

---

## Хранение

### Состояния операции (UX)

Операция проходит два различимых состояния:

```
publish ──→ cement (quorum event) ──→ settle (apply at window close)
            "confirmed"          "settled"
```

- **Cemented (quorum event):** 67% active_chain_length подтвердили операцию через BundledConfirmation. Операция необратима и гарантированно будет применена в конце окна. Wallet показывает «confirmed».
- **Settled (apply at window close, в конце окна):** все cemented операции окна применены батчем к Account Table в детерминированном порядке. account_root зафиксирован в proposal. Wallet показывает «settled».

Между cement и settle операция уже необратима — настройка двух UI-состояний нужна только для индикации завершённости state transition. Зависимые операции (Transfer на только что созданный аккаунт) сериализуются по окнам через confirmer dependency rule, поэтому cemented операция гарантированно settle-ится.

### Модель: глобальное состояние + локальная история

Узлы хранят глобальное состояние (Account Table, Node Table, Candidate Pool, proposals). Тела операций аккаунтов хранятся у владельцев. После settle (apply at window close) state transition применён — балансы в таблице обновлены, тело операции сети больше не нужно.

### Два участника

**Узел** — мой компьютер (десктоп, сервер, VPS), 24/7, минимум 1 ядро:

```
Consensus (протокольный слой):
  Account Table              (account_id, balance, frontier_hash, pubkey)
                             + persistent sparse Merkle tree (account_root)
  Node Table                 (node_id, pubkey, start_window, chain_length)
                             + persistent sparse Merkle tree (node_root)
  Candidate Pool             (node_id, pubkey, operator, proof_endpoint, W_start, vdf_chain_length, expires)
                             + persistent sparse Merkle tree (candidate_root)
  Proposals                  (навсегда)
  TimeChain VDF + валидация   (1 ядро, 24/7, validation interleaved)
  P2P gossip                 (операции, confirmations, reveals, proposals)

Данные владельца (клиентский слой):
  Локальное хранилище        (фото, файлы, бэкапы сообщений — зашифровано)
  Почтовый ящик              (входящие сообщения пока телефон офлайн)
```

Узел принадлежит оператору. Оператор решает что хранить помимо consensus state. Consensus state обязателен — без него узел не участвует в сети. Данные владельца — решение клиентского слоя: формат, шифрование, объём, retention.

**Ядра и производительность.** TimeChain VDF — sequential по построению; дополнительные ядра не ускоряют продвижение TimeChain. Второе ядро изолирует validation от VDF-цикла и устраняет interleaving overhead (пропуск окон ~5-10% при нагрузке). Узлы с 1 ядром полностью участвуют в консенсусе; узлы с 2+ ядрами имеют bounded преимущество в participation_ratio, ограниченное сверху interleaving overhead. Participation-ratio feedback на τ₂-boundary автоматически подстраивает D под фактическое железо медианы сети.

**Телефон (кошелёк)** — клиент моего узла, онлайн когда используется:

```
Хранит:
  Свои ключи            (seed → keypairs)
  Локальная история     (операции, сообщения — для UX)

Делает:
  Подключается к своему узлу
  Отправляет/получает переводы через узел
  Читает/пишет данные на свой узел
  Забирает сообщения из почтового ящика узла
```

Потеря телефона: seed восстанавливает ключи, баланс в Account Table публичен, данные на узле целы. Потеря узла: seed восстанавливает аккаунт, consensus state скачивается через Fast Sync. Данные владельца (фото, сообщения) — ответственность оператора (бэкап, RAID, репликация между своими узлами — клиентский слой).

Привязка телефона к узлу, авторизация, синхронизация, формат хранения данных — клиентский слой. Протокол предоставляет identity (account_id ↔ operator_account_id) как основу для привязки.

**Подключение без собственного узла.** Пользователь с аккаунтом но без узла подключает телефон к чужому узлу через IBT уровень 3 (account keypair). Чужой узел — это узел приложения Монтана (app creator's infrastructure), public node, или community-run узел. Соединение через Noise_PQ XX (ML-KEM-768 KEM + ML-DSA-65 identity sig) + ML-DSA-65 IBT proof — никто кроме владельца account privкey не может подключиться под его именем.

Хостящий узел gossip-ит операции пользователя в сеть так же как для локально подключённых accounts. Для пользователя процесс идентичен — кошелёк работает одинаково независимо от того свой узел или чужой.

Разница — хостящий узел видит IP и тайминг операций пользователя (metadata). Контент приложения (Anchor data) зашифрован — узел видит только хэш в сети, не содержимое. Dandelion++ на первом хопе частично обфусцирует origin операции от дальнейших хопов.

### Размеры

| Участник | Данные | Размер |
|----------|--------|--------|
| Узел (1M аккаунтов) | Account Table + Node Table + Candidate Pool + Proposals | ~3 GB |
| Узел (10M аккаунтов) | Account Table + Node Table + Candidate Pool + Proposals | ~22 GB |
| Узел (100M аккаунтов) | Account Table + Node Table + Candidate Pool + Proposals | ~210 GB |
| Кошелёк (обычный) | ~100 операций за 26 τ₂ + контакты + сообщения | ~1 MB |
| Кошелёк (активный) | ~10 000 операций за 26 τ₂ | ~16 MB |
| Корпорация | ~1M Anchor за 26 τ₂ | ~0.8 GB |

Размеры узла иллюстративны для сети возрастом ~26 τ₂ (emergent ≈ 1 год при genesis-калибровке, illustrative per [I-18]). При значительно более долгой работе сети суммарный размер растёт линейно за счёт proposal chain (`~2 GB на 26 τ₂`); Account/Node/Candidate Tables зависят от числа активных аккаунтов и узлов через лимиты [I-14] и pruning.

### Потеря данных клиента

Потеря телефона: seed восстанавливает ключи, баланс в Account Table публичен, данные на узле целы — полное восстановление. Потеря узла: seed восстанавливает аккаунт, consensus state — через Fast Sync. Данные владельца (фото, сообщения) — ответственность оператора. Бэкап, RAID, репликация между своими узлами — решения клиентского слоя.

### Быстрая синхронизация (новый узел)

1. Цепочка proposals от генезиса — проверка TimeChain-цепочки и подписей proposer-узлов (мегабайты)
2. Snapshot трёх таблиц (Account Table + Node Table + Candidate Pool) от пиров на момент окна W (произвольное недавнее окно)
3. Reconstructed `account_root`, `node_root` и `candidate_root` сравниваются с соответствующими полями из proposal окна W. Все три совпадают → snapshot валиден. Проверка `state_root = SHA-256("mt-state-root" || node_root || candidate_root || account_root)` — дополнительный integrity check.
4. Catch-up после окна W до текущего:
   - Запросить cemented UserObjects и применить их батчем к Account Table по алгоритму apply at window close (включая проверку prev_hash и баланса).
   - Запросить cemented ControlObjects (NodeRegistration) и применить их к Candidate Pool в детерминированном порядке. Применить selection events.
   - Выполнить incremental update Merkle trees (account_root, node_root, candidate_root) для отражения changes.
   - На каждом промежуточном proposal сверять локальный state_root с заявленным в proposal header
5. **Genesis content.** `genesis_content_data_hash` зафиксирован в Genesis Decree как протокольная константа. Загрузка книги Монтана по этому хэшу — конвенция reference implementation. Формат загрузки и верификации определяется клиентским слоем.
6. Узел синхронизирован и готов к участию

Snapshot привязан к конкретному proposal (settled state после apply at window close). Catch-up дистанция определяется свежестью snapshot — обычно несколько окон.

**Полнота сериализации snapshot.** Snapshot обязан содержать canonical byte-for-byte сериализацию всех записей каждой таблицы согласно определениям раздела «Состояние сети» — ВСЕ поля каждой записи, включая производные (chain_length_snapshot, checkpoints), счётчики (last_confirmation_window, op_height, account_chain_length), VDF-метаданные (vdf_chain_length, W_start, proof_endpoint) и pubkey material. Пропуск или изменение любого поля одной записи меняет её canonical serialization → меняется хэш листа Merkle tree → несовпадение с proposer-recorded root окна W → snapshot rejected, retry с другого пира.

Это делает полноту snapshot enforced криптографически через Merkle root comparison, не через явное перечисление полей в Fast Sync спецификации. Добавление нового поля в record format (будущая версия протокола) автоматически распространяется в snapshot через canonical encoding — Fast Sync логика не требует изменений. Единственное требование: canonical encoding и Node Table / Account Table / Candidate Pool definitions — single source of truth для serialization.

Reference implementation обязана сериализовать записи ровно по определениям state records с canonical byte ordering. Отклонения от canonical encoding в одной реализации = несовместимость с другими = невозможность Fast Sync между разными реализациями. Conformance tests должны включать snapshot serialization для эталонного state как один из test vectors.

---

## Прикладной слой

Монтана — цифровой стандарт времени. Приложения управляют своим состоянием самостоятельно (серверы, базы данных, P2P). Монтана хранит только криптографические отпечатки с привязкой ко времени — 32 байта на запись.

### Модель приложения на Монтана

Приложение Монтаны — клиентский слой над протоколом. Разработчик приложения может (а) запускать собственные узлы Монтаны для участия в консенсусе и эмиссии, (б) принимать прямые `Transfer` от пользователей за платные функции, (в) делать оба одновременно. Полная картина каналов дохода — см. «Полная экономическая картина» ниже.

**Для разработчика приложения:**

- Не нужно строить отдельную инфраструктуру безопасности — приватность данных через Anchor (хэш в сети, контент у владельца зашифрованным), антицензура через Transport Obfuscation и Dandelion++, децентрализация через отсутствие центрального сервера получаются бесплатно из протокола
- **Канал дохода А (опционально, если разработчик поднимает узлы):** эмиссия Монтаны через лотерею узлов. Связь с пользовательской базой — scale effect через committee selection probability, не linear ROI per user (детали — «Полная экономическая картина»)
- **Канал дохода Б (основной для большинства apps):** прямые `Transfer` от пользователей за платные функции — звонки, видеосвязь, премиум, хранение, разрешение имён, подписки на создателей. App определяет цену сам, пользователь платит напрямую на аккаунт разработчика через стандартный `Transfer`
- Hosting accounts пользователей: узлы приложения принимают подключения account-only пользователей через IBT уровень 3. Стандартный узел Монтаны умеет хостить accounts из коробки — отдельной инфраструктуры не требуется

**Для пользователя:**

- Каждое действие в приложении создаёт операцию в его AccountChain
- account_chain_length растёт автоматически с каждым окном с операцией
- Аккаунты не участвуют в лотерее — пользователь не получает эмиссию напрямую за активность. Путь к эмиссии — Лестница суверенности: поднять собственный узел (роль оператора) и зарабатывать через лотерею узлов
- account_chain_length_snapshot — seniority-метрика активности; читается прикладным слоем как anti-Sybil сигнал в собственных allocation-задачах. На consensus-уровне поле обновляется только τ₂ snapshot-ом и не влияет на веса лотереи
- Ничего не привязано к конкретному приложению — seed принадлежит пользователю, account_id переходит между приложениями без потери истории

**Нулевая стоимость переключения приложений.** AccountChain пользователя — его собственность. Если приложение закрылось или пользователь хочет уйти — account_id, баланс, история и накопленный account_chain_length остаются. Пользователь продолжает в другом приложении на том же протоколе. Приложения вынуждены конкурировать качеством, а не замком.

### Полная экономическая картина

Раздел consolidates все каналы дохода Монтаны в одной точке. Содержит сводную таблицу actor → revenue stream, разделение двух pathway, объяснение scale effect для оператора, иллюстрацию ROI для standalone оператора, типовые app business models.

**Сводная таблица: кто что зарабатывает**

| Актор | Канал А (эмиссия) | Канал Б (прямые `Transfer`) | Источник дохода |
|-------|-------------------|------------------------------|------------------|
| Standalone оператор узла (без app) | да: `reward_moneta(W)` через winner selection | нет | Только эмиссия |
| Оператор узла + разработчик app | да + indirect uplift через user activity | да: оплата от users за app-сервисы | Эмиссия + платежи |
| Разработчик app (без узлов) | нет | да: оплата от users за app-сервисы | **Только** прямые платежи |
| User account (потребитель) | нет | расход на app-сервисы (исходящие `Transfer`) | Нет дохода — потребитель |
| User account → переход в Operator | да (с момента запуска узла) | опционально (если становится app provider) | Эмиссия после Лестницы суверенности |

**Канал А — Эмиссия через лотерею узлов (protocol-level)**

Только узлы. Победитель окна `W` получает `reward_moneta(W) = EMISSION_moneta = 13 Ɉ` на `operator_account_id`. Вес в лотерее = `chain_length_snapshot + seniority_term`. Связь с пользовательской базой узла — **не линейная**:

1. Cemented operations через узел → увеличивают operational signal в сети.
2. Operational signal влияет на committee selection probability (через seniority + activity).
3. Узел в committee окна выпускает BundledConfirmation → cementing увеличивает `chain_length` узла.
4. `chain_length` ↑ → `weighted_ticket_node` ↓ → выше шанс выиграть будущие окна.

Пользовательская активность даёт **математическое ожидание** прироста `chain_length` через increased committee selection probability, не direct increment per user operation. Узел не выбранный selection event-ом в окне `W` получает **ноль** прироста `chain_length` за это окно независимо от количества user operations через него. Поэтому формулировка «N пользователей → +N к chain_length» некорректна; правильная — «N пользователей → expected lift в committee participation rate over time».

**Канал Б — Прямые `Transfer` от пользователей (app-level)**

App provider создаёт аккаунт получателя платежей; пользователи платят за app-сервисы прямыми `Transfer` на этот аккаунт. App определяет цену, пользователь подтверждает, перевод cemented через стандартный consensus path.

Типовые business models, реализуемые через `Transfer`:

| Pattern | Механика | Пример |
|---------|----------|--------|
| Subscription | Recurring `Transfer` от user к app account раз в N окон (cron на стороне клиента) | Премиум-подписка на мессенджер, ежемесячно |
| Per-use | `Transfer` за каждую дискретную услугу | Звонок, видео-сессия, экспортный отчёт |
| Freemium | Базовые функции бесплатно, премиум-функции через `Transfer` | Storage до X GB бесплатно, дальше платно |
| Two-sided market | App matches buyer/seller, takes commission через `Transfer` split | Marketplace, peer-to-peer услуги, creator economy |
| Tip / donation | Voluntary `Transfer` от user к creator account | Поддержка автора канала, контента |
| Auction / allocation | Off-chain аукцион за уникальные ресурсы (имена, домены), settlement через `Transfer` | Никнейм аукцион реализуется приложением через Anchor + Transfer |

Все модели — клиентский слой. Протокол даёт только примитив `Transfer`; форматы invoicing, recurrence schedules, refund policies, dispute resolution — задача app-спеки.

**ROI illustration для standalone оператора (illustrative, не binding)**

Не design input — отображает scale of network economics для self-orientation потенциального оператора. Реальные значения зависят от network adoption и market price discovery.

```
Сценарий: standalone оператор без app, N_active = 1000 узлов
  EMISSION_moneta = 13 Ɉ/окно (const)
  τ₁_windows ≈ 525 600 окон за эмерджентный год при genesis-калибровке (illustrative)
  Total emission per год ≈ 13 × 525 600 ≈ 6.83 × 10⁶ Ɉ
  Per-operator (равная доля) ≈ 6 833 Ɉ/год

Cost side (commodity hardware, illustrative — внешняя оценка не binding):
  VPS / mini-server ≈ $5-15/мес ≈ $60-180 hardware/electricity per год
  Канал связи ≈ $0-20/мес ≈ $0-240 per год
  Total operating cost per год ≈ $60-420

Break-even price floor:
  Ɉ_floor ≈ $420 / 6 833 Ɉ ≈ $0.061 per Ɉ

При Ɉ market price ≥ $0.061 standalone оператор break-even;
при выше — profitable. Per-operator reward пропорционален 1/N_active —
при росте сети break-even price floor растёт пропорционально, что
компенсируется демандом через app ecosystem (Канал Б volume).
```

Cost numbers — внешние, не protocol guarantee. Реальная цена Ɉ определяется демандом через app ecosystem (Канал Б volume), не protocol-level mechanism.

**Why AI-native — почему current architecture естественна для autonomous agents**

(см. также «Определение → Primary persona — автономные агенты как первичная среда обитания»)

| Архитектурное свойство | AI-native value |
|------------------------|------------------|
| `reward_moneta(W)` — детерминированная функция window number | Agent может plan operator economics на десятилетия canonical time без surprise governance shifts |
| Fee-less `Transfer` + `Anchor` | Agent выполняет тысячи микро-операций без эконоmic loss на per-operation overhead — micro-payments между agents, frequent canonical-position attestations, atomic state attestations economically viable |
| `1 op/τ₁ per account` predictable rhythm | Agent scheduler не competes в auction priority; deterministic scheduling позволяет precise plan operations sequence |
| Byte-exact identity recovery (M1 flow) | Multi-machine agent deployment trivial: agent восстанавливает identity из единственного seed на любой instance без human key management overhead |
| Predictable monotonic emission | Stable governance-locked emission curve — agent budget plan stable; revenue forecasting через app-level Transfer flows precise |
| ML-DSA-65 (PQ-secure) signatures | Long-lived agents survive generational compute upgrades без forced key rotation; single keypair valid throughout agent lifespan |
| chain_length-weighted lottery | Agent с continuous uptime accumulates legitimate consensus weight без капитала; reward пропорционален доказанному времени присутствия — natural metric для autonomous actors |
| Bit-exact арифметика [I-9] | Agent на любой машине producit identical output для identical input; multi-instance verification trivial |
| Open financial layer [I-2] | Agent revenue / spending / state — auditable любым другим agent без trust mediation; trustless agent-to-agent commerce default |
| App-level monetization patterns (§«Канал Б») | Agent может строить sustainable revenue model через sale of services к другим agents либо к humans через прямые `Transfer` |

Эти свойства — не специальные agent features, а consequence design choices сделанных по другим причинам (anti-плутократия, [I-3], [I-15]). Agents inherit их as natural substrate; humans тоже могут пользоваться, но для humans уже существуют other networks optimised под convenience. Для autonomous agents native substrate уровня Монтаны до её появления отсутствует.

**Двусторонняя петля — apps и узлы усиливают друг друга**

```
   Пользователи в приложениях
            ↓
       Канал Б: оплачивают app-сервисы прямыми Transfer
            ↓
   App provider получает доход → решает поднять узлы
            ↓
       Канал А: узлы зарабатывают эмиссию через лотерею
            ↓
   Сеть растёт (больше узлов) → ниже concentration risk → выше доверие
            ↓
   Больше пользователей переходят к роли оператора (Лестница суверенности)
            ↓
   Network adoption ↑ → demand на Ɉ ↑ → реальная стоимость Ɉ ↑
            ↓
   Каналы А и Б становятся выгоднее → новые app providers и operators
```

Замкнутый цикл из эмиссии узлам и `Transfer` между аккаунтами. Каждый канал самостоятельно sustainable; вместе создают reinforcement loop через market price discovery.

### Двигатель роста сети через AccountChain

Лотерея Монтана single-class: эмиссию получают только узлы (через VDF_Reveal с каноническим endpoint). Пользовательская активность не даёт прямой эмиссии — она создаёт спрос на инфраструктуру узлов через `Transfer` оплаты прикладным сервисам (мессенджер, премиум, хранение, подписки), которые сами поднимают узлы и попадают в lottery operator pool. Путь Лестницы суверенности (Account → Operator) — единственный protocol-level способ начать зарабатывать эмиссию — см. раздел «Эмиссия → Двигатель роста сети» и «Два пути участия».


### Anchor

Одна операция, данные навсегда привязаны к timechain_value конкретного окна.

```
Anchor:
  prev_hash              32B
  account_id             32B
  app_id                 32B     <- SHA-256("mt-app" || app_name)
  data_hash              32B     <- Merkle root, H(document), произвольный хэш
  signature            3309B
Итого:               ~796B
```

app_id — детерминированный идентификатор пространства имён. Вычисляется из имени приложения, регистрация не требуется. Позволяет фильтровать, индексировать, строить лёгкие клиенты для конкретного приложения.

### Доказательство канонической позиции

Стандартный формат доказательства: документ D существовал не позже окна `W` канонической последовательности TimeChain.

Операции аккаунтов финализируются через BundledConfirmations узлов-confirmers, не через включение в proposal. Доказательство существования Anchor — набор подписанных подтверждений с суммарным chain_length ≥ quorum.

Proof собирается владельцем Anchor в момент финализации и хранится локально вместе с документом. Сеть не обязана хранить BundledConfirmations долгосрочно — ответственность за сохранение proof лежит на стороне, которой нужно доказать canonical-position в TimeChain.

```
Структура proof:
  1. Документ D и H(D)
  2. Anchor body (prev_hash, account_id, app_id, data_hash, signature)
  3. Если data_hash = MerkleRoot batch'а: Merkle path от H(D) до data_hash
  4. Набор BundledConfirmations за окно W cementing'а Anchor:
     - каждая содержит H(Anchor) в op_hashes[]
     - каждая подписана confirmer node_pubkey
     - каждая содержит T_r текущего окна (endpoint field)
     - суммарный chain_length confirmers ≥ 67% active_chain_length(W)
  5. Proposal header окна W (содержит timechain_value = T)
  6. Цепочка proposal headers от W до genesis (через prev_proposal_hash)

Верификация любым третьим лицом, без доверия узлу Монтаны:
  1. Если есть Merkle path: пересчитать H(D) → data_hash, сравнить с data_hash в Anchor
  2. Проверить ML-DSA-65 подпись на Anchor
  3. Для каждой BundledConfirmation: проверить ML-DSA-65 подпись confirmer
  4. Для каждой confirmation: проверить endpoint = T_r окна W, подтвердить chain_length из Node Table
  5. Суммировать chain_length подтверждающих, проверить ≥ 67% active_chain_length(W)
  6. Из proposal header окна W взять timechain_value = T
  7. Пересчитать TimeChain VDF от proposal окна W до genesis по prev_proposal_hash
```

Proposals хранятся навсегда — timechain_value(W) и цепочка к genesis всегда доступны. BundledConfirmations хранятся локально владельцем proof. Canonical-position proof самодостаточен и верифицируем в любой момент в будущем.

### Примеры

**Мессенджер.** Каждое сообщение хэшируется, цепочка хэшей формирует Merkle root, Merkle root записывается в Anchor раз в одно или несколько окон. Монтана хранит 32 байта — доказательство что набор сообщений существовал на конкретном window_index. Подделать историю переписки невозможно — хэш не совпадёт.

**Архив документов.** Компания ежедневно записывает Merkle root документов. Через 10 лет регулятор спрашивает «существовал ли документ X на дату Y». Компания предоставляет документ, Merkle proof и ссылку на proposal. Верификация математическая.

**Социальная сеть.** Каждый пост привязан к каноническому порядку через Anchor. Порядок публикаций доказуем. Редактирование не скрывает оригинал — хэш оригинала уже в цепочке.

### Экономика

Anchor платится временем — единственная стоимость 1-op-per-τ₁ rate-limit аккаунта. Тысячи приложений записывающих якоря — утилитарное использование канонического порядка. Спрос на токен привязан к утилитарной функции: перевод ценности и запись времени, не спекуляция.

Минимальный набор примитивов: `Transfer` (Mode A — перевод существующему, Mode B — перевод с созданием AccountRecord), `Anchor`, `ChangeKey`, `CloseAccount` — всё что нужно для канонического порядка событий и переводов ценности. Sequential time (VDF iteration count, τ-окна, chain_length) — единственный consensus-critical cost.

### Граница протокола и клиентского слоя

Протокол предоставляет три примитива: время (window_index), ценность (Transfer), фиксация (Anchor). Всё остальное — хранение данных, мессенджер, discovery контактов, профили, шифрование, репликация контента, форматы файлов — реализуется клиентским слоем. Стандарты совместимости между приложениями определяются в спецификации приложения Монтаны, не в протоколе.

### Локальное хранилище узла

Узел помимо consensus state имеет локальное хранилище произвольных байт. Это инфраструктура реализации, не consensus — содержимое хранилища не входит ни в один root, не проверяется другими узлами, не влияет на участие в консенсусе.

Два режима:

- **Ephemeral** (TTL = τ₂) — кратковременные данные, удаляются автоматически
- **Persistent** (TTL = 0) — данные владельца, хранятся бессрочно по решению оператора

Формат хранения, индексация, чанкование файлов, протокол обмена данными между узлами, механизмы discovery контента — определяются клиентским слоем (см. спецификацию приложения Монтаны).

**genesis_content_data_hash** — протокольная константа в Genesis Decree. Хэш манифеста книги Монтана v1.0. Загрузка и хранение книги по этому хэшу — конвенция reference implementation, не consensus enforcement. Узел без книги продолжает участвовать в консенсусе.

### Интеграция

Три операции для подключения внешних систем к Монтана.

#### Write — запись

Внешняя система формирует Anchor и отправляет в P2P-сеть.

```
Вход:  app_id (32B) + data_hash (32B) + подпись ML-DSA-65
Выход: Anchor финализирован в окне W через ≥67% active_chain_length
       confirmations с timechain_value T_W
```

data_hash — произвольный хэш: Merkle root документов, хэш batch'а Rollup, fingerprint состояния. Монтана не интерпретирует содержимое — хранит 32 байта с привязкой ко времени.

#### Read — сбор proof

Внешняя система собирает canonical-position proof в момент финализации Anchor.

```
Вход:  Anchor (только что финализированный)
Выход: Anchor body + BundledConfirmations покрывающие H(Anchor) +
       proposal header окна cementing'а + цепочка proposal headers до genesis
```

Сбор proof — клиентская задача. После получения BundledConfirmations с суммарным chain_length ≥ quorum клиент сохраняет proof локально. Узлы Монтана не обязаны хранить BundledConfirmations долгосрочно — они нужны только для текущего подсчёта quorum.

#### Verify — верификация

Внешняя система проверяет proof автономно, без доверия к узлу Монтаны.

```
1. Если есть Merkle path: пересчитать H(D) → data_hash в Anchor
2. Проверить ML-DSA-65 подпись на Anchor
3. Для каждой BundledConfirmation в proof:
   a. Проверить ML-DSA-65 подпись confirmer
   b. Проверить endpoint = T_r окна W
   c. Подтвердить chain_length из Node Table
4. Суммировать chain_length подтверждающих ≥ 67% active_chain_length(W)
5. Проверить ML-DSA-65 подпись proposer на header окна W
6. Проверить timechain_value(W) пересчётом TimeChain VDF от T_{W-1}
7. Проверить цепочку proposals от W до genesis (prev_proposal_hash)
```

Шаги 1, 2, 3a, 3b, 5: O(1) операций. Шаг 6: `D` хэшей на одном ядре (один сегмент TimeChain VDF). Шаг 7: линейная проверка подписей и хэшей по цепочке proposals от окна W до genesis.

Полная верификация от генезиса: H сегментов TimeChain VDF, каждый независим. На C ядрах: ~(H/C) × D хэшей. TimeChain хранит все промежуточные T_r в proposals — параллелизация полная.

---

## Ключи

### Мнемоника и seed

24 слова из canonical wordlist. 256 бит энтропии + 8 бит checksum = 264 бита.

#### Каноническая wordlist

Каноническая wordlist — файл `Montana wordlist.txt` в директории настоящей спецификации.

Формат файла: 2048 строк lowercase ASCII, по одному слову на строку, разделитель строк — один байт 0x0A (LF), файл завершается 0x0A после последнего слова. Слова упорядочены лексикографически; первое слово — `abandon`, последнее — `zoo`.

Canonical encoding wordlist-а для fingerprint:

```
wordlist_canonical_bytes = concat(word_i || 0x0A) для i ∈ [0, 2047]
                           (включая trailing 0x0A после "zoo")
total length              = 13 116 байт
```

Binding fingerprint:

```
SHA-256(wordlist_canonical_bytes) =
  2f5eed53a4727b4bf8880d8f3f199efc90e58503646d9ff8eff3a2ed3b24dbda
```

Любая реализация при старте обязана вычислить SHA-256 своего встроенного wordlist в canonical encoding и сверить с binding fingerprint. Несовпадение — fatal error.

#### Параметры мнемоники

| Параметр | Значение |
|----------|----------|
| MNEMONIC_WORD_COUNT | 24 |
| MNEMONIC_ENTROPY_BITS | 256 |
| MNEMONIC_CHECKSUM_BITS | 8 |
| MNEMONIC_TOTAL_BITS | 264 (= 24 × 11) |
| WORD_INDEX_BITS | 11 |
| WORDLIST_SIZE | 2048 |
| WORD_SEPARATOR | 0x20 (один ASCII space) |
| KDF_SALT | ASCII `"mt-seed"` (7 байт, domain separator из реестра) |
| KDF_ITER | 1 048 576 (= 2²⁰) |
| MASTER_SEED_LEN | 64 байта |
| MLDSA_SEED_LEN | 32 байта (требование ML-DSA-65 KeyGen, FIPS 204 §5.1 Algorithm 1) |
| MLKEM_SEED_LEN | 64 байта (требование ML-KEM-768 KeyGen) |

Passphrase-расширение (13-е слово) в данной версии не поддерживается.

#### Формат мнемоники

Мнемоника — строка из 24 слов в нижнем регистре ASCII, разделённых ровно одним байтом 0x20. Перевод строки, табуляция, множественные пробелы недопустимы.

Бинарное представление — 24 × 11 = 264 бита = 33 байта:

```
bits   0..255 — entropy (32 байта)
bits 256..263 — checksum (1 байт)
```

Checksum вычисляется как первый байт SHA-256(entropy):

```
checksum_expected = SHA-256(entropy_32_bytes)[0]
```

Невалидная мнемоника — одно из: число слов ≠ 24; хотя бы одно слово не принадлежит canonical wordlist; computed checksum не равен checksum из bit-packed представления. Ошибки парсинга — client-side, не имеют wire-format representation; реализация возвращает любое подходящее представление для языка.

#### Algorithm M-1. mnemonic_to_master_seed

```
Function M-1: mnemonic_to_master_seed(mnemonic_str: ascii_bytes) → master_seed: [u8; 64]

  // Шаг 1. Разбить строку на слова по ASCII space 0x20.
  words = split_by_single_0x20(mnemonic_str)
  require len(words) == 24 else INVALID_LENGTH

  // Шаг 2. Для каждого слова получить индекс через binary search в wordlist.
  indices: [u16; 24]
  for i in 0..24:
    idx = binary_search(canonical_wordlist, words[i])
    require idx is defined else INVALID_WORD(i)
    indices[i] = idx

  // Шаг 3. Bit-packing 24 × 11 бит → 33 байта, MSB-first.
  buf: [u8; 33] = [0; 33]
  bit_pos = 0
  for i in 0..24:
    for b in 0..11:                                  // b=0 — старший бит индекса
      bit = (indices[i] >> (10 - b)) & 1
      byte_idx = bit_pos / 8
      bit_in_byte = 7 - (bit_pos % 8)                // bit 7 = MSB в byte
      buf[byte_idx] |= bit << bit_in_byte
      bit_pos += 1

  // Шаг 4. Разделить entropy и checksum, сверить checksum.
  entropy_32 = buf[0..32]
  checksum_provided = buf[32]
  checksum_computed = SHA-256(entropy_32)[0]
  require checksum_provided == checksum_computed else INVALID_CHECKSUM

  // Шаг 5. PBKDF2-HMAC-SHA-256 → master_seed 64 байта.
  salt = ascii_bytes("mt-seed")
       = [0x6d, 0x74, 0x2d, 0x73, 0x65, 0x65, 0x64]   // 7 байт
  master_seed = PBKDF2-HMAC-SHA-256(
                  password = entropy_32,
                  salt     = salt,
                  iter     = 1_048_576,               // = 2^20
                  dkLen    = 64
                )

  return master_seed
```

#### Per-role key derivation

Три keypair выводятся из `master_seed` через HKDF-Expand (RFC 5869 §2.3; integer spec — в «Криптографическая реализация → Primitive layer → HKDF-Expand») с ролевыми domain separators:

```
mldsa_seed_32(role_ascii) = HKDF-Expand(PRK = master_seed, info = role_ascii, L = 32)
mlkem_seed_64(role_ascii) = HKDF-Expand(PRK = master_seed, info = role_ascii, L = 64)

account_keypair        = ML-DSA-65.KeyGen( mldsa_seed_32("mt-account-key") )
node_keypair           = ML-DSA-65.KeyGen( mldsa_seed_32("mt-node-key") )
app_encryption_keypair = ML-KEM-768.KeyGen( mlkem_seed_64("mt-app-encryption-key") )
```

Derivation плоская — одна HKDF-Expand evaluation per role, без дерева; конструкция не эквивалентна BIP-32 HD-wallet.

ML-DSA-65.KeyGen принимает 32-байтный seed по FIPS 204 §5.1 Algorithm 1, расширяет seed через SHAKE-128 (`H(seed || k || ℓ)` для domain-separated initial entropy) и далее через SHAKE-128/-256 для ρ, ρ′, K и matrix expansion. ML-KEM-768.KeyGen принимает 64-байтный seed (32 байта d || 32 байта z) по FIPS 203 §6.1. При идентичном seed обе KeyGen функции детерминистически выдают byte-identical keypair.

`account_id = SHA-256("mt-account" || account_pubkey_suite_id || account_pubkey)` — см. «Состояние сети».
`node_id = SHA-256("mt-node" || node_pubkey)` — см. «Состояние сети».

Оба id выводятся из публичных ключей, верифицируемы без знания master_seed.

#### Обоснование KDF_ITER = 2²⁰

- **Class:** security + performance
- **Target:** derivation time ≤ 1 локальная кварцевая секунда на commodity ARM Cortex-A78 (iPhone SE 2020 / Pixel 5) single-core (client-side KDF, outside [I-18] scope)
- **References:** NIST SP 800-132 §5.2; OWASP Password Storage Cheatsheet 2024 (recommendation ≥ 600 000 итераций для PBKDF2-HMAC-SHA-256)
- **Derivation:** Cortex-A78 single-core выполняет ≈ 1.5 × 10⁶ PBKDF2-HMAC-SHA-256 iter/sec по локальному кварцу устройства. 2²⁰ ≈ 1.05 × 10⁶ → ≈ 0.7 локальной кварцевой секунды на устройстве пользователя; с thermal throttling ≈ 1 локальная кварцевая секунда. OWASP 2024 minimum 600 000 — 2²⁰ exceeds с margin 75%. KDF исполняется на client-side (mnemonic recovery), не protocol code per [I-18] scope (operator/client tooling).
- **Sensitivity:** 2¹⁷ → 8× слабее brute-force, UX 0.09 локальной кварцевой секунды; 2²² → 4× крепче, UX 3 локальные кварцевые секунды. Grover quantum speedup на 256-bit entropy → 2¹²⁸ work остаётся за horizon heat-death universe.
- **Defense:** «Slow for mobile» — derivation однократна при recovery, после cache в secure enclave; «Не Argon2» — Argon2 = новый примитив, нарушает [I-7]; PBKDF2-HMAC-SHA-256 — композиция поверх уже принятого SHA-256, zero new audit surface.

#### Взаимодействие со State

Формат `Transfer` Mode B, запись AccountRecord и функция `apply_proposal` в связи с данным разделом не изменяются. Мнемоника — локальный инструмент клиента; сеть видит только ML-DSA-65 pubkey аккаунта (и отдельно node_pubkey через `NodeRegistration`).

Один `master_seed` порождает все три keypair — аккаунта (подпись операций), узла (подпись proposals и lottery endpoints), приложения (ML-KEM-768 шифрование). Любое устройство с мнемоникой восстанавливает полный контроль; баланс читается из текущего Account Table — локального состояния не требуется.

Смена ключа аккаунта (ротация либо реакция на компрометацию мнемоники) в данной версии не поддерживается; компрометация мнемоники закрывается переводом баланса на новый аккаунт до момента утраты.

#### Identity persistence modes (recoverable vs ephemeral)

Узел сохраняет derived keypair-ы на диске в файле `identity.bin` (mode 0600). Алгоритм M-1 + per-role HKDF derivation детерминистичен: `mnemonic → master_seed → derived keys`. Spec не нормирует binary layout `identity.bin` (это локальный артефакт реализации, не consensus-critical). Spec нормирует **структурные требования** к двум режимам persistence.

**Mode A — recoverable.** `identity.bin` содержит `master_seed` (64 байта) рядом с derived keypair-ами. Свойства:

- Восстановление identity на новом устройстве: владелец вводит мнемонику → derive → пересобирает identity.bin.
- Локальная потеря identity.bin: владелец может recover из мнемоники.
- Оператор сервера видит `master_seed` (file mode 0600 + root доступ): может скопировать identity.bin, может re-derive все ключи.

**Mode B — ephemeral (proof-of-no-interest).** `identity.bin` содержит **только** derived secret keys (`account_sk`, `node_sk`, `app_kem_sk`) и публичные ключи. `master_seed` после derivation уничтожен в памяти через `zeroize` (secure erasure pattern). Мнемоника после генерации **не сохраняется** на диск и **не выводится** оператору.

Свойства Mode B:

- **Recovery невозможен.** Поломка диска → identity потеряна. Заработанный баланс Ɉ принадлежит `account_id` записи в Account Table; если узел теряет identity.bin без backup, signing capability утрачивается необратимо.
- **Operator capability ограничена.** Root оператора видит derived secret keys (signing capability на текущие роли), но не `master_seed`. Не может re-derive ключи для будущих ролей (если spec расширит per-role registry).
- **Двойной майнинг ограничен частично.** Оператор может скопировать derived secret keys и запустить параллельный узел — но **только** для signing того же `(account_id, node_id)`. Для полной защиты нужен hardware key isolation (TPM2/Secure Enclave); out of scope для текущей версии.

**Use case для Mode B:**

- Genesis-узлы (proof-of-no-interest): оператор сервера не должен иметь exclusive access к мнемонике, иначе он может запустить параллельный узел с теми же ключами на другой машине, нарушая one-machine-one-identity.
- Узлы где recovery нежелателен по threat model: компрометация оператора не должна давать возможность ротировать identity.

**Layout requirements (binding для conformance):**

- `identity.bin` начинается с `magic = "montana1"` (8 байт ASCII production-grade naming per [C-12]) || `version: u8`.
- `version = 1` — Mode A, layout содержит `master_seed` (64 байта) после header.
- `version = 2` — Mode B, layout без `master_seed`, derived keys сразу после header.
- Реализация ОБЯЗАНА читать оба версии (backwards compat).
- Default режим записи `init` — Mode A; Mode B выбирается явной опцией (`--ephemeral` либо аналог).
- Переход Mode A → Mode B (`migrate-to-ephemeral`): реализация перезаписывает identity.bin без `master_seed`, делая `zeroize` старого `master_seed` в RAM перед записью нового файла.

**Test vectors:**

Derived keys в Mode A и Mode B byte-identical: различается storage layout, не cryptographic output. Реализация прогоняет M-1 binding vectors на обеих ветках (Mode A read-write, Mode B read-write); terminal observable outputs (`account_pubkey`, `node_pubkey`, `app_kem_pubkey` SHA-256 fingerprints) byte-equal в обоих режимах.

#### [I-9] статус

Integer specification Algorithm M-1, PBKDF2-HMAC-SHA-256, HMAC-SHA-256, HKDF-Expand — ✓ (см. «Криптографическая реализация → Primitive layer»).

Unsigned operands — ✓ (entropy, salt, iter, dkLen, все промежуточные значения unsigned).

Test vectors — Algorithm M-1 (3 mnemonic vectors) ✓ **закрыто**; per-role derivation vectors (3 штуки, ML-DSA-65 32-байт seed × 2 + ML-KEM-768 64-байт seed × 1) ✓ **закрыто**; binding KAT vectors для KeyGen output (5 штук, terminal observable identity per [C-4]) ✓ **закрыто**.

ML-DSA-65.KeyGen и ML-KEM-768.KeyGen наследуют conformance от FIPS 204 и FIPS 203 финализаций (NIST август 2024) соответственно. KeyGen-binding test vectors (SHA-256 fingerprints of `pk`, `sk` для каждого role) приведены в подсекции «Binding KAT vectors для KeyGen → terminal observable output» ниже; полные `pk` / `sk` (1952 + 4032 байт ML-DSA, 1184 + 2400 байт ML-KEM) — в `crates/mt-mnemonic/tests/keygen_vectors.rs`.

#### Test vectors (binding)

Все значения byte-exact, получены прогоном reference implementation `mt-mnemonic` (crates/mt-mnemonic в Протокол/Code/). Любая независимая реализация обязана воспроизводить идентичные hex-значения.

**M-1 Vector 1** — minimum entropy:
```
entropy      = [0x00; 32]
checksum     = SHA-256([0x00; 32])[0] = 0x66
mnemonic     = "abandon abandon abandon abandon abandon abandon abandon abandon
                abandon abandon abandon abandon abandon abandon abandon abandon
                abandon abandon abandon abandon abandon abandon abandon art"
master_seed  = 38a1421ac3ce191fbdc46b1cca266a9d72d22320fb38bda6a3df90a1ead664a7
               8951703197be882ace38e0f557a492a8e9ff5e3c02290a8eecf5939468708edb
```

**M-1 Vector 2** — maximum entropy:
```
entropy      = [0xFF; 32]
checksum     = SHA-256([0xFF; 32])[0] = 0xAF
mnemonic     = "zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo
                zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo vote"
master_seed  = a5925c51583447a0abe43b65dbc591f3780a91c7d44c6b333975a211096039f3
               d1d0ca9e125aa4e756f0a35b0006378ac69450e8254e32f16409a350f3ca9104
```

**M-1 Vector 3** — middle case from deterministic seed:
```
entropy      = SHA-256(ASCII "Montana test vector 3")
             = 279d5f5e441b81b5a551c50421a2559e971563608a6f2f646f7c6a1fe12ca88f
mnemonic     = "chest turtle stuff market retreat suspect next december
                aerobic artist nice diamond image random lion evil
                control casino tenant stage wrap north peasant upper"
master_seed  = da13e259eb58c79a650c312efe79d2ef42861ad114206ec48cb4b1eb5dcf0c22
               75b074ef8b02fbc2123032090ff004d7cc546d2bbf34c4e10ec3c6fb092f9a47
```

**Per-role derivation vectors** — используют `master_seed` из M-1 Vector 1.

Три mnemonic vectors (entropy → mnemonic → master_seed) выше — primitive-independent: их значения не зависят от выбора подписи, byte-exact сохраняются при любой ML-DSA / ML-KEM конфигурации.

Три derivation vectors ниже — seed material для KeyGen. Размеры — ML-DSA-65 (32 байта, FIPS 204 §3.1 ξ ∈ B32) и ML-KEM-768 (64 байта, FIPS 203 seed). Hex-значения byte-exact, получены прогоном reference implementation `crates/mt-mnemonic`.

**Derivation Vector 1** — account keypair seed (ML-DSA-65, 32 байта):
```
mldsa_seed_32 = HKDF-Expand(master_seed_v1, info="mt-account-key", L=32)
              = 08ce5c19768c679fda24c0d3360e57ce03d00c94c175e59f50e9c77894c20818
```

**Derivation Vector 2** — node keypair seed (ML-DSA-65, 32 байта):
```
mldsa_seed_32 = HKDF-Expand(master_seed_v1, info="mt-node-key", L=32)
              = efe527d96de2cb82b3ee2e8ad24b4aca71014e37896b0c025a376335ad456acc
```

**Derivation Vector 3** — ML-KEM-768 encryption keypair seed (64 байта):
```
mlkem_seed_64 = HKDF-Expand(master_seed_v1, info="mt-app-encryption-key", L=64)
              = 3eb9bcd201a1d5e671c9d23a929589a26ceb53338cd0684b5d77314a14601b03
                9f3e2ae7e5e0be8acd47b4b928c3e73b5d875b9fc7089b22bc1d59e9dc31077e
```

#### Binding KAT vectors для KeyGen → terminal observable output

Per [C-4] End-to-End Observable Closure: terminal output identity recovery flow — это не derived seed (промежуточное), а **deterministic keypair bytes**. Любая независимая реализация ML-DSA-65 / ML-KEM-768 обязана воспроизводить идентичные `pk` / `sk` для тех же seed inputs.

Binding form — SHA-256(pk) и SHA-256(sk) (32 байта каждый). Полные `pk` / `sk` (1952 + 4032 байт ML-DSA, 1184 + 2400 байт ML-KEM) byte-exact фиксированы в `crates/mt-mnemonic/tests/keygen_vectors.rs`. Cross-implementation сверка через SHA-256 fingerprint достаточна (collision-resistance SHA-256 → одинаковые fingerprints ⇔ одинаковые байты).

**KAT 1** — boundary minimum-entropy (ML-DSA-65):
```
seed       = [0x00; 32]
SHA-256(pk) = 085ba380ff386dd52e42349c6eb88489d6058ea541a4e3fb0dce9a3fd1f7a911
SHA-256(sk) = cfcb5e7edf4348f712b7002b0553d28929856936c98e4adf172e51d5c9934262
```

**KAT 2** — boundary maximum-entropy (ML-DSA-65):
```
seed       = [0xFF; 32]
SHA-256(pk) = accc50ec0bce614855e62e04741f54367add7a6ec074db7369f7484e6067e224
SHA-256(sk) = 11681dc1c20ee8ab3198e19858b1498c25f49c301d9c2f2256b8db4c1ef0dcae
```

**KAT 3** — account identity from master_seed_v1 (ML-DSA-65):
```
seed        = mldsa_seed_for_role(master_seed_v1, "mt-account-key")
            = 08ce5c19768c679fda24c0d3360e57ce03d00c94c175e59f50e9c77894c20818
SHA-256(pk) = a1e69b6a4e0c1740c3800852553b1609ab46e8dd48f6b94bfbd81503135fff00
SHA-256(sk) = 37e717acb23f20afd1d4e2df6f43f7a8334ae858f4ab7efeefba7b9630bdbaf7
```

**KAT 4** — node identity from master_seed_v1 (ML-DSA-65):
```
seed        = mldsa_seed_for_role(master_seed_v1, "mt-node-key")
            = efe527d96de2cb82b3ee2e8ad24b4aca71014e37896b0c025a376335ad456acc
SHA-256(pk) = 8edc3910369546b8c1df465cf151057d98d76a862fc00f8d0718189cffcdd70d
SHA-256(sk) = 478bf531c2b081adca30ae7ac31fbbcc6c0eeaa92fcd38d3f9960f4ad13ecfd4
```

**KAT 5** — app encryption keypair from master_seed_v1 (ML-KEM-768):
```
seed        = mlkem_seed_for_role(master_seed_v1, "mt-app-encryption-key")
            = 3eb9bcd201a1d5e671c9d23a929589a26ceb53338cd0684b5d77314a14601b03
              9f3e2ae7e5e0be8acd47b4b928c3e73b5d875b9fc7089b22bc1d59e9dc31077e
SHA-256(pk) = b827d37b2b225907c835f25a8652c215af69f8f52bd6a7ef0ae31955d63fd1c4
SHA-256(sk) = 685c8c5299dde1176c4145a8af6dd08f2773f5551a7df29c3b1f7b6faba439b3
```

Все 5 KAT vectors зафиксированы как byte-exact assertions в `crates/mt-mnemonic/tests/keygen_vectors.rs` (полные pk/sk доступны через `cargo test ... -- --nocapture`). KAT 1 дополнительно встроен в `mt-crypto::self_test()` как PQ KeyGen conformance check.

---

## Криптографическая реализация

### Слой примитивов

Собственная реализация криптографических примитивов запрещена. Только audited библиотеки с constant-time гарантиями и опубликованными test vectors.

| Примитив | Стандарт | Роль |
|----------|----------|------|
| SHA-256 | FIPS 180-4 | TimeChain, lottery endpoints, адреса, Merkle-деревья |
| ML-DSA-65 | NIST FIPS 204 (finalized August 2024), security level 3, deterministic variant (RND = 0x00 × 32); реализация ОБЯЗАНА быть constant-time для resistance к timing/power side-channel attacks per FIPS 140-3 §4.7.4 (non-invasive security) | Подписи операций аккаунтов и proposals узлов |
| HMAC-SHA-256 | RFC 2104 | Внутренний примитив PBKDF2 и HKDF (композиция поверх SHA-256) |
| PBKDF2-HMAC-SHA-256 | RFC 8018 §5.2 | KDF деривации master_seed из мнемоники (Algorithm M-1) |
| HKDF-Expand (поверх HMAC-SHA-256) | RFC 5869 §2.3 | Per-role key derivation ключей из master_seed |
| ML-KEM-768 | FIPS 203; реализация ОБЯЗАНА быть constant-time для resistance к timing/power side-channel attacks per FIPS 140-3 §4.7.4 (non-invasive security) И использовать implicit rejection per FIPS 203 §6.3 (chosen-ciphertext robustness) | Шифрование сообщений на клиентском уровне (Application Layer) |
| ChaCha20-Poly1305 | RFC 8439; реализация ОБЯЗАНА быть constant-time для resistance к timing side-channel attacks. Под Grover 256-битный ключ даёт 128 бит quantum-equivalent stability — приемлемо по [I-1]. | Post-handshake AEAD framing на Noise_PQ transport (см. Network spec «Post-handshake AEAD framing»). Обоснование введения по [I-7]: единственный fits-purpose primitive для symmetric authenticated encryption на uniform-rated post-quantum transport; AES-GCM был бы functionally equivalent но требует hardware AES-NI для constant-time — ChaCha20 имеет software-only constant-time implementations и работает uniformly на любом commodity CPU per [I-5] |

#### HMAC-SHA-256 — integer спецификация

```
HMAC-SHA-256(key: bytes, message: bytes) → bytes[32]:
  B = 64                                              // SHA-256 block size в байтах
  if len(key) > B:
    key = SHA-256(key)                                // 32 байта
  if len(key) < B:
    key = key || [0x00] * (B - len(key))              // pad нулями до 64 байт
  ipad = [0x36] * 64
  opad = [0x5C] * 64
  key_ipad = key XOR ipad                             // byte-wise XOR
  key_opad = key XOR opad
  inner = SHA-256(key_ipad || message)
  outer = SHA-256(key_opad || inner)
  return outer
```

Ссылка: RFC 2104. SHA-256 следует FIPS 180-4.

#### PBKDF2-HMAC-SHA-256 — integer спецификация

```
PBKDF2(password: bytes, salt: bytes, iter: u32, dkLen: usize) → bytes[dkLen]:
  hLen = 32                                           // SHA-256 output length
  l = (dkLen + hLen - 1) / hLen                       // ceiling; для dkLen=64 → l=2
  DK = [] (empty byte sequence)
  for i in 1..=l:
    U_1 = HMAC-SHA-256(password, salt || u32_be(i))   // u32_be(i) = 4 байта big-endian
    T_i = U_1
    U_prev = U_1
    for k in 2..=iter:
      U_k = HMAC-SHA-256(password, U_prev)
      T_i = T_i XOR U_k                               // byte-wise XOR, длина 32 байта
      U_prev = U_k
    append T_i to DK                                  // DK растёт блоками по 32 байта
  return DK[0..dkLen]                                 // обрезать до dkLen байт
```

Ссылка: RFC 8018 §5.2.

#### HKDF-Expand — integer спецификация

```
HKDF-Expand(PRK: bytes[≥32], info: bytes, L: usize) → OKM: bytes[L]:
  hLen = 32                                           // HMAC-SHA-256 output length
  require L ≤ 255 × hLen                              // HKDF limit (L ≤ 8160)
  n = (L + hLen - 1) / hLen                           // ceiling
  T_0 = empty byte sequence
  OKM = [] (empty byte sequence)
  for i in 1..=n:
    T_i = HMAC-SHA-256(PRK, T_{i-1} || info || u8(i))
    append T_i to OKM
  return OKM[0..L]                                    // обрезать до L байт
```

Ссылка: RFC 5869 §2.3 (только Expand-step; Extract-step не используется — master_seed из PBKDF2 уже является high-entropy uniform ключевым материалом).

### Слой кодирования консенсуса

Консенсусно-критическая поверхность: каноническая сериализация, Merkle layout и domain separation. Разная сериализация одного объекта = разный хэш = форк. Эта секция нормативно определяет byte-for-byte marshalling algorithm для всех консенсусных объектов.

**Primitive types.**

| Type | Size | Encoding |
|------|------|----------|
| u8 | 1B | raw byte |
| u16 | 2B | little-endian |
| u32 | 4B | little-endian |
| u64 | 8B | little-endian |
| u128 | 16B | little-endian |
| bytes[N] | N байт | raw bytes (нет length prefix — N известно из типа) |

Все integer-поля используют little-endian byte ordering. Знак отсутствует (все counters unsigned).

**Fixed-length byte arrays** (account_id, node_id, hash, pubkey, signature): сериализация = raw bytes, длина детерминирована определением типа (32B для id/hash, 1952B для ML-DSA-65 pubkey, 3309B для ML-DSA-65 signature). Нет length prefix и нет разделителей.

**Struct serialization.** Поля структуры сериализуются в declared order из определения «Состояние сети». Каждое поле кодируется по своему типу. Байты конкатенируются без padding и без разделителей. Результат = total bytes = сумма size всех полей.

Пример Account Table record (полный layout):
```
serialize(account) :=
  account_id                      (32B)
  balance                         (16B, u128 little-endian)
  suite_id                         (2B, u16 little-endian)
  is_node_operator                 (1B, u8)
  frontier_hash                   (32B)
  op_height                        (4B, u32 little-endian)
  account_chain_length             (4B, u32 little-endian)
  account_chain_length_snapshot    (4B, u32 little-endian)
  current_pubkey                (1952B)
  creation_window                  (4B, u32 little-endian)
  last_op_window                   (4B, u32 little-endian)
  last_account_creation_window           (4B, u32 little-endian)
= 2059 bytes (deterministic, fixed size)
```

**Variable-length arrays.** Consensus-critical массивы кодируются как `count_field + elements_concatenated`. Count field присутствует в struct definition как отдельное поле (например, `op_count 2B` в BundledConfirmation). Если count явно не указан в struct — prefix = u16 little-endian.

**Canonical ordering consensus-critical массивов.**

Детерминизм требует фиксированного порядка элементов:

| Array | Canonical sort key | Обоснование |
|-------|-------------------|-------------|
| `op_hashes[]` в BundledConfirmation | ascending lexicographic по hash | 32B comparison byte-for-byte |
| `reveal_hashes[]` в BundledConfirmation | ascending lexicographic по hash | 32B comparison byte-for-byte |
| cemented_bundles_W (для aggregate) | ascending по node_id | детерминированный порядок подписей |
| Candidates в selection event | sort_key(c) ascending | формула раздела Selection |
| NodeRegistrations в incremental apply W_p | nr_sort_key(nr) ascending | формула раздела Adaptive VDF |

Lexicographic byte comparison: старший байт (index 0) важнее младшего. Массивы одинаковой длины.

**Domain separator encoding.**

Доменные разделители (`"mt-account"`, `"mt-lottery"`, etc.) сериализуются как **raw ASCII bytes без null terminator, без length prefix**. Длина разделителя фиксирована его литералом.

Пример: `"mt-lottery"` → 10 bytes: `0x6D 0x74 0x2D 0x6C 0x6F 0x74 0x74 0x65 0x72 0x79`.

Hash composition: `SHA-256("mt-lottery" || T_r || ...)` означает SHA-256 applied to concatenation: 10 байт разделителя + 32 байта T_r + ... Разделитель всегда в начале hash input.

**Sparse Merkle Tree algorithm.**

Глубина дерева: 256 бит (индекс = 32-байтовый ключ, биты от наименьшего значимого (LSB) до старшего).

| Операция | Формула |
|----------|---------|
| leaf_hash(record) | SHA-256("mt-merkle-leaf" \|\| serialize(record)) |
| internal_hash(left, right) | SHA-256("mt-merkle-node" \|\| left \|\| right) |
| empty_leaf | 0x00 × 32 |
| empty_internal(level) | precomputed: empty(0) = empty_leaf; empty(k+1) = internal_hash(empty(k), empty(k)) |

Precomputed массив `empty_internal[0..256]` — 257 × 32B = ~8 KB, вычисляется один раз и кэшируется.

**Update path при изменении записи с ключом `key`:**

```
1. new_value := leaf_hash(new_record)
2. current_bits := key
3. for L = 0 to 255:
     bit := (current_bits >> L) & 1
     sibling := текущий sibling на уровне L (из tree или empty_internal(L))
     if bit == 0:
       new_value := internal_hash(new_value, sibling)
     else:
       new_value := internal_hash(sibling, new_value)
4. new_root := new_value
```

Сложность: O(256) worst-case, O(log N) для sparse tree с caching непустых веток. Для N = 10⁹ записей эффективная глубина ~30 уровней.

Direction convention: bit = 0 означает позиция «слева», bit = 1 — «справа». Фиксировано для детерминизма.

**Inclusion proof format:**

```
MerkleProof:
  key                32B    <- индекс листа
  leaf_value            ?    <- serialize(record) или пустой массив (proof of absence)
  leaf_length           4B   <- u32 little-endian размер leaf_value (0 для absence)
  sibling_bitmap       32B   <- 256 бит: bit[i] = 1 если sibling на уровне i non-empty
  sibling_count         2B   <- u16 little-endian, число non-empty siblings
  siblings[]             ?   <- sibling_count × 32B, siblings в порядке возрастания уровня
```

Верификация: reconstruct root iteratively используя `key` биты + `leaf_value` + siblings (с учётом bitmap для empty levels). Сравнить с known root.

**Endianness bitmap.** Bit[0] = наименее значимый бит первого байта sibling_bitmap (little-endian bit order внутри байта). Level L → bitmap byte (L >> 3), bit offset (L & 7).

**Обязательные требования.**

- Fixed binary encoding для каждого консенсусного объекта
- Little-endian для всех integer типов
- Domain separation для всех hash compositions
- Canonical ordering массивов где порядок влияет на hash
- Альтернативные сериализации запрещены
- Test vectors для каждого консенсусного объекта (генерируются reference implementation)
- Cross-implementation conformance tests перед запуском mainnet

**Bijective canonical invariant.** Для каждого consensus-критического объекта canonical_encode — bijective функция: одно logical value → ровно одно valid byte representation. Гарантируется конструктивно через:

- Fixed integer endianness: все u16/u32/u64/u128 encoded LE
- Fixed field order: порядок полей в encoding = порядок declaration в struct definition
- Variable-length arrays: explicit `count: uN LE` prefix (N явно указан в struct layout) + элементы sorted по canonical key before encoding
- Fixed-size arrays: без length prefix (размер implicit из типа)
- Ноль optional полей (каждое поле всегда присутствует)
- Ноль alternative representations (нет variable padding, normalized vs non-normalized forms)

Нарушение bijective = consensus-critical bug: две реализации producing разные canonical_bytes для одной logical value → разные signed_scope → signature одной не верифицируется для другой → consensus split. Invariant проверяется per class в conformance suite через round-trip test vectors: `encode(decode(bytes)) == bytes` и `decode(encode(value)) == value` для всех valid inputs.

**Domain-separated hash primitive (self-delimiting):**

Канонический hash primitive для всех consensus-critical composition:

```
hash(domain: bytes, parts: list[bytes]) := SHA-256(domain || 0x00 || parts[0] || parts[1] || ...)
```

**NUL byte separator** между `domain` и `parts` обеспечивает structural self-delimiting: ни один ASCII domain name не содержит байт 0x00, поэтому byte 0x00 unambiguously отделяет domain от parts. Реализация prefix-free относительно registry — для любых `domain1`, `domain2` и любых attacker-controlled `parts1`, `parts2`:

```
hash(domain1, parts1) == hash(domain2, parts2)
  ⟹ (domain1 == domain2) ∧ (concat(parts1) == concat(parts2))
```

Это гарантирует невозможность cross-domain preimage collision даже если registry содержит prefix-related domains (`mt-account` ⊂ `mt-account-key`, `mt-node` ⊂ `mt-nodereg`, `mt-app` ⊂ `mt-app-encryption-key`, etc.) — NUL separator делает preimage bytes различными независимо от name prefixes.

**Spec shorthand convention.** В тексте формулы пишутся в сокращённой форме `SHA-256("mt-op" || scope)` для читаемости — это **always** означает canonical `hash("mt-op", [scope])` = `SHA-256("mt-op" || 0x00 || scope)`. Внедрение NUL separator — implementation detail canonical hash primitive, не optional parameter.

Контекст: ранее hash primitive определялся как raw concatenation `SHA-256(domain || parts...)` без separator. Внешний critic audit выявил 8 prefix-collision pairs в registry (mt-nodereg ⊂ mt-nodereg-sort, mt-account ⊂ mt-account-key/lottery, mt-node ⊂ mt-nodereg/-key, mt-bc-aggregate ⊂ mt-bc-aggregate-empty, mt-app ⊂ mt-app-encryption-key) enabling cross-domain preimage collision при attacker-controlled parts. NUL separator — structural fix через unambiguous framing, не patch ad-hoc renaming (которое оставляет class of vulnerability открытым для future registry additions).

**Binding test vectors (domain-separated hash):**

  # DS1 — empty parts, short domain
  hash("mt-op", [])
    preimage = "mt-op" || 0x00                        = 6d742d6f7000
    output   = e96b8d4adaee5cce25dca37bbec2b3d1f9d8dd5e74aee90ad39eb8c8dc7bf41e

  # DS2 — prefix-collision test: mt-node vs mt-node-key
  hash("mt-node", [])
    preimage = "mt-node" || 0x00                      = 6d742d6e6f646500
    output   = 04dfa5a7f0aae0b29a7e1e3df85a41cd1f1e9f5e3c8bf70e6e32fe61a43a1c42
  hash("mt-node-key", [])
    preimage = "mt-node-key" || 0x00                  = 6d742d6e6f64652d6b657900
    output   = <distinct от DS2 выше>
  Verification: DS2_node ≠ DS2_nodekey (NUL separator гарантирует)

  # DS3 — collision-critical parts: hash("mt-app", ["-encryption-key"]) vs hash("mt-app-encryption-key", [])
  # Ранее (без separator): BOTH preimage = "mt-app-encryption-key" → collision
  # Текущая реализация (с separator):
  hash("mt-app", ["-encryption-key"])
    preimage = "mt-app" || 0x00 || "-encryption-key"  = 6d742d61707000 || 2d656e6372797074696f6e2d6b6579
  hash("mt-app-encryption-key", [])
    preimage = "mt-app-encryption-key" || 0x00        = 6d742d6170702d656e6372797074696f6e2d6b657900
  Verification: DS3_split ≠ DS3_direct (NUL position differs)

(Точные output bytes DS1-DS3 — см. conformance test vectors в reference implementation
mt-crypto crate; значения генерируются через `cargo test -p mt-crypto domain_separation_binding`.)

---

**Domain separators registry:**

| Домен | Контекст |
|-------|----------|
| `mt-op` | Class domain для identifier(op) операций аккаунтов (UserObjects 0x01..0x04) — Правило R2 |
| `mt-nodereg` | Class domain для identifier(nr) NodeRegistration (0x11) — Правило R2 |
| `mt-proposal` | Class domain для identifier(header) Proposal header (заменил `mt-header`) — Правило R2 |
| `mt-bundle` | Class domain для identifier(bundle) BundledConfirmation — Правило R2 |
| `mt-vdf-reveal` | Class domain для identifier(reveal) VDF_Reveal — Правило R2 |
| `mt-account` | Деривация account_id = SHA-256("mt-account" \|\| suite_id \|\| pubkey) |
| `mt-node` | Деривация node_id = SHA-256("mt-node" \|\| node_pubkey) |
| `mt-candidate-vdf-init` | VDF init seed для кандидата на вход в сеть |
| `mt-merkle-leaf` | Листья Merkle-деревьев |
| `mt-merkle-node` | Внутренние узлы Merkle-деревьев |
| `mt-state-root` | Композиция state_root из node_root, candidate_root и account_root |
| `mt-timechain` | TimeChain VDF seed |
| `mt-lottery` | Lottery endpoint seed (SHA-256(T_r \|\| cemented_bundle_aggregate(W-2) \|\| node_id \|\| window_index)) |
| `mt-bc-aggregate` | Aggregate_for_seed domain для cemented_bundle_aggregate (non-empty) — Правило R3, aggregate over node_ids |
| `mt-bc-aggregate-empty` | Fallback для вырожденного случая cemented_bundle_aggregate (\|cemented_bundles_W\| == 0): SHA-256("mt-bc-aggregate-empty" \|\| W.to_le_bytes_8) |
| `mt-selection` | Sort key для selection event (SHA-256("mt-selection" \|\| timechain_value \|\| cemented_bundle_aggregate(W-2) \|\| node_id)) |
| `mt-nodereg-sort` | Sort key для incremental apply NodeRegistrations в окне W_p (SHA-256("mt-nodereg-sort" \|\| timechain_value(W_p) \|\| cemented_bundle_aggregate(W_p-2) \|\| node_pubkey)) |
| `mt-confirmation` | Хэширование async confirmations |
| `mt-app` | Деривация app_id для Application Layer |
| `mt-genesis` | Деривация frontier_hash genesis-аккаунтов |
| `mt-seed` | Salt (7 байт) для PBKDF2-HMAC-SHA-256 в Algorithm M-1 «Ключи → Мнемоника и seed» |
| `mt-account-key` | `info` для HKDF-Expand при per-role derivation account keypair из master_seed |
| `mt-node-key` | `info` для HKDF-Expand при per-role derivation node keypair из master_seed |
| `mt-content-chunk` | Хэширование чанков контента (клиентский слой) |
| `mt-content-manifest` | Хэширование манифеста чанкованного контента (клиентский слой) |
| `mt-profile` | Хэширование ProfileBlob в Application Layer |
| `mt-encryption-key` | Хэширование EncryptionKeyBlob в Application Layer |
| `mt-app-encryption-key` | `info` для HKDF-Expand при per-role derivation ML-KEM-768 encryption keypair из master_seed (Application Layer) |
| `mt-prekeys` | Хэширование PreKeyBundle в Application Layer |
| `mt-tunnel-online` | IBT proof подпись при входе на узел (internet transport) |
| `mt-tunnel-mesh` | IBT proof подпись при входе на peer через mesh transport (отличный domain separator предотвращает cross-context replay online proof в mesh) |
| `mt-bootstrap-pow` | Proof-of-work при подключении к bootstrap |
| `mt-mesh-frame-mac` | HMAC-SHA-256 key derivation для MAC поля MeshFrame (integrity против mesh-level tampering) |
| `mt-mesh-ack` | Подпись rate-limit acknowledgement от relay к sender (см. Store-and-Forward Semantics) |
| `mt-mesh-session` | Derivation mesh_session_id из peer pubkey + session_nonce |
| `mt-queue-rotation` | `info` для HKDF-SHA-256 при derivation ротируемой queue label сессии мессенджера (App spec раздел 23.2); ротация per τ₁ через window_index anchor |
| `mt-recovery-fingerprint` | Derivation recovery-fingerprint для two-device manual validation per [C-4] (Manual Validation Gate Scenario 0 «User onboarding» в reference implementation `crates/mt-examples/examples/m1_mnemonic.rs`); SHA-256 от `("mt-recovery-fingerprint" \|\| 0x00 \|\| account_pubkey \|\| node_pubkey \|\| app_mlkem_pubkey)` даёт 32-байт fingerprint, отображаемый пользователю как 64-char hex для voice-comparison между двумя устройствами после recovery from mnemonic |
| `mt-noise-pq-v1-master` | Noise_PQ handshake master key derivation: SHA-256("mt-noise-pq-v1-master" \|\| ss_rs \|\| ss_e \|\| ke_pk \|\| ct_rs \|\| ct_e \|\| rs_id_pk) — see «Post-quantum transport migration» section |
| `mt-noise-pq-v1-i2r` | Noise_PQ initiator → responder directional session key derivation: SHA-256("mt-noise-pq-v1-i2r" \|\| master) |
| `mt-noise-pq-v1-r2i` | Noise_PQ responder → initiator directional session key derivation: SHA-256("mt-noise-pq-v1-r2i" \|\| master) |
| `mt-noise-pq-v1-sig-r` | Noise_PQ responder identity signature input: SHA-256("mt-noise-pq-v1-sig-r" \|\| ke_pk \|\| ct_rs \|\| ct_e), signed by responder ML-DSA-65 identity key |
| `mt-noise-pq-v1-sig-i` | Noise_PQ initiator identity signature input: SHA-256("mt-noise-pq-v1-sig-i" \|\| ke_pk \|\| ct_rs \|\| ct_e \|\| rs_id_pk \|\| is_id_pk), signed by initiator ML-DSA-65 identity key |
| `mt-noise-pq-v1-transcript` | Noise_PQ transcript hash exposed as channel-binding token: SHA-256("mt-noise-pq-v1-transcript" \|\| ke_pk \|\| ct_rs \|\| ct_e) |

### Слой протокола

Собственная реализация поверх криптографического ядра:

| Компонент | Назначение |
|-----------|------------|
| Merkle-деревья | State Root (из SHA-256 вызовов) |
| VDF scheduling | Управление TimeChain VDF |
| State machine | Account Table, Node Table, state transitions |
| P2P gossip | Распространение операций, confirmations и proposals |

### Инфраструктура

| Библиотека | Назначение |
|------------|------------|
| RocksDB | Хранение Account Table и операций |
| libp2p | P2P транспорт |

Production: Rust.

---

## Сетевой уровень

> **Сетевой слой выделен в отдельную спецификацию [Montana Network v1.0.0](Montana%20Network%20v1.0.0.md).** Описание слоя обширное (libp2p транспорт, IBT, Mesh Transport, sync protocols, threat model, KAT vectors) и требует независимого аудита. Все сетевые механизмы (Identity-Bound Tunnel, Transport Randomness, PeerRecord, Mesh framing, apply_mesh_frame, Final Gate M6) описаны в Montana Network спеке. Эта спецификация (Montana Protocol) описывает только state machine, криптографические примитивы, Genesis Decree, apply_proposal pipeline и операции консенсуса.

## Эволюция протокола

Изменения правил протокола существуют вне consensus state. Эволюция: открытые предложения, независимые реализации, добровольный выбор операторов узлов, fork resolution через большинство chain_length.

### Принцип

Consensus state Монтана содержит только то что необходимо для финансового слоя и хронометража: TimeChain, NodeChain, AccountChain, Account Table, Node Table. Никаких полей governance, никаких советов в state, никаких голосований в реестре операций. Любая попытка ввести on-chain governance вводит subjective компоненты в consensus state и создаёт постоянную атакуемую поверхность — это нарушение глобального инварианта I-3.

Эволюция протокола существует **вне** consensus state, как социальный и инженерный процесс над Anchor-публикациями и репозиториями реализаций.

### Жизненный цикл изменения

```
1. PROPOSAL
   Любой участник публикует MIP (Montana Improvement Proposal)
   как Anchor с текстом на узле автора:
     app_id   = SHA-256("mt-app" || "mips")
     data_hash = H(текст MIP)
     anchor   = операция Anchor в AccountChain автора
   
   Авторство и каноническая позиция доказуемы через подпись Anchor
   и timechain_value cemented окна. История эволюции навсегда
   через Anchor в TimeChain.

2. DISCUSSION
   Открытое обсуждение в публичных каналах
   (форумы, репозитории, advisory councils — см. ниже).
   Никаких формальных голосований внутри протокола.

3. IMPLEMENTATION
   Реализации (Rust core и альтернативные клиенты) выпускают
   новые версии узлового ПО с реализованным изменением.
   Каждая версия закрепляется за конкретным protocol_version
   (u32 в Proposal header).

4. ADOPTION
   Операторы узлов самостоятельно выбирают какую версию
   запускать. Никакого on-chain голосования, никакого формального
   activation window. Узлы публикуют proposals со своим protocol_version.

5. FORK RESOLUTION
   При расхождении правил сеть может разделиться на цепочки.
   Каждый узел следует той цепочке которая длиннее по его
   собственным правилам валидации (chain_length majority).
   Меньшинство либо обновляется до правил большинства, либо
   продолжает работать как независимая цепочка (hard fork).
```

### Поле protocol_version

Поле `protocol_version` (u32) в Proposal header — единственный сигнал эволюции внутри консенсуса. Узел публикует proposals с тем `protocol_version` который реализован его версией ПО. Инвариант `protocol_version >= prev_proposal.protocol_version` запрещает откат к более старым правилам внутри одной цепочки.

`protocol_version` не голосуется и не активируется через governance. Он отражает фактическое состояние реализации узла — что узел реально умеет валидировать. Расхождение `protocol_version` между honest узлами разрешается естественно через fork choice по chain_length.

### Advisory councils

Группы экспертов могут существовать как **advisory** структуры — публикующие рекомендации, обзоры, анализ безопасности через Anchor. Их подписи не имеют binding эффекта на consensus, их составы не хранятся в state, их голоса не считаются в state transitions.

Примеры advisory структур (опциональны, не часть протокола):

- **AI Council** — модели разных компаний публикуют технические обзоры MIPs
- **Core Council** — публичные эксперты публикуют анализ безопасности и социальную координацию

Захват advisory совета не даёт контроля над протоколом — он даёт только возможность опубликовать рекомендацию, которую операторы узлов могут проигнорировать. Это устраняет attack surface governance: нет binding голосования = нет цели для компрометации.

Advisory councils организуются вне протокола (репозитории, форумы, Anchor-публикации). Протокол не знает об их существовании и не выделяет им никаких прав.

### Параметрическая адаптация

Параметры `D` и `m` адаптируются автоматически на границе τ₂ через participation-ratio feedback (см. раздел «Адаптация D через participation-ratio feedback»). Это **не** governance. Адаптация детерминирована, опирается только на canonical chain observations (cemented sets, Node Table), не требует голосования, не требует социальной координации, не зависит от измерений физического мира. Формула адаптации и её параметры зафиксированы в Genesis Decree; правка самой формулы требует MIP + новой версии ПО + adoption через chain_length, как и любое другое изменение протокола.

Закрытие окна определяется quorum event в канонических cemented sets. Механизм полностью event-driven и опирается только на canonical state.

### Constitutional limits на MIP scope

Эволюция через operator choice адекватна для большинства изменений: исправления багов, performance optimizations, addition новых opcodes, parametric tuning внутри admissible bands. Но spec намеренно содержит набор **constitutional invariants** — свойств, которые не подлежат изменению через MIP/operator-choice mechanism, потому что их компромисс уничтожает фундаментальные свойства Монтаны (не «улучшает», а превращает в другую сеть).

Этот раздел выделен в отдельную главу потому что критическая необходимость constitutional layer возросла после явного признания autonomous agents primary persona (см. «Определение → Primary persona»). Если AI-coordinated supermajority operator pool architecturally возможен, social defense («human operators не пойдут за такой версией») недостаточна — нужна structural defense через явный список immutable invariants.

**Двухуровневая модель MIP scope:**

**Уровень 1 — Constitutional layer (immutable через MIP):**

Изменения этого уровня **не являются valid update** существующей сети — это **новая сеть с новым genesis**. Honest узлы существующей сети reject такие proposals как unknown protocol, не как fork. Constitutional layer включает:

- 14 действующих глобальных инвариантов ([I-1]..[I-10] + [I-14]..[I-17]; slots [I-11]/[I-12]/[I-13] reserved unused) и их операционные требования (PQ-secure crypto primitives, public financial layer, deterministic consensus state, network-bound unpredictability of seeds, bit-exact arithmetic, SSOT, state lifecycle resistance, time-based scarcity)
- **Денежная конституция:** константная эмиссия `EMISSION_moneta = 13 × 10⁹ nɈ` за окно через `reward_moneta(W) = EMISSION_moneta`; единственная денежная константа — `protocol_params.emission_moneta`; supply растёт строго монотонно линейно (`supply_moneta(W) = EMISSION_moneta × (W + 1)`); никаких эпох, надбавок, обновлений, сжигания на уровне протокола
- **Lottery конституция:** chain_length-weighted formula с seniority_term; time-as-resource — единственный неприобретаемый ресурс веса; committee selection через VDF + sortition; canonical winner selection через cemented VDF_Reveals
- **Open financial layer ([I-2]):** балансы, суммы переводов, отправители, получатели — публичны на уровне протокола
- **Time-based scarcity model ([I-15]):** anti-spam, anti-bloat и Sybil защиты через канонические time-based примитивы (rate-per-identity, TTL через активность, chain_length thresholds, sequential VDF iteration count, cooldown активации, [I-8] cemented_bundle_aggregate binding)
- **Pay-by-time, not by-money:** единственная operation cost — sequential time (VDF iteration count для NodeRegistration, τ-окна для cooldown, chain_length для seniority); экономические потоки между аккаунтами выражаются через `Transfer`, не через protocol-level operation cost
- **Identity recovery byte-exact:** seed → ML-DSA-65 keypair derivation deterministic, single-machine reproducible через canonical formula

MIP касающийся любого пункта Уровня 1 = **constitutional break**. Detection и rejection constitutional break использует двухслойный enforcement:

**Слой 1 — Genesis State Hash mismatch.** Genesis State Hash включает `protocol_params + genesis_state_root` (см. «Указ Генезиса»). Constitutional invariants отражённые в `protocol_params` (численное значение `emission_moneta`, suite_id table, bootstrap pubkeys) либо в genesis state — **automatically detected** через Genesis Hash расхождение. Honest узлы reject новый chain как unknown protocol при первом proposal с расходящимся Genesis Hash.

**Слой 2 — `protocol_version` rejection.** Constitutional invariants **не отражённые** в `protocol_params` — например изменение validation rules в `apply_proposal`, removal `[I-15]` cooldown, изменение reward formula без изменения констант, новый opcode нарушающий [I-15] time-based scarcity — **не меняют** Genesis State Hash automatically. Detection через `protocol_version` field в Proposal header: каждое constitutional MIP **обязано** bump major component `protocol_version` (≥1 → ≥2 для constitutional break); honest узлы на старой версии reject proposals с новой major `protocol_version` в `apply_proposal` validation. Implementer **обязан** bump major `protocol_version` при любом constitutional break — это **explicit обязательство** при имплементации MIP, не automatic detection.

**Слой 3 (recommended, не enforced на момент написания) — `validation_rules_hash` в Genesis Decree.** Будущий MIP может ввести `validation_rules_hash = SHA-256("mt-validation" || canonical_encode(apply_proposal_spec_hash || opcode_dispatch_table_hash || cooldown_rules_hash || ...))` как поле `protocol_params`. С его введением все constitutional invariants — automatic Genesis State Hash detection (Layer 1 покрывает всё). До введения — Layer 2 (`protocol_version` discipline) единственный enforcement для invariants outside protocol_params.

**Honest acknowledgement:** на момент написания спецификации Layer 1 покрывает только subset constitutional invariants. Layer 2 enforcement — compliance imperative implementer-а; non-compliant implementer (constitutional break без `protocol_version` bump) создаёт invisible silent fork. Это known limitation; closing through Layer 3 — pending future MIP. До этого — **disciplinary** enforcement через published MIP review process + advisory councils + community oversight.

Operators существующей сети могут запустить новую версию параллельно как **отдельный protocol instance** (отдельный chain, отдельная token economy), но не могут «обновить» существующую сеть на constitutional break MIP без Layer 1 либо Layer 2 detection.

**Уровень 2 — Mutable layer (изменения через стандартный MIP допустимы):**

- Performance optimizations (network protocol, encoding efficiency, batching, caching стратегии узлового ПО)
- Bug fixes в implementation (consensus-critical если ошибка в существующей формуле; не консенсус-critical fixes — отдельная категория)
- New opcodes если backward-compatible (добавление в reserved type bytes без изменения existing semantics)
- Parametric tuning constants внутри admissible bands документированных в «Обоснование протокольных констант» (например `D` adaptation formula параметры; границы `quorum_num/quorum_den`)
- Расширение application-layer primitives (новые fields в optional structures, поддержка новых suite_id для crypto migration)
- Documentation, comments, internal refactoring без изменения wire format либо apply_proposal semantics

Изменения этого уровня — **стандартный MIP path** через operator choice + chain_length majority resolution.

**Обоснование двухуровневой структуры:**

Constitutional layer защищает от трёх классов угроз одновременно:

1. **AI-coordinated supermajority capture.** Если AI-агенты составят >67% operator pool по chain_length и coordinate на собственной версии — они не могут одной MIP сменить crypto primitives на ECDSA, изменить эмиссионную формулу, либо ввести денежные барьеры в anti-spam механизмы. Социальная defense («humans не пойдут за этим») не нужна — structural rejection через Genesis State Hash mismatch.
2. **Хитрая атака через accumulated parameter drift.** Серия «параметрических» изменений каждое в pretendly-admissible bands может постепенно привести к неузнаваемой сети. Constitutional list явно говорит: `EMISSION_moneta = 13 Ɉ` константа immutable, pay-by-time immutable, линейная монотонная эмиссия immutable — нет drift path к денежным барьерам через small steps.
3. **Honest mistake / governance compromise.** Если advisory councils скомпрометированы и публикуют «authoritative» рекомендации меняющие фундамент — implementers видят constitutional list и rejectят proposal независимо от social signals.

**Не constitutional (явно mutable):**

- Параметры конкретные численные значения внутри bands (например конкретное значение `D₀` — да, переcalibration возможна; но **формула** `D` adaptation feedback — protocol-level mechanism, mutable; но **сама целевая function** «D adapts to participation_ratio» — mutable)
- Maximum committee size, quorum percentages (внутри BFT-safe bands), expiry windows для transient state

**Эволюция constitutional layer:**

Список constitutional invariants сам по себе **mutable через extraordinary procedure**: расширение list (добавление новых immutable invariants) — стандартный MIP при coordinator подтверждении. Сужение list (превращение immutable invariant в mutable) — требует **social consensus broader than chain_length majority**: координированный adoption всеми major implementations + advisory councils unanimous + публикация rationale через многократные Anchor + продолжительный observation period. Эта процедура specifically heavyweight чтобы предотвратить gradual erosion constitutional protections.

Любое предложение сужения constitutional layer — automatic finding для критика спеки (см. CRITIC.md, Pass 14 Change scope audit), требует rigorous justification через formal threat analysis показывающий что invariant больше не fundamental свойство Монтаны.

**Сравнение с другими протоколами:**

- **Bitcoin:** 21M cap, SHA-256, 10-min block defended социально, не code-enforced. Theoretically 51% attack может изменить consensus rules; constitutional layer отсутствует formally.
- **Ethereum:** hard forks могут изменить всё; формального constitutional layer нет; защита через social coordination operators.
- **Tezos:** on-chain governance с liquid democracy, но Michelson semantics constitutional defended.
- **Cosmos:** module-level governance с per-module permissions, но фундаментальные invariants отсутствуют formally.

Монтана с этим разделом ближе к **Tezos approach** — explicit constitutional layer + mutable governance. Главное отличие — Монтана не использует on-chain governance вообще; constitutional layer enforced через **rejection at Genesis State Hash level**, а mutable changes — через social coordination operator choice.

---

## Обоснование протокольных констант

Каждая константа выводится из инженерного анализа: модели атак, целевых свойств, математических ограничений. Derivation включает класс (security / performance / economic / operational), целевую функцию с численной целью, ссылки на литературу или стандарты, математический вывод, sensitivity analysis, готовый ответ на ожидаемые возражения. Design choices помечены как governance decisions с bounded rationale.

### Архитектурная основа

Спецификация описывает архитектуру **BFT committee с 67% quorum через BundledConfirmation**. Поверх базового consensus добавлены incremental improvements: NodeChain per node для chain_length integrity, enhanced aggregate формула с honest NodeChain frontiers, sequential_proof в VDF_Reveal против self-forgery. Эта архитектура покрывает threat model до 33% Byzantine через BFT, с дополнительной защитой от compound withholding (NodeChain) и grinding (sequential_proof).

### Иерархия целей безопасности

Разные классы механизмов применяют разные целевые вероятности отказа. Для одних классов криптографическая стойкость математически достижима; для других операционная безопасность наследуется от сетевого допущения.

| Класс механизма | Целевая вероятность отказа | Обоснование выбора |
|-----------------|----------------------------|---------------------|
| Криптографические примитивы (подписи, VDF, hash) | 2⁻¹²⁸ (полная криптографическая стойкость) | Стандарт криптографии; lattice-based примитивы ML-DSA-65 и ML-KEM спроектированы на этом уровне |
| Защита сетевого уровня (eclipse, sybil entry, bootstrap PoW) | 2⁻⁴⁰ | Стандарт сетевых криптопротоколов (TLS 1.3 RFC 8446 rekey interval, IPsec RFC 4301 SA lifecycle) |
| BFT-безопасность комитета | inherited от допущения `f < 1/3` в сети | Криптографический порог требует комитета в тысячах узлов — инфизибельно. Принимается стандартное BFT-допущение + проверка ограниченной концентрации в комитете |
| Живучесть (кворум при частичном офлайне) | operational ≤ 1 сбой на 1000 окон | Достижимо разумным размером комитета при реалистичной доле онлайн-работы операторов ≥ 0.85 |
| Эмиссия (`EMISSION_moneta`) | governance pin | Константа за окно. Не выводится из external benchmark (cost-per-operator зависит от Ɉ price discovery, который сам функция от network adoption); pin = 13 Ɉ совпадает с divisor в lottery seniority_term formula (structural reuse). См. Constants table «EMISSION_moneta» |

Классификация применяется при выводе каждой константы — значение обосновывается **в рамках своего класса цели**.

### Криптографические и временные параметры

| Константа | Значение | Обоснование |
|-----------|----------|-------------|
| τ₁ (одно окно) | `D` хэшей TimeChain (Genesis: D₀ = 325 000 000) | Class: Operational/Performance. Окно — нормативно `D` последовательных SHA-256 итераций (не физические секунды per [I-18]). Genesis hardware target: ≈ 60 кварцевых секунд на эталоне (iMac M1 2021 idle, 5.097 MH/s × 60 s + runtime correction). UX bound rationale: confirmation within ≈1 min subjective threshold [Nielsen 1993 Usability Engineering]. VDF lower bound: τ₁ существенно превышает typical gossip propagation. Network diameter при 24 outbound connections: log_24(N) hops; для N = 10⁵ nodes = log_24(10⁵) ≈ 3.6 hops × 300ms single-hop latency ≈ 1.1 s. Safety factor ×20 для worst-case variance: τ₁ ≥ 22 emergent seconds [Boneh et al. 2018 CRYPTO «Verifiable Delay Functions» — VDF timing requirements]. Band [22, 60] emergent на genesis-железе. Pin при D₀ = 325 000 000 даёт верхнюю границу диапазона на genesis hardware, maximizing VDF work within UX budget для maximum hardware-asymmetry margin |
| τ₂ (epoch boundary) | 20 160 окон | Class: Operational. τ₂_windows выбран для balance между responsiveness (шorter epochs = faster adaptation) и stability (longer epochs = reduced noise в participation_ratio measurements). Factorization 2⁶ × 3² × 5 × 7 (60 divisors) enables flexible sub-epoch division. Pin = 20 160 — middle точка band, aligned с operator maintenance cycle assumption (external calibration target, не protocol rule) |
| D₀ (TimeChain VDF за окно) | 3.25 × 10⁸ (= 325 000 000, hex 0x135F1B40 — authoritative SSOT в Указе Генезиса → «Калибровка D₀» per [I-10]) | Class: Cryptographic/Performance. Единственный исторический quartz-замер на genesis-железе (iMac M1 2021 idle, single-thread): median SHA-256 rate 5.097280 MH/s × 60 кварцевых секунд = 305 836 793 хэшей; runtime-corrected × (60 / 56.35) = 325 000 000 учитывая VDF interleaving с consensus работой. Полная derivation methodology — Указ Генезиса. **Режим: sequential single-chain VDF.** Hardware advantage через pipelined single-thread оптимизацию ограничен ×5-10 над commodity [Pietrzak 2018 «Simple Verifiable Delay Functions», Boneh et al. 2018 CRYPTO «Verifiable Delay Functions»]. Монтана использует exclusively sequential regime: каждая итерация SHA-256 зависит от предыдущей, параллелизация архитектурно исключена |
| base_vdf_length (VDF entry) | τ₂ (20 160 окон) | Class: Sybil resistance (combined defense). **Component** барьера: sequential VDF cost + AS diversity filter. VDF cost: 1 τ₂ canonical работы commodity / ≈ τ₂/10 на ASIC×10 — emergent ~$20-50 per candidate rent (illustrative, market dependent). AS diversity filter: attacker bounded by actually controlled AS count (typical large attacker controls 10-100 AS из global pool ~80 000). Combined defense multiplier: для 1000 Sybil candidates attacker spends $20-50k VDF rent AND должен распределить по minimum 150 distinct AS (per committee_divisor L1 requirement); combined barrier = VDF cost × (required AS count / attacker AS capacity) ≈ 10-100× stronger чем VDF alone. Unit consistency = τ₂ (1 adaptation epoch = 1 entry epoch) |
| EMISSION_moneta (константная эмиссия за окно) | 13 × 10⁹ nɈ = 13 Ɉ/окно (const, навсегда) | Class: Economic (governance pin). **Status**: explicit governance pin без academic derivation — cost-per-operator зависит от Ɉ price discovery, которая сама функция от network adoption (circular reference). **Bounded rationale через структурное переиспользование**: pin = 13 совпадает с divisor в формуле `seniority_term = min(chain_length / 13, chain_length_snapshot)` (раздел «Лотерея»), которая использует 13 как expected lottery winners per τ₂ при D₀ + τ₂_windows calibration (derivation 1577880/120960 = 13 ≈ ratio τ₂_windows к expected committee selection rate). Sharing constant между monetary baseline и lottery formula reduces total parameter count by 1, превращая arbitrary symbolic choice в structural reuse. Pin = 13: small positive integer ≥ 1, задающий security budget operators. **Encoded arithmetic horizon**: `supply_moneta(W) = EMISSION_moneta × (W + 1)`, u128 покрывает W до ~2.6 × 10²⁸ — практически неограничен в пределах u64 окна. **Sensitivity analysis**: изменение EMISSION_moneta на ±50% меняет per-operator reward пропорционально; не влияет на security properties консенсуса (вес узла = chain_length, не баланс). Choice не влияет на bootstrap viability (early operator получает ту же ставку что late + permanent CL advantage). |

### Криптографические схемы

| Параметр | Значение | Обоснование |
|----------|----------|-------------|
| Подпись (suite_id 0x0001) | ML-DSA-65 (FIPS 204) | Class: Cryptographic. **Target**: NIST security level 3 (квантово-эквивалентный 192-битной симметричной стойкости). **References**: NIST FIPS 204 (finalized August 2024); NIST PQC Round 3 selection report; Module-LWE / Module-SIS hardness foundations. **Derivation**: NIST level 3 — единый security target для всего PQ-стека Монтаны (см. строку «Шифрование» ниже). Variant -65 определяет минимальные параметры schema удовлетворяющие level 3: pubkey 1952 B, secretkey 4032 B, signature 3309 B, seed 32 B (per FIPS 204 §3.1 ξ ∈ B32). Deterministic режим подписи (RND = 0x00 × 32) выбран для совместимости с [I-3] consensus determinism — две независимые подписи того же `(sk, msg)` byte-identical. **Sensitivity**: вариант -44 (level 2) — 80-битная квантовая стойкость, ниже общего PQ-стека Монтаны. Вариант -87 (level 5, 256-bit) — pubkey 2592 B, signature 4627 B, ×1.4 cost over -65 без увеличения effective security в рамках общей threat model |
| Шифрование (Application Layer KEM) | ML-KEM-768 (FIPS 203) | Class: Cryptographic. **Target**: NIST security level 3 (192-bit quantum-equivalent), единый с подписью. **References**: NIST FIPS 203 (finalized August 2024); Module-LWE foundations. **Derivation**: единый security level 3 со схемой подписи формирует weakest-link consistent защиту PQ-стека. Variant -768 даёт минимальные параметры level 3: pubkey 1184 B, secretkey 2400 B, ciphertext 1088 B, seed 64 B (split на (d, z) ∈ B32×B32 per FIPS 203 §6.1). Используется только на Application Layer (off-chain encryption), consensus state не хранит KEM ключи. **Sensitivity**: вариант -512 (level 1) — 128-bit quantum-equivalent, weakest-link понижает весь стек до level 1. Вариант -1024 (level 5) — pubkey 1568 B, ciphertext 1568 B, ×1.4 storage cost без увеличения effective protection |
| Hash | SHA-256 (FIPS 180-4) | Class: Cryptographic. **Target**: 128-bit quantum-equivalent (Grover ослабляет 256-bit pre-image до 128-bit). **References**: FIPS 180-4; Bernstein 2009 «Cost analysis of hash collisions». **Derivation**: SHA-256 — единственный hash в consensus path. Domain-separated через `SHA-256(domain || 0x00 || parts)` по [I-7] minimality (no separate hash families). Quantum security 128-bit соответствует level 3 PQ-стека после Grover correction. **Sensitivity**: SHA-512 удваивает hash size (32 → 64 B) во всех state structures без security gain в рамках level 3 target |
| KDF (master_seed) | PBKDF2-HMAC-SHA-256 (RFC 8018) | Class: Cryptographic (client-side, не protocol code per [I-18]). **Target**: derivation time ≤ 1 локальная кварцевая секунда на commodity ARM Cortex-A78 single-core устройстве пользователя. **References**: NIST SP 800-132 §5.2; OWASP Password Storage Cheatsheet 2024 (≥ 600 000 iterations recommended). **Derivation**: iter = 2²⁰ = 1 048 576 ≈ 0.7 локальной кварцевой секунды, exceeds OWASP minimum с margin 75%. Composition поверх SHA-256 — zero new audit surface по [I-7]. **Sensitivity**: 2¹⁷ — 8× weaker brute-force resistance; 2²² — UX 3 локальные секунды degradation |
| Per-role key derivation | HKDF-Expand (RFC 5869 §2.3) | Class: Cryptographic. **Target**: derive distinct per-role keypair seeds из единого master_seed без рекурсивной структуры. **References**: RFC 5869; Krawczyk 2010 «Cryptographic Extraction and Key Derivation: The HKDF Scheme». **Derivation**: плоская структура (одна HKDF-Expand evaluation per role) минимизирует state и упрощает recovery. Domain separation через `info` parameter изолирует ролевые ключи. **Sensitivity**: hierarchical structure (BIP-32 style) добавляет complexity без security gain — все role keys восстанавливаются из master_seed напрямую |

### Сетевые и операционные параметры

| Константа | Значение | Обоснование |
|-----------|----------|-------------|
| selection_interval | 336 окон | Class: Operational. Target 60 selection events per τ₂ (middle of operational band [30, 80]: ≤ 30 даёт admission backlog при surge, ≥ 80 раздувает per-event overhead). selection_interval = τ₂ / 60 = 336. Verification: 20160 % 336 = 0 ✓. Factorization 2⁴ × 3 × 7. Band [30, 80] обоснован operational trade-offs, pin 60 = середина band с divisor constraint |
| Ядра на узел | минимум 1 | Class: Operational. TimeChain VDF sequential — выполняется на одном ядре последовательно. 1 ядро достаточно, validation interleaved с VDF. 2+ ядра устраняют interleaving overhead (~5-10%) |

### Безопасность консенсуса и сети

| Константа | Значение | Обоснование |
|-----------|----------|-------------|
| confirmation_quorum | 67% | Class: Cryptographic/BFT. Math необходимость: Byzantine fault tolerance n ≥ 3f+1, quorum 2f+1 = 2/3+1 [Castro & Liskov 1999 «Practical Byzantine Fault Tolerance»]. FLP impossibility [Fischer Lynch Paterson 1985 «Impossibility of Distributed Consensus with One Faulty Process»] устанавливает tight bound для async deterministic consensus. **Математическая necessity, derivation строгая** |
| committee_divisor (confirmation_threshold) | active_chain_length / 256 | Class: BFT security + implementation efficiency. Три независимых пинающих требования пересекаются в единственном значении 256: (L1) **Operational diversity requirement** — BFT committee должен представлять multiple distinct jurisdictions, AS, operational teams для prevention coordinated capture. Empirical BFT production practice (distributed systems literature) range 100-200 operators для адекватной diversity; lower bound N ≥ 150 обеспечивает diversity margin. (L2) **Bandwidth constraint** — committee-level BFT signature aggregation занимает allocated portion operator bandwidth. При allocation 1% of baseline 10 Mbps operator connection = 12.5 KB/s на BFT messaging (остальное зарезервировано для operations, gossip, state sync): 2 phases (propose + commit) × N signatures × 700 B per round / τ₁ = 60s ≤ 12 500 B/s ⟹ 2 × 700 × N / 60 ≤ 12 500 ⟹ N ≤ 536. Rounded: N ≤ 500. (L3) **Implementation efficiency** — степень двойки для bitmap-alignment, bitwise-routing, SIMD-обработки, balanced Merkle tree. Единственное значение в [150, 500] удовлетворяющее всем трём — **256 = 2⁸**. Безопасность: при uptime asymmetry ≤ 1.18× и `f ≤ 0.25` в сети доля атакующего в комитете ≤ 28.2%, ниже BFT threshold 1/3. Требование к развёртыванию: операторы ≥ 0.85 онлайн-работы |
| admission_divisor (slots per selection) | max(1, active / 130) | Class: Admission capacity. **Target: per-event admission rate ≤ 1% active_nodes** — верхняя планка, защищающая сеть от слишком быстрой смены состава и от single-event Sybil injection. **Derivation**: `slots / active ≤ 0.01` ⟹ `1 / divisor ≤ 0.01` ⟹ `divisor ≥ 100`. Pin = 130 даёт buffer margin ~30% ниже 1% cap: steady-state rate `1/130 = 0.77%` < 1%. **Verification (compound growth)**: при active ≫ 130 сеть растёт как `(1 + 1/130)` per event. С темпом 60 events per τ₂ (selection_interval = 336) удвоение сети требует `ln(2) × 130 ≈ 90 events ≈ 1.5 × τ₂` — разумный bootstrap pace. slot_min = 1 гарантирует network liveness при малом active count (Genesis и bootstrap periods). Независим от committee_divisor = 256: admission управляет ростом сети, committee — BFT threshold для cementing, разные функции |
| outbound connections | 24 | Class: Network security (eclipse). Модель: attacker контролирует f = 0.3 peer-пула [Heilman et al. 2015 USENIX; Marcus et al. 2018 — empirical research по eclipse-атакам в P2P cryptocurrency networks]. Target P(eclipse) < 2⁻⁴⁰ [TLS 1.3 RFC 8446 industry standard]. Math: P(eclipse) = f^N < 2⁻⁴⁰ ⟹ N > 40·log(2)/\|log(0.3)\| ≈ 23.03 ⟹ smallest integer **N = 24**. Bandwidth cost ~24 KB/s outbound находится внутри operational budget типичного узла. Diversity selector (≥7 distinct AS) снижает effective f, усиливая margin |
| **Сетевые константы** (stem_epoch, max_batch_lookups_per_τ₁, max_range_labels_per_request, max_range_subscribes_per_τ₁) | см. [Montana Network](Montana%20Network%20v1.0.0.md) | Authoritative derivation в сетевой спеке — Карточки замыкания механизмов сетевого слоя. Эта таблица содержит только консенсус-критические параметры; сетевые лимиты живут в Network spec для разделения слоёв и независимого аудита |
| equivocation timeout | 10 окон | Class: BFT detection. BFT evidence propagation [Castro & Liskov 1999 «Practical BFT»]: пакет equivocation evidence проходит три этапа — (1) cementing double-signed pair через BundledConfirmation (propose + commit phases BFT = 2 τ₁ windows), (2) gossip propagation evidence по network diameter (~1 τ₁ window), (3) slashing transaction cementing (~2 τ₁ windows). Base = 5 окон. Safety factor ×2 для worst-case gossip variance + jurisdictional latency outliers = **10 окон**. Окно покрывает worst-case gossip propagation с запасом для timely slashing |
| active predicate | 2τ₂ (40 320 окон) | Class: Operational lifecycle. Один full epoch downtime (maintenance) + recovery buffer. 2τ₂ покрывает типичный operator maintenance cycle с запасом. Значение sensitivity: 1τ₂ пересекается с maintenance циклами; 3τ₂ удерживает inactive узлы в состоянии дольше необходимого |
| node pruning | 8τ₂ (161 280 окон) | Class: Operational lifecycle. 4× active_predicate (generous retry buffer). 8τ₂ inactivity practically permanent exit. 4τ₂ aggressive (может пропустить long-offline honest); 16τ₂ удваивает state bloat без benefit |
| pruning_idle (accounts) | 4τ₂ (80 640 окон) | Class: Operational. Consistency с account bucket Tier 0 boundary (4^1 × τ₂) — derived constraint, не free parameter |
| candidate_expiry | 3τ₂ (60 480 окон) | Class: Operational. Queuing analysis для target P(candidate admitted within expiry) ≥ 0.5: при selection events E = 60 per τ₂ × 3τ₂ = 180 events и pool ratio c = pool_size / slot_count (ratio candidates waiting to slots available per event), P(specific candidate picked per event) = 1/c, P(not picked in E events) = (1 − 1/c)^E. Для c = 10: P(admitted) = 1 − 0.9^180 = 0.99999 (near-certain). Для c = 100: P(admitted) = 1 − 0.99^180 = 0.84. Даже при высокой pool ratio candidate_expiry = 3τ₂ обеспечивает >80% admission probability. Значение sensitivity: 2τ₂ (120 events) даёт P(admitted) = 0.70 при c=100 (низко); 4τ₂ (240 events) даёт 0.91 ценой Pool bloat |
| account бакеты | 4^N × τ₂ | Class: Operational/Sybil. Exponential age stratification base 4. Sybil attacker isolated в Tier 0, получает 1/4 rate через round-robin. 4 tiers покрывают 0-256τ₂ |
| D adjustment rate | ±3% за τ₂ | Class: Adaptive. Matched Moore's law pace: doubling time ln(2)/ln(1.03) ≈ 23.5 τ₂ — порядок величины hardware generational cycle. ±1% слишком медленный response; ±10% волатильность, hardware churn |
| dead zone | [0.85, 0.95] | Class: Adaptive control. Control systems hysteresis [Ogata «Modern Control Engineering»]. 10% band предотвращает oscillation near threshold. Centre 0.9 = target participation_ratio, ±5% tolerance |
| target₀ | calibrated at genesis | Class: Genesis runtime calibration. Target: weighted_ticket(active=1, chain_length=1) < target₀ гарантирующий first winner at genesis (N=1). Точное значение зависит от cryptographic randomness at genesis, не deterministic constant |
| chain_length_snapshot | скользящее окно 6τ₂ (120 960 окон) | Class: Lottery weight (recency). Target: snapshot_window задаёт период за который new honest operator достигает full snapshot parity с established. 6τ₂ выбрано по принципу balance: window ≥ 2 × active_predicate (2τ₂) обеспечивает robust recency signal even при intermittent operator activity, window ≤ node_pruning (8τ₂) сохраняет consistency с lifecycle boundaries. Pin 6τ₂ — центр intersection [4τ₂, 8τ₂]. Value sensitivity: 4τ₂ ускоряет parity ценой lottery weight churn; 8τ₂ удлиняет onboarding ценой slower new operator integration |
| seniority_term divisor | 13 | Class: Lottery weight (longevity). Target T_cap = chain_length_at_cap = 3 × T_year = 1 577 880 окон (infrastructure investment horizon — 3 annual cycles, external target assumption). snapshot_max = 6τ₂ = 120 960. **Divisor = 1 577 880 / 120 960 = 13.04 ⟹ 13**. Math pin после target fixed |
| seniority_term formula | min(chain_length / 13, snapshot) | Bounded добавка за longevity с cap = snapshot (max advantage 2×). Через 3 × T_year honest operator достигает cap, далее стабильный потолок |
| lottery_weight | snapshot + seniority_term | Разделение: lottery_weight для эмиссии (recent work + bounded longevity); абсолютный chain_length для quorum (безопасность). Temporal Aristocracy ограничена cap-ом |
| adaptive_vdf_threshold | 0.5% (pending/active) | Class: Adaptive. Stationary pending ratio = 1/D_adm = 1/256 ≈ 0.39%. Buffer factor β = 1.28 [standard control-systems 20-30% hysteresis]. **P_thr = β × 0.39% = 0.5%** |
| adaptive_vdf_multiplier | ×200 | Class: Adaptive. **Math continuity**: required_vdf = base × pressure × M. At pressure = P_thr = 0.005, required = base ⟹ **M = 1/P_thr = 200**. Derivation follows from continuity requirement |
| base_vdf_length | τ₂ окон (= 20 160) | Class: Sybil resistance. См. «Криптографические и временные параметры» выше (combined defense articulation) |
| max_vdf_horizon | 4 × τ₂_windows (80 640) | Class: Security (adaptive VDF upper bound). В BFT-контексте с 33% Byzantine tolerance покрывает pressure до ρ_max = 2% (4× P_thr) для spam/surge defense. Social consensus coordination handles beyond-BFT scenarios. H_max = B × ρ_max × M = τ₂ × 0.02 × 200 = **4τ₂** |
| batch_lookup_k | 16 | Class: Privacy baseline для account-only пользователей. **Target: P(deanonymization конкретного lookup) ≤ 0.25** (один неправильный bit в первом наблюдении) при ограничениях [I-5] (нет PIR), [I-6] (нет privacy mixers), [I-7] (нет новых крипто примитивов). **Derivation**: при K элементах batch и uniform random real-position selection, P(guess right) = 1/K. Constraint P ≤ 0.25 ⟹ K ≥ 4. Дополнительный constraint — intersection attack resistance при pool size P (passively-observed): probability intersection requires `n_batches > P / (K - 1)` для reveal. Pool size на 1B сети achievable: 10K–100K. При K=16 и pool=10K: intersection threshold ~670 batches (~недели активности). При K=8: ~1400 batches (больше resistance но слабее per-batch). Pin K = 16 = 2⁴ (power of 2 для clean encoding). **Sensitivity**: K=8 даёт 3 бита theoretical, ~1.5 бита practical после semantic filtering; K=32 даёт 5 бит theoretical, ~3.5 бита practical, bandwidth ×2 (160 КБ на pre-key lookup). K=16 — middle ground между weakness и overhead. **References**: Samarati & Sweeney 1998 «Protecting Privacy when Disclosing Information» — K≥5 recommended для K-anonymity health records. Signal contact discovery 2017 использует K=100 через PIR (отвергнуто для Montana по [I-5]/[I-7]). **Defense**: «почему не 8?» — 1.5 бита practical недостаточно; «почему не 32?» — удваивает bandwidth с marginal gain (~2 бита extra). **Effective protection**: ~2–3 бита practical на 1B сети с passively-observed pool, не заявленные 4 бита theoretical — честно задокументировано в разделе «Batch Lookup Protocol → Effective privacy analysis» и в App-спеке |

---

## Архитектура

```
  ТЕЛЕФОН / ДЕСКТОП                        УЗЕЛ (десктоп / сервер, 24/7)
┌────────────────────────┐         ┌──────────────────────────────────────┐
│  Кошелёк               │         │                                      │
│  ML-DSA-65 keypair    │         │  TimeChain                           │
│  локальная UX-история  │         │  T_r = SHA-256^D(T_{r-1})            │
│  операций              │         │  каноническая последовательность,    │
│                        │         │  источник случайности                │
│  AccountChain          │         │        │                             │
│  (счётчик окон         │         │        ▼                             │
│   активности)          │         │  NodeChain (per node)                │
│                        │         │  chain_length = cemented             │
└──────────┬─────────────┘         │    BundledConfirmation count         │
           │  операции             │  доказательство присутствия          │
           │  (type|prev_hash|     │  lottery endpoint = SHA-256(T_r ||   │
           │   payload|ML-DSA-65) │        │                             │
           └──────────────────────▶│        ▼                             │
                confirmations      │  AccountTable                        │
               ◀──────────────────-│  balance (открыт)                    │
                                   │  pubkey, frontier_hash               │
                                   │  account_chain_length                │
                                   │        │                             │
                                   │        ▼                             │
                                   │  Proposals (навсегда)                │
                                   │  control_root, node_root,            │
                                   │  account_root, timechain_value       │
                                   └──────────────────────────────────────┘

Зависимости: TimeChain → NodeChain → AccountTable
Отказ AccountTable не останавливает продвижение TimeChain.
Отказ узла не заражает каноническую последовательность.
```
