# Montana v1.0.0 — mainnet readiness for external audit

**Release tag.** `v1.0.0` (2026-05-22).
**Annotated tag SHA.** `a260ba9005c48763fadad0de5797bae48989215e`.
**Repository.** https://github.com/efir369999/Montana
**Release page.** https://github.com/efir369999/Montana/releases/tag/v1.0.0
**Live mesh explorer.** https://efir.org/explorer/
**Audit walkthrough.** [`AUDIT-WALKTHROUGH-v1.0.0.md`](AUDIT-WALKTHROUGH-v1.0.0.md) — runnable ten-step checklist from a fresh shell.

This document is the single landing page for an external auditor, cryptographer, or stakeholder evaluating whether the Montana v1.0.0 mainnet tag is in audit-ready state. Every claim below is anchored in a file or live endpoint that the auditor reads or probes directly.

---

## 1. Readiness summary

| Readiness dimension | Status | Evidence |
|---------------------|--------|----------|
| Release tagged on `main` | ✅ ready | `git rev-parse v1.0.0` ⇒ `a260ba9005c48763fadad0de5797bae48989215e` |
| GitHub Release page live | ✅ ready | https://github.com/efir369999/Montana/releases/tag/v1.0.0 |
| Release notes published | ✅ ready | [`Code/RELEASE-v1.0.0.md`](../Code/RELEASE-v1.0.0.md) |
| CI green on the tag | ✅ ready | `cargo fmt --all -- --check` + `cargo clippy --workspace --all-targets -- -D warnings` + `cargo test --workspace --release` |
| Workspace compiles to a single binary | ✅ ready | `cargo build --release -p montana-node` |
| Genesis manifest bundled in the install path | ✅ ready | [`Code/scripts/genesis-manifest.json`](../Code/scripts/genesis-manifest.json) (SHA-256: `f42a9e2d5d76c41285ee933e9172540981237b8e3935dc169886ae61df6c6f8e`) |
| Public install script | ✅ ready | [`Code/scripts/install-vps.sh`](../Code/scripts/install-vps.sh) |
| Live Genesis cohort | ✅ ready | Moscow + Frankfurt + Helsinki, all TCP-reachable on the libp2p port |
| External operators on the live mesh | ✅ ready | Yerevan + New York + the maintainer's macOS workstation visible at https://efir.org/explorer/data.json under `discovered_peers[]`, each carrying a peer-id-keyed `label` field |
| Desktop monitor app (macOS) | ✅ ready | [`Code/desktop/MontanaMonitor/`](../Code/desktop/MontanaMonitor/) — single-file SwiftUI status-bar app, builds via `bash build.sh`, reads the same `data.json` the explorer renders |
| Production transport: Noise_PQ XX | ✅ ready | ML-KEM-768 + ML-DSA-65 + ChaCha20-Poly1305; classical handshakes removed |
| M7 fast-sync algorithmic gate | ✅ ready | `mt-sync` crate, 17 unit tests, byte-equal cross-implementation conformance |
| M7 server-side dispatcher | ✅ ready | `MsgType::FastSyncRequest` is answered by chunked `FastSyncResponse` in `montana-node` |
| Spec deviation log up to date | ✅ ready | [`Code/docs/SPEC_DEVIATIONS.md`](../Code/docs/SPEC_DEVIATIONS.md) |
| Sixteen-finding 2026-05-19 audit closed | ✅ ready | Disposition: [`montana-response-to-2026-05-19-audit.md`](montana-response-to-2026-05-19-audit.md) |
| Maintainer-side critic audit pass | ✅ ready | Eight findings — two closed, six escalated: [`critic-audit-v1.0.0-mainnet.md`](critic-audit-v1.0.0-mainnet.md) |
| Audit scope document | ✅ ready | [`AUDIT-SCOPE-v1.0.0.md`](AUDIT-SCOPE-v1.0.0.md) |
| Audit walkthrough | ✅ ready | [`AUDIT-WALKTHROUGH-v1.0.0.md`](AUDIT-WALKTHROUGH-v1.0.0.md) |
| Reading list | ✅ ready | [`README-external-audit-v1.0.0.md`](README-external-audit-v1.0.0.md) |
| Explorer auto-refresh + auto-discovery | ✅ ready | Cron every 30 s; HTML auto-refresh every 15 s; mirrored at [`Code/scripts/montana-explorer-collect.py`](../Code/scripts/montana-explorer-collect.py) |

---

## 2. The three priority-1 audit asks

The maintainer's request to the external reviewer is concentrated on three lines of work. Each line has a single normative spec section, a single implementation entry point, and a single set of automated tests that bind the wire-level or invariant-level claim.

### 2.1 Noise_PQ XX transport — transcript binding

| Item | Pointer |
|------|---------|
| Spec  | [`Montana Network v1.1.0.md`](../Montana%20Network%20v1.1.0.md) §«Noise_PQ XX wire format» |
| Code  | [`Code/crates/mt-noise-pq/src/xx_handshake.rs`](../Code/crates/mt-noise-pq/src/xx_handshake.rs) + [`Code/crates/mt-noise-pq/src/xx_libp2p_upgrade.rs`](../Code/crates/mt-noise-pq/src/xx_libp2p_upgrade.rs) |
| libp2p plug-in | [`Code/crates/mt-net-transport/src/xx_noise_pq_upgrade.rs`](../Code/crates/mt-net-transport/src/xx_noise_pq_upgrade.rs) |
| Tests | `cargo test -p mt-noise-pq --release` + `cargo test -p mt-net-transport --release` |

**Reviewer's question:** is the ML-DSA-65 identity signature in msg2 / msg3 bound to the post-ML-KEM-768-decapsulation transcript hash, not the pre-decapsulation hash? Is the PeerId derivation (SHA-256 multihash, libp2p sha2-256 multihash code 0x12) sound against identity-substitution attacks during the handshake?

### 2.2 Sequential SHA-256 delay function reduction

| Item | Pointer |
|------|---------|
| Spec  | [`Whitepaper Montana.md`](../Whitepaper%20Montana.md) §5 (threat model + attack-class subsections); [`Montana Network v1.1.0.md`](../Montana%20Network%20v1.1.0.md) §«Lookback Leadership» |
| Code  | [`Code/crates/mt-timechain/src/lib.rs`](../Code/crates/mt-timechain/src/lib.rs) |
| Disclaimer | The construction is explicitly **not** a SSHA in the Boneh / Pietrzak / Wesolowski sense; there is no proof of correct evaluation |

**Reviewer's question:** what is the reduction from the cementing rule to the unforgeability of `t_r(W)` under the proposer-verifier asymmetry? Is the two-window lookback (`cemented_bundle_aggregate(W − 2)`) sufficient against grinding when the proposer holds a hardware advantage ×K over a verifier?

### 2.3 Constant-time discipline of the crypto-native shim

| Item | Pointer |
|------|---------|
| Code  | [`Code/crates/mt-crypto-native/`](../Code/crates/mt-crypto-native/) — C bindings over OpenSSL 3.5.5 LTS pinned via `openssl-src = "=300.5.5+3.5.5"` |
| Tests | [`Code/crates/mt-crypto/tests/security_invariants.rs`](../Code/crates/mt-crypto/tests/security_invariants.rs) — 13 automated invariants |
| Security cards | [`Code/docs/security-cards.md`](../Code/docs/security-cards.md) — per-primitive secret-site enumeration |

**Reviewer's question:** is the production crypto path constant-time end-to-end for ML-DSA-65 / ML-KEM-768 per the row-level requirement in [`Whitepaper Montana.md`](../Whitepaper%20Montana.md) §13? Are secret-material allocations protected by `mlock` and zeroed on `Drop`?

---

## 3. Open items carried into v1.0.1

These are spec-vs-code deviations or document-hygiene findings that the maintainer-side critic audit identified and escalated. None of them block the v1.0.0 mainnet tag. Closure paths are written into the deviation log.

| Item | Severity | Closure path |
|------|----------|--------------|
| **DEV-012 Phase B + C** — multi-confirmer cementing in the Active phase | post-mainnet | [`Code/docs/SPEC_DEVIATIONS.md`](../Code/docs/SPEC_DEVIATIONS.md) §DEV-012 |
| **DEV-015** — M7 client-side handler (drain chunks + verify + LocalState swap) | post-mainnet | [`Code/docs/SPEC_DEVIATIONS.md`](../Code/docs/SPEC_DEVIATIONS.md) §DEV-015 |
| **F-003** — Cyrillic content in `Montana Network v1.1.0.md` (1796 hits) | high | [`critic-audit-v1.0.0-mainnet.md`](critic-audit-v1.0.0-mainnet.md) §F-003 |
| **F-004** — Whitepaper §Nash retains a temporal marker | low | [`critic-audit-v1.0.0-mainnet.md`](critic-audit-v1.0.0-mainnet.md) §F-004 |
| **F-005..F-008** — Cyrillic content in four supporting docs (audit-checklist / security-cards / build-from-source / VERSION) | medium-low | [`critic-audit-v1.0.0-mainnet.md`](critic-audit-v1.0.0-mainnet.md) §F-005–F-008 |
| **MONT-001** — independent constant-time review of `mt-crypto-native` | high | Priority-1 audit ask, see §2.3 above |

---

## 4. The live mesh as evidence

The auditor can corroborate every claim above against the running network without any privileged access.

| Probe | Command | Expected |
|-------|---------|----------|
| Explorer JSON live | `curl -sS https://efir.org/explorer/data.json` | `updated` field within ~30 s of `date -u`; three Genesis nodes plus the Yerevan operator |
| Genesis TCP reachability | `python3 -c "import json; [print(p['multiaddr']) for p in json.load(open('Code/scripts/genesis-manifest.json'))['peers']]"` + `nc -z` on each | three `open` results |
| Repo sources at the tag | `git clone https://github.com/efir369999/Montana.git && cd Montana && git checkout v1.0.0 && git rev-parse v1.0.0` | `a260ba9005c48763fadad0de5797bae48989215e` |
| Build + test | `cd Code && cargo test --workspace --release` | exit 0 |
| Onboard a fresh operator | `sudo bash Code/scripts/install-vps.sh` on a clean Linux VPS | new node visible in `discovered_peers[]` within ~16 minutes |

---

## 5. Engagement model

| Channel | Use |
|---------|-----|
| Public findings | https://github.com/efir369999/Montana/issues, label `mainnet-v1.0.0` |
| Cryptography list channel | Plaintext mail to `cryptography@metzdowd.com`, body references the v1.0.0 tag SHA |
| Acknowledgment SLA | Seven days |
| Written disposition SLA | Thirty days |
| Confidentiality | None. Public on-record review only |
| Bug bounty | None. The repository is dual-licensed Apache-2.0 / MIT; the protocol is non-token |

---

## 6. Audit-bundle index

| Document | Purpose |
|----------|---------|
| [`MAINNET-READINESS-v1.0.0.md`](MAINNET-READINESS-v1.0.0.md) | This document — single landing page |
| [`AUDIT-WALKTHROUGH-v1.0.0.md`](AUDIT-WALKTHROUGH-v1.0.0.md) | Runnable ten-step verification from a fresh shell |
| [`AUDIT-SCOPE-v1.0.0.md`](AUDIT-SCOPE-v1.0.0.md) | Scope boundaries — priority-1 / priority-2 / out-of-scope |
| [`README-external-audit-v1.0.0.md`](README-external-audit-v1.0.0.md) | Recommended reading order |
| [`critic-audit-v1.0.0-mainnet.md`](critic-audit-v1.0.0-mainnet.md) | Maintainer-side critic findings (two closed, six escalated) |
| [`montana-response-to-2026-05-19-audit.md`](montana-response-to-2026-05-19-audit.md) | Disposition for the sixteen-finding consolidated review |
| [`montana-deep-retrospective-2026-05-21.md`](montana-deep-retrospective-2026-05-21.md) | Empirical record of the four-node mesh under operational load before the v1.0.0 tag |
| [`transport-identifier-leakage.md`](transport-identifier-leakage.md) | Byte-by-byte analysis showing Noise_PQ XX has no plaintext long-term identifier on the wire — the MTProto `auth_key_id` correlation class is structurally not reachable |
| [`Code/docs/SPEC_DEVIATIONS.md`](../Code/docs/SPEC_DEVIATIONS.md) | Complete spec-vs-code deviation log |

— Montana maintainer, 2026-05-22.
