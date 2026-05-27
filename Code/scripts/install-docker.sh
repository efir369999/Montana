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
# Pre-stageable secrets (optional — operator-supplied via scp BEFORE running):
#   /etc/montana-vpn/privkey      Reality x25519 private key. Presence => the
#                                  node joins the universal-key Montana VPN
#                                  federation (same UUID/PBK/SID/SNI as
#                                  Helsinki/Frankfurt/US). Absence => generate
#                                  fresh standalone keys.
#   /etc/montana-vpn/orch-token   Orchestrator admin secret. Presence => after
#                                  compose-up the installer POSTs /vpn/node/
#                                  register so this node appears in the public
#                                  https://montana.quest/vpn/sub subscription.
#
# Operator metadata (used by orchestrator register payload; sensible defaults
# from the VPS itself when omitted):
# Country, city and coordinates are auto-detected from the node's public IP
# (ip-api.com) when the variables below are omitted; override only if needed:
#   MONTANA_ALIAS=<hostname-short>            short lowercase alias
#   MONTANA_CITY=<city name>                  auto from geo-IP; e.g. Yerevan
#   MONTANA_LABEL=<human label>               auto from city; any UTF-8
#   MONTANA_COUNTRY=<two-letter ISO>          auto from geo-IP; e.g. AM, FI, DE
#   MONTANA_HOSTING=<provider name>           e.g. WorkTitans
#   MONTANA_COORDS='lat,lon'                  auto from geo-IP; e.g. 40.18,44.51
#
# Other environment knobs:
#   MONTANA_DECOY_HOST=www.googletagmanager.com   Reality dest SNI
#   MONTANA_CLIENT_EMAIL=montana-universal        xray client email tag
#   MONTANA_NODE_TAG=$(hostname)                  inbound tag suffix
#   MONTANA_REPO_URL=https://github.com/efir369999/Montana.git
#   MONTANA_REPO_BRANCH=main
#   MONTANA_WIPE_LEGACY=1                         purge prior native systemd install
#   MONTANA_ORCH_URL=https://montana.quest/vpn/node
#   MONTANA_SKIP_VERIFY=0                         skip post-install self-checks

set -euo pipefail

# ── configuration ───────────────────────────────────────────────────────────
REPO_URL="${MONTANA_REPO_URL:-https://github.com/efir369999/Montana.git}"
REPO_BRANCH="${MONTANA_REPO_BRANCH:-main}"
INSTALL_DIR="/opt/montana"
RUNTIME_DIR="$INSTALL_DIR/Code/docker/runtime"
VPN_DIR="/etc/montana-vpn"
VPN_PRIVKEY_FILE="$VPN_DIR/privkey"
ORCH_TOKEN_FILE="$VPN_DIR/orch-token"
XRAY_CONF="$VPN_DIR/xray-config.json"
NGX_CONF="$VPN_DIR/nginx-decoy.conf"
DECOY_HTML="$VPN_DIR/decoy-index.html"

DECOY_HOST="${MONTANA_DECOY_HOST:-www.googletagmanager.com}"
CLIENT_EMAIL="${MONTANA_CLIENT_EMAIL:-montana-universal}"
NODE_TAG="${MONTANA_NODE_TAG:-$(hostname -s 2>/dev/null || echo node)}"
WIPE_LEGACY="${MONTANA_WIPE_LEGACY:-1}"
ORCH_URL="${MONTANA_ORCH_URL:-https://montana.quest/vpn/node}"
SKIP_VERIFY="${MONTANA_SKIP_VERIFY:-0}"

# Allow passing orch-token via env (alternative to pre-staging the file).
if [ -n "${MONTANA_ORCH_TOKEN:-}" ] && [ ! -s "$ORCH_TOKEN_FILE" ]; then
  mkdir -p "$VPN_DIR" && chmod 0700 "$VPN_DIR"
  install -m 0600 /dev/stdin "$ORCH_TOKEN_FILE" <<<"$MONTANA_ORCH_TOKEN"
  log "orch-token written from MONTANA_ORCH_TOKEN env"
fi

# Universal Montana VPN client metadata — public, distributed in VLESS subs.
UNIVERSAL_UUID="e6d355e2-2d79-4c96-a373-3b0e6b6f4b0d"
UNIVERSAL_SID="302805bc0c25e504"
UNIVERSAL_PRIVKEY="cL7D6FCqH5nWcQlHCKH9uNr-RNwCt5peRAqt8tl9mXs"

log()  { printf '\033[1;32m[install-docker]\033[0m %s\n' "$*"; }
warn() { printf '\033[1;33m[install-docker]\033[0m %s\n' "$*" >&2; }
die()  { printf '\033[1;31m[install-docker] ERROR:\033[0m %s\n' "$*" >&2; exit 1; }
ok()   { printf '\033[1;32m[verify ✓]\033[0m %s\n' "$*"; }
bad()  { printf '\033[1;31m[verify ✗]\033[0m %s\n' "$*" >&2; }

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
  systemctl daemon-reload 2>/dev/null || true
  systemctl reset-failed 2>/dev/null || true

  # Native xray uninstall — only if there's something to remove.
  if [ -x /usr/local/bin/xray ] && [ -f /etc/systemd/system/xray.service ]; then
    bash -c "$(curl -fsSL https://github.com/XTLS/Xray-install/raw/main/install-release.sh)" \
      @ remove --purge >/dev/null 2>&1 || true
  fi
  rm -f /usr/local/bin/xray /usr/local/bin/xctl
  rm -rf /usr/local/etc/xray /usr/local/share/xray /var/log/xray

  if dpkg -l 2>/dev/null | grep -qE '^ii  nginx'; then
    DEBIAN_FRONTEND=noninteractive apt-get remove --purge -y nginx nginx-core nginx-common nginx-full >/dev/null 2>&1 || true
    DEBIAN_FRONTEND=noninteractive apt-get autoremove -y >/dev/null 2>&1 || true
  fi
  rm -rf /etc/nginx /var/www/decoy

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

# ── 6. xray config — always universal (pre-staged or built-in federation key) ────────────────
mkdir -p "$VPN_DIR" && chmod 0700 "$VPN_DIR"
install -m 0644 "$RUNTIME_DIR/nginx-decoy.conf"   "$NGX_CONF"
install -m 0644 "$RUNTIME_DIR/decoy-index.html"   "$DECOY_HTML"

if [ -s "$VPN_PRIVKEY_FILE" ]; then
  VPN_MODE=universal
  PRIV="$(tr -d ' \n\r' < "$VPN_PRIVKEY_FILE")"
  UUID="$UNIVERSAL_UUID"
  SID="$UNIVERSAL_SID"
  log "VPN mode: universal (privkey pre-staged at $VPN_PRIVKEY_FILE)"
else
  VPN_MODE=universal
  log "VPN mode: universal (auto — no pre-staged privkey, using built-in federation key)"
  PRIV="$UNIVERSAL_PRIVKEY"
  UUID="$UNIVERSAL_UUID"
  SID="$UNIVERSAL_SID"
  install -m 0600 /dev/stdin "$VPN_PRIVKEY_FILE" <<<"$PRIV"
fi

PBK="$(docker run --rm teddysun/xray:26.2.6 xray x25519 -i "$PRIV" 2>&1 \
  | awk -F': ' '/Password|ublic/ {print $NF; exit}' | tr -d ' \r')"
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
log "building montana-node image (10-30 min on small VPS)..."
docker compose down --remove-orphans >/dev/null 2>&1 || true
BUILD_LOG=/var/log/montana-compose.log
: > "$BUILD_LOG"
( BUILDKIT_PROGRESS=plain docker compose build >"$BUILD_LOG" 2>&1; echo $? >/tmp/.mt_build_rc ) &
BPID=$!
BSTART=$(date +%s); BEST=380; SPIN='|/-\'; SI=0
if [ -t 1 ]; then
  while kill -0 "$BPID" 2>/dev/null; do
    N=$(grep -c 'Compiling ' "$BUILD_LOG" 2>/dev/null) || N=0
    CUR=$(grep 'Compiling ' "$BUILD_LOG" 2>/dev/null | tail -1 | sed -E 's/.*Compiling ([^ ]+).*/\1/') || CUR=""
    EL=$(( $(date +%s) - BSTART ))
    PCT=$(( N * 100 / BEST )); if [ "$PCT" -gt 99 ]; then PCT=99; fi
    FILLED=$(( PCT * 28 / 100 ))
    BAR="$(printf '%*s' "$FILLED" '' | tr ' ' '#')$(printf '%*s' $((28 - FILLED)) '' | tr ' ' '.')"
    printf '\r\033[1;32m[build]\033[0m [%s] %3d%% %3dc %5ds %c %-22.22s' "$BAR" "$PCT" "$N" "$EL" "${SPIN:$SI:1}" "${CUR:-preparing}"
    SI=$(( (SI + 1) % 4 )); sleep 2
  done
  printf '\r\033[K'
fi
wait "$BPID" 2>/dev/null || true
BRC=$(cat /tmp/.mt_build_rc 2>/dev/null || echo 1); rm -f /tmp/.mt_build_rc
NC=$(grep -c 'Compiling ' "$BUILD_LOG" 2>/dev/null) || NC=0
[ "$BRC" = 0 ] || die "image build failed (rc=$BRC) — see $BUILD_LOG; tail: $(tail -20 "$BUILD_LOG")"
log "image built ($NC crates in $(( $(date +%s) - BSTART ))s). starting stack..."
docker compose up -d 2>&1 | tail -20

# ── 8. wait for identity ─────────────────────────────────────────────────────
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

# ── 9. orchestrator register (only when token + universal mode present) ─────
PUBLIC_IP="$(curl -fs --max-time 8 https://api.ipify.org || echo '')"
# geo-IP self-identification: derive country / city / coordinates from the
# node's public address so the operator never types a city name by hand.
GEO_CC=''; GEO_CITY=''; GEO_LAT=''; GEO_LON=''
if [ -n "$PUBLIC_IP" ]; then
  GEO_JSON="$(curl -fs --max-time 8 "http://ip-api.com/json/${PUBLIC_IP}?fields=status,countryCode,city,lat,lon&lang=ru" || echo '')"
  if [ -n "$GEO_JSON" ] && [ "$(printf '%s' "$GEO_JSON" | jq -r '.status' 2>/dev/null)" = "success" ]; then
    GEO_CC="$(printf '%s' "$GEO_JSON" | jq -r '.countryCode // empty' 2>/dev/null)"
    GEO_CITY="$(printf '%s' "$GEO_JSON" | jq -r '.city // empty' 2>/dev/null)"
    GEO_LAT="$(printf '%s' "$GEO_JSON" | jq -r '.lat // empty' 2>/dev/null)"
    GEO_LON="$(printf '%s' "$GEO_JSON" | jq -r '.lon // empty' 2>/dev/null)"
    log "geo-IP: country=$GEO_CC city=$GEO_CITY coords=$GEO_LAT,$GEO_LON"
  fi
fi
ORCH_RESP=''
if [ -s "$ORCH_TOKEN_FILE" ] && [ "$VPN_MODE" = "universal" ] && [ -n "$PUBLIC_IP" ]; then
  TOKEN="$(tr -d ' \n\r' < "$ORCH_TOKEN_FILE")"
  ALIAS="${MONTANA_ALIAS:-$NODE_TAG}"
  COUNTRY="${MONTANA_COUNTRY:-${GEO_CC:-XX}}"
  CITY="${MONTANA_CITY:-${GEO_CITY:-${ALIAS^}}}"
  LABEL="${MONTANA_LABEL:-$CITY}"
  HOSTING="${MONTANA_HOSTING:-unknown}"
  COORDS="${MONTANA_COORDS:-${GEO_LAT:-0},${GEO_LON:-0}}"
  LAT="$(echo "$COORDS" | cut -d, -f1)"
  LON="$(echo "$COORDS" | cut -d, -f2)"
  log "registering with orchestrator at $ORCH_URL/register (alias=$ALIAS, country=$COUNTRY)..."
  # Give xray a moment so the Reality probe by the orchestrator succeeds.
  sleep 4
  PAYLOAD=$(jq -nc \
    --arg alias "$ALIAS" --arg ip "$PUBLIC_IP" --arg country "$COUNTRY" \
    --arg city "$CITY" \
    --arg hosting "$HOSTING" --arg label "$LABEL" --argjson lat "$LAT" --argjson lon "$LON" \
    --arg pbk "$PBK" --arg uuid "$UUID" --arg sid "$SID" --arg secret "$TOKEN" \
    '{alias:$alias,ip:$ip,country:$country,city:$city,hosting:$hosting,label:$label,coords:[$lat,$lon],reality_pbk:$pbk,reality_uuid:$uuid,reality_sid:$sid,secret:$secret}')
  ORCH_RESP="$(curl -sk --max-time 20 -X POST -H 'Content-Type: application/json' -d "$PAYLOAD" "$ORCH_URL/register" || true)"
  log "orchestrator response: $ORCH_RESP"
  if command -v jq >/dev/null 2>&1 && [ -n "$ORCH_RESP" ]; then
    MR="$(printf '%s' "$ORCH_RESP" | jq -r '.moscow_reachable // empty' 2>/dev/null || true)"
    RTT="$(printf '%s' "$ORCH_RESP" | jq -r '.moscow_rtt_ms // empty' 2>/dev/null || true)"
    CEN="$(printf '%s' "$ORCH_RESP" | jq -r '.cascade.enabled // empty' 2>/dev/null || true)"
    CRE="$(printf '%s' "$ORCH_RESP" | jq -r '.cascade.reason // empty' 2>/dev/null || true)"
    if [ "$MR" = "true" ]; then log "Moscow cross-check: reachable (TCP :443, RTT ${RTT}ms)"; else log "Moscow cross-check: NOT reachable from Moscow datacenter"; fi
    CFRONTS="$(printf '%s' "$ORCH_RESP" | jq -r '(.cascade.fronts // []) | join(", ")' 2>/dev/null || true)"
    if [ "$CEN" = "true" ]; then log "Cascade: ENABLED via ${CFRONTS:-de.montana.quest} (reason: $CRE) — clients enter at any of 5 fronts, traffic exits on THIS node"; else log "Cascade: not needed — direct connection"; fi
  fi
fi

# ── 10. self-verification ───────────────────────────────────────────────────
if [ "$SKIP_VERIFY" != "1" ]; then
  log ""
  log "running post-install self-verification..."
  PASS=0; FAIL=0
  vcheck() { if eval "$1"; then ok "$2"; PASS=$((PASS+1)); else bad "$2"; FAIL=$((FAIL+1)); fi; }

  vcheck "docker ps --format '{{.Names}} {{.Status}}' | grep -q 'montana-node.*healthy\|montana-node.*Up'" \
         "container montana-node up"
  vcheck "docker ps --format '{{.Names}} {{.Status}}' | grep -q 'montana-xray.*Up'" \
         "container montana-xray up"
  vcheck "docker ps --format '{{.Names}} {{.Status}}' | grep -q 'montana-nginx-decoy.*Up'" \
         "container montana-nginx-decoy up"

  # outbound peer TCP probe — bootstrap peers from genesis manifest
  if [ -f "$INSTALL_DIR/Code/scripts/genesis-manifest.json" ]; then
    PEER_HOSTS="$(jq -r '.peers[] | .multiaddr' "$INSTALL_DIR/Code/scripts/genesis-manifest.json" \
      | sed -nE 's|/ip4/([0-9.]+)/tcp/([0-9]+)|\1 \2|p')"
    while read -r ph pp; do
      [ -z "$ph" ] && continue
      vcheck "timeout 5 bash -c '</dev/tcp/$ph/$pp' 2>/dev/null" \
             "peer TCP reachable: $ph:$pp"
    done <<<"$PEER_HOSTS"
  fi

  # local Reality TLS handshake to :443
  vcheck "echo Q | timeout 8 openssl s_client -connect 127.0.0.1:443 -servername '$DECOY_HOST' -brief 2>&1 | grep -q 'CONNECTION ESTABLISHED'" \
         "local TLS handshake :443 via Reality cover SNI"

  # decoy :80
  vcheck "curl -sf --max-time 8 -o /dev/null 'http://127.0.0.1/'" "decoy :80 returns 200"

  # ESTABLISHED peer connections from montana-node
  EST="$(ss -tnp 2>/dev/null | grep montana-node | wc -l)"
  vcheck "[ '$EST' -ge 1 ]" "at least 1 ESTABLISHED p2p connection (got $EST)"

  # /vpn/sub membership (universal mode only — fresh keys aren't aggregated)
  if [ "$VPN_MODE" = "universal" ] && [ -n "$PUBLIC_IP" ]; then
    SUB="$(curl -sk --max-time 10 "${ORCH_URL%/node}/sub" | base64 -d 2>/dev/null || true)"
    ALIAS_LOWER="$(echo "${MONTANA_ALIAS:-$NODE_TAG}" | tr '[:upper:]' '[:lower:]')"
    # match either the alias label or this server's public IP appearing in any VLESS URL
    vcheck "echo \"\$SUB\" | grep -qi -E '${ALIAS_LOWER}\\.montana\\.quest|${PUBLIC_IP//./\\.}'" \
           "node appears in https://montana.quest/vpn/sub subscription"
  fi

  log ""
  if [ "$FAIL" = "0" ]; then
    log "self-verification: $PASS/$PASS checks passed"
  else
    warn "self-verification: $PASS passed / $FAIL failed — review checks above"
  fi
fi

# ── 11. final report ────────────────────────────────────────────────────────
log ""
log "================================================================"
log "  INSTALL COMPLETE"
log "================================================================"
log ""
log "Containers:"
docker compose ps --format 'table {{.Name}}\t{{.Status}}' 2>/dev/null || docker compose ps
log ""
log "Montana node identity (24-word mnemonic — write it down NOW):"
echo "----------------------------------------------------------------"
docker exec montana-node cat /var/lib/montana/mnemonic.txt 2>/dev/null \
  || warn "mnemonic.txt not yet flushed — run: docker exec montana-node cat /var/lib/montana/mnemonic.txt"
echo "----------------------------------------------------------------"
log ""
log "VPN client subscription (VLESS Reality):"
echo "vless://${UUID}@${PUBLIC_IP:-<host-ip>}:443?encryption=none&flow=xtls-rprx-vision&security=reality&sni=${DECOY_HOST}&fp=chrome&pbk=${PBK}&sid=${SID}&type=tcp#montana-${NODE_TAG}"
log ""
if [ -n "$ORCH_RESP" ]; then
  log "Orchestrator: $ORCH_RESP"
  log "Public subscription (decoded):  curl -sk ${ORCH_URL%/node}/sub | base64 -d"
fi
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
