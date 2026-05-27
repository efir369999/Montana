#!/bin/sh
# Montana node container entrypoint.
#
# Runs as root just long enough to make the named-volume mountpoint writable
# by user montana, then drops privileges via runuser. On first start, the
# init step prints a 24-word mnemonic to stdout and saves it to mnemonic.txt.
#
# Network test mode (production binary, test parameters) via env:
#   MONTANA_MNEMONIC               fixed identity (so a test manifest can pre-list this node)
#   MONTANA_GENESIS_MANIFEST_B64   base64 of a custom genesis manifest (test cohort)
#   MONTANA_D_TEST_OVERRIDE        small D → fast windows → fast admission VDF

set -eu

DATA_DIR="/var/lib/montana"
MNEMONIC_FILE="$DATA_DIR/mnemonic.txt"
MANIFEST="/etc/montana/genesis-manifest.json"
LISTEN="${MONTANA_LISTEN:-/ip4/0.0.0.0/tcp/8444}"

chown -R montana:montana "$DATA_DIR"

# Test cohort: a supplied genesis manifest overrides the image-baked production one.
if [ -n "${MONTANA_GENESIS_MANIFEST_B64:-}" ]; then
  echo "$MONTANA_GENESIS_MANIFEST_B64" | base64 -d > "$DATA_DIR/genesis-manifest.json"
  chown montana:montana "$DATA_DIR/genesis-manifest.json"
  MANIFEST="$DATA_DIR/genesis-manifest.json"
  echo "[entrypoint] TEST MODE: using supplied genesis manifest"
fi

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

DTEST=""
if [ -n "${MONTANA_D_TEST_OVERRIDE:-}" ]; then
  DTEST="--d-test-override $MONTANA_D_TEST_OVERRIDE"
  echo "[entrypoint] TEST MODE: $DTEST"
fi

exec runuser -u montana -- /usr/local/bin/montana-node start \
  --data-dir "$DATA_DIR" \
  --listen "$LISTEN" \
  --genesis-manifest "$MANIFEST" \
  $DTEST
