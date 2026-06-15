#!/bin/bash
# Montana all-in-one container entrypoint (v1.0.2).
#
# Starts: nginx :80 (decoy), xray :443 (Reality), montana-node :8444 (foreground).
# Auto-registers with Moscow orchestrator on first start so node appears in
# https://montana.quest/vpn/sub within ~5 minutes.
#
# Requires: --network host (so :443 and :8444 are reachable from the open internet).
#
# Env overrides:
#   MONTANA_LISTEN                  default /ip4/0.0.0.0/tcp/8444
#   MONTANA_MNEMONIC                fixed identity for test cohorts
#   MONTANA_GENESIS_MANIFEST_B64    base64 of a custom manifest (test cohort)
#   MONTANA_D_TEST_OVERRIDE         small D → fast windows (test)
#   MONTANA_FASTSYNC_LAG_THRESHOLD  override fast-sync lag threshold
#   MONTANA_ALIAS                   per-city alias used in orchestrator registration
#   MONTANA_NODE_TAG                3-char xray inbound tag (defaults to alias[:3])
#   MONTANA_HOSTING                 hosting provider label
#   MONTANA_ORCH_TOKEN              override built-in orchestrator admin token
#   MONTANA_ORCH_URL                override orchestrator URL (default Moscow)
#   MONTANA_DISABLE_VPN=1           skip xray + nginx + register (node-only mode)

set -eu

DATA_DIR="/var/lib/montana"
VPN_DIR="/etc/montana-vpn"
MNEMONIC_FILE="$DATA_DIR/mnemonic.txt"
MANIFEST="/etc/montana/genesis-manifest.json"
LISTEN="${MONTANA_LISTEN:-/ip4/0.0.0.0/tcp/8444}"

# Federation secrets baked into the image (universal Reality key + orch token
# shared across the Montana mesh; per-host UUID generated below).
BUILTIN_PRIVKEY="cL7D6FCqH5nWcQlHCKH9uNr-RNwCt5peRAqt8tl9mXs"
BUILTIN_PBK="EkTs2aGKnFNgFZ0f7wgft2sJp3VjwFQqIrwkZKM4gD8"
BUILTIN_SID="302805bc0c25e504"
BUILTIN_SNI="www.googletagmanager.com"
BUILTIN_ORCH_TOKEN="b517e7888473d905d26eba58c444f7cad927978c5ef3a77b5baa8bb6c296c948"
DEFAULT_ORCH_URL="https://montana.quest/api"

mkdir -p "$DATA_DIR" "$VPN_DIR"
chown -R montana:montana "$DATA_DIR"

# ── test cohort: custom genesis manifest ─────────────────────────────────────
if [ -n "${MONTANA_GENESIS_MANIFEST_B64:-}" ]; then
  echo "$MONTANA_GENESIS_MANIFEST_B64" | base64 -d > "$DATA_DIR/genesis-manifest.json"
  chown montana:montana "$DATA_DIR/genesis-manifest.json"
  MANIFEST="$DATA_DIR/genesis-manifest.json"
  echo "[entrypoint] TEST MODE: using supplied genesis manifest"
fi

# ── 1. Init montana identity (first run only) ───────────────────────────────
if [ ! -f "$DATA_DIR/identity.bin" ]; then
  echo "================================================================"
  echo "  Montana node — first run on this volume"
  echo "  Generating identity. Save the 24 mnemonic words below."
  echo "================================================================"
  if [ -n "${MONTANA_MNEMONIC:-}" ]; then
    runuser -u montana -- /usr/local/bin/montana-node init --data-dir "$DATA_DIR" \
      --mnemonic "$MONTANA_MNEMONIC" | tee "$MNEMONIC_FILE"
  else
    runuser -u montana -- /usr/local/bin/montana-node init --data-dir "$DATA_DIR" \
      | tee "$MNEMONIC_FILE"
  fi
  chmod 0400 "$MNEMONIC_FILE"
  chown montana:montana "$MNEMONIC_FILE"
  echo "  Mnemonic saved to $MNEMONIC_FILE (mode 0400)."
fi

# ── 2. VPN setup (skippable via MONTANA_DISABLE_VPN=1) ──────────────────────
if [ "${MONTANA_DISABLE_VPN:-0}" != "1" ]; then
  # Per-host UUID, generated once and persisted to volume
  if [ ! -f "$VPN_DIR/uuid" ]; then
    /usr/local/bin/xray uuid > "$VPN_DIR/uuid"
    chmod 0600 "$VPN_DIR/uuid"
  fi
  CLIENT_UUID="$(cat "$VPN_DIR/uuid")"

  # Detect public IP for orchestrator registration + VLESS URL display
  PUBLIC_IP="$(curl -sf --max-time 5 https://api.ipify.org 2>/dev/null \
    || curl -sf --max-time 5 https://ifconfig.me 2>/dev/null \
    || hostname -I | awk '{print $1}')"

  # Per-city alias (defaults to short hash of UUID; operator overrides via env)
  ALIAS="${MONTANA_ALIAS:-$(echo "$CLIENT_UUID" | cut -c1-6)}"
  NODE_TAG="${MONTANA_NODE_TAG:-$(echo "$ALIAS" | cut -c1-3)}"
  HOSTING="${MONTANA_HOSTING:-unknown}"

  # Render xray Reality config — uses universal pbk/sid + this host's UUID
  cat > /etc/xray.config.json <<XRAY_CFG
{
  "log": {"loglevel": "warning", "access": "/var/log/xray/access.log", "error": "/var/log/xray/error.log"},
  "dns": {"servers": ["1.1.1.1", "8.8.8.8"], "queryStrategy": "UseIP"},
  "inbounds": [{
    "tag": "reality-${NODE_TAG}-entry",
    "listen": "0.0.0.0", "port": 443, "protocol": "vless",
    "settings": {"clients": [{"id": "${CLIENT_UUID}", "email": "${ALIAS}@montana", "flow": "xtls-rprx-vision"}], "decryption": "none"},
    "streamSettings": {
      "network": "tcp", "security": "reality",
      "realitySettings": {
        "show": false, "dest": "${BUILTIN_SNI}:443", "xver": 0,
        "serverNames": ["${BUILTIN_SNI}"],
        "privateKey": "${BUILTIN_PRIVKEY}",
        "shortIds": ["${BUILTIN_SID}"]
      }
    },
    "sniffing": {"enabled": true, "destOverride": ["http", "tls", "quic"], "routeOnly": false}
  }],
  "outbounds": [{"protocol": "freedom", "tag": "direct"}]
}
XRAY_CFG

  # Start nginx :80 decoy
  nginx -t >/dev/null 2>&1 && nginx
  echo "[entrypoint] nginx :80 (decoy) started"

  # Start xray :443 Reality (background)
  /usr/local/bin/xray run -c /etc/xray.config.json >> /var/log/xray/run.log 2>&1 &
  XRAY_PID=$!
  echo "[entrypoint] xray :443 (Reality) started pid=$XRAY_PID alias=$ALIAS uuid=${CLIENT_UUID:0:8}…"

  # Register with Moscow orchestrator (idempotent — orchestrator overwrites prior entry by IP)
  if [ -n "$PUBLIC_IP" ] && [ -n "${MONTANA_ORCH_TOKEN:-$BUILTIN_ORCH_TOKEN}" ]; then
    ORCH_URL="${MONTANA_ORCH_URL:-$DEFAULT_ORCH_URL}"
    ORCH_TOKEN="${MONTANA_ORCH_TOKEN:-$BUILTIN_ORCH_TOKEN}"
    # Geo lookup
    GEO_JSON="$(curl -sf --max-time 5 "http://ip-api.com/json/${PUBLIC_IP}?fields=country,countryCode,city,lat,lon,isp" 2>/dev/null || echo '{}')"
    COUNTRY="$(echo "$GEO_JSON" | jq -r '.country // ""')"
    COUNTRY_CODE="$(echo "$GEO_JSON" | jq -r '.countryCode // ""')"
    CITY="$(echo "$GEO_JSON" | jq -r '.city // ""')"
    PAYLOAD=$(jq -n \
      --arg secret "$ORCH_TOKEN" \
      --arg ip "$PUBLIC_IP" \
      --arg alias "$ALIAS" \
      --arg uuid "$CLIENT_UUID" \
      --arg country "$COUNTRY" \
      --arg cc "$COUNTRY_CODE" \
      --arg city "$CITY" \
      --arg hosting "$HOSTING" \
      --arg pbk "$BUILTIN_PBK" \
      --arg sid "$BUILTIN_SID" \
      --arg sni "$BUILTIN_SNI" \
      '{secret:$secret, ip:$ip, alias:$alias, uuid:$uuid, country:$country, country_code:$cc, city:$city, hosting:$hosting, pbk:$pbk, sid:$sid, sni:$sni, role:"vpn-backend"}')
    echo "[entrypoint] registering with $ORCH_URL/register (alias=$ALIAS country=$COUNTRY city=$CITY)"
    RESP="$(curl -sk --max-time 20 -X POST -H 'Content-Type: application/json' -d "$PAYLOAD" "$ORCH_URL/register" 2>/dev/null || true)"
    if [ -n "$RESP" ]; then
      echo "[entrypoint] orch response: $RESP"
    else
      echo "[entrypoint] orch register failed or no response (network blocked?); subscription may not include this node"
    fi
  fi

  # Print VLESS URL
  echo "================================================================"
  echo "  Montana VPN endpoint up. Personal VLESS URL:"
  echo "================================================================"
  echo "vless://${CLIENT_UUID}@${PUBLIC_IP}:443?type=tcp&headerType=none&security=reality&fp=chrome&sni=${BUILTIN_SNI}&pbk=${BUILTIN_PBK}&sid=${BUILTIN_SID}#montana-${ALIAS}"
  echo
fi

# ── 3. start montana-node (foreground, tini supervises) ─────────────────────
DTEST=""
if [ -n "${MONTANA_D_TEST_OVERRIDE:-}" ]; then
  DTEST="--d-test-override $MONTANA_D_TEST_OVERRIDE"
  echo "[entrypoint] TEST MODE: $DTEST"
fi
FASTSYNC_ENV=""
if [ -n "${MONTANA_FASTSYNC_LAG_THRESHOLD:-}" ]; then
  FASTSYNC_ENV="MONTANA_FASTSYNC_LAG_THRESHOLD=$MONTANA_FASTSYNC_LAG_THRESHOLD"
fi
exec runuser -u montana -- env $FASTSYNC_ENV /usr/local/bin/montana-node start \
  --data-dir "$DATA_DIR" \
  --listen "$LISTEN" \
  --genesis-manifest "$MANIFEST" \
  $DTEST
