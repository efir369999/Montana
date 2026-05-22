#!/usr/bin/env bash
# Build MontanaMonitor.app — macOS status-bar app that monitors the live mesh
# via https://efir.org/explorer/data.json. Reads-only, no privileged access.
#
# Usage: bash build.sh        — builds and copies the app bundle to ~/Applications
#        bash build.sh run    — builds and launches immediately
set -euo pipefail

HERE="$(cd "$(dirname "$0")" && pwd)"
cd "$HERE"

BUNDLE="$HERE/MontanaMonitor.app"
CONTENTS="$BUNDLE/Contents"
MACOS="$CONTENTS/MacOS"
RESOURCES="$CONTENTS/Resources"

rm -rf "$BUNDLE"
mkdir -p "$MACOS" "$RESOURCES"

swift build --configuration release --package-path "$HERE"
cp "$HERE/.build/release/MontanaMonitor" "$MACOS/MontanaMonitor"

cat > "$CONTENTS/Info.plist" <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleExecutable</key>           <string>MontanaMonitor</string>
  <key>CFBundleIdentifier</key>           <string>quest.montana.monitor</string>
  <key>CFBundleName</key>                 <string>Montana Monitor</string>
  <key>CFBundleShortVersionString</key>   <string>1.0.0</string>
  <key>CFBundleVersion</key>              <string>1.0.0</string>
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

# Install + register LaunchAgent so the app auto-launches on login.
PLIST_SRC="$HERE/quest.montana.monitor.plist"
PLIST_DEST="$HOME/Library/LaunchAgents/quest.montana.monitor.plist"
LOG_DIR="$HOME/Applications/Montana/data/logs"
mkdir -p "$LOG_DIR"
sed -e "s|__APP_PATH__|$DEST/Contents/MacOS/MontanaMonitor|" \
    -e "s|__LOG_OUT__|$LOG_DIR/monitor.log|" \
    -e "s|__LOG_ERR__|$LOG_DIR/monitor.err.log|" \
    "$PLIST_SRC" > "$PLIST_DEST"

# Reload the LaunchAgent so the new binary takes over.
launchctl unload "$PLIST_DEST" 2>/dev/null || true
launchctl load -w "$PLIST_DEST"
echo "LaunchAgent registered at $PLIST_DEST"

if [ "${1:-}" = "run" ]; then
    open "$DEST"
fi
