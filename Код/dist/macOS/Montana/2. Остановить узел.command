#!/bin/bash
PLIST="$HOME/Library/LaunchAgents/org.montana.node.plist"
LABEL="org.montana.node"
clear
echo "=========================================="
echo "  Montana — остановка узла"
echo "=========================================="
echo

if [ ! -f "$PLIST" ]; then
  echo "узел не установлен через launchd ($PLIST не найден)"
  echo
  read -n 1 -s -r -p "Нажмите любую клавишу чтобы закрыть окно..."
  exit 0
fi

if ! launchctl list "$LABEL" >/dev/null 2>&1; then
  echo "узел уже остановлен."
  echo
  read -n 1 -s -r -p "Нажмите любую клавишу чтобы закрыть окно..."
  exit 0
fi

echo "Останавливаю узел и убираю из автозапуска..."
launchctl unload -w "$PLIST" 2>/dev/null || true
sleep 1

if launchctl list "$LABEL" >/dev/null 2>&1; then
  echo
  echo "ВНИМАНИЕ: узел всё ещё в списке launchd. Попробуйте ещё раз."
else
  echo
  echo "ГОТОВО. Узел остановлен."
  echo "State сохранён в data/ — следующий запуск продолжит с того окна."
  echo
  echo "Чтобы запустить снова — дабл-клик «1. Запуск и логи узла»."
fi
echo
read -n 1 -s -r -p "Нажмите любую клавишу чтобы закрыть окно..."
