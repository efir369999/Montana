#!/bin/bash
# Montana VPS full install — узел Montana + VPN endpoint на одном Linux VPS.
#
# Что делает:
#   1. Запускает scripts/install-vps.sh    — узел Montana (systemd + identity + start)
#   2. Запускает montana-vpn/install.sh    — VPN endpoint (xray Reality + nginx decoy)
#
# Узел и VPN — два независимых systemd-сервиса. Можно остановить любой,
# второй продолжит работать. Конфигурация каждого описана в своём README.
#
# Использование:
#   sudo bash scripts/install-vps-full.sh
#
# Опции через env vars (опционально):
#   DECOY_HOST=www.cloudflare.com    — dest SNI для Reality (default googletagmanager)
#   CLIENT_EMAIL=alice               — email-метка клиента в xray
#   SKIP_NODE=1                      — пропустить установку узла (только VPN)
#   SKIP_VPN=1                       — пропустить установку VPN (только узел)

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
die() { printf '\033[1;31m[install-vps-full] ОШИБКА:\033[0m %s\n' "$*" >&2; exit 1; }

[ "$(id -u)" = "0" ] || die "требуется sudo/root"

[ -f "$INSTALL_VPS" ] || die "не найден $INSTALL_VPS"
[ -f "$INSTALL_VPN" ] || die "не найден $INSTALL_VPN"

if [ "${SKIP_NODE:-0}" != "1" ]; then
  log ""
  log "================================================================"
  log "  ШАГ 1/2 — установка узла Montana"
  log "================================================================"
  log ""
  bash "$INSTALL_VPS"
else
  log "SKIP_NODE=1 — пропускаю установку узла"
fi

if [ "${SKIP_VPN:-0}" != "1" ]; then
  log ""
  log "================================================================"
  log "  ШАГ 2/2 — установка VPN endpoint"
  log "================================================================"
  log ""
  bash "$INSTALL_VPN"
else
  log "SKIP_VPN=1 — пропускаю установку VPN"
fi

log ""
log "================================================================"
log "  ВСЁ ГОТОВО"
log "================================================================"
log ""
log "Узел Montana:  systemctl status montana-node"
log "VPN endpoint:  systemctl status xray"
log "decoy nginx:   systemctl status nginx"
log ""
log "Логи узла:     journalctl -u montana-node -f"
log "Логи VPN:      journalctl -u xray -f"
log ""
log "VLESS URL для клиента — выведен выше в шаге 2."
log "Бэкап мнемоники узла — выведен в шаге 1, сохрани в надёжное место."
