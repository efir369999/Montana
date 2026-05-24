#!/bin/sh
# Montana node entrypoint.
# On first run, generates a fresh identity (24-word mnemonic printed to stdout
# and saved to /var/lib/montana/mnemonic.txt mode 0400). On subsequent runs,
# skips init and starts the node directly.

set -eu

DATA_DIR="/var/lib/montana"
MNEMONIC_FILE="$DATA_DIR/mnemonic.txt"
MANIFEST="/etc/montana/genesis-manifest.json"
LISTEN="${MONTANA_LISTEN:-/ip4/0.0.0.0/tcp/8444}"

if [ ! -f "$DATA_DIR/identity.bin" ]; then
  echo "================================================================"
  echo "  Montana node — first run on this volume"
  echo "  Generating identity. The 24 mnemonic words below are the ONLY"
  echo "  backup. Save them now (they will not be regenerated)."
  echo "================================================================"
  /usr/local/bin/montana-node init --data-dir "$DATA_DIR" | tee "$MNEMONIC_FILE"
  chmod 0400 "$MNEMONIC_FILE"
  echo "================================================================"
  echo "  Mnemonic also saved to $MNEMONIC_FILE (mode 0400)."
  echo "  Recover it later with: docker exec montana-node cat $MNEMONIC_FILE"
  echo "================================================================"
fi

exec /usr/local/bin/montana-node start \
  --data-dir "$DATA_DIR" \
  --listen "$LISTEN" \
  --genesis-manifest "$MANIFEST"
