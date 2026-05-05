#!/bin/bash
# Montana — единый installer для любого Mac (fresh OR recovery).
#
# Использование:
#
#   FRESH (новый identity, генерируется 24 слова):
#     bash install-anywhere.sh
#
#   RECOVERY (восстановить identity из backup на втором Mac):
#     INSTALL_MNEMONIC_OR_SEED='word1 word2 ... word24' bash install-anywhere.sh
#     либо
#     INSTALL_MNEMONIC_OR_SEED='<64 hex символа master_seed>' bash install-anywhere.sh
#
#   Кастомные пути:
#     INSTALL_DIR=/custom/path DATA_DIR=/custom/path/data bash install-anywhere.sh
#
# Что делает:
#   1. Сборка montana-node release (Rust, ~1-2 мин cold)
#   2. Установка binary в ~/Applications/Montana/montana-node
#   3. Identity: init заново ИЛИ recovery из переданного mnemonic/seed
#   4. launchd-агент org.montana.node (auto-restart, переживает logout)
#   5. Сборка Montana.app (SwiftUI, ~10 сек)
#   6. Установка Montana.app в /Applications/Montana.app
#   7. Регистрация в LaunchServices (Dock-иконка)
#   8. Запуск Montana.app
#
# Mac должен иметь: Xcode Command Line Tools (xcode-select --install).
# Rust toolchain установится автоматически если нет.

set -euo pipefail

SCRIPT_PATH="${BASH_SOURCE[0]}"
if [ -L "$SCRIPT_PATH" ]; then SCRIPT_PATH="$(readlink "$SCRIPT_PATH")"; fi
SCRIPT_DIR="$(cd "$(dirname "$SCRIPT_PATH")" && pwd)"
PROTOCOL_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
CODE_DIR="$PROTOCOL_ROOT/Code"
APP_SRC_DIR="$SCRIPT_DIR/Montana"

INSTALL_DIR="${INSTALL_DIR:-$HOME/Applications/Montana}"
DATA_DIR="${DATA_DIR:-$INSTALL_DIR/data}"
LOGS_DIR="$DATA_DIR/logs"
LAUNCH_AGENTS_DIR="$HOME/Library/LaunchAgents"
PLIST_PATH="$LAUNCH_AGENTS_DIR/org.montana.node.plist"
SERVICE_LABEL="org.montana.node"
APP_DEST="/Applications/Montana.app"

log()  { printf '\033[1;32m[montana]\033[0m %s\n' "$*"; }
warn() { printf '\033[1;33m[montana]\033[0m %s\n' "$*" >&2; }
die()  { printf '\033[1;31m[montana] ОШИБКА:\033[0m %s\n' "$*" >&2; exit 1; }

# --- 1. платформа + tooling ---
[ "$(uname -s)" = "Darwin" ] || die "только macOS"
log "macOS $(sw_vers -productVersion 2>/dev/null), arch $(uname -m)"
log "Montana-Protocol: $PROTOCOL_ROOT"

[ -f "$CODE_DIR/Cargo.toml" ] || die "не найден $CODE_DIR/Cargo.toml"
[ -d "$APP_SRC_DIR" ]         || die "не найден $APP_SRC_DIR"

if ! xcode-select -p >/dev/null 2>&1; then
  warn "Xcode Command Line Tools не установлены — открываю установщик"
  xcode-select --install 2>/dev/null || true
  die "после установки CLT повторите команду"
fi

if ! command -v cargo >/dev/null 2>&1; then
  log "ставлю Rust toolchain (rustup minimal)..."
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable --profile minimal --no-modify-path
fi
export PATH="$HOME/.cargo/bin:/opt/homebrew/bin:/usr/local/bin:$PATH"
command -v cargo >/dev/null 2>&1 || die "cargo не доступен"
command -v swift >/dev/null 2>&1 || die "swift не доступен (нужен Xcode CLT)"
log "Rust: $(cargo --version) · Swift: $(swift --version | head -1)"

# --- 2. остановка старого узла ---
if launchctl list "$SERVICE_LABEL" >/dev/null 2>&1; then
  log "останавливаю текущий узел перед обновлением..."
  launchctl unload "$PLIST_PATH" 2>/dev/null || true
fi

# --- 3. сборка montana-node ---
log "собираю montana-node release..."
cargo build --release --manifest-path "$CODE_DIR/Cargo.toml" -p montana-node 2>&1 | tail -3

# --- 4. install binary + commands ---
mkdir -p "$INSTALL_DIR" "$DATA_DIR" "$LOGS_DIR" "$LAUNCH_AGENTS_DIR"
cp -f "$CODE_DIR/target/release/montana-node" "$INSTALL_DIR/montana-node"
chmod 0755 "$INSTALL_DIR/montana-node"
cp -f "$CODE_DIR/dist/macOS/Montana"/*.command "$INSTALL_DIR/" 2>/dev/null || true
cp -f "$CODE_DIR/dist/macOS/Montana/README.txt" "$INSTALL_DIR/" 2>/dev/null || true
chmod 0755 "$INSTALL_DIR"/*.command 2>/dev/null || true
xattr -dr com.apple.quarantine "$INSTALL_DIR" 2>/dev/null || true

# --- 5. identity — fresh OR recovery ---
RECOVERY="${INSTALL_MNEMONIC_OR_SEED:-}"
if [ -n "$RECOVERY" ]; then
  WORD_COUNT=$(echo "$RECOVERY" | tr -s ' ' '\n' | grep -c .)
  HEX_LEN=${#RECOVERY}
  if [ "$WORD_COUNT" -eq 24 ]; then
    log "RECOVERY: identity из 24-словной мнемоники"
    "$INSTALL_DIR/montana-node" init --data-dir "$DATA_DIR" --mnemonic "$RECOVERY" --force
  elif [ "$HEX_LEN" -eq 64 ] && echo "$RECOVERY" | grep -qiE '^[0-9a-f]+$'; then
    log "RECOVERY: identity из 64-hex master_seed"
    "$INSTALL_DIR/montana-node" init --data-dir "$DATA_DIR" --entropy "$RECOVERY" --force
  else
    die "INSTALL_MNEMONIC_OR_SEED должно быть либо 24 слова через пробел, либо 64 hex; получено: $WORD_COUNT слов / $HEX_LEN символов"
  fi
elif [ ! -f "$DATA_DIR/identity.bin" ]; then
  log ""
  log "================================================================"
  log "  ГЕНЕРАЦИЯ IDENTITY — запишите 24 слова мнемоники!"
  log "================================================================"
  log ""
  "$INSTALL_DIR/montana-node" init --data-dir "$DATA_DIR"
  log ""
  log "Мнемоника также в логе установщика. Скопируйте из Terminal scrollback."
  log "Для recovery на втором Mac:"
  log "  INSTALL_MNEMONIC_OR_SEED='<24 слова>' bash install-anywhere.sh"
  log "================================================================"
else
  log "identity уже существует — пропускаю init (для recovery передайте INSTALL_MNEMONIC_OR_SEED)"
fi

# --- 6. launchd plist ---
log "устанавливаю LaunchAgent $PLIST_PATH..."
cat > "$PLIST_PATH" <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key><string>$SERVICE_LABEL</string>
    <key>ProgramArguments</key>
    <array>
        <string>$INSTALL_DIR/montana-node</string>
        <string>start</string>
        <string>--data-dir</string>
        <string>$DATA_DIR</string>
    </array>
    <key>RunAtLoad</key><true/>
    <key>KeepAlive</key>
    <dict><key>SuccessfulExit</key><false/><key>Crashed</key><true/></dict>
    <key>ThrottleInterval</key><integer>10</integer>
    <key>StandardOutPath</key><string>$LOGS_DIR/montana.log</string>
    <key>StandardErrorPath</key><string>$LOGS_DIR/montana.err.log</string>
    <key>WorkingDirectory</key><string>$DATA_DIR</string>
    <key>EnvironmentVariables</key>
    <dict><key>PATH</key><string>/usr/local/bin:/usr/bin:/bin</string></dict>
    <key>ProcessType</key><string>Standard</string>
</dict>
</plist>
PLIST
chmod 0644 "$PLIST_PATH"

# --- 7. сборка Montana.app ---
log "собираю Montana.app..."
bash "$APP_SRC_DIR/build.sh" 2>&1 | tail -4
APP_BUILT="$APP_SRC_DIR/build/Montana.app"
[ -d "$APP_BUILT" ] || die "Montana.app не собрался: $APP_BUILT"

# --- 8. установка Montana.app в /Applications/ ---
log "копирую Montana.app в /Applications/..."
if [ -d "$APP_DEST" ]; then
  rm -rf "$APP_DEST"
fi
cp -R "$APP_BUILT" "$APP_DEST"
xattr -dr com.apple.quarantine "$APP_DEST" 2>/dev/null || true
touch "$APP_DEST"
/System/Library/Frameworks/CoreServices.framework/Versions/A/Frameworks/LaunchServices.framework/Versions/A/Support/lsregister -f "$APP_DEST" 2>/dev/null || true

# --- 9. запуск узла ---
log "запускаю узел через launchctl load..."
launchctl load -w "$PLIST_PATH"
sleep 2
if launchctl list "$SERVICE_LABEL" 2>/dev/null | grep -q "$SERVICE_LABEL"; then
  STATE=$(launchctl list "$SERVICE_LABEL" | awk '/PID/ {print $3}')
  log "узел запущен, PID=$STATE"
else
  warn "не удалось загрузить службу — проверьте $LOGS_DIR/montana.err.log"
fi

# --- 10. запуск приложения ---
log "запускаю Montana.app..."
open "$APP_DEST" || warn "не смог открыть приложение автоматически"

# --- 11. финальный отчёт ---
log ""
log "================================================================"
log "  MONTANA УСТАНОВЛЕНА И ЗАПУЩЕНА"
log "================================================================"
log ""
log "Узел:        $INSTALL_DIR/montana-node (launchd: $SERVICE_LABEL)"
log "Данные:      $DATA_DIR"
log "Логи:        $LOGS_DIR/montana.log"
log "Приложение:  $APP_DEST"
log ""
log "Identity:"
"$INSTALL_DIR/montana-node" inspect --data-dir "$DATA_DIR" 2>&1 | grep -E "^(account_id|node_id|libp2p_peer_id|master_seed_fp)" | sed 's/^/  /'
log ""
log "Backup и второй Mac: см. Apps/macOS/INSTALL.md"
log ""
