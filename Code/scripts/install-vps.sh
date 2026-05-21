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
