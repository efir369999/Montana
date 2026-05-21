# Montana v1.0.0 — first mainnet release

**Release date:** 2026-05-22
**Tag:** `v1.0.0`
**Spec target:** Protocol v35.25.1 + Network v1.1.0 + App v3.12.0
**Reference implementation:** Rust workspace 18 crates, `0.1.3`

This is the first mainnet release. It promotes `v1.0.0-rc.3` with the M7 fast-sync gate closed at the algorithmic level. The live four-node mesh — Moscow (bootstrap), Frankfurt, Helsinki, and Yerevan — has been running on Noise_PQ XX since 2026-05-21 15:54 UTC.

---

## Scope

### Production-ready in this release

- **Noise_PQ XX transport.** TCP → Noise_PQ XX → Yamux is the production handshake stack. ML-KEM-768 ephemeral key encapsulation on both sides, ML-DSA-65 identity signatures over the transcript, ChaCha20-Poly1305 AEAD on the established session. Wire format byte-exact to Network specification v1.1.0 (msg1 1184 B / msg2 7533 B / msg3 6349 B). PeerId derived as SHA-256 multihash (libp2p sha2-256 code 0x12) of each peer's ML-DSA-65 identity public key.
- **Genesis cohort live.** Moscow, Frankfurt, Helsinki — full 6/6 pairwise mesh negotiating `/montana/noise-pq-xx/1.0.0` and exchanging Ping/Pong heartbeats every 5 s.
- **External operator path verified.** A clean Linux VPS clones `github.com/efir369999/Montana`, runs `Code/scripts/install-vps.sh`, and within ~16 minutes the new node is visible in all three Genesis peers' journals and exchanging heartbeats. End-to-end verified on the Yerevan VPS `<exit-am>` on 2026-05-21.
- **M7 fast-sync snapshot mechanism.** `mt-sync` crate ships:
  - `Snapshot::from_tables(anchor_window, &AccountTable, &NodeTable, &CandidatePool)` — re-encodes typed records into canonical wire form.
  - `Snapshot::to_wire_chunks(records_per_chunk)` — flat-indexed chunked delivery across Account / Node / Candidate tables.
  - `Snapshot::build_tables()` — typed insertion back into live `mt_state` tables.
  - `SnapshotVerifier::verify(snapshot, expected_state_root)` — recomputes `state_root` via the production Sparse Merkle algorithm and the same `compute_state_root` domain-separated combiner the proposer uses, with byte-equal cross-implementation conformance proved by 17 unit tests.
  - The server-side dispatcher in `montana-node` answers `MsgType::FastSyncRequest` envelopes from peers by building a Snapshot at the current window and broadcasting chunked responses.
- **Auto-sync infrastructure.**
  - `montana-manifest-sync.timer` (every 10 min) — SSH-probes the live XX peer_id of mos/fra/zel, compares against the bundled `Code/scripts/genesis-manifest.json` in the repository, and pushes to `origin/main` on key rotation.
  - `montana-vpn-key-sync.timer` (every 5 min) — pulls Helsinki's `/var/lib/montana-net/my-vpn.json` (canonical xray Reality config), parses UUID/PBK/SID/SNI, writes `/etc/montana-vpn/keys.json` on Moscow; the Flask `/vpn/sub` endpoint reads keys with a 30 s cache.
  - Explorer `data.json` collector (every 1 min) — discovers external operators automatically via heartbeat scan of Genesis journals; new peers appear at [efir.org/explorer/](https://efir.org/explorer/) with their public IP, last-heartbeat age, and the set of Genesis witnesses.
- **Closed Metzdowd findings.** All sixteen findings of the CISO-as-a-Service Team consolidated review of 2026-05-19 are addressed. Disposition: twelve accepted and fixed by construction in the whitepaper (WP-1..WP-12); two rejected with spec citations (MONT-003 race condition, WP-8 sub-claim); MONT-001 closed by spec patch (constant-time requirement on ML-DSA-65 / ML-KEM-768 rows); MONT-002 closed by `online_session_nonce` addition to the IBT online proof; MONT-004 documented as pre-mainnet operating state; DEV-014 (post-quantum transport migration) closed by switching the production transport to Noise_PQ XX. Full disposition in [`External-Audit/montana-response-to-2026-05-19-audit.md`](External-Audit/montana-response-to-2026-05-19-audit.md).

### Carried into v1.0.1 (post-mainnet hot-fix track)

| ID | Title | Closure path |
|----|-------|--------------|
| DEV-012 Phase B+C | Multi-confirmer cementing in the Active phase — proposer-side BC accumulator with quorum + follower-side per-bundle validation against canonical `T_r(W)` and the cemented Proposal envelope schema bump from v1.1 to v1.2. The bootstrap-proposer path is the mainnet-baseline. | Phase B: proposer-side BC accumulator + envelope schema bump. Phase C: follower-side validate per-bundle on cemented set + state_root convergence verification across the 4-node cohort. |
| M7 client-side handler | Drain `FastSyncResponse` chunks on the receiver, verify against the anchor `ProposalHeader.state_root`, and swap the local state's tables. New operators currently replay history. | Wire the chunk accumulator + verifier + `LocalState` swap into `start.rs`. |
| MONT-001 constant-time audit | Independent cryptographer pass over the ML-DSA-65 / ML-KEM-768 hot paths in mt-crypto-native. | External audit scope after the mainnet tag. |

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
4. The local node enters Phase 1 Bootstrap → CandidateVdf (sequential SHA-256 chain to `vdf_chain_length ≥ τ₂`, approximately fourteen days of wall-clock).

The install path is the canonical onboarding flow for the Metzdowd cryptography list audience and any independent operator who wants to join the live mesh.

---

## Verification artifacts

- **CI on the release tag.** `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace --release`. All four green on `v1.0.0`.
- **Live mesh stats.** Snapshot at [`STATUS.md`](STATUS.md). Current `data.json` at [efir.org/explorer/data.json](https://efir.org/explorer/data.json) reflects mos/fra/zel + any external discovered peers in real time.
- **External-audit response.** [`External-Audit/montana-response-to-2026-05-19-audit.md`](External-Audit/montana-response-to-2026-05-19-audit.md) — sixteen-finding disposition.
- **SPEC deviations log.** [`Code/docs/SPEC_DEVIATIONS.md`](Code/docs/SPEC_DEVIATIONS.md) — every known spec-vs-code deviation with closure path.

---

## License

Apache-2.0 OR MIT, at the operator's choice (see [`LICENSE`](LICENSE), [`Code/LICENSE-APACHE`](Code/LICENSE-APACHE), [`Code/LICENSE-MIT`](Code/LICENSE-MIT)).

---

## Contact for security review

- Issues and findings: [github.com/efir369999/Montana/issues](https://github.com/efir369999/Montana/issues)
- Mainnet-readiness review scope: tag the issue `mainnet-v1.0.0`
- No email, no Discord, no Telegram — public on-record review only
