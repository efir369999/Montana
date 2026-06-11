#!/bin/bash
# Montana node-only entrypoint. No VPN. The genesis manifest is mounted from the
# host (never baked into the image), so a node's genesis is decoupled from the
# binary and cannot silently drift between machines.
#
# Env:
#   MONTANA_LISTEN              default /ip4/0.0.0.0/tcp/8444
#   MONTANA_MNEMONIC           fixed identity (otherwise generated on first run)
#   MONTANA_GENESIS_MANIFEST   manifest path (default /etc/montana/genesis-manifest.json)
#   MONTANA_MANIFEST_SHA256    if set, refuse to start unless manifest sha matches
set -eu

DATA_DIR="/var/lib/montana"
MANIFEST="${MONTANA_GENESIS_MANIFEST:-/etc/montana/genesis-manifest.json}"
LISTEN="${MONTANA_LISTEN:-/ip4/0.0.0.0/tcp/8444}"

mkdir -p "$DATA_DIR"
chown -R montana:montana "$DATA_DIR"

# 1. Generate identity once per data volume.
if [ ! -f "$DATA_DIR/identity.bin" ]; then
  echo "[entrypoint] first run on this volume — generating node identity"
  if [ -n "${MONTANA_MNEMONIC:-}" ]; then
    runuser -u montana -- /usr/local/bin/montana-node init --data-dir "$DATA_DIR" --mnemonic "$MONTANA_MNEMONIC" | tee "$DATA_DIR/mnemonic.txt"
  else
    runuser -u montana -- /usr/local/bin/montana-node init --data-dir "$DATA_DIR" | tee "$DATA_DIR/mnemonic.txt"
  fi
  chmod 0400 "$DATA_DIR/mnemonic.txt"
  chown montana:montana "$DATA_DIR/mnemonic.txt"
fi

# 2. Optional manifest pin — refuse to start on sha mismatch (anti-drift).
if [ -f "$MANIFEST" ] && [ -n "${MONTANA_MANIFEST_SHA256:-}" ]; then
  actual="$(sha256sum "$MANIFEST" | cut -d' ' -f1)"
  if [ "$actual" != "$MONTANA_MANIFEST_SHA256" ]; then
    echo "[entrypoint] FATAL: manifest sha256 $actual != pinned $MONTANA_MANIFEST_SHA256"
    exit 1
  fi
  echo "[entrypoint] manifest sha256 verified: $actual"
fi

# 3. Start the node.
# The node requires --listen and --genesis-manifest TOGETHER (cross-machine mode)
# or NEITHER (singleton mode). Never --listen alone.
if [ -f "$MANIFEST" ]; then
  set -- start --data-dir "$DATA_DIR" --listen "$LISTEN" --genesis-manifest "$MANIFEST"
  echo "[entrypoint] cross-machine mode — manifest $MANIFEST, listen $LISTEN"
else
  set -- start --data-dir "$DATA_DIR"
  echo "[entrypoint] singleton mode — no manifest, no --listen"
fi
exec runuser -u montana -- /usr/local/bin/montana-node "$@"
