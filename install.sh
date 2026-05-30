#!/bin/bash
# Montana ONE-COMMAND install — thin shim that downloads + runs the canonical
# Code/scripts/install-docker.sh (universal Reality privkey + orchestrator
# auto-registration + Montana VPN cascade auto-detect).
#
# After completion the new node:
#   - Runs montana-node (TimeChain consensus, P2P :8444) — appears in /api/peers
#   - Runs xray Reality endpoint :443 with the SHARED universal pbk/sid that
#     all Montana VPN endpoints use; per-host UUID is generated locally
#   - Registers itself with the Moscow orchestrator (built-in admin token);
#     appears in https://montana.quest/vpn/sub within ~5 min (systemd timer)
#   - If host IP is in a blocked CIDR or unreachable from Moscow: orchestrator
#     auto-provisions a cascade front via Frankfurt so the city stays reachable
#   - Auto-detects country / city / coords from public IP via ip-api.com
#
# Usage (clean Ubuntu/Debian VPS, root):
#   curl -sSL https://raw.githubusercontent.com/efir369999/Montana/main/install.sh | sudo bash
#
# Optional env overrides (see Code/scripts/install-docker.sh header for full list):
#   MONTANA_ALIAS=mycity         per-city hostname / cascade routing tag
#   MONTANA_NODE_TAG=myc         xray inbound tag (max 3 chars)
#   MONTANA_HOSTING=mycloud      hosting provider label in registry metadata
#   MONTANA_ORCH_TOKEN=xxx       override built-in orchestrator admin token
#
# This shim is intentionally minimal — it only fetches and exec's the canonical
# script; all logic lives in Code/scripts/install-docker.sh.

set -euo pipefail
URL="https://raw.githubusercontent.com/efir369999/Montana/main/Code/scripts/install-docker.sh"

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
