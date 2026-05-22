#!/bin/bash
# Montana node — one-command install on a clean Linux VPS.
#
# Usage (one line on the VPS):
#   curl -sSL https://raw.githubusercontent.com/efir369999/Montana/main/Code/scripts/install-vps.sh | sudo bash
#
# Or after `git clone`:
#   sudo bash /opt/montana/Code/scripts/install-vps.sh
#
# Defaults:
#   - listen on /ip4/0.0.0.0/tcp/8444 (Noise_PQ XX over TCP)
#   - dial the three-node Genesis cohort from Code/scripts/genesis-manifest.json
#   - the new node joins as a candidate; live mesh heartbeats appear in the
#     three Genesis peers' logs within seconds of `systemctl start`
#
# Overrides:
#   MONTANA_LISTEN=/ip4/0.0.0.0/tcp/PORT      change listen port (default 8444)
#   MONTANA_GENESIS_MANIFEST=/path/to/file    use a custom manifest file
#   MONTANA_REPO_BRANCH=main                  override branch (default main)
#   INSTALL_VPN=1                             also install Xray Reality VPN
#                                              backend on :443 (joins the
#                                              federated /vpn/sub pool)
#
# Steps:
#   1. Verify root and detect OS (Ubuntu / Debian / Fedora / RHEL / Alpine)
#   2. Install system build dependencies
#   3. Install Rust toolchain via rustup (skip if cargo already present)
#   4. Clone or fast-forward the repository at /opt/montana
#   5. Build the release binary
#   6. Create system user `montana` and data directory /var/lib/montana
#   7. Generate identity (prints 24-word recovery mnemonic — save it!)
#   8. Deploy /etc/montana/genesis-manifest.json from the repo bundle
#   9. Install systemd unit with hardening + cross-machine networking
#  10. Enable and start the service
#
# After step 10, the local node dials the three Genesis peers, negotiates
# Noise_PQ XX (/montana/noise-pq-xx/1.0.0), and starts exchanging Ping/Pong
# heartbeats. The new node appears in mos/fra/zel journals as
# `[network] CONNECTION ESTABLISHED peer=<your XX peer_id> label=unknown`.

set -euo pipefail

REPO_URL="${MONTANA_REPO_URL:-https://github.com/efir369999/Montana.git}"
REPO_BRANCH="${MONTANA_REPO_BRANCH:-main}"
INSTALL_DIR="/opt/montana"
DATA_DIR="/var/lib/montana"
ETC_DIR="/etc/montana"
BIN_DST="/usr/local/bin/montana-node"
USER_NAME="montana"
SERVICE_FILE="/etc/systemd/system/montana-node.service"
DEFAULT_LISTEN="/ip4/0.0.0.0/tcp/8444"
DEFAULT_MANIFEST_SRC="$INSTALL_DIR/Code/scripts/genesis-manifest.json"
DEFAULT_MANIFEST_DST="$ETC_DIR/genesis-manifest.json"

log()  { printf '\033[1;32m[install-vps]\033[0m %s\n' "$*"; }
warn() { printf '\033[1;33m[install-vps]\033[0m %s\n' "$*" >&2; }
die()  { printf '\033[1;31m[install-vps] ERROR:\033[0m %s\n' "$*" >&2; exit 1; }

# Step 1: root + OS detection
if [ "$(id -u)" != "0" ]; then
  die "root privileges required. Run: curl -sSL <URL> | sudo bash"
fi
if [ ! -f /etc/os-release ]; then
  die "cannot detect OS — /etc/os-release missing"
fi
. /etc/os-release
OS_ID="${ID:-unknown}"
log "detected OS: ${PRETTY_NAME:-$OS_ID}"

# Step 2: system deps
log "installing system dependencies..."
case "$OS_ID" in
  ubuntu|debian)
    export DEBIAN_FRONTEND=noninteractive
    apt-get update -qq
    apt-get install -y -qq build-essential clang pkg-config git curl perl ca-certificates >/dev/null
    ;;
  fedora|rhel|centos|rocky|almalinux)
    dnf install -y -q gcc gcc-c++ clang pkgconf-pkg-config git curl perl ca-certificates make >/dev/null
    ;;
  alpine)
    apk add --no-cache build-base clang pkgconfig git curl perl linux-headers ca-certificates >/dev/null
    ;;
  *)
    die "unsupported OS: $OS_ID. Supported: ubuntu, debian, fedora, rhel, centos, rocky, almalinux, alpine"
    ;;
esac

# Step 3: Rust toolchain
if ! command -v cargo >/dev/null 2>&1; then
  log "installing Rust toolchain via rustup..."
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | \
    sh -s -- -y --default-toolchain stable --profile minimal --no-modify-path
  export PATH="/root/.cargo/bin:$PATH"
else
  log "Rust toolchain present: $(cargo --version)"
fi
export PATH="${HOME:-/root}/.cargo/bin:/root/.cargo/bin:$PATH"

# Step 4: clone / update repo
if [ -d "$INSTALL_DIR/.git" ]; then
  log "updating repository $INSTALL_DIR..."
  cd "$INSTALL_DIR"
  git fetch origin "$REPO_BRANCH"
  git reset --hard "origin/$REPO_BRANCH"
else
  log "cloning $REPO_URL (branch $REPO_BRANCH) → $INSTALL_DIR..."
  rm -rf "$INSTALL_DIR"
  git clone --branch "$REPO_BRANCH" --single-branch "$REPO_URL" "$INSTALL_DIR"
fi

# Step 5: build release binary
SOURCE_DIR="$INSTALL_DIR/Code"
if [ ! -d "$SOURCE_DIR" ]; then
  die "expected directory '$SOURCE_DIR' not found in repository"
fi
cd "$SOURCE_DIR"
log "building montana-node release (5–30 minutes on first run)..."
cargo build --release -p montana-node 2>&1 | tail -5

# Step 6: install binary
install -m 0755 target/release/montana-node "$BIN_DST"
log "binary installed: $BIN_DST"

# Step 7: system user + data dir
if ! id "$USER_NAME" >/dev/null 2>&1; then
  log "creating system user $USER_NAME..."
  useradd -r -s /usr/sbin/nologin -d "$DATA_DIR" -M "$USER_NAME" 2>/dev/null \
    || useradd -r -s /bin/false -d "$DATA_DIR" "$USER_NAME"
fi
mkdir -p "$DATA_DIR"
chown -R "$USER_NAME:$USER_NAME" "$DATA_DIR"
chmod 0750 "$DATA_DIR"

# Step 8: identity (only if missing)
if [ ! -f "$DATA_DIR/identity.bin" ]; then
  log "generating identity (24-word mnemonic)..."
  echo
  echo "================================================================"
  echo "  NOTE: 24 mnemonic words will be printed below."
  echo "  Write them down in a safe place — this is the entire backup."
  echo "  Lose them → lose the node and all earned Ɉ."
  echo "================================================================"
  echo
  sudo -u "$USER_NAME" "$BIN_DST" init --data-dir "$DATA_DIR"
  echo
else
  log "identity already exists ($DATA_DIR/identity.bin) — skipping init"
fi

# Step 9: deploy Genesis manifest
LISTEN_ADDR="${MONTANA_LISTEN:-$DEFAULT_LISTEN}"
MANIFEST_PATH="${MONTANA_GENESIS_MANIFEST:-$DEFAULT_MANIFEST_DST}"

mkdir -p "$ETC_DIR"
chmod 0755 "$ETC_DIR"

if [ -z "${MONTANA_GENESIS_MANIFEST:-}" ]; then
  # default path: copy the bundled manifest to /etc/montana
  if [ ! -f "$DEFAULT_MANIFEST_SRC" ]; then
    die "expected bundled manifest at $DEFAULT_MANIFEST_SRC not found in repository"
  fi
  install -m 0644 "$DEFAULT_MANIFEST_SRC" "$DEFAULT_MANIFEST_DST"
  log "deployed default Genesis manifest to $DEFAULT_MANIFEST_DST"
else
  if [ ! -f "$MANIFEST_PATH" ]; then
    warn "custom manifest $MANIFEST_PATH not yet present — node will retry on systemd restart loop"
  else
    log "using custom manifest at $MANIFEST_PATH"
  fi
fi

# Step 10: systemd unit
log "installing systemd unit at $SERVICE_FILE..."
cat > "$SERVICE_FILE" <<UNIT
[Unit]
Description=Montana Node (cross-machine, Proof-of-Time, Noise_PQ XX)
Documentation=https://github.com/efir369999/Montana
After=network.target
Wants=network-online.target

[Service]
Type=simple
User=$USER_NAME
Group=$USER_NAME
ExecStart=$BIN_DST start --data-dir $DATA_DIR --listen $LISTEN_ADDR --genesis-manifest $MANIFEST_PATH
Restart=on-failure
RestartSec=10
StandardOutput=journal
StandardError=journal

# Hardening per systemd security best-practice
NoNewPrivileges=yes
PrivateTmp=yes
ProtectSystem=strict
ProtectHome=yes
ReadWritePaths=$DATA_DIR
ProtectKernelTunables=yes
ProtectKernelModules=yes
ProtectControlGroups=yes
RestrictRealtime=yes
RestrictSUIDSGID=yes
LockPersonality=yes
MemoryDenyWriteExecute=no
SystemCallArchitectures=native

# Resource limits — node is single-threaded, one core is enough
CPUQuota=110%
LimitNOFILE=4096

[Install]
WantedBy=multi-user.target
UNIT

systemctl daemon-reload
systemctl enable montana-node.service >/dev/null 2>&1
systemctl restart montana-node.service

# Final report
sleep 3
log ""
log "================================================================"
log "  INSTALL COMPLETE"
log "================================================================"
log ""
log "Binary:        $BIN_DST"
log "Data:          $DATA_DIR"
log "User:          $USER_NAME"
log "Listen:        $LISTEN_ADDR"
log "Manifest:      $MANIFEST_PATH"
log "Service:       montana-node.service"
log ""
log "--- service status ---"
systemctl --no-pager status montana-node.service | head -10 || true
log ""
log "Useful commands:"
log "  systemctl status montana-node       # current status"
log "  journalctl -u montana-node -f       # follow logs"
log "  systemctl stop montana-node         # stop the node"
log "  systemctl restart montana-node      # restart"
log "  $BIN_DST status --data-dir $DATA_DIR    # phase + balance"
log ""
log "Node lifecycle:"
log "  Phase 1: Bootstrap → CandidateVdf   (sequential SHA-256 chain to vdf_chain_length ≥ τ₂)"
log "  Phase 2: CandidateVdf → Registered  (NodeRegistration via canonical apply_*)"
log "  Phase 3: Registered → Active        (selection event on next W where W % 336 == 0)"
log "  Phase 4: Active                     (emission 13 Ɉ per window via apply_proposal)"
log ""
log "This is the spec-defined Sybil barrier — there is no shortcut."
log "The node survives VPS restarts (Restart=on-failure) and resumes at the same window."
log ""
log "Within seconds of start, your node negotiates Noise_PQ XX with the three"
log "Genesis peers (moscow / frankfurt / helsinki) listed in the manifest. To"
log "confirm the connection appears in the live mesh, ask one of the Genesis"
log "operators to grep their journal for your local XX PeerId."

# ───────────────────────────────────────────────────────────────────────
# Optional: Montana Xray Reality VPN backend
# ───────────────────────────────────────────────────────────────────────
# Run with INSTALL_VPN=1 to also stand up the Reality VPN endpoint on :443
# alongside montana-node. The endpoint joins the federated /vpn/sub pool
# served at https://montana.quest/vpn/sub. Universal shared keypair is
# used so every Montana VPN-backend node accepts the same client config
# (see project_montana_vpn_universal_key.md).
#
# Defaults (overridable via env):
#   INSTALL_VPN=0                — set to 1 to install
#   VPN_UNIVERSAL_UUID           — shared UUID for all Montana VPN clients
#   VPN_UNIVERSAL_PRIVKEY        — shared Reality x25519 private key
#   VPN_UNIVERSAL_SID            — shared Reality short_id
#   VPN_SNI                      — Reality dest SNI

if [ "${INSTALL_VPN:-0}" = "1" ]; then
    log "--- installing Xray Reality VPN backend on :443 ---"
    VPN_UNIVERSAL_UUID="${VPN_UNIVERSAL_UUID:-e6d355e2-2d79-4c96-a373-3b0e6b6f4b0d}"
    VPN_UNIVERSAL_PRIVKEY="${VPN_UNIVERSAL_PRIVKEY:-cL7D6FCqH5nWcQlHCKH9uNr-RNwCt5peRAqt8tl9mXs}"
    VPN_UNIVERSAL_SID="${VPN_UNIVERSAL_SID:-302805bc0c25e504}"
    VPN_SNI="${VPN_SNI:-www.googletagmanager.com}"
    XRAY_VERSION="${XRAY_VERSION:-26.2.6}"

    if ! command -v xray >/dev/null 2>&1; then
        log "  installing xray-core v${XRAY_VERSION}..."
        bash -c "$(curl -L https://github.com/XTLS/Xray-install/raw/main/install-release.sh)" \
            @ install --version "${XRAY_VERSION}" -u root >/dev/null 2>&1 || \
            log "  WARN xray install failed; continuing"
    fi

    if command -v xray >/dev/null 2>&1; then
        mkdir -p /usr/local/etc/xray /var/log/xray /var/lib/montana-net
        cat > /usr/local/etc/xray/config.json <<XRAY
{
  "log": {"loglevel": "warning", "access": "/var/log/xray/access.log", "error": "/var/log/xray/error.log"},
  "dns": {"servers": ["1.1.1.1", "8.8.8.8"], "queryStrategy": "UseIP"},
  "inbounds": [{
      "tag": "reality-entry",
      "listen": "0.0.0.0",
      "port": 443,
      "protocol": "vless",
      "settings": {
        "clients": [{"id": "${VPN_UNIVERSAL_UUID}", "email": "montana-universal", "flow": "xtls-rprx-vision"}],
        "decryption": "none"
      },
      "streamSettings": {
        "network": "tcp",
        "security": "reality",
        "realitySettings": {
          "show": false,
          "dest": "${VPN_SNI}:443",
          "xver": 0,
          "serverNames": ["${VPN_SNI}"],
          "privateKey": "${VPN_UNIVERSAL_PRIVKEY}",
          "shortIds": ["${VPN_UNIVERSAL_SID}"]
        }
      },
      "sniffing": {"enabled": true, "destOverride": ["http", "tls", "quic"]}
  }],
  "outbounds": [
    {"tag": "direct", "protocol": "freedom", "settings": {"domainStrategy": "UseIP"}},
    {"tag": "blocked", "protocol": "blackhole"},
    {"tag": "dns-out", "protocol": "dns"}
  ],
  "routing": {"rules": [
    {"type": "field", "port": "53", "outboundTag": "dns-out"},
    {"type": "field", "ip": ["geoip:private"], "outboundTag": "blocked"}
  ]}
}
XRAY

        # Public-safe metadata file for federated /vpn/sub aggregator
        VPN_PUBKEY=$(xray x25519 -i "${VPN_UNIVERSAL_PRIVKEY}" 2>/dev/null | awk -F': ' '/PublicKey/ {print $2}')
        cat > /var/lib/montana-net/my-vpn.json <<META
{
  "UUID": "${VPN_UNIVERSAL_UUID}",
  "PBK": "${VPN_PUBKEY}",
  "SID": "${VPN_UNIVERSAL_SID}",
  "SNI": "${VPN_SNI}"
}
META
        chmod 644 /var/lib/montana-net/my-vpn.json

        systemctl daemon-reload
        systemctl enable xray >/dev/null 2>&1 || true
        systemctl restart xray
        sleep 2

        if systemctl is-active --quiet xray; then
            log "  xray active on :443"
            log "  vless://${VPN_UNIVERSAL_UUID}@<this-host>:443?flow=xtls-rprx-vision&security=reality&sni=${VPN_SNI}&pbk=${VPN_PUBKEY}&sid=${VPN_UNIVERSAL_SID}&type=tcp"
            log ""
            log "To enroll this node in https://montana.quest/vpn/sub aggregator,"
            log "ask the orchestrator operator to POST to /api/orchestrator/register"
            log "with this node's IP, alias, country, and label."
        else
            log "  WARN xray failed to start; check journalctl -u xray"
        fi
    fi
fi
