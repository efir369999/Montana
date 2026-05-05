#!/bin/bash
# Сборка Montana.app — SwiftUI desktop wrapper над launchd-узлом montana-node.
#
# Шаги:
#   1. swift build -c release        — собрать executable + bundle ресурсов
#   2. собрать Montana.app/ structure: Contents/{MacOS,Resources}
#   3. скопировать executable + Info.plist + Montana.icns + Montana_Montana.bundle (SPM resources)
#   4. финальный путь: Apps/macOS/Montana/build/Montana.app

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BUILD_DIR="$ROOT/build"
APP="$BUILD_DIR/Montana.app"
CONTENTS="$APP/Contents"
MACOS="$CONTENTS/MacOS"
RESOURCES="$CONTENTS/Resources"

log() { printf '\033[1;32m[build]\033[0m %s\n' "$*"; }
die() { printf '\033[1;31m[build] ОШИБКА:\033[0m %s\n' "$*" >&2; exit 1; }

log "swift build -c release"
cd "$ROOT"
swift build -c release --product Montana

BIN_PATH="$ROOT/.build/release/Montana"
[ -x "$BIN_PATH" ] || die "executable не собран: $BIN_PATH"

log "сборка .app bundle"
rm -rf "$APP"
mkdir -p "$MACOS" "$RESOURCES"

cp "$BIN_PATH" "$MACOS/Montana"
cp "$ROOT/Info.plist" "$CONTENTS/Info.plist"
cp "$ROOT/Resources/Montana.icns" "$RESOURCES/Montana.icns"

# SPM ресурс-bundle (Montana_Montana.bundle) копируем целиком в Resources
SPM_BUNDLE="$ROOT/.build/release/Montana_Montana.bundle"
if [ -d "$SPM_BUNDLE" ]; then
  cp -R "$SPM_BUNDLE" "$RESOURCES/"
fi

# обновим mtime — Finder/Launch Services перечитает bundle
touch "$APP"

log "готово: $APP"
log "запуск: open '$APP'"
