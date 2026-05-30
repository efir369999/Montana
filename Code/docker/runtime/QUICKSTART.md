# Montana node — quickstart

Run a full Montana mainnet node with one command.

## Option 1: pre-built image (recommended, ~30 seconds)

```bash
docker volume create montana-data
docker run -d \
  --name montana-node \
  --network host \
  --restart unless-stopped \
  -v montana-data:/var/lib/montana \
  ghcr.io/efir369999/montana-node:latest
```

The container:
- Listens on TCP :8444 (Noise_PQ XX → Yamux transport)
- Dials the 5-node bootstrap mesh (moscow, frankfurt, vilnius, armenia, nicosia) from the embedded `genesis-manifest.json`
- On first launch generates a 24-word mnemonic and writes it to `/var/lib/montana/mnemonic.txt` (mode 0400). **Save it immediately:**
  ```bash
  docker exec montana-node cat /var/lib/montana/mnemonic.txt
  ```

## Option 2: build from source (~5 minutes)

```bash
git clone https://github.com/efir369999/Montana.git /opt/montana
cd /opt/montana/Code/docker/runtime
docker compose up -d --build
```

Rebuilds `montana-node:local` from the current `main` branch.

## Verify

```bash
# Local — current_window advances every ~30 seconds:
docker exec montana-node /usr/local/bin/montana-node status --data-dir /var/lib/montana

# Live mesh — your node should appear in /api/peers of any bootstrap peer:
curl -sk https://efir.org:8443/montana-api/peers
```

After ~24 hours of cemented Proposal sync the node appears in `/api/nodes` as `Candidate`.
After ~14 days of sequential SHA-256 chain computation plus the next selection event it transitions to `Active` and starts participating in the lottery.

## Optional: VPN exit-node

To also operate the node as a Reality VLESS endpoint on the same VPS:

```bash
docker compose up -d xray nginx-decoy
```

`xray` listens on :443 (TLS Reality, disguised as `googletagmanager.com`), `nginx-decoy` on :80 serves a decoy landing page.

## Network state (current)

- **Cohort:** 5 nodes — `moscow` (bootstrap) + `frankfurt` + `vilnius` + `armenia` + `nicosia`
- **Build:** `26` (sha `b6e79bdc1e8b…`), image tag `v1.0.1-build26`
- **Proposer model:** bootstrap-only (Lookback rotation deferred to v1.0.2 pending DEV-021b drain refactor — see `Code/docs/SPEC_DEVIATIONS.md`)
- **Multi-confirmer cementing:** typically `bundles=2-3` per window
- **Reveal lottery (DEV-021):** distributes per-window emission winner across the full active cohort
- **Window time:** ~30–60 seconds per cemented Proposal

## Support

Open an issue: https://github.com/efir369999/Montana/issues
