#!/usr/bin/env bash
# Build MontanaMonitor.app — macOS status-bar + VPN client for Montana mesh.
# Bundles xray-core for Reality VPN; subscription pulled from montana.quest/vpn/sub.
set -euo pipefail

HERE="$(cd "$(dirname "$0")" && pwd)"
cd "$HERE"

BUNDLE="$HERE/MontanaMonitor.app"
CONTENTS="$BUNDLE/Contents"
MACOS="$CONTENTS/MacOS"
RESOURCES="$CONTENTS/Resources"

XRAY_VERSION="v26.3.27"
ARCH="$(uname -m)"
case "$ARCH" in
    arm64)  XRAY_ASSET="Xray-macos-arm64-v8a.zip" ;;
    x86_64) XRAY_ASSET="Xray-macos-64.zip" ;;
    *) echo "unsupported arch: $ARCH" >&2; exit 1 ;;
esac
XRAY_URL="https://github.com/XTLS/Xray-core/releases/download/${XRAY_VERSION}/${XRAY_ASSET}"
XRAY_CACHE="$HERE/.cache/${XRAY_VERSION}-${ARCH}"
XRAY_BIN="$XRAY_CACHE/xray"

if [ ! -x "$XRAY_BIN" ]; then
    mkdir -p "$XRAY_CACHE"
    echo "downloading xray-core ${XRAY_VERSION} (${ARCH})…"
    curl -fSL --retry 3 -o "$XRAY_CACHE/xray.zip" "$XRAY_URL"
    (cd "$XRAY_CACHE" && unzip -o -q xray.zip)
    chmod +x "$XRAY_BIN"
fi

rm -rf "$BUNDLE"
mkdir -p "$MACOS" "$RESOURCES"

(cd "$HERE/../../" && cargo build -p mt-bindings --release)
WORKSPACE_ROOT="$HERE/../.."
mkdir -p "$HERE/Resources/mt-bindings"
cp "$WORKSPACE_ROOT/target/release/libmt_bindings.a" "$HERE/Resources/mt-bindings/libmt_bindings.a"
cp "$WORKSPACE_ROOT/crates/mt-bindings/include/mt_bindings.h" "$HERE/Sources/MontanaBindings/mt_bindings.h"

swift build --configuration release --package-path "$HERE"
cp "$HERE/.build/release/MontanaMonitor" "$MACOS/MontanaMonitor"
cp "$XRAY_BIN" "$RESOURCES/xray"
chmod +x "$RESOURCES/xray"

cat > "$CONTENTS/Info.plist" <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleExecutable</key>           <string>MontanaMonitor</string>
  <key>CFBundleIdentifier</key>           <string>quest.montana.monitor</string>
  <key>CFBundleName</key>                 <string>Montana</string>
  <key>CFBundleShortVersionString</key>   <string>1.1.0</string>
  <key>CFBundleVersion</key>              <string>1.1.0</string>
  <key>LSMinimumSystemVersion</key>       <string>14.0</string>
  <key>LSUIElement</key>                  <true/>
  <key>NSHighResolutionCapable</key>      <true/>
</dict>
</plist>
PLIST

DEST="$HOME/Applications/MontanaMonitor.app"
rm -rf "$DEST"
cp -R "$BUNDLE" "$DEST"
echo "installed at $DEST"

PLIST_SRC="$HERE/quest.montana.monitor.plist"
PLIST_DEST="$HOME/Library/LaunchAgents/quest.montana.monitor.plist"
LOG_DIR="$HOME/Applications/Montana/data/logs"
mkdir -p "$LOG_DIR"
sed -e "s|__APP_PATH__|$DEST/Contents/MacOS/MontanaMonitor|" \
    -e "s|__LOG_OUT__|$LOG_DIR/monitor.log|" \
    -e "s|__LOG_ERR__|$LOG_DIR/monitor.err.log|" \
    "$PLIST_SRC" > "$PLIST_DEST"

launchctl unload "$PLIST_DEST" 2>/dev/null || true
launchctl load -w "$PLIST_DEST"
echo "LaunchAgent registered at $PLIST_DEST"

if [ "${1:-}" = "run" ]; then
    open "$DEST"
fi
