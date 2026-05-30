#!/bin/bash
# Montana ONE-COMMAND install — full stack on a clean Linux VPS.
#
# Brings up:
#   - montana-node     TimeChain consensus, P2P :8444 (Noise_PQ XX)
#   - xray             Xray Reality VLESS on :443 (TLS-fronted as googletagmanager.com)
#   - nginx-decoy      decoy landing page on :80
#
# All three containers run via docker compose, share host network.
# Pre-built montana-node image pulled from ghcr.io/efir369999/montana-node:latest.
#
# Usage (clean Linux VPS, root):
#   curl -sSL https://raw.githubusercontent.com/efir369999/Montana/main/install.sh | sudo bash
#
# After completion the script prints:
#   - 24-word mnemonic (only backup of node identity)
#   - VLESS URL (paste into v2rayN / Hiddify / Sing-box / Streisand client)

set -euo pipefail

log()  { printf '\033[1;32m[install]\033[0m %s\n' "$*"; }
warn() { printf '\033[1;33m[install]\033[0m %s\n' "$*" >&2; }
die()  { printf '\033[1;31m[install] ERROR:\033[0m %s\n' "$*" >&2; exit 1; }

[ "$(id -u)" = "0" ] || die "root required: sudo bash $0"

# ── 1. Docker install (skip if present) ─────────────────────────────────────
if ! command -v docker >/dev/null 2>&1; then
  log "installing docker (apt path)"
  apt-get update -qq
  apt-get install -y -qq ca-certificates curl gnupg
  install -m 0755 -d /etc/apt/keyrings
  curl -fsSL https://download.docker.com/linux/debian/gpg | gpg --dearmor -o /etc/apt/keyrings/docker.gpg
  chmod a+r /etc/apt/keyrings/docker.gpg
  CODENAME=$(. /etc/os-release && echo "$VERSION_CODENAME")
  echo "deb [arch=$(dpkg --print-architecture) signed-by=/etc/apt/keyrings/docker.gpg] https://download.docker.com/linux/debian $CODENAME stable" \
    > /etc/apt/sources.list.d/docker.list
  apt-get update -qq
  apt-get install -y -qq docker-ce docker-ce-cli containerd.io docker-buildx-plugin docker-compose-plugin
  systemctl enable --now docker
else
  log "docker already present: $(docker --version)"
fi

# ── 2. Pull pre-built montana-node image from GHCR ──────────────────────────
MONTANA_IMG="ghcr.io/efir369999/montana-node:latest"
log "pulling $MONTANA_IMG"
docker pull "$MONTANA_IMG" >/dev/null

# ── 3. Generate Reality keys + UUID + SID ───────────────────────────────────
INSTALL_DIR="/opt/montana"
mkdir -p "$INSTALL_DIR" /etc/montana-vpn /var/log/xray

log "generating Reality x25519 keypair via xray"
XRAY_KEYS=$(docker run --rm teddysun/xray:26.2.6 xray x25519 2>/dev/null)
PRIVATE_KEY=$(echo "$XRAY_KEYS" | awk '/Private key:/ {print $3}')
PUBLIC_KEY=$(echo "$XRAY_KEYS"  | awk '/Public key:/  {print $3}')
[ -n "$PRIVATE_KEY" ] && [ -n "$PUBLIC_KEY" ] || die "xray keypair generation failed"

UUID=$(docker run --rm teddysun/xray:26.2.6 xray uuid 2>/dev/null)
SID=$(head -c 8 /dev/urandom | xxd -p)
DECOY_HOST="${DECOY_HOST:-www.googletagmanager.com}"
CLIENT_EMAIL="${CLIENT_EMAIL:-operator}"
LISTEN_IP=$(hostname -I | awk '{print $1}')

cat > /etc/montana-vpn/xray-config.json <<XRAY_CFG
{
  "log": {"loglevel": "warning", "access": "/var/log/xray/access.log", "error": "/var/log/xray/error.log"},
  "dns": {"servers": ["1.1.1.1", "8.8.8.8"], "queryStrategy": "UseIP"},
  "inbounds": [{
    "listen": "0.0.0.0", "port": 443, "protocol": "vless",
    "settings": {"clients": [{"id": "$UUID", "email": "$CLIENT_EMAIL", "flow": "xtls-rprx-vision"}], "decryption": "none"},
    "streamSettings": {
      "network": "tcp", "security": "reality",
      "realitySettings": {
        "show": false,
        "dest": "$DECOY_HOST:443",
        "xver": 0,
        "serverNames": ["$DECOY_HOST"],
        "privateKey": "$PRIVATE_KEY",
        "shortIds": ["$SID"]
      }
    },
    "sniffing": {"enabled": true, "destOverride": ["http", "tls", "quic"], "routeOnly": false}
  }],
  "outbounds": [{"protocol": "freedom", "tag": "direct"}]
}
XRAY_CFG

# ── 4. Compose stack ────────────────────────────────────────────────────────
cat > $INSTALL_DIR/docker-compose.yml <<COMPOSE
services:
  montana-node:
    image: $MONTANA_IMG
    container_name: montana-node
    restart: unless-stopped
    network_mode: host
    volumes:
      - montana-data:/var/lib/montana
    healthcheck:
      test: ["CMD-SHELL", "test -f /var/lib/montana/identity.bin"]
      interval: 30s
      timeout: 5s
      retries: 5

  xray:
    image: teddysun/xray:26.2.6
    container_name: montana-xray
    restart: unless-stopped
    network_mode: host
    volumes:
      - /etc/montana-vpn/xray-config.json:/etc/xray/config.json:ro
      - /var/log/xray:/var/log/xray
    depends_on:
      - nginx-decoy

  nginx-decoy:
    image: nginx:alpine
    container_name: montana-nginx-decoy
    restart: unless-stopped
    network_mode: host
    command: ["nginx", "-g", "daemon off;"]

volumes:
  montana-data:
COMPOSE

# ── 5. Bring up the stack ───────────────────────────────────────────────────
log "bringing up the stack"
cd "$INSTALL_DIR"
docker compose up -d
sleep 6

# ── 6. Output mnemonic + VLESS URL ──────────────────────────────────────────
log ""
log "================================================================"
log "  Montana node + VPN installed."
log "================================================================"
log ""
log "Node identity (24-word mnemonic — save offline, only backup):"
echo
docker exec montana-node cat /var/lib/montana/mnemonic.txt 2>/dev/null || warn "(mnemonic file not yet written; retry: docker exec montana-node cat /var/lib/montana/mnemonic.txt)"
echo
log "Node peer_id (visible in mesh /api/peers):"
docker exec montana-node /usr/local/bin/montana-node inspect --data-dir /var/lib/montana 2>/dev/null | grep libp2p_peer_id || true
log ""
log "VPN VLESS URL (paste into v2rayN / Hiddify / Sing-box / Streisand):"
echo
echo "vless://$UUID@$LISTEN_IP:443?encryption=none&flow=xtls-rprx-vision&security=reality&sni=$DECOY_HOST&fp=chrome&pbk=$PUBLIC_KEY&sid=$SID&type=tcp#montana-vpn"
echo
log "Verification:"
log "  docker ps                              # 3 containers up"
log "  docker exec montana-node /usr/local/bin/montana-node status --data-dir /var/lib/montana"
log "  curl -sk https://efir.org:8443/montana-api/peers | grep -o '$(docker exec montana-node /usr/local/bin/montana-node inspect --data-dir /var/lib/montana 2>/dev/null | awk '/libp2p_peer_id/ {print $3}')'   # should print your peer_id once Moscow sees you"
log ""
