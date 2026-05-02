#!/bin/bash
DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
DATA="$DIR/data"
PLIST="$HOME/Library/LaunchAgents/org.montana.node.plist"
LABEL="org.montana.node"
LOG="$DATA/logs/montana.log"
ERR="$DATA/logs/montana.err.log"

clear
echo "=========================================="
echo "  Montana — запуск и логи узла"
echo "=========================================="
echo

if [ ! -f "$PLIST" ]; then
  echo "ОШИБКА: $PLIST не найден."
  echo "Запустите installer (install-local-mac.sh) сначала."
  echo
  read -n 1 -s -r -p "Нажмите любую клавишу чтобы закрыть окно..."
  exit 1
fi

# Запуск если не запущен
if ! launchctl list "$LABEL" >/dev/null 2>&1; then
  echo "Загружаю узел через launchd..."
  launchctl load -w "$PLIST"
  sleep 2
fi

PID=$(launchctl list "$LABEL" 2>/dev/null | awk '/PID/ {print $3}' | tr -d ';')
if [ -z "$PID" ] || [ "$PID" = "-" ]; then
  echo "ОШИБКА: служба не запущена. Логи: $ERR"
  echo
  read -n 1 -s -r -p "Нажмите любую клавишу чтобы закрыть окно..."
  exit 1
fi

while true; do
  printf '\033[3J\033[H\033[2J'
  echo "=========================================="
  echo "  Montana — узел работает (PID=$PID, Ctrl-C для выхода — узел продолжит)"
  echo "=========================================="
  echo
  "$DIR/montana-node" status --data-dir "$DATA" 2>/dev/null
  echo
  echo "------------------------------------------"
  echo "  Логи окон (последние 30, обновляется live)"
  echo "------------------------------------------"
  grep "^окно " "$LOG" 2>/dev/null | tail -n 30
  if [ -s "$ERR" ]; then
    echo
    echo "--- stderr ---"
    tail -n 5 "$ERR"
  fi
  sleep 1
done
