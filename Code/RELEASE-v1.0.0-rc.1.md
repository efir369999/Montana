# Montana v1.0.0-rc.1 — first mainnet release candidate

**Release date:** 2026-05-21
**Tag:** `v1.0.0-rc.1`
**Spec target:** Protocol v35.25.1 + Network v1.1.0 + App v3.12.0
**Reference implementation:** Rust workspace 17 crates, `0.1.2`

This is the first mainnet-readiness candidate after the closure of the Metzdowd security review (issue #1, sixteen findings) and the production deployment of Noise_PQ XX as the network transport. The release is published for external review against the live three-node Genesis cohort.

---

## Scope

### Production-ready in this release

- **Noise_PQ XX transport.** TCP → Noise_PQ XX → Yamux is the production handshake stack. ML-KEM-768 ephemeral key encapsulation on both sides, ML-DSA-65 identity signatures over the transcript, ChaCha20-Poly1305 AEAD on the established session. Wire format byte-exact to Network specification v1.1.0 (msg1 1184 B / msg2 7533 B / msg3 6349 B). PeerId derived as SHA-256 multihash (libp2p sha2-256 code 0x12) of each peer's ML-DSA-65 identity public key.
- **Three-node Genesis cohort.** Moscow, Frankfurt, Helsinki — full 6/6 pairwise mesh negotiating `/montana/noise-pq-xx/1.0.0` and exchanging Ping/Pong heartbeats every 5 s. Live since 2026-05-21 15:54 UTC.
- **External operator onboarding.** A clean Linux VPS clones `github.com/efir369999/Montana`, runs `Code/scripts/install-vps.sh`, and within ~16 minutes the new node is visible in all three Genesis peers' journals as `[network] CONNECTION ESTABLISHED peer=<XX peer_id> label=unknown` and exchanging heartbeats. End-to-end verified on a fresh Yerevan VPS at `<exit-am>` on 2026-05-21.
- **Auto-sync infrastructure.**
  - `montana-manifest-sync.timer` (every 10 min) — SSH-probes the live XX peer_id of mos/fra/zel, compares against the bundled `Code/scripts/genesis-manifest.json` in the repository, and pushes to `origin/main` on key rotation.
  - Explorer `data.json` collector (every 1 min) — discovers external operators automatically via heartbeat scan of Genesis journals; new peers appear at [efir.org/explorer/](https://efir.org/explorer/) with their public IP, last-heartbeat age, and the set of Genesis witnesses.
- **Closed Metzdowd findings.** All sixteen findings of the CISO-as-a-Service Team consolidated review of 2026-05-19 are addressed. Disposition: twelve accepted and fixed by construction in the whitepaper (WP-1..WP-12); two rejected with spec citations (MONT-003 race condition, WP-8 sub-claim); MONT-001 closed by spec patch (constant-time requirement on ML-DSA-65 / ML-KEM-768 rows); MONT-002 closed by `online_session_nonce` addition to the IBT online proof; MONT-004 documented as pre-mainnet operating state; DEV-014 (post-quantum transport migration) closed by switching the production transport to Noise_PQ XX. Full disposition in [`External-Audit/montana-response-to-2026-05-19-audit.md`](External-Audit/montana-response-to-2026-05-19-audit.md).

### Open blockers before v1.0.0 mainnet promotion

| ID | Title | Closure path |
|----|-------|--------------|
| DEV-012 | Singleton-only proposal generation in Active phase — multi-node `apply_proposal` across the Genesis cohort is not yet wired. | Implement cross-machine `apply_proposal` from peers; verify all three Genesis nodes converge on identical state_root after every window; document in M9 Phase 2 closure note. |
| M7      | Fast-sync snapshot mechanism. | Implement `mt-sync` crate with snapshot delivery rooted in the current Merkle state root; benchmark at the billion-account target. |
| Phase 3 part 3 | Cross-machine 24-hour soak of Noise_PQ XX. | Passive; heartbeats are flowing on the three Genesis peers + Armenia external operator continuously since 2026-05-21 15:54 UTC. |
| MONT-001 | ML-DSA-65 constant-time discipline verification. | Per `[C-5]` capability checklist already filled; mainnet promotion requires an independent constant-time audit of the production crypto path (mt-crypto-native via openssl-src 3.5 LTS). |

Promotion criterion to v1.0.0: all four items closed, and an independent cryptographer pass over the v1.0.0-rc.1 tag returns zero high-severity findings.

---

## How to join the live network

```bash
# On any clean Linux VPS (Ubuntu, Debian, Fedora, RHEL, Alpine — root):
git clone https://github.com/efir369999/Montana.git /opt/montana
sudo bash /opt/montana/Code/scripts/install-vps.sh
```

After the install completes:

1. `systemctl status montana-node` shows `active (running)`.
2. The journal shows `[network] CONNECTION ESTABLISHED peer=Q… label=moscow|frankfurt|helsinki` for each Genesis peer.
3. Within one minute, the new node appears at [efir.org/explorer/](https://efir.org/explorer/) under "Discovered peers" with the new node's public IP, last-heartbeat age, and the three Genesis witnesses.
4. The local node enters Phase 1 Bootstrap → CandidateSsha (sequential SHA-256 chain to `ssha_chain_length ≥ τ₂`, approximately fourteen days of wall-clock).

The install path is the canonical onboarding flow for the Metzdowd cryptography list audience and any independent operator who wants to join the live mesh.

---

## Verification artifacts

- **CI on the release tag.** `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace --release`. All four are green on `v1.0.0-rc.1`.
- **Live mesh stats.** Snapshot at [`STATUS.md`](STATUS.md). Current `data.json` at [efir.org/explorer/data.json](https://efir.org/explorer/data.json) reflects mos/fra/zel + any external discovered peers in real time.
- **External-audit response.** [`External-Audit/montana-response-to-2026-05-19-audit.md`](External-Audit/montana-response-to-2026-05-19-audit.md) — sixteen-finding disposition.
- **SPEC deviations log.** [`Code/docs/SPEC_DEVIATIONS.md`](Code/docs/SPEC_DEVIATIONS.md) — every known spec-vs-code deviation with closure path.

---

## License

Apache-2.0 OR MIT, at the operator's choice (see [`LICENSE`](LICENSE), [`Code/LICENSE-APACHE`](Code/LICENSE-APACHE), [`Code/LICENSE-MIT`](Code/LICENSE-MIT)).

---

## Contact for security review

- Issues and findings: [github.com/efir369999/Montana/issues](https://github.com/efir369999/Montana/issues)
- Next-round review with mainnet-readiness scope: tag the issue `mainnet-readiness`
- No email, no Discord, no Telegram — public on-record review only
