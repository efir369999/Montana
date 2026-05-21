#!/bin/bash
# Montana VPS full install — Montana node + companion VPN endpoint on one Linux VPS.
#
# What it does:
#   1. Runs scripts/install-vps.sh    — Montana node (systemd + identity + start)
#   2. Runs montana-vpn/install.sh    — VPN endpoint (xray Reality + nginx decoy)
#
# The node and the VPN are two independent systemd services. Either can be
# stopped without affecting the other. Each service has its own README.
#
# Usage:
#   sudo bash scripts/install-vps-full.sh
#
# Optional environment overrides:
#   DECOY_HOST=www.cloudflare.com    Reality dest SNI (default googletagmanager)
#   CLIENT_EMAIL=alice               xray client email label
#   SKIP_NODE=1                      install VPN only (skip node)
#   SKIP_VPN=1                       install node only (skip VPN)

set -euo pipefail

SCRIPT_PATH="${BASH_SOURCE[0]}"
if [ -L "$SCRIPT_PATH" ]; then
  SCRIPT_PATH="$(readlink "$SCRIPT_PATH")"
fi
SCRIPT_DIR="$(cd "$(dirname "$SCRIPT_PATH")" && pwd)"
CODE_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

INSTALL_VPS="$SCRIPT_DIR/install-vps.sh"
INSTALL_VPN="$CODE_DIR/montana-vpn/install.sh"

log() { printf '\033[1;32m[install-vps-full]\033[0m %s\n' "$*"; }
die() { printf '\033[1;31m[install-vps-full] ERROR:\033[0m %s\n' "$*" >&2; exit 1; }

[ "$(id -u)" = "0" ] || die "root privileges required"

[ -f "$INSTALL_VPS" ] || die "missing: $INSTALL_VPS"
[ -f "$INSTALL_VPN" ] || die "missing: $INSTALL_VPN"

if [ "${SKIP_NODE:-0}" != "1" ]; then
  log ""
  log "================================================================"
  log "  STEP 1/2 — Montana node install"
  log "================================================================"
  log ""
  bash "$INSTALL_VPS"
else
  log "SKIP_NODE=1 — skipping node install"
fi

if [ "${SKIP_VPN:-0}" != "1" ]; then
  log ""
  log "================================================================"
  log "  STEP 2/2 — VPN endpoint install"
  log "================================================================"
  log ""
  bash "$INSTALL_VPN"
else
  log "SKIP_VPN=1 — skipping VPN install"
fi

log ""
log "================================================================"
log "  DONE"
log "================================================================"
log ""
log "Montana node:  systemctl status montana-node"
log "VPN endpoint:  systemctl status xray"
log "decoy nginx:   systemctl status nginx"
log ""
log "Node logs:     journalctl -u montana-node -f"
log "VPN logs:      journalctl -u xray -f"
log ""
log "VLESS URL for the client was printed above in step 2."
log "The node's mnemonic backup was printed in step 1 — keep it safe."
