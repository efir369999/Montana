#!/bin/bash
# Montana — one-command autonomous node install on ANY clean Linux server.
#
#   curl -fsSL https://raw.githubusercontent.com/efir369999/Montana/main/Code/scripts/install-node.sh | sudo bash
#
# Fully out-of-the-box, zero manual steps:
#   - installs Docker if missing
#   - fetches the canonical genesis manifest automatically (montana.quest)
#   - auto-detects city/country from geo-IP (no IP published)
#   - builds & runs the node-only image
#   - the node self-reports to the explorer API every 30s → appears in the
#     live network at https://montana.quest/net automatically
set -euo pipefail
REPO_URL="${MONTANA_REPO_URL:-https://github.com/efir369999/Montana.git}"
REPO_BRANCH="${MONTANA_REPO_BRANCH:-main}"
WORKDIR="${MONTANA_BUILD_DIR:-/opt/montana-src}"
MANIFEST_URL="${MONTANA_MANIFEST_URL:-https://montana.quest/genesis-manifest.json}"
IMAGE="montana-node:local"

echo "== Montana autonomous node install =="
command -v docker >/dev/null 2>&1 || { echo "[1/6] installing Docker"; curl -fsSL https://get.docker.com | sh; }
echo "[1/6] Docker: $(docker --version)"

echo "[2/6] fetching source"
rm -rf "$WORKDIR"; git clone --depth 1 -b "$REPO_BRANCH" "$REPO_URL" "$WORKDIR"

echo "[3/6] fetching canonical genesis manifest"
mkdir -p /etc/montana
curl -fsSL "$MANIFEST_URL" -o /etc/montana/genesis-manifest.json
echo "    manifest sha256=$(sha256sum /etc/montana/genesis-manifest.json | cut -c1-16)"

echo "[4/6] auto-detecting location (city/country from geo-IP)"
GEO="$(curl -fsSL --max-time 6 'http://ip-api.com/json?fields=city,countryCode,isp,org' 2>/dev/null || echo '{}')"
CITY="${MONTANA_CITY:-$(printf '%s' "$GEO" | sed -n 's/.*"city":"\([^"]*\)".*/\1/p')}"
CC="${MONTANA_COUNTRY:-$(printf '%s' "$GEO" | sed -n 's/.*"countryCode":"\([^"]*\)".*/\1/p')}"
ALIAS="${MONTANA_ALIAS:-$(printf '%s' "${CITY:-node}" | tr 'A-Z ' 'a-z-' )}"
[ -n "$CITY" ] || CITY="$ALIAS"
echo "    alias=$ALIAS city=$CITY country=$CC"

echo "[5/6] building node image"
docker build -t "$IMAGE" -f "$WORKDIR/Code/docker/runtime/Dockerfile.node" "$WORKDIR" >/dev/null

echo "[6/6] starting node (joins network + self-reports to explorer)"
docker rm -f montana-node 2>/dev/null || true
docker run -d --name montana-node --network host --restart unless-stopped \
  -v montana-data:/var/lib/montana \
  -v /etc/montana/genesis-manifest.json:/etc/montana/genesis-manifest.json:ro \
  -e MONTANA_ALIAS="$ALIAS" -e MONTANA_LABEL="$CITY" -e MONTANA_COUNTRY="$CC" \
  "$IMAGE"
echo "== done. live in ~1 min at https://montana.quest/net =="
echo "logs: docker logs -f montana-node"
