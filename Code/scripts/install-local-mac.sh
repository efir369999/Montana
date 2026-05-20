#!/bin/bash
# Montana node — локальная установка на macOS из текущей рабочей копии.
# БЕЗ клонирования с GitHub. Все процессы автоматически по очереди:
# build → install → identity → launchd → start → Finder.
#
# Использование (одна команда в Terminal.app):
#   bash "/Users/kh./Python/Ничто/Montana/Russian/Протокол/Code/scripts/install-local-mac.sh"
#
# Узел запускается через launchd (Mac-аналог systemd) — переживает logout
# и перезагрузку Mac, автоматически рестартует при падении.

set -euo pipefail

SCRIPT_PATH="${BASH_SOURCE[0]}"
if [ -L "$SCRIPT_PATH" ]; then
  SCRIPT_PATH="$(readlink "$SCRIPT_PATH")"
fi
SCRIPT_DIR="$(cd "$(dirname "$SCRIPT_PATH")" && pwd)"
SOURCE_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

# Локация установки: задаётся env var INSTALL_DIR либо default ~/Applications/Montana.
# Пример с custom путём:
#   INSTALL_DIR=/path/to/Montana_Node bash scripts/install-local-mac.sh
INSTALL_DIR="${INSTALL_DIR:-$HOME/Applications/Montana}"
DATA_DIR="$INSTALL_DIR/data"
LOGS_DIR="$DATA_DIR/logs"
LAUNCH_AGENTS_DIR="$HOME/Library/LaunchAgents"
PLIST_PATH="$LAUNCH_AGENTS_DIR/org.montana.node.plist"
SERVICE_LABEL="org.montana.node"

log() { printf '\033[1;32m[install-local-mac]\033[0m %s\n' "$*"; }
warn() { printf '\033[1;33m[install-local-mac]\033[0m %s\n' "$*" >&2; }
die() { printf '\033[1;31m[install-local-mac] ОШИБКА:\033[0m %s\n' "$*" >&2; exit 1; }

# --- Шаг 1: проверка платформы ---
[ "$(uname -s)" = "Darwin" ] || die "это installer для macOS"
log "macOS $(sw_vers -productVersion 2>/dev/null || echo unknown), arch $(uname -m)"
log "источник: $SOURCE_DIR"

# --- Шаг 2: проверка структуры репозитория ---
[ -f "$SOURCE_DIR/Cargo.toml" ] || die "не найден Cargo.toml в $SOURCE_DIR"
[ -d "$SOURCE_DIR/crates/montana-node" ] || die "не найден crates/montana-node"
[ -d "$SOURCE_DIR/dist/macOS/Montana" ] || die "не найден dist/macOS/Montana"

# --- Шаг 3: Xcode Command Line Tools ---
if ! xcode-select -p >/dev/null 2>&1; then
  warn "Xcode Command Line Tools не установлены."
  warn "Откроется системный диалог — нажмите «Установить» и дождитесь завершения."
  xcode-select --install || true
  die "после установки CLT повторите команду"
fi

# --- Шаг 4: Rust toolchain ---
if ! command -v cargo >/dev/null 2>&1; then
  log "устанавливаю Rust toolchain (rustup minimal)..."
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | \
    sh -s -- -y --default-toolchain stable --profile minimal --no-modify-path
fi
export PATH="$HOME/.cargo/bin:$PATH"
command -v cargo >/dev/null 2>&1 || die "cargo не доступен после rustup install"
log "Rust: $(cargo --version)"

# --- Шаг 5: остановить running узел (если есть) ---
if launchctl list "$SERVICE_LABEL" >/dev/null 2>&1; then
  log "останавливаю текущий узел перед обновлением бинаря..."
  launchctl unload "$PLIST_PATH" 2>/dev/null || true
fi

# --- Шаг 6: build ---
cd "$SOURCE_DIR"
log "собираю montana-node release..."
cargo build --release -p montana-node 2>&1 | tail -3

# --- Шаг 7: install в ~/Applications/Montana/ ---
mkdir -p "$INSTALL_DIR" "$DATA_DIR" "$LOGS_DIR" "$LAUNCH_AGENTS_DIR"
cp -f "$SOURCE_DIR/target/release/montana-node" "$INSTALL_DIR/montana-node"
chmod 0755 "$INSTALL_DIR/montana-node"
cp -f "$SOURCE_DIR/dist/macOS/Montana"/*.command "$INSTALL_DIR/" 2>/dev/null || true
cp -f "$SOURCE_DIR/dist/macOS/Montana/README.txt" "$INSTALL_DIR/"
chmod 0755 "$INSTALL_DIR"/*.command 2>/dev/null || true
xattr -dr com.apple.quarantine "$INSTALL_DIR" 2>/dev/null || true

# --- Шаг 8: identity (init если нет) ---
if [ ! -f "$DATA_DIR/identity.bin" ]; then
  log ""
  log "================================================================"
  log "  ГЕНЕРАЦИЯ IDENTITY"
  log "================================================================"
  log ""
  log "Сейчас сгенерируются 24 слова мнемоники + ключи ML-DSA-65 +"
  log "ML-KEM-768. ВНИМАНИЕ: запишите 24 слова в надёжное место."
  log "Потеряете → потеряете доступ к узлу и всем заработанным Ɉ."
  log ""
  "$INSTALL_DIR/montana-node" init --data-dir "$DATA_DIR"
  log ""
  log "================================================================"
  if [ -t 0 ] || [ -e /dev/tty ]; then
    read -r -p "Нажмите Enter ПОСЛЕ того как записали 24 слова в надёжное место..." _ </dev/tty || true
  fi
else
  log "identity уже существует ($DATA_DIR/identity.bin) — пропускаю генерацию"
fi

# --- Шаг 9: установка LaunchAgent plist ---
log "устанавливаю LaunchAgent $PLIST_PATH..."
cat > "$PLIST_PATH" <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>$SERVICE_LABEL</string>
    <key>ProgramArguments</key>
    <array>
        <string>$INSTALL_DIR/montana-node</string>
        <string>start</string>
        <string>--data-dir</string>
        <string>$DATA_DIR</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <dict>
        <key>SuccessfulExit</key>
        <false/>
        <key>Crashed</key>
        <true/>
    </dict>
    <key>ThrottleInterval</key>
    <integer>10</integer>
    <key>StandardOutPath</key>
    <string>$LOGS_DIR/montana.log</string>
    <key>StandardErrorPath</key>
    <string>$LOGS_DIR/montana.err.log</string>
    <key>WorkingDirectory</key>
    <string>$DATA_DIR</string>
    <key>EnvironmentVariables</key>
    <dict>
        <key>PATH</key>
        <string>/usr/local/bin:/usr/bin:/bin</string>
    </dict>
    <key>ProcessType</key>
    <string>Standard</string>
    <key>LowPriorityIO</key>
    <false/>
    <key>Nice</key>
    <integer>0</integer>
</dict>
</plist>
PLIST
chmod 0644 "$PLIST_PATH"

# --- Шаг 10: запуск через launchd ---
log "запускаю узел через launchctl load..."
launchctl load -w "$PLIST_PATH"

# Дать узлу секунду чтобы стартовать
sleep 2

# --- Шаг 11: проверка что запустилось ---
if launchctl list "$SERVICE_LABEL" 2>/dev/null | grep -q "$SERVICE_LABEL"; then
  STATE=$(launchctl list "$SERVICE_LABEL" | awk '/PID/ {print $3}')
  if [ -n "$STATE" ] && [ "$STATE" != "-" ]; then
    log "узел запущен, PID=$STATE"
  else
    warn "узел загружен в launchd, но PID не получен. Проверьте логи: $LOGS_DIR/montana.err.log"
  fi
else
  die "не удалось загрузить службу $SERVICE_LABEL"
fi

# --- Шаг 12: финальный отчёт + Finder ---
log ""
log "================================================================"
log "  УСТАНОВКА ЗАВЕРШЕНА. УЗЕЛ ЗАПУЩЕН."
log "================================================================"
log ""
log "Бинарь:    $INSTALL_DIR/montana-node"
log "Данные:    $DATA_DIR"
log "Логи:      $LOGS_DIR/montana.log (stdout) + montana.err.log (stderr)"
log "Service:   $SERVICE_LABEL (launchd, RunAtLoad=true, restart on crash)"
log ""
log "Узел работает в фоне через launchd:"
log "  • переживает закрытие Terminal.app"
log "  • переживает logout"
log "  • переживает перезагрузку Mac (auto-start при логине)"
log "  • перезапускается автоматически при падении"
log ""
log "Полезные команды:"
log "  tail -f \"$LOGS_DIR/montana.log\"                    # логи realtime"
log "  $INSTALL_DIR/montana-node status --data-dir \"$DATA_DIR\"   # phase + balance"
log "  launchctl unload \"$PLIST_PATH\"             # остановить узел"
log "  launchctl load -w \"$PLIST_PATH\"            # запустить заново"
log ""
log "Жизненный цикл узла (canonical apply_proposal pipeline byte-exact spec):"
log "  Phase 1: Bootstrap → CandidateVdf  (~10 часов VDF до vdf_chain_length ≥ τ₂)"
log "  Phase 2: CandidateVdf → Registered (apply_noderegistrations_batch)"
log "  Phase 3: Registered → Active       (apply_selection_event на W % 336 == 0)"
log "  Phase 4: Active                    (13 Ɉ per окно через apply_proposal)"
log ""
log "Для повторной установки (после изменений в коде):"
log "  bash \"$SCRIPT_PATH\""
log ""
open "$INSTALL_DIR" 2>/dev/null || warn "не удалось открыть Finder автоматически"
