#!/bin/bash
# Montana — one-command node-only install on a clean Linux host (no VPN layer).
#
#   curl -fsSL https://raw.githubusercontent.com/efir369999/Montana/main/Code/scripts/install-node.sh | sudo bash
#
# Result: a single Docker container running ONLY the Montana consensus node on
# :8444. No xray, no nginx, no VPN, no orchestrator registration.
#
# Genesis manifest (decoupled from the image — never baked in):
#   To JOIN the canonical Montana network, place the canonical manifest at
#   /etc/montana/genesis-manifest.json BEFORE running this script (scp it, or
#   pass MONTANA_GENESIS_MANIFEST_B64). With a pinned sha (MONTANA_MANIFEST_SHA256)
#   the node refuses to start on any mismatch. With no manifest present the node
#   bootstraps as its own standalone genesis.
#
# Env knobs:
#   MONTANA_REPO_URL          default https://github.com/efir369999/Montana.git
#   MONTANA_REPO_BRANCH       default main
#   MONTANA_GENESIS_MANIFEST_B64   base64 of the manifest (alt to scp)
#   MONTANA_MANIFEST_SHA256        pin: refuse to start on mismatch
#   MONTANA_MNEMONIC               fixed identity (otherwise generated)
#   MONTANA_LISTEN                 default /ip4/0.0.0.0/tcp/8444
set -euo pipefail

REPO_URL="${MONTANA_REPO_URL:-https://github.com/efir369999/Montana.git}"
REPO_BRANCH="${MONTANA_REPO_BRANCH:-main}"
WORKDIR="${MONTANA_BUILD_DIR:-/opt/montana-src}"
IMAGE="montana-node:local"

echo "== Montana node install =="

# 1. Docker engine.
if ! command -v docker >/dev/null 2>&1; then
  echo "[1/5] installing Docker"
  curl -fsSL https://get.docker.com | sh
else
  echo "[1/5] Docker present: $(docker --version)"
fi

# 2. Source.
echo "[2/5] fetching source ($REPO_BRANCH)"
rm -rf "$WORKDIR"
git clone --depth 1 -b "$REPO_BRANCH" "$REPO_URL" "$WORKDIR"

# 3. Build node-only image (context = repo root).
echo "[3/5] building $IMAGE (this takes several minutes)"
docker build -t "$IMAGE" -f "$WORKDIR/Code/docker/runtime/Dockerfile.node" "$WORKDIR"

# 4. Genesis manifest (optional, host-mounted).
MANIFEST_ARGS=()
if [ -n "${MONTANA_GENESIS_MANIFEST_B64:-}" ]; then
  mkdir -p /etc/montana
  echo "$MONTANA_GENESIS_MANIFEST_B64" | base64 -d > /etc/montana/genesis-manifest.json
fi
if [ -f /etc/montana/genesis-manifest.json ]; then
  echo "[4/5] manifest present, sha256=$(sha256sum /etc/montana/genesis-manifest.json | cut -d' ' -f1)"
  MANIFEST_ARGS+=(-v /etc/montana/genesis-manifest.json:/etc/montana/genesis-manifest.json:ro)
else
  echo "[4/5] no manifest — standalone bootstrap genesis"
fi

# 5. Run.
echo "[5/5] starting container"
docker rm -f montana-node 2>/dev/null || true
docker run -d --name montana-node --network host --restart unless-stopped \
  -v montana-data:/var/lib/montana \
  "${MANIFEST_ARGS[@]}" \
  ${MONTANA_MANIFEST_SHA256:+-e MONTANA_MANIFEST_SHA256=$MONTANA_MANIFEST_SHA256} \
  ${MONTANA_MNEMONIC:+-e MONTANA_MNEMONIC=$MONTANA_MNEMONIC} \
  ${MONTANA_LISTEN:+-e MONTANA_LISTEN=$MONTANA_LISTEN} \
  "$IMAGE"

echo "== done =="
echo "Logs:    docker logs -f montana-node"
echo "Status:  docker exec montana-node montana-node status --data-dir /var/lib/montana"
