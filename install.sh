#!/bin/bash
# Montana ONE-COMMAND install — thin shim that downloads + runs the canonical
# Code/scripts/install-node.sh (node-only Docker image, consensus participation).
#
# After completion the new node:
#   - Runs montana-node (TimeChain consensus, P2P :8444) — appears in /api/peers
#   - Auto-detects country / city / coords from public IP via ip-api.com
#
# Usage (clean Ubuntu/Debian VPS, root):
#   curl -sSL https://raw.githubusercontent.com/efir369999/Montana/main/install.sh | sudo bash
#
# This shim is intentionally minimal — it only fetches and exec's the canonical
# script; all logic lives in Code/scripts/install-node.sh.

set -euo pipefail
URL="https://raw.githubusercontent.com/efir369999/Montana/main/Code/scripts/install-node.sh"

[ "$(id -u)" = "0" ] || { printf '[install] root required: sudo bash %s\n' "$0" >&2; exit 1; }

if ! command -v curl >/dev/null 2>&1; then
  apt-get update -qq && apt-get install -y -qq curl
fi

printf '[install] fetching canonical installer from %s\n' "$URL"
TMP=$(mktemp)
trap "rm -f $TMP" EXIT
curl -fsSL "$URL" -o "$TMP"
chmod +x "$TMP"
exec bash "$TMP" "$@"
