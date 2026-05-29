# Montana Manifesto

**Version:** 2.0.2
**Date:** 2026-05-29
**Author:** Alejandro Montana
**Repository:** [github.com/efir369999/Montana](https://github.com/efir369999/Montana)

A single declaration of what Montana is and refuses to be, published in three languages from one canonical version. The three texts say the same thing.

- [English](Manifesto%20EN.md) — for the Metzdowd cryptography list and independent reviewers
- [Русский](Manifesto%20RU.md) — голос автора
- [中文](Manifesto%20ZH.md) — 中文版本

The English version is canonical for cryptographic claims; the Russian version is canonical for the author's voice. For the academic specification of the protocol, see [Whitepaper Montana.md](../Whitepaper%20Montana.md) and [Montana Protocol v35.25.1](../Montana%20Protocol%20v35.25.1.md).

The manifesto is written in the academic register of a position paper, addressed simultaneously to the general reader and the cryptographer. The pain of ordinary users with money systems is named; the architectural response is grounded in the full protocol; the technical sections (§V–§IX) are the proof that the human-facing claims are achievable, not the headline of the document.

**v2.0.2 (2026-05-29) — critic pass closure.**

Three findings from the adversarial review of v2.0.1 are addressed.

- *§III post-quantum bullet rewritten.* The marketing line «Designed to be safe for your children's children» is replaced with a defensible analytical claim: «at NIST security level 3 (≈ 192-bit quantum-equivalent strength)». The lay register of §III is preserved; the consumer-marketing voice is removed.
- *§IX entry barrier carries the integer derivation inline.* The prior phrasing «approximately fourteen days of wall-clock on a commodity x86_64 core» is replaced with the full derivation: `vdf_entry_windows = 20 160 windows × D = 6.552 × 10¹² SHA-256 hashes`, fourteen days of wall-clock at the genesis-hardware calibration. The number is now peer-review defensible.
- *§IV present-tense for four ecosystem components is softened.* Messenger, Junona, Pluton, and Vera Montana are framed in design-stage language («a reference implementation is in development», «is defined as», «designed to», «at the design stage»). Shipping components (Rail, Money, Wallet, Anchor, VPN, Hub) keep present-tense. The reader can no longer mistake design vision for production reality.

**v2.0.1 (2026-05-29) — tone correction.** The «last nail in the coffin of two failed worlds» language and the «blue ocean» framing of v2.0.0 are replaced with analytical equivalents. The substance is unchanged. The thesis line now reads: *Montana is an architecture that closes the failures of both fiat and cryptocurrency on a single foundation.* §IV is named «The Ecosystem». §XI closes with the analytical thesis and «the ether the name promised».

**v2.0.0 (2026-05-29) — major reframe.** The thesis shifted from architectural rail-vs-currency positioning (a cryptographer-facing axis) to the failure of two money systems for the people who use them. §I «Two Worlds That Failed» names the fiat failures (inflation, debanking, censorship, surveillance, 30% remittance corridors, uneconomic small payments, asset-price-driven housing exclusion, sanction targeting, CBDC-as-surveillance) and the crypto failures (Bitcoin-as-digital-gold, Ethereum-as-MEV-land, stablecoins-on-broken-banks, DeFi-as-Ponzi, failing exchanges). The same underlying cause: rail reliability coupled to native-asset price, rail neutrality contingent on next-block producer, rail primitives that Shor breaks. §IV introduced the full Montana ecosystem. §III «Montana in Plain Terms» lays out user-facing properties.

**v1.3.0 (2026-05-29):** Ethereum recognized and critiqued alongside Bitcoin in §I; no extractable ordering added to §V as a rail property; §IX closes with «the ether the name promised».

**v1.2.0 (2026-05-28):** Rail-not-currency positioning from the Metzdowd thread. Bitcoin's title conflated two distinct jobs (stable unit of account vs. neutral settlement and ordering rail). Montana picks the second on purpose. `Ɉ` is the rail's reward unit, not a stable unit of account. Dormant non-zero accounts are never touched.

**v1.1.0 (2026-05-28):** Terminology aligned with Montana Protocol v35.25.1: sequential delay computation / iterated SHA-256 hash chain, not VDF. Finality at window cementing (~one minute on commodity x86_64), not «300 ms». Twenty-three crates, not twelve.

---

**Symbol:** **Ɉ** — Montana.

Alejandro Montana
