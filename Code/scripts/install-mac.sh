#!/bin/bash
# Montana node — установка одной командой на macOS.
#
# Использование (одна строка в Terminal.app):
#   curl -sSL https://raw.githubusercontent.com/efir369999/Montana/main/Code/scripts/install-mac.sh | bash
#
# Что делает:
#   1. Проверяет macOS + Xcode CLT (запрашивает установку если нет)
#   2. Ставит Rust toolchain через rustup (если нет)
#   3. Клонирует репозиторий в $HOME/.cache/montana-source
#   4. Собирает release бинарь
#   5. Копирует бинарь + .command-обёртки в ~/Applications/Montana/
#   6. Открывает Finder в папке узла
#
# НЕ требует sudo. Всё в $HOME пользователя.

set -euo pipefail

REPO_URL="${MONTANA_REPO_URL:-https://github.com/efir369999/Montana.git}"
REPO_BRANCH="${MONTANA_REPO_BRANCH:-main}"
SOURCE_CACHE="$HOME/.cache/montana-source"
# Локация установки: env var INSTALL_DIR либо default ~/Applications/Montana.
INSTALL_DIR="${INSTALL_DIR:-$HOME/Applications/Montana}"
DATA_DIR="$INSTALL_DIR/data"

log() { printf '\033[1;32m[install-mac]\033[0m %s\n' "$*"; }
warn() { printf '\033[1;33m[install-mac]\033[0m %s\n' "$*" >&2; }
die() { printf '\033[1;31m[install-mac] ОШИБКА:\033[0m %s\n' "$*" >&2; exit 1; }

# --- Шаг 1: проверка платформы ---
if [ "$(uname -s)" != "Darwin" ]; then
  die "это installer для macOS. Для Linux VPS используйте install-vps.sh"
fi
log "обнаружен macOS $(sw_vers -productVersion 2>/dev/null || echo unknown), arch $(uname -m)"

# --- Шаг 2: Xcode Command Line Tools ---
if ! xcode-select -p >/dev/null 2>&1; then
  warn "Xcode Command Line Tools не установлены."
  warn "Сейчас откроется системный диалог установки CLT — нажмите «Установить»."
  warn "После завершения установки запустите этот скрипт снова."
  xcode-select --install || true
  die "дождитесь окончания установки Xcode CLT и повторите команду"
fi

# --- Шаг 3: Rust toolchain ---
if ! command -v cargo >/dev/null 2>&1; then
  log "устанавливаю Rust toolchain (rustup minimal)..."
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | \
    sh -s -- -y --default-toolchain stable --profile minimal --no-modify-path
fi
export PATH="$HOME/.cargo/bin:$PATH"
if ! command -v cargo >/dev/null 2>&1; then
  die "cargo не доступен после rustup install. Перезапустите Terminal."
fi
log "Rust: $(cargo --version)"

# --- Шаг 4: clone/update repo ---
mkdir -p "$(dirname "$SOURCE_CACHE")"
if [ -d "$SOURCE_CACHE/.git" ]; then
  log "обновляю репозиторий $SOURCE_CACHE..."
  cd "$SOURCE_CACHE"
  git fetch origin "$REPO_BRANCH"
  git reset --hard "origin/$REPO_BRANCH"
else
  log "клонирую $REPO_URL (branch $REPO_BRANCH) → $SOURCE_CACHE..."
  rm -rf "$SOURCE_CACHE"
  git clone --branch "$REPO_BRANCH" --single-branch "$REPO_URL" "$SOURCE_CACHE"
fi

# --- Шаг 5: build бинарь ---
SOURCE_DIR="$SOURCE_CACHE/Code"
if [ ! -d "$SOURCE_DIR" ]; then
  die "директория '$SOURCE_DIR' не найдена в репозитории"
fi
cd "$SOURCE_DIR"
log "собираю montana-node release (5-10 минут на первом запуске)..."
cargo build --release -p montana-node 2>&1 | tail -3

# --- Шаг 6: install в ~/Applications/Montana/ ---
mkdir -p "$INSTALL_DIR" "$DATA_DIR"

log "копирую бинарь и обёртки в $INSTALL_DIR..."
cp -f target/release/montana-node "$INSTALL_DIR/montana-node"
chmod 0755 "$INSTALL_DIR/montana-node"

cp -f "$SOURCE_DIR/dist/macOS/Montana"/*.command "$INSTALL_DIR/"
cp -f "$SOURCE_DIR/dist/macOS/Montana/README.txt" "$INSTALL_DIR/"
chmod 0755 "$INSTALL_DIR"/*.command

# Снять quarantine attribute если есть (на случай если файлы скачивались через Safari)
xattr -dr com.apple.quarantine "$INSTALL_DIR" 2>/dev/null || true

# --- Шаг 7: финальный отчёт + открытие Finder ---
log ""
log "================================================================"
log "  УСТАНОВКА ЗАВЕРШЕНА"
log "================================================================"
log ""
log "Бинарь:        $INSTALL_DIR/montana-node"
log "Данные:        $DATA_DIR"
log "Обёртки:       $INSTALL_DIR/*.command"
log ""
log "Дальнейшие шаги:"
log "  1. Дабл-клик «1. Создать identity»  — генерирует 24 слова + ключи"
log "  2. Дабл-клик «6. Запустить узел»    — БОЕВОЙ РЕЖИМ, реальный VDF"
log ""
log "Жизненный цикл узла на M-class Mac (production D из Genesis Decree (mt-genesis)):"
log "  Phase 1: Bootstrap → CandidateVdf  (~10 часов wall-clock VDF)"
log "  Phase 2: CandidateVdf → Registered (NodeRegistration через canonical)"
log "  Phase 3: Registered → Active       (selection event на W % 336 == 0)"
log "  Phase 4: Active                    (13 Ɉ per окно через apply_proposal)"
log ""
log "Открываю Finder в папке узла..."
open "$INSTALL_DIR" 2>/dev/null || warn "не удалось открыть Finder автоматически — откройте $INSTALL_DIR вручную"
