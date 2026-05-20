#!/bin/bash
# Установка git pre-commit hook ([C-7]/[C-8]/[C-10] enforcement).
# Запустить один раз после клона репозитория:
#   bash scripts/install-hooks.sh

set -e

REPO_ROOT="$(git rev-parse --show-toplevel)"
mkdir -p "$REPO_ROOT/.git/hooks"

install_hook() {
  local name="$1"
  local src="$REPO_ROOT/scripts/${name}.sh"
  local dst="$REPO_ROOT/.git/hooks/${name}"
  if [ ! -f "$src" ]; then
    echo "ОШИБКА: $src не найден"
    exit 1
  fi
  chmod +x "$src"
  if [ -L "$dst" ]; then
    rm "$dst"
  elif [ -f "$dst" ]; then
    echo "ВНИМАНИЕ: $dst уже существует. Сохраняю как .backup"
    mv "$dst" "$dst.backup"
  fi
  ln -s "$src" "$dst"
  echo "  $dst → $src"
}

echo "Установка git hooks:"
install_hook "pre-commit"
install_hook "commit-msg"
echo
echo "Активные gate-ы:"
echo "  [C-7]  No-shortcut на apply_* (запрет прямого mut на state-таблицах)"
echo "  [C-8]  Mandatory SC trace block в commit message"
echo "  [C-10] Mandatory deviation tracker (SPEC_DEVIATIONS.md sync)"
echo "  +      cargo fmt + cargo clippy на consensus-critical commits"
