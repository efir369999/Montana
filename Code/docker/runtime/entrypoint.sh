#!/bin/sh
# Montana node container entrypoint.
#
# Runs as root just long enough to make the named-volume mountpoint writable
# by user montana, then drops privileges via runuser. On first start, the
# init step prints a 24-word mnemonic to stdout and saves it to mnemonic.txt
# inside the volume (mode 0400, owner montana).

set -eu

DATA_DIR="/var/lib/montana"
MNEMONIC_FILE="$DATA_DIR/mnemonic.txt"
MANIFEST="/etc/montana/genesis-manifest.json"
LISTEN="${MONTANA_LISTEN:-/ip4/0.0.0.0/tcp/8444}"

# Make the named volume mountpoint writable by montana.
chown -R montana:montana "$DATA_DIR"

if [ ! -f "$DATA_DIR/identity.bin" ]; then
  echo "================================================================"
  echo "  Montana node — first run on this volume"
  echo "  Generating identity. The 24 mnemonic words below are the ONLY"
  echo "  backup. Save them now (they will not be regenerated)."
  echo "================================================================"
  runuser -u montana -- /usr/local/bin/montana-node init --data-dir "$DATA_DIR" \
    | tee "$MNEMONIC_FILE"
  chmod 0400 "$MNEMONIC_FILE"
  chown montana:montana "$MNEMONIC_FILE"
  echo "================================================================"
  echo "  Mnemonic also saved to $MNEMONIC_FILE (mode 0400, owner montana)."
  echo "  Retrieve later with: docker exec montana-node cat $MNEMONIC_FILE"
  echo "================================================================"
fi

exec runuser -u montana -- /usr/local/bin/montana-node start \
  --data-dir "$DATA_DIR" \
  --listen "$LISTEN" \
  --genesis-manifest "$MANIFEST"
