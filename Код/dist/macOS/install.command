#!/bin/bash
set -e
DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
REPO_ROOT="$( cd "$DIR/../.." && pwd )"
SRC="$DIR/Montana"
DEST="${INSTALL_DIR:-$HOME/Applications/Montana}"
BIN_BUILT="$REPO_ROOT/target/release/montana-node"

clear
echo "=========================================="
echo "  Montana — установка узла на macOS"
echo "=========================================="
echo
echo "Repo:        $REPO_ROOT"
echo "Источник:    $SRC"
echo "Назначение:  $DEST"
echo

if [ ! -x "$BIN_BUILT" ]; then
  echo "Бинарь не найден. Собираю release..."
  cd "$REPO_ROOT"
  cargo build --release -p montana-node
  echo
fi

echo "Создаю $DEST"
mkdir -p "$DEST/data"

echo "Копирую бинарь"
cp -f "$BIN_BUILT" "$DEST/montana-node"
chmod +x "$DEST/montana-node"

echo "Копирую обёртки и README"
cp -f "$SRC"/*.command "$DEST/"
cp -f "$SRC/README.txt" "$DEST/"
chmod +x "$DEST"/*.command

echo "Снимаю quarantine (если есть)"
xattr -dr com.apple.quarantine "$DEST" 2>/dev/null || true

echo
echo "------------------------------------------"
echo "  УСТАНОВЛЕНО."
echo "  Открываю Finder в папке узла..."
echo "------------------------------------------"
open "$DEST"
echo
read -n 1 -s -r -p "Нажмите любую клавишу чтобы закрыть это окно..."
echo
