#!/bin/bash
# Montana — one-command Docker install on a clean Linux VPS.
#
# Result: a Helsinki-equivalent VPN-backend Montana node, fully containerised.
#   - montana-node    container on host network, p2p :8444
#   - xray            container on host network, Reality VLESS-XTLS-Vision :443
#   - nginx-decoy     container on host network, plain HTTP :80
#   - ufw             host firewall opening 22/80/443/8444
#
# Usage on a clean VPS (root):
#   curl -fsSL https://raw.githubusercontent.com/efir369999/Montana/main/Code/scripts/install-docker.sh \
#     | sudo bash
#
# Optional pre-staged secrets (place BEFORE running the installer):
#   /etc/montana-vpn/privkey    — Reality x25519 private key. If present and
#                                  MONTANA_VPN_MODE=universal is set in env,
#                                  the node joins the universal-key Montana
#                                  VPN federation (same client UUID/PBK/SID
#                                  as Helsinki/Frankfurt/US). Without this
#                                  file, fresh keys are generated.
#
# Optional environment variables:
#   MONTANA_VPN_MODE=universal   — use universal Montana VPN keys (needs the
#                                   privkey file above). Default: generate fresh.
#   MONTANA_DECOY_HOST=www.googletagmanager.com   Reality dest SNI.
#   MONTANA_CLIENT_EMAIL=montana-universal        xray client email tag.
#   MONTANA_NODE_TAG=$(hostname)                  inbound tag suffix.
#   MONTANA_REPO_URL=https://github.com/efir369999/Montana.git
#   MONTANA_REPO_BRANCH=main
#   MONTANA_WIPE_LEGACY=1        Also remove any prior native systemd install.

set -euo pipefail

REPO_URL="${MONTANA_REPO_URL:-https://github.com/efir369999/Montana.git}"
REPO_BRANCH="${MONTANA_REPO_BRANCH:-main}"
INSTALL_DIR="/opt/montana"
RUNTIME_DIR="$INSTALL_DIR/Code/docker/runtime"
VPN_DIR="/etc/montana-vpn"
VPN_PRIVKEY_FILE="$VPN_DIR/privkey"
XRAY_CONF="$VPN_DIR/xray-config.json"
NGX_CONF="$VPN_DIR/nginx-decoy.conf"
DECOY_HTML="$VPN_DIR/decoy-index.html"

VPN_MODE="${MONTANA_VPN_MODE:-fresh}"
DECOY_HOST="${MONTANA_DECOY_HOST:-www.googletagmanager.com}"
CLIENT_EMAIL="${MONTANA_CLIENT_EMAIL:-montana-universal}"
NODE_TAG="${MONTANA_NODE_TAG:-$(hostname -s 2>/dev/null || echo node)}"
WIPE_LEGACY="${MONTANA_WIPE_LEGACY:-1}"

# Universal Montana VPN client metadata — public, distributed in VLESS subscriptions.
# Matches Helsinki / Frankfurt / US backends.
UNIVERSAL_UUID="e6d355e2-2d79-4c96-a373-3b0e6b6f4b0d"
UNIVERSAL_SID="302805bc0c25e504"

log()  { printf '\033[1;32m[install-docker]\033[0m %s\n' "$*"; }
warn() { printf '\033[1;33m[install-docker]\033[0m %s\n' "$*" >&2; }
die()  { printf '\033[1;31m[install-docker] ERROR:\033[0m %s\n' "$*" >&2; exit 1; }

retry() {
  local n="$1"; shift
  local i=1
  while [ "$i" -le "$n" ]; do
    if "$@"; then return 0; fi
    warn "attempt $i/$n failed for: $*"
    i=$((i+1)); sleep $((i*3))
  done
  return 1
}

# ── 1. preconditions ─────────────────────────────────────────────────────────
[ "$(id -u)" = "0" ] || die "root privileges required"
[ -f /etc/os-release ] || die "/etc/os-release missing"
. /etc/os-release
OS_ID="${ID:-unknown}"
case "$OS_ID" in
  ubuntu|debian) ;;
  *) die "unsupported OS: $OS_ID. Supported: ubuntu, debian" ;;
esac
log "OS: ${PRETTY_NAME:-$OS_ID}"

# ── 2. wipe prior native systemd install if any ──────────────────────────────
if [ "$WIPE_LEGACY" = "1" ]; then
  log "wiping any prior native systemd install of montana-node / xray / nginx..."
  systemctl stop montana-node xray nginx 2>/dev/null || true
  systemctl disable montana-node xray nginx 2>/dev/null || true
  rm -f /etc/systemd/system/montana-node.service \
        /etc/systemd/system/xray.service /etc/systemd/system/xray@.service
  rm -rf /etc/systemd/system/montana-node.service.d \
         /etc/systemd/system/xray.service.d /etc/systemd/system/xray@.service.d
  systemctl daemon-reload || true
  systemctl reset-failed 2>/dev/null || true

  # Native xray uninstall (idempotent)
  if [ -x /usr/local/bin/xray ] || [ -f /usr/local/etc/xray/config.json ]; then
    retry 2 bash -c "curl -fsSL https://github.com/XTLS/Xray-install/raw/main/install-release.sh \
      -o /tmp/xray-uninst.sh && bash /tmp/xray-uninst.sh remove --purge" || true
    rm -f /usr/local/bin/xray /usr/local/bin/xctl
    rm -rf /usr/local/etc/xray /usr/local/share/xray /var/log/xray
  fi

  # Native nginx uninstall (keep package only if it's still serving non-decoy content)
  if dpkg -l 2>/dev/null | grep -qE '^ii  nginx'; then
    apt-get remove --purge -y nginx nginx-core nginx-common nginx-full 2>&1 | tail -2 || true
    apt-get autoremove -y 2>&1 | tail -1 || true
  fi
  rm -rf /etc/nginx /var/www/decoy

  # Montana data (keeps backup copy under /root)
  if [ -d /var/lib/montana ]; then
    BK="/root/montana-pre-docker-backup-$(date +%s)"
    mkdir -p "$BK"
    cp -a /var/lib/montana "$BK/" 2>/dev/null || true
    cp -a /etc/montana "$BK/" 2>/dev/null || true
    log "backed up legacy data to $BK"
  fi
  rm -rf /var/lib/montana /etc/montana
  id montana >/dev/null 2>&1 && userdel -r montana 2>/dev/null || true
  rm -f /usr/local/bin/montana-node

  log "legacy wipe complete"
fi

# ── 3. apt deps + docker + compose plugin ────────────────────────────────────
log "installing apt deps (curl, ca-certificates, gnupg, openssl, jq, ufw, git)..."
export DEBIAN_FRONTEND=noninteractive
retry 3 apt-get update -qq
retry 3 apt-get install -y -qq curl ca-certificates gnupg openssl jq ufw git lsb-release >/dev/null

if ! command -v docker >/dev/null 2>&1; then
  log "installing Docker Engine + compose plugin via official apt repo..."
  install -m 0755 -d /etc/apt/keyrings
  retry 3 bash -c "curl -fsSL https://download.docker.com/linux/${OS_ID}/gpg \
    | gpg --dearmor -o /etc/apt/keyrings/docker.gpg"
  chmod a+r /etc/apt/keyrings/docker.gpg
  CODENAME="$(. /etc/os-release && echo "${VERSION_CODENAME:-bookworm}")"
  echo "deb [arch=$(dpkg --print-architecture) signed-by=/etc/apt/keyrings/docker.gpg] \
    https://download.docker.com/linux/${OS_ID} ${CODENAME} stable" > /etc/apt/sources.list.d/docker.list
  retry 3 apt-get update -qq
  retry 3 apt-get install -y -qq docker-ce docker-ce-cli containerd.io \
    docker-buildx-plugin docker-compose-plugin >/dev/null
fi
systemctl enable --now docker >/dev/null 2>&1 || true
docker --version
docker compose version

# ── 4. firewall ──────────────────────────────────────────────────────────────
log "configuring ufw (22, 80, 443, 8444)..."
ufw allow 22/tcp     comment 'SSH'                          >/dev/null 2>&1 || true
ufw allow 80/tcp     comment 'decoy nginx'                  >/dev/null 2>&1 || true
ufw allow 443/tcp    comment 'VLESS+TCP+Reality+Vision'     >/dev/null 2>&1 || true
ufw allow 8444/tcp   comment 'Montana p2p Noise_PQ XX'      >/dev/null 2>&1 || true
yes | ufw --force enable >/dev/null 2>&1 || true

# ── 5. clone repo ────────────────────────────────────────────────────────────
if [ -d "$INSTALL_DIR/.git" ]; then
  log "updating repo at $INSTALL_DIR..."
  (cd "$INSTALL_DIR" && retry 3 git fetch --depth 1 origin "$REPO_BRANCH" && \
    git reset --hard "origin/$REPO_BRANCH")
else
  log "cloning $REPO_URL → $INSTALL_DIR..."
  retry 3 git clone --branch "$REPO_BRANCH" --depth 1 "$REPO_URL" "$INSTALL_DIR"
fi
[ -d "$RUNTIME_DIR" ] || die "expected $RUNTIME_DIR not found — repo layout drifted"

# ── 6. xray config — Helsinki-style universal or fresh keys ─────────────────
mkdir -p "$VPN_DIR" && chmod 0700 "$VPN_DIR"
install -m 0644 "$RUNTIME_DIR/nginx-decoy.conf"   "$NGX_CONF"
install -m 0644 "$RUNTIME_DIR/decoy-index.html"   "$DECOY_HTML"

if [ "$VPN_MODE" = "universal" ]; then
  [ -s "$VPN_PRIVKEY_FILE" ] || die "MONTANA_VPN_MODE=universal but $VPN_PRIVKEY_FILE missing/empty.
Stage it first:  scp helsinki-privkey root@<host>:$VPN_PRIVKEY_FILE && chmod 600 $VPN_PRIVKEY_FILE"
  PRIV="$(tr -d ' \n\r' < "$VPN_PRIVKEY_FILE")"
  UUID="$UNIVERSAL_UUID"
  SID="$UNIVERSAL_SID"
  log "VPN mode: universal (Helsinki-equivalent universal Montana key)"
else
  log "VPN mode: fresh keys (standalone Reality endpoint, not in Montana federation)"
  # One-shot xray container call to generate x25519 keypair.
  KEYS="$(docker run --rm teddysun/xray:latest xray x25519 2>&1 || true)"
  PRIV="$(echo "$KEYS" | awk -F': ' '/Private[ _]key:|PrivateKey:/ {print $NF; exit}' | tr -d ' \r')"
  PBK_FRESH="$(echo "$KEYS" | awk -F': ' '/Public[ _]key:|Password.*PublicKey/ {print $NF; exit}' | tr -d ' \r')"
  [ -n "$PRIV" ] && [ -n "$PBK_FRESH" ] || die "failed to derive fresh x25519 keypair from xray container"
  UUID="$(cat /proc/sys/kernel/random/uuid)"
  SID="$(openssl rand -hex 8)"
  install -m 0600 /dev/stdin "$VPN_PRIVKEY_FILE" <<<"$PRIV"
fi

# Derive PBK from PRIV via xray container (works in both modes for consistent output).
PBK="$(docker run --rm teddysun/xray:latest xray x25519 -i "$PRIV" 2>&1 \
  | awk -F': ' '/Public[ _]key:|Password.*PublicKey/ {print $NF; exit}' | tr -d ' \r')"
[ -n "$PBK" ] || die "failed to derive PublicKey from PrivateKey"

sed \
  -e "s|{{CLIENT_UUID}}|$UUID|g" \
  -e "s|{{CLIENT_EMAIL}}|$CLIENT_EMAIL|g" \
  -e "s|{{NODE_TAG}}|$NODE_TAG|g" \
  -e "s|{{DECOY_HOST}}|$DECOY_HOST|g" \
  -e "s|{{REALITY_PRIVATE_KEY}}|$PRIV|g" \
  -e "s|{{REALITY_SHORT_ID}}|$SID|g" \
  "$RUNTIME_DIR/xray-config.json.template" > "$XRAY_CONF"
chmod 0640 "$XRAY_CONF"

# ── 7. compose up (build + start) ────────────────────────────────────────────
cd "$RUNTIME_DIR"
log "building montana-node image and bringing the stack up (build is 10-30 min on small VPS)..."
docker compose down --remove-orphans >/dev/null 2>&1 || true
docker compose up -d --build 2>&1 | tail -20

# ── 8. wait for node identity to appear ──────────────────────────────────────
log "waiting up to 5 min for montana-node to write identity.bin..."
i=0
while [ "$i" -lt 60 ]; do
  if docker exec montana-node test -f /var/lib/montana/identity.bin 2>/dev/null; then
    break
  fi
  i=$((i+1)); sleep 5
done
docker exec montana-node test -f /var/lib/montana/identity.bin \
  || die "identity.bin not created after 5 min — inspect: docker logs montana-node"

# ── 9. report ────────────────────────────────────────────────────────────────
PUBLIC_IP="$(curl -fs --max-time 8 https://api.ipify.org || echo '<host-ip>')"
log ""
log "================================================================"
log "  INSTALL COMPLETE"
log "================================================================"
log ""
log "Containers:"
docker compose ps --format 'table {{.Name}}\t{{.Status}}\t{{.Ports}}' 2>/dev/null || docker compose ps
log ""
log "Montana node identity:"
docker exec montana-node cat /var/lib/montana/mnemonic.txt 2>/dev/null | tail -40 || \
  warn "mnemonic.txt not yet flushed — run: docker logs montana-node"
log ""
log "VPN client subscription (VLESS Reality):"
echo "vless://${UUID}@${PUBLIC_IP}:443?encryption=none&flow=xtls-rprx-vision&security=reality&sni=${DECOY_HOST}&fp=chrome&pbk=${PBK}&sid=${SID}&type=tcp#montana-${NODE_TAG}"
log ""
log "Useful commands:"
log "  docker compose -f $RUNTIME_DIR/docker-compose.yml ps"
log "  docker logs -f montana-node"
log "  docker exec montana-node /usr/local/bin/montana-node status --data-dir /var/lib/montana"
log "  docker logs montana-xray"
log "  docker compose -f $RUNTIME_DIR/docker-compose.yml down       # stop"
log "  docker compose -f $RUNTIME_DIR/docker-compose.yml down -v    # stop + wipe identity"
log ""
log "Node lifecycle:"
log "  Phase 1: CandidateVdf — sequential SHA-256 chain to vdf_chain_length >= τ₂"
log "  Phase 2: Registered   — NodeRegistration via apply_proposal on next selection window"
log "  Phase 3: Active       — emission 13 Ɉ per window via apply_proposal"
