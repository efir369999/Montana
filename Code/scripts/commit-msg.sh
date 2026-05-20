#!/bin/bash
# spec, раздел [C-8] Mandatory SC trace block в commit message.
#
# commit-msg hook получает $1 = путь к commit message file.
# Запускается ПОСЛЕ pre-commit hook, имеет доступ к фактическому тексту
# commit message который сейчас будет применён.

set -e

REPO_ROOT="$(git rev-parse --show-toplevel)"
COMMIT_MSG_FILE="$1"

if [ -z "$COMMIT_MSG_FILE" ] || [ ! -f "$COMMIT_MSG_FILE" ]; then
  exit 0
fi

# Список изменённых consensus-critical файлов (staged for this commit)
CONSENSUS_CHANGED=$(cd "$REPO_ROOT" && git diff --cached --name-only --diff-filter=ACM \
  | grep -E 'crates/(mt-(state|account|entry|consensus|lottery|timechain)|montana-node)/.*\.rs$' \
  || true)

if [ -z "$CONSENSUS_CHANGED" ]; then
  # Не consensus-critical commit — SC trace не обязателен
  exit 0
fi

if ! grep -q "^SC trace:" "$COMMIT_MSG_FILE"; then
  echo "ОТКАЗ [C-8]: consensus-critical файлы изменены, в commit message отсутствует блок 'SC trace:'"
  echo
  echo "Изменены:"
  echo "$CONSENSUS_CHANGED" | sed 's/^/  /'
  echo
  echo "Добавьте в commit message блок (с строки начинающейся 'SC trace:'):"
  echo
  echo "  SC trace:"
  echo "    Spec section:    \"<название раздела>\""
  echo "    Spec quote:      \"<дословная цитата>\""
  echo "    Code location:   crates/<crate>/src/<file>:NNN-MMM"
  echo "    Test:            crates/<crate>/tests/<test>.rs::<fn>"
  echo "    Inv check:       [I-X, C-Y, ...]"
  echo "    Deviation count: 0 (либо N с reference на DEV-NNN)"
  exit 1
fi

exit 0
