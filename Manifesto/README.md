# Montana Manifesto

**Version:** 1.1.0
**Date:** 2026-05-28
**Author:** Alejandro Montana
**Repository:** [github.com/efir369999/Montana](https://github.com/efir369999/Montana)

A single declaration of what Montana is and refuses to be, published in three languages from one canonical version. The three texts say the same thing.

- [English](Manifesto%20EN.md) — for the Metzdowd cryptography list and independent reviewers
- [Русский](Manifesto%20RU.md) — голос автора
- [中文](Manifesto%20ZH.md) — 中文版本

The English version is canonical for cryptographic claims; the Russian version is canonical for the author's voice. For the academic specification of the protocol, see [Whitepaper Montana.md](../Whitepaper%20Montana.md) and [Montana Protocol v35.25.1](../Montana%20Protocol%20v35.25.1.md).

**v1.1.0 (2026-05-28):**

- *Terminology* aligned with Montana Protocol v35.25.1. The primitive is named «sequential delay computation» / «iterated SHA-256 hash chain», not VDF. Montana's chain is deliberately not a verifiable delay function in the Boneh-Pietrzak-Wesolowski sense (see §II for the rationale). Consensus is named Proof of Time. The smallest unit is `moneta`; `1 Ɉ = 10⁹ moneta`; the international ticker is `MONT`.
- *Cash-system frame* foregrounded. §I makes explicit that Bitcoin's title — *A Peer-to-Peer Electronic Cash System* — was never delivered, and identifies the two missing pieces Montana takes on: a cash-system tokenomics (§V) and an economics of time (§VI). §IX names Montana as the cash system Bitcoin promised, built on top of the economics of time the digital-money tradition has not yet built.
- *Finality claim corrected* against spec v35.25.1. Asynchronous finality is at window cementing — within a single window of the canonical order (approximately one minute of wall-clock at the genesis-hardware calibration), not the obsolete «~300 ms» claim carried from an early draft. The wall-clock duration of a window is an emergent property of the operating hardware, not part of consensus state.
- *Reference-implementation count corrected.* Twenty-three crates in `Code/crates/`, not twelve.

---

**Symbol:** **Ɉ** — Montana.

Alejandro Montana
