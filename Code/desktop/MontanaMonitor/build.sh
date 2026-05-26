#!/usr/bin/env bash
# Build MontanaMonitor.app — macOS status-bar + VPN client for Montana mesh.
# Bundles xray-core for Reality VPN; subscription pulled from montana.quest/vpn/sub.
set -euo pipefail

HERE="$(cd "$(dirname "$0")" && pwd)"
cd "$HERE"

# Auto-increment build number (CFBundleVersion). Stored в .build-number,
# растёт монотонно на каждую пересборку чтобы macOS не путал кеш .app.
BUILD_FILE="$HERE/.build-number"
if [ -f "$BUILD_FILE" ]; then
    BUILD=$(($(cat "$BUILD_FILE") + 1))
else
    BUILD=4
fi
echo "$BUILD" > "$BUILD_FILE"
echo "→ build #$BUILD"

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
(cd "$HERE/../../" && cargo build -p montana-node --release)
WORKSPACE_ROOT="$HERE/../.."
mkdir -p "$HERE/Resources/mt-bindings"
cp "$WORKSPACE_ROOT/target/release/libmt_bindings.a" "$HERE/Resources/mt-bindings/libmt_bindings.a"
cp "$WORKSPACE_ROOT/crates/mt-bindings/include/mt_bindings.h" "$HERE/Sources/MontanaBindings/mt_bindings.h"

swift build --configuration release --package-path "$HERE"
cp "$HERE/.build/release/MontanaMonitor" "$MACOS/MontanaMonitor"
cp "$XRAY_BIN" "$RESOURCES/xray"
cp "$WORKSPACE_ROOT/target/release/montana-node" "$RESOURCES/montana-node"
chmod +x "$RESOURCES/montana-node"
cp "$HERE/Resources/montana-vpn-proxy" "$RESOURCES/montana-vpn-proxy"
chmod +x "$RESOURCES/montana-vpn-proxy"
chmod +x "$RESOURCES/xray"

cp "$HERE/Resources/Montana.icns" "$RESOURCES/Montana.icns"

cat > "$CONTENTS/Info.plist" <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleExecutable</key>           <string>MontanaMonitor</string>
  <key>CFBundleIdentifier</key>           <string>quest.montana.core</string>
  <key>CFBundleName</key>                 <string>Montana</string>
  <key>CFBundleDisplayName</key>          <string>Montana</string>
  <key>CFBundleIconFile</key>             <string>Montana</string>
  <key>CFBundleShortVersionString</key>   <string>0.1</string>
  <key>CFBundleVersion</key>              <string>$BUILD</string>
  <key>LSMinimumSystemVersion</key>       <string>14.0</string>
  <key>NSHighResolutionCapable</key>      <true/>
  <key>NSQuitAlwaysKeepsWindows</key>     <false/>
  <key>NSSupportsAutomaticTermination</key> <false/>
  <key>NSPrincipalClass</key>             <string>NSApplication</string>
</dict>
</plist>
PLIST

DEST="$HOME/Applications/Montana.app"
rm -rf "$DEST"
cp -R "$BUNDLE" "$DEST"
echo "installed at $DEST"

# Package as DMG for distribution через montana.quest/vpn/
DMG_OUT="$HERE/dist/Montana-0.1.dmg"
mkdir -p "$HERE/dist"
rm -f "$DMG_OUT"
TMPDIR_DMG=$(mktemp -d)
cp -R "$BUNDLE" "$TMPDIR_DMG/Montana.app"
ln -s /Applications "$TMPDIR_DMG/Applications"
hdiutil create -volname "Montana 0.1" -srcfolder "$TMPDIR_DMG" -ov -format UDZO "$DMG_OUT" >/dev/null
rm -rf "$TMPDIR_DMG"
echo "DMG ready: $DMG_OUT ($(du -h "$DMG_OUT" | cut -f1))"

# Снимаем старый launchd agent от menu-bar версии если был
PLIST_DEST="$HOME/Library/LaunchAgents/quest.montana.monitor.plist"
[ -f "$PLIST_DEST" ] && launchctl unload "$PLIST_DEST" 2>/dev/null && rm -f "$PLIST_DEST"

# Auto-launch локально для просмотра — kill предыдущий процесс, открыть новый.
osascript -e 'tell application "Montana" to quit' 2>/dev/null || pkill -f "/Applications/Montana.app" 2>/dev/null
sleep 1
open "$DEST"
echo "→ launched Montana.app (build #$BUILD)"
