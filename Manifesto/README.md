# Montana Manifesto

**Version:** 1.3.0
**Date:** 2026-05-29
**Author:** Alejandro Montana
**Repository:** [github.com/efir369999/Montana](https://github.com/efir369999/Montana)

A single declaration of what Montana is and refuses to be, published in three languages from one canonical version. The three texts say the same thing.

- [English](Manifesto%20EN.md) — for the Metzdowd cryptography list and independent reviewers
- [Русский](Manifesto%20RU.md) — голос автора
- [中文](Manifesto%20ZH.md) — 中文版本

The English version is canonical for cryptographic claims; the Russian version is canonical for the author's voice. For the academic specification of the protocol, see [Whitepaper Montana.md](../Whitepaper%20Montana.md) and [Montana Protocol v35.25.1](../Montana%20Protocol%20v35.25.1.md).

**v1.3.0 (2026-05-29):**

- *Ethereum recognized and critiqued* alongside Bitcoin in §I. *Ether* in Ethereum's title is the medium that fills space — the substrate on which everything moves; the proposition was correct, but the implementation charges fees denominated in its own asset, runs on classical elliptic-curve signatures Shor breaks, and exposes an extractive ordering layer (MEV) that turns neutrality into a marketing claim rather than a property. The name promised the ether; the implementation did not deliver it. Bitcoin missed *cash*; Ethereum missed *ether*; both for the same underlying reason — rail reliability coupled to the price of the native asset, neutrality contingent on whoever produced the next block, primitive set Shor breaks.
- *No extractable ordering* is added to §V as a property of the rail. Proposer discretion over which operations to include is zero; only cemented BundledConfirmations enter the chain; operation ordering inside a window is fixed by canonical τ₁-rate rules, not by the proposer's local mempool view. There is no MEV-equivalent position over the order of operations.
- *§IX closes with* «the ether the name promised» / «эфир, как его обещало имя» / «那个名字所承诺的「以太」». The author's domain is `efir.org`. The Ɉ is the rail's reward unit. The name precedes the implementation.

**v1.2.0 (2026-05-28):**

- *Positioning refined* in response to the Metzdowd cryptography list thread. Bitcoin's title conflated two distinct jobs: being a stable unit of account (which needs an accountable issuer with reserve and buy-back) and being a neutral settlement and ordering rail (which cannot be that issuer). Montana picks the second on purpose. The stable usable currency lives one floor up, issued by parties with balance sheets. Montana provides the substrate any such currency can run on.
- *§I rewritten* as "Two Jobs Bitcoin Conflated". §V renamed "Properties of the Rail" (was "Cash-System Tokenomics"). §IX rewritten as "What Montana Is" with the rail-not-currency closing. `Ɉ` is named as the rail's protocol-level reward unit, not a promise of stable purchasing power.
- *Dormant non-zero accounts are never touched* — added to §III.

**v1.1.0 (2026-05-28):**

- *Terminology* aligned with Montana Protocol v35.25.1. The primitive is named «sequential delay computation» / «iterated SHA-256 hash chain», not VDF. Montana's chain is deliberately not a verifiable delay function in the Boneh-Pietrzak-Wesolowski sense (see §II for the rationale). The smallest unit is `moneta`; `1 Ɉ = 10⁹ moneta`; the international ticker is `MONT`.
- *Finality claim corrected*: asynchronous finality is at window cementing — within a single window of the canonical order (approximately one minute of wall-clock at the genesis-hardware calibration), not the obsolete «~300 ms».
- *Reference-implementation count corrected* to 23 crates (from 12).

---

**Symbol:** **Ɉ** — Montana.

Alejandro Montana
