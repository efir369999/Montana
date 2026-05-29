# Montana Manifesto

**Version:** 2.0.3
**Date:** 2026-05-29
**Author:** Alejandro Montana
**Repository:** [github.com/efir369999/Montana](https://github.com/efir369999/Montana)

A single declaration of what Montana is and refuses to be, published in three languages from one canonical version. The three texts say the same thing.

- [English](Manifesto%20EN.md) — for the Metzdowd cryptography list and independent reviewers
- [Русский](Manifesto%20RU.md) — голос автора
- [中文](Manifesto%20ZH.md) — 中文版本

The English version is canonical for cryptographic claims; the Russian version is canonical for the author's voice. For the academic specification of the protocol, see [Whitepaper Montana.md](../Whitepaper%20Montana.md) and [Montana Protocol v35.25.1](../Montana%20Protocol%20v35.25.1.md).

The manifesto is written in the academic register of a position paper, addressed simultaneously to the general reader and the cryptographer. The pain of ordinary users with money systems is named; the architectural response is grounded in the full protocol; the technical sections (§V–§IX) are the proof that the human-facing claims are achievable, not the headline of the document.

**v2.0.3 (2026-05-29) — full critic-pass closure.**

Three residual issues from the v2.0.2 re-pass are addressed.

- *«approximately» qualifiers in §III and §VIII* are replaced by either inline integer derivation or abstract framing. §V now carries the canonical genesis-hardware derivation `D / 5.097 × 10⁶ ≈ 63.8 seconds per window` once; §III drops the wall-clock estimate in favor of «within one window of the canonical order»; §VIII and §IX reference the §V derivation rather than repeating «approximately one minute». Every numerical claim in the manifesto is now traceable to a derivation or a constant.
- *«production-grade» in §II* is replaced with the analytical equivalent «Post-quantum succinct VDF constructions remain at research-grade status; none has published security audits or standardization at the level of FIPS 203 / FIPS 204». The construction's status is now stated in defensible terms.
- The v2.0.2 §III «at NIST security level 3» and §IX integer derivation (`vdf_entry_windows = 20 160 windows × D = 6.552 × 10¹² SHA-256 hashes`, fourteen days at the genesis-hardware calibration) are kept and remain peer-review defensible.

**v2.0.2 (2026-05-29) — critic-pass first round.** The marketing line «Designed to be safe for your children's children» of §III is replaced with a defensible analytical claim. §IX adds the integer derivation for the node-entry barrier. §IV softens present-tense for Messenger, Junona, Pluton, and Vera Montana (all design-stage); shipping components (Rail, Money, Wallet, Anchor, VPN, Hub) keep present-tense.

**v2.0.1 (2026-05-29) — tone correction.** The «last nail in the coffin of two failed worlds» language and the «blue ocean» framing of v2.0.0 are replaced with analytical equivalents. The thesis line: *Montana is an architecture that closes the failures of both fiat and cryptocurrency on a single foundation.* §IV is named «The Ecosystem». §XI closes with the analytical thesis and «the ether the name promised».

**v2.0.0 (2026-05-29) — major reframe.** The thesis shifted from architectural rail-vs-currency positioning to the failure of two money systems for the people who use them. §I «Two Worlds That Failed» names the fiat and crypto failures. §IV introduced the full Montana ecosystem. §III «Montana in Plain Terms» lays out user-facing properties.

**v1.3.0 (2026-05-29):** Ethereum recognized and critiqued alongside Bitcoin in §I; no extractable ordering added to §V as a rail property; §IX closes with «the ether the name promised».

**v1.2.0 (2026-05-28):** Rail-not-currency positioning from the Metzdowd thread. Bitcoin's title conflated two distinct jobs. Montana picks the second on purpose. `Ɉ` is the rail's reward unit, not a stable unit of account. Dormant non-zero accounts are never touched.

**v1.1.0 (2026-05-28):** Terminology aligned with Montana Protocol v35.25.1: sequential delay computation / iterated SHA-256 hash chain, not VDF. Finality at window cementing, not «300 ms». Twenty-three crates, not twelve.

---

**Symbol:** **Ɉ** — Montana.

Alejandro Montana
