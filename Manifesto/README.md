# Montana Manifesto

**Version:** 1.2.0
**Date:** 2026-05-28
**Author:** Alejandro Montana
**Repository:** [github.com/efir369999/Montana](https://github.com/efir369999/Montana)

A single declaration of what Montana is and refuses to be, published in three languages from one canonical version. The three texts say the same thing.

- [English](Manifesto%20EN.md) — for the Metzdowd cryptography list and independent reviewers
- [Русский](Manifesto%20RU.md) — голос автора
- [中文](Manifesto%20ZH.md) — 中文版本

The English version is canonical for cryptographic claims; the Russian version is canonical for the author's voice. For the academic specification of the protocol, see [Whitepaper Montana.md](../Whitepaper%20Montana.md) and [Montana Protocol v35.25.1](../Montana%20Protocol%20v35.25.1.md).

**v1.2.0 (2026-05-28):**

- *Positioning refined* in response to the Metzdowd cryptography list thread. Bitcoin's title — *A Peer-to-Peer Electronic Cash System* — conflated two distinct jobs: being a stable unit of account (which needs an accountable issuer with a reserve and a buy-back capability), and being a neutral settlement and ordering rail (which cannot be that issuer and should not pretend to be). Montana picks the second on purpose. The stable usable currency that consumers see in daily life lives one floor up, issued by parties with balance sheets. Montana provides the substrate any such currency can run on.
- *§I rewritten* as "Two Jobs Bitcoin Conflated". §V renamed from "Cash-System Tokenomics" to "Properties of the Rail". §IX rewritten as "What Montana Is" with the rail-not-currency closing. `Ɉ` is named as the rail's protocol-level reward unit, not a promise of stable purchasing power.
- *Dormant non-zero accounts are never touched* — added to §III. Only empty `AccountRecord` entries (`balance == 0`, idle ≥ `4 × τ₂`) are garbage-collected. A balance, once credited, belongs to its key forever.

**v1.1.0 (2026-05-28):**

- *Terminology* aligned with Montana Protocol v35.25.1. The primitive is named «sequential delay computation» / «iterated SHA-256 hash chain», not VDF. Montana's chain is deliberately not a verifiable delay function in the Boneh-Pietrzak-Wesolowski sense (see §II for the rationale). Consensus is named Proof of Time. The smallest unit is `moneta`; `1 Ɉ = 10⁹ moneta`; the international ticker is `MONT`.
- *Finality claim corrected* against spec v35.25.1. Asynchronous finality is at window cementing — within a single window of the canonical order (approximately one minute of wall-clock at the genesis-hardware calibration), not the obsolete «~300 ms» claim carried from an early draft.
- *Reference-implementation count corrected* to 23 crates (from 12).

---

**Symbol:** **Ɉ** — Montana.

Alejandro Montana
