# Montana v1.0.0 — external audit walkthrough

This document is the runnable checklist for a third-party reviewer who wants to verify the v1.0.0 mainnet tag end-to-end from a fresh shell, without privileged access to any maintainer infrastructure. Every step is a public probe — no SSH key, no API token, no email handshake required.

**Anchors as of audit-tag time (2026-05-22):**

| Item | Value |
|------|-------|
| Release tag | `v1.0.0` |
| Annotated tag SHA | `a260ba9005c48763fadad0de5797bae48989215e` |
| Top of `main` at tag | commit `14a8dac` (2026-05-22 00:31:03 +0300) |
| Workspace version | `1.0.0` (18 crates) |
| Spec target | Protocol v35.25.1 + Network v1.1.0 + App v3.12.0 |
| Repository | https://github.com/efir369999/Montana |
| Live mesh explorer | https://efir.org/explorer/ |
| Explorer data.json | https://efir.org/explorer/data.json |

**Hash anchors for the four most-cited artifacts (SHA-256, computed from the v1.0.0 checkout):**

```
f42a9e2d5d76c41285ee933e9172540981237b8e3935dc169886ae61df6c6f8e  Code/scripts/genesis-manifest.json
e8b31b2e4ee0fc031587da754f3634e42f36be5f3d02f4f50b5a3c7adf174b9a  Whitepaper Montana.md
b5fd519d22dd1535a90c84bbc239c301a25f267c7fce5c2d1a9fc4fccb338174  Montana Network v1.1.0.md
3b463b3fa43629556cd7d4f6ab6ccac86231a22a0c69dd2271eddd366c1cabe6  Code/RELEASE-v1.0.0.md
```

Each subsequent step prints the command the auditor runs and the property the output establishes.

---

## Step 1 — clone the repository at the tagged commit

```
git clone https://github.com/efir369999/Montana.git montana-audit
cd montana-audit
git checkout v1.0.0
git rev-parse v1.0.0
```

**Expected output:** `a260ba9005c48763fadad0de5797bae48989215e`.

**Establishes:** the auditor is reading the same byte-stream the v1.0.0 GitHub release was cut from. Any divergence at this step is a tampered clone; abort and re-fetch.

---

## Step 2 — verify artifact hashes byte-for-byte

```
shasum -a 256 \
  Code/scripts/genesis-manifest.json \
  "Whitepaper Montana.md" \
  "Montana Network v1.1.0.md" \
  Code/RELEASE-v1.0.0.md
```

Compare against the four hash anchors at the top of this document. A single byte of drift fails the check.

**Establishes:** the published whitepaper, network spec, release notes, and pinned genesis manifest are exactly the artifacts the maintainer publishes against. The auditor's reasoning from this point uses those four files as authoritative.

---

## Step 3 — reproduce the CI gate on the v1.0.0 tag

The CI gate that the v1.0.0 tag passed is three commands. The auditor reproduces them on a Linux or macOS host with Rust stable (1.95+) installed.

```
cd Code
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --release
```

**Expected:** all three exit 0. A representative full pass takes 15–25 minutes on commodity hardware (release-mode build + 200+ tests).

**Establishes:** the entire Rust workspace compiles clean, passes lint at warning-as-error, and every test passes. The maintainer commits no failing or skipped test in `main`.

If any step fails, that is a regression against the v1.0.0 tag and a blocking finding. Open a GitHub issue at https://github.com/efir369999/Montana/issues with label `mainnet-v1.0.0` and the exact failure output.

---

## Step 4 — probe the live mesh from outside

The three Genesis nodes — Moscow, Frankfurt, Helsinki — list their multiaddrs in `Code/scripts/genesis-manifest.json`. The bundle is what every operator dials on first start. From any host with `nc`, `python3`, or `curl`:

```
python3 -c "
import json
m = json.load(open('Code/scripts/genesis-manifest.json'))
for p in m['peers']:
    ip = p['multiaddr'].split('/ip4/')[1].split('/')[0]
    port = p['multiaddr'].split('/tcp/')[1]
    print(p['label'], ip, port)
" | while read label ip port; do
    nc -z -w 5 "$ip" "$port" && echo "$label tcp/$port open"
done
```

**Expected:** three `open` lines — one per Genesis peer. The TCP socket on the libp2p port (`8444` per the bundled manifest) is what handles the Noise_PQ XX handshake.

External operators discovered by the live mesh appear at `efir.org/explorer/data.json` under `discovered_peers[]` with a public-safe label (city or `external`) — the raw IP is intentionally masked in the explorer JSON per the public-artifact rule. The auditor's own node, after Step 6, appears there.

**Establishes:** the three Genesis nodes whose multiaddrs are pinned in the manifest are TCP-reachable on the libp2p port that handles the Noise_PQ XX handshake. A failure to reach a Genesis node means either the node is down or a network path between the auditor and the node is broken; both are recorded in subsequent `data.json` snapshots, the auditor can corroborate.

---

## Step 5 — verify the explorer reflects the live state

```
curl -sS https://efir.org/explorer/data.json | python3 -m json.tool | head -80
```

The `updated` field must be within ~60 seconds of `date -u`. The `nodes` array must contain Moscow, Frankfurt, Helsinki entries with `status: active` and a `current_window` value advancing at roughly one window per minute. The `discovered_peers` array must contain the Yerevan operator (peer_id `Qma3XZ8mJZDD4MbtJVNxCyS2sYYn9BQRzxYvfiXiMbNCp9`, `remote_ip: yerevan`) with `last_heartbeat_seconds_ago` ≤ 60.

**Establishes:** the explorer is a live read of the journal output of the Moscow orchestrator node, refreshed every 30 seconds via the cron in `/etc/cron.d/montana-explorer`. The collector that produces `data.json` is mirrored in the repo at [`Code/scripts/montana-explorer-collect.py`](../Code/scripts/montana-explorer-collect.py) — the auditor reads the exact source that writes the file. Raw IPs are masked to city labels in the rendered JSON per the public-artifact rule; the auditor sees the same state any operator on the network sees.

---

## Step 6 — onboard the auditor's own node

The strongest evidence that the install path described in the release notes works is to run it. On any clean Linux VPS (Ubuntu, Debian, Fedora, RHEL, Alpine) as root:

```
git clone https://github.com/efir369999/Montana.git /opt/montana
cd /opt/montana
git checkout v1.0.0
bash Code/scripts/install-vps.sh
```

Within roughly 16 minutes of starting `install-vps.sh`, the auditor's node should:

1. Compile montana-node from source (~12 minutes on commodity 2-vCPU).
2. Generate the auditor's own ML-DSA-65 identity (`identity.bin`).
3. Start the systemd unit `montana-node.service`.
4. Dial the three Genesis peers from the bundled manifest.
5. Complete the Noise_PQ XX handshake with each of the three peers.

The auditor's own journal — `journalctl -u montana-node -f` — must show:

```
[network] CONNECTION ESTABLISHED peer=Q… label=moscow    remote=<moscow_multiaddr>
[network] CONNECTION ESTABLISHED peer=Q… label=frankfurt remote=<frankfurt_multiaddr>
[network] CONNECTION ESTABLISHED peer=Q… label=helsinki  remote=<helsinki_multiaddr>
[network] heartbeat OK peer=Q… request_id=…
```

Within ~60 seconds of the auditor's connection-established events, `https://efir.org/explorer/data.json` must contain the auditor's peer_id in `discovered_peers[]` with the auditor's public IP.

**Establishes:** the install flow described in `Code/RELEASE-v1.0.0.md` works end-to-end against the live mesh, the auditor's node successfully negotiates Noise_PQ XX with all three Genesis peers, and the auto-discovery in the explorer collector picks up the new node within one minute.

This is the most expensive step (it costs ~$5 of VPS time for the verification window), but it is the only step that proves the install-path end-to-end against the live network.

---

## Step 7 — inspect the Noise_PQ XX handshake at wire level

The handshake state machine and the libp2p plug-in live at:

- [`Code/crates/mt-noise-pq/src/xx_handshake.rs`](../Code/crates/mt-noise-pq/src/xx_handshake.rs) — 3-message state machine (msg1 1184 B / msg2 7533 B / msg3 6349 B).
- [`Code/crates/mt-noise-pq/src/xx_libp2p_upgrade.rs`](../Code/crates/mt-noise-pq/src/xx_libp2p_upgrade.rs) — async drive over `tokio::io::{AsyncRead, AsyncWrite}`.
- [`Code/crates/mt-net-transport/src/xx_noise_pq_upgrade.rs`](../Code/crates/mt-net-transport/src/xx_noise_pq_upgrade.rs) — `InboundConnectionUpgrade` + `OutboundConnectionUpgrade` impls for libp2p `SwarmBuilder`.
- [`Montana Network v1.1.0.md`](../Montana%20Network%20v1.1.0.md), §«Noise_PQ XX wire format» — normative byte layout.

KAT vectors that bind the wire format byte-exact:

```
cargo test -p mt-noise-pq --release
cargo test -p mt-net-transport --release
```

Both targets pass on the v1.0.0 tag. The auditor reads the tests to see what wire-format claims they bind.

**Audit asks** (priority 1 per `External-Audit/AUDIT-SCOPE-v1.0.0.md` §1.1):

- Does the ML-DSA-65 identity signature in msg2 / msg3 cover the post-ML-KEM-768-decapsulation transcript hash, not the pre-decapsulation hash?
- Is the PeerId-from-ML-DSA-65 binding (SHA-256 multihash, libp2p sha2-256 multihash code 0x12) sound against identity-substitution attacks during the handshake?
- Does any field in the transcript carry attacker-grindable content?

Findings against this section go to https://github.com/efir369999/Montana/issues with label `mainnet-v1.0.0` and severity in the body.

---

## Step 8 — inspect the sequential SHA-256 delay function and its reduction

Code: [`Code/crates/mt-timechain/src/lib.rs`](../Code/crates/mt-timechain/src/lib.rs).
Spec: [`Whitepaper Montana.md`](../Whitepaper%20Montana.md) §5 (threat model, attack-class subsections) + §13 (cryptographic primitives table); [`Montana Network v1.1.0.md`](../Montana%20Network%20v1.1.0.md) §«Lookback Leadership».

The implementation is **explicitly not** a SSHA in the Boneh / Pietrzak / Wesolowski sense — there is no proof of correct evaluation. The disclaimer is in §5 of the whitepaper and is repeated in `Code/docs/SPEC_DEVIATIONS.md` under the spec target row.

**Audit asks** (priority 1 per `AUDIT-SCOPE-v1.0.0.md` §1.2):

- What is the reduction from the cementing rule to the unforgeability of `t_r(W)` under the proposer-verifier asymmetry?
- Is the two-window lookback (`cemented_bundle_aggregate(W − 2)`) sufficient against grinding when the proposer has hardware advantage ×K over a verifier?

---

## Step 9 — inspect the constant-time discipline of the crypto-native shim

Code: [`Code/crates/mt-crypto-native/`](../Code/crates/mt-crypto-native/) (C bindings over OpenSSL 3.5.5 LTS pinned via `openssl-src = "=300.5.5+3.5.5"`).
Tests: [`Code/crates/mt-crypto/tests/security_invariants.rs`](../Code/crates/mt-crypto/tests/security_invariants.rs) — 13 automated invariants on the secret-material handling.
Security cards: [`Code/docs/security-cards.md`](../Code/docs/security-cards.md) — per-primitive secret-site enumeration. (Note: this file contains Cyrillic prose annotations — see F-006 in `critic-audit-v1.0.0-mainnet.md`.)

**Audit asks** (priority 1 per `AUDIT-SCOPE-v1.0.0.md` §1.3):

- Is the production crypto path constant-time end-to-end for ML-DSA-65 / ML-KEM-768 per the row-level spec requirement in `Whitepaper Montana.md` §13?
- Are secret-material allocations protected by `mlock` and zeroed on `Drop`?

---

## Step 10 — file findings, expect acknowledgment within seven days

- Findings → GitHub issues at https://github.com/efir369999/Montana/issues with label `mainnet-v1.0.0`.
- Alternative channel → plaintext mail to the Metzdowd Cryptography List (`cryptography@metzdowd.com`) referencing the v1.0.0 tag SHA in the body.
- Acknowledgment SLA → seven days. Written disposition SLA → thirty days.
- Confidentiality → none. Public on-record review only.
- Bug bounty → none. The repository is dual-licensed Apache-2.0 / MIT; the protocol is non-token.

---

## Companion documents

| Document | Purpose |
|----------|---------|
| [`AUDIT-SCOPE-v1.0.0.md`](AUDIT-SCOPE-v1.0.0.md) | Scope boundaries — priority-1 asks, priority-2 asks, out-of-scope, known maintainer-side deviations |
| [`README-external-audit-v1.0.0.md`](README-external-audit-v1.0.0.md) | Recommended reading order across the audit-bundle documents |
| [`critic-audit-v1.0.0-mainnet.md`](critic-audit-v1.0.0-mainnet.md) | Eight findings from the maintainer-side critic audit pass (two closed, six escalated to author) |
| [`montana-response-to-2026-05-19-audit.md`](montana-response-to-2026-05-19-audit.md) | Disposition matrix for the sixteen-finding CISO-as-a-Service Team consolidated review |
| [`montana-deep-retrospective-2026-05-21.md`](montana-deep-retrospective-2026-05-21.md) | Empirical record of the four-node mesh under operational load before v1.0.0 |
| [`Code/docs/SPEC_DEVIATIONS.md`](../Code/docs/SPEC_DEVIATIONS.md) | Complete spec-vs-code deviation log with closure status per entry |

— Montana maintainer, 2026-05-22.
