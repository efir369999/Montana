#!/bin/bash
# Montana VPN — установщик xray Reality endpoint на Linux VPS.
#
# Что делает:
#   1. Ставит system deps (xray, nginx, ufw, curl, jq)
#   2. Генерирует свежие Reality keys + UUID + shortId на самом хосте
#   3. Подставляет их в xray-config.json.template → /usr/local/etc/xray/config.json
#   4. Поднимает nginx :80 с decoy-страницей (камуфляж от пассивных проверяющих)
#   5. Прописывает systemd unit + drop-in для xray
#   6. Открывает 22/80/443 в ufw, остальное закрывает
#   7. Включает BBR + fq_codel через sysctl
#   8. Запускает xray, печатает VLESS URL клиенту
#
# Использование:
#   sudo bash install.sh                        # с настройками по умолчанию
#   sudo DECOY_HOST=www.googletagmanager.com bash install.sh
#   sudo CLIENT_EMAIL=alice@montana bash install.sh
#
# Идемпотентен: повторный запуск не ломает настройки и не пересоздаёт ключи
# (они в /etc/montana-vpn/state.env, удалить файл = пересоздать).

set -euo pipefail

DECOY_HOST="${DECOY_HOST:-www.googletagmanager.com}"
CLIENT_EMAIL="${CLIENT_EMAIL:-montana-client}"
STATE_DIR="/etc/montana-vpn"
STATE_FILE="$STATE_DIR/state.env"
XRAY_CONF_DIR="/usr/local/etc/xray"
XRAY_CONF="$XRAY_CONF_DIR/config.json"
XRAY_LOG_DIR="/var/log/xray"
DECOY_ROOT="/var/www/decoy"
NGINX_SITE="/etc/nginx/sites-available/decoy"
SYSCTL_FILE="/etc/sysctl.d/99-montana-vpn.conf"

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
TEMPLATE_DIR="$SCRIPT_DIR/config-template"

log() { printf '\033[1;32m[montana-vpn]\033[0m %s\n' "$*"; }
warn() { printf '\033[1;33m[montana-vpn]\033[0m %s\n' "$*" >&2; }
die() { printf '\033[1;31m[montana-vpn] ОШИБКА:\033[0m %s\n' "$*" >&2; exit 1; }

[ "$(id -u)" = "0" ] || die "требуется sudo/root"
[ -f /etc/os-release ] || die "/etc/os-release отсутствует"
. /etc/os-release
OS_ID="${ID:-unknown}"
log "OS: ${PRETTY_NAME:-$OS_ID}"

[ -d "$TEMPLATE_DIR" ] || die "config-template/ не найден рядом со скриптом ($TEMPLATE_DIR)"

log "ставлю system deps..."
case "$OS_ID" in
  ubuntu|debian)
    export DEBIAN_FRONTEND=noninteractive
    apt-get update -qq
    apt-get install -y -qq nginx ufw curl jq ca-certificates >/dev/null
    ;;
  fedora|rhel|centos|rocky|almalinux)
    dnf install -y -q nginx firewalld curl jq ca-certificates >/dev/null
    ;;
  alpine)
    apk add --no-cache nginx curl jq ca-certificates >/dev/null
    ;;
  *)
    die "неподдерживаемый OS: $OS_ID"
    ;;
esac

if ! command -v xray >/dev/null 2>&1; then
  log "ставлю xray (официальный installer)..."
  bash -c "$(curl -L https://github.com/XTLS/Xray-install/raw/main/install-release.sh)" @ install >/dev/null
else
  log "xray уже установлен: $(xray version 2>&1 | head -1)"
fi

mkdir -p "$STATE_DIR" "$XRAY_CONF_DIR" "$XRAY_LOG_DIR" "$DECOY_ROOT"
chmod 0700 "$STATE_DIR"

if [ -f "$STATE_FILE" ]; then
  log "переиспользую ключи из $STATE_FILE (удалите файл чтобы перегенерить)"
  # shellcheck disable=SC1090
  . "$STATE_FILE"
else
  log "генерирую свежие Reality keys + UUID + shortId..."
  KEYS=$(xray x25519)
  REALITY_PRIVATE_KEY=$(echo "$KEYS" | awk '/Private key:|PrivateKey:/ {print $NF}')
  REALITY_PUBLIC_KEY=$(echo "$KEYS"  | awk '/Public key:|Password:/ {print $NF}')
  CLIENT_UUID=$(xray uuid)
  REALITY_SHORT_ID=$(openssl rand -hex 8)
  cat > "$STATE_FILE" <<STATE
DECOY_HOST="$DECOY_HOST"
CLIENT_EMAIL="$CLIENT_EMAIL"
CLIENT_UUID="$CLIENT_UUID"
REALITY_PRIVATE_KEY="$REALITY_PRIVATE_KEY"
REALITY_PUBLIC_KEY="$REALITY_PUBLIC_KEY"
REALITY_SHORT_ID="$REALITY_SHORT_ID"
STATE
  chmod 0600 "$STATE_FILE"
fi

log "генерирую xray config.json из шаблона..."
sed \
  -e "s|{{CLIENT_UUID}}|$CLIENT_UUID|g" \
  -e "s|{{CLIENT_EMAIL}}|$CLIENT_EMAIL|g" \
  -e "s|{{DECOY_HOST}}|$DECOY_HOST|g" \
  -e "s|{{REALITY_PRIVATE_KEY}}|$REALITY_PRIVATE_KEY|g" \
  -e "s|{{REALITY_SHORT_ID}}|$REALITY_SHORT_ID|g" \
  "$TEMPLATE_DIR/xray-config.json.template" > "$XRAY_CONF"
chmod 0644 "$XRAY_CONF"
chown -R nobody:nogroup "$XRAY_LOG_DIR" 2>/dev/null || chown -R nobody:nobody "$XRAY_LOG_DIR" 2>/dev/null || true

log "ставлю xray.service + drop-in..."
install -m 0644 "$TEMPLATE_DIR/xray.service" /etc/systemd/system/xray.service
install -d -m 0755 /etc/systemd/system/xray.service.d
install -m 0644 "$TEMPLATE_DIR/xray.service.d/10-donot_touch_single_conf.conf" /etc/systemd/system/xray.service.d/10-donot_touch_single_conf.conf

log "поднимаю nginx :80 decoy..."
install -m 0644 "$TEMPLATE_DIR/decoy-index.html" "$DECOY_ROOT/index.html"
install -m 0644 "$TEMPLATE_DIR/nginx-decoy.conf" "$NGINX_SITE"
ln -sf "$NGINX_SITE" /etc/nginx/sites-enabled/decoy
rm -f /etc/nginx/sites-enabled/default
nginx -t >/dev/null 2>&1 || die "nginx config невалиден"
systemctl enable nginx >/dev/null 2>&1 || true
systemctl restart nginx

log "включаю BBR + fq_codel через sysctl..."
install -m 0644 "$TEMPLATE_DIR/sysctl-bbr.conf" "$SYSCTL_FILE"
sysctl -p "$SYSCTL_FILE" >/dev/null 2>&1 || true

if command -v ufw >/dev/null 2>&1; then
  log "настраиваю ufw (22, 80, 443)..."
  ufw allow 22/tcp comment 'SSH' >/dev/null 2>&1 || true
  ufw allow 80/tcp comment 'decoy nginx' >/dev/null 2>&1 || true
  ufw allow 443/tcp comment 'VLESS+TCP+Reality' >/dev/null 2>&1 || true
  ufw --force enable >/dev/null 2>&1 || true
fi

log "запускаю xray..."
systemctl daemon-reload
systemctl enable xray.service >/dev/null 2>&1
systemctl restart xray.service
sleep 2

PUBLIC_IP=$(curl -s4 https://api.ipify.org 2>/dev/null || hostname -I | awk '{print $1}')

log ""
log "================================================================"
log "  УСТАНОВКА VPN ЗАВЕРШЕНА"
log "================================================================"
log ""
log "Server:        $PUBLIC_IP:443"
log "Decoy SNI:     $DECOY_HOST"
log "Client UUID:   $CLIENT_UUID"
log "Reality PK:    $REALITY_PUBLIC_KEY"
log "Reality SID:   $REALITY_SHORT_ID"
log "Flow:          xtls-rprx-vision"
log ""
log "VLESS URL для клиента (импортировать в v2rayN/Hiddify/Streisand):"
log ""
echo "vless://${CLIENT_UUID}@${PUBLIC_IP}:443?encryption=none&flow=xtls-rprx-vision&security=reality&sni=${DECOY_HOST}&fp=chrome&pbk=${REALITY_PUBLIC_KEY}&sid=${REALITY_SHORT_ID}&type=tcp#montana-vpn"
log ""
log "Полные ключи и состояние: $STATE_FILE (mode 0600)"
log ""
log "Управление:"
log "  systemctl status xray              # статус VPN"
log "  systemctl restart xray             # перезапуск"
log "  journalctl -u xray -f              # логи"
log ""
log "Узел Montana — отдельный слой; устанавливается scripts/install-vps.sh"
log "(или scripts/install-vps-full.sh — узел + VPN одной командой)."
