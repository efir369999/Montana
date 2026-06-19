#!/bin/bash
# Reset montana-node consensus state, keep identity, restart the node service.
#
# Use after a genesis change (the Genesis State Hash differs from the on-disk
# tables): the node re-bootstraps from the new genesis. identity.bin is
# preserved so the node keeps its keypair / XX peer_id and its label on the
# explorer. Works on macOS (launchd org.montana.node) and Linux (systemd
# montana-node); auto-detects the data directory.
#
# Usage:
#   bash reset-state.sh                 # auto-detect data dir + service
#   DATA_DIR=/custom/path bash reset-state.sh
set -euo pipefail

log()  { printf '\033[1;32m[reset-state]\033[0m %s\n' "$*"; }
warn() { printf '\033[1;33m[reset-state]\033[0m %s\n' "$*" >&2; }
die()  { printf '\033[1;31m[reset-state] ОШИБКА:\033[0m %s\n' "$*" >&2; exit 1; }

OS="$(uname -s)"

# --- locate data dir ---
if [ -n "${DATA_DIR:-}" ]; then
  :
elif [ "$OS" = "Darwin" ]; then
  for d in "$HOME/Applications/Montana/data" \
           "$HOME/Library/Application Support/Montana/node"; do
    [ -f "$d/identity.bin" ] && DATA_DIR="$d" && break
  done
else
  DATA_DIR="/var/lib/montana"
fi
[ -n "${DATA_DIR:-}" ] || die "data-dir не найден; задайте DATA_DIR=... явно"
[ -f "$DATA_DIR/identity.bin" ] || die "в $DATA_DIR нет identity.bin — это не data-dir узла"
log "data-dir: $DATA_DIR"

# --- stop service ---
if [ "$OS" = "Darwin" ]; then
  LABEL="org.montana.node"
  PLIST="$HOME/Library/LaunchAgents/$LABEL.plist"
  if launchctl list 2>/dev/null | grep -q "$LABEL"; then
    log "останавливаю launchd $LABEL"
    launchctl bootout "gui/$(id -u)/$LABEL" 2>/dev/null || launchctl unload "$PLIST" 2>/dev/null || true
  fi
else
  if systemctl is-active --quiet montana-node 2>/dev/null; then
    log "останавливаю systemd montana-node"
    sudo systemctl stop montana-node
  fi
fi

# --- wipe consensus state, keep identity.bin ---
log "стираю consensus-state (identity.bin сохраняю)"
find "$DATA_DIR" -mindepth 1 ! -name 'identity.bin' -delete
REMAIN="$(find "$DATA_DIR" -mindepth 1 | wc -l | tr -d ' ')"
log "в data-dir осталось файлов: $REMAIN (ожидается 1 — identity.bin)"

# --- restart service ---
if [ "$OS" = "Darwin" ]; then
  if [ -f "$PLIST" ]; then
    log "запускаю launchd $LABEL"
    launchctl bootstrap "gui/$(id -u)" "$PLIST" 2>/dev/null || launchctl load "$PLIST" 2>/dev/null || true
  else
    warn "plist $PLIST не найден — запустите узел вручную или переустановите (install-local-mac.sh)"
  fi
else
  log "запускаю systemd montana-node"
  sudo systemctl start montana-node
fi

log "готово — узел перезагружен на новый genesis"
