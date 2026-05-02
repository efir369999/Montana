#!/bin/bash
# spec, разделы [C-7] No-shortcut на apply_*, [C-8] Mandatory SC trace block,
# [C-10] Mandatory deviation tracker
#
# Установка: создать симлинк из этого файла в .git/hooks/pre-commit:
#   ln -sf "$(pwd)/scripts/pre-commit.sh" .git/hooks/pre-commit
# (либо запустить scripts/install-hooks.sh)
#
# Этот hook применяется ТОЛЬКО на коммиты в crates/, не на role/spec правки.

set -e

REPO_ROOT="$(git rev-parse --show-toplevel)"
cd "$REPO_ROOT"

CHANGED=$(git diff --cached --name-only --diff-filter=ACM)
CONSENSUS_CHANGED=$(echo "$CHANGED" | grep -E 'crates/(mt-(state|account|entry|consensus|lottery|timechain)|montana-node)/.*\.rs$' || true)

# Gate 1 [C-8] SC trace block — проверяется в commit-msg hook (он получает
# commit message file как $1; pre-commit запускается ДО записи message
# и не имеет доступа к нему). См. scripts/commit-msg.sh.

# Gate 2 [C-10]: SPEC DEVIATION в коде требует DEV-N reference
BAD_DEV=$(git diff --cached -- 'crates/**/*.rs' 2>/dev/null | grep "^+.*SPEC DEVIATION" | grep -v "DEV-[0-9]" || true)
if [ -n "$BAD_DEV" ]; then
  echo "ОТКАЗ [C-10]: SPEC DEVIATION без DEV-N reference"
  echo "$BAD_DEV"
  echo
  echo "Каждое // SPEC DEVIATION: ОБЯЗАНО ссылаться на конкретный DEV-NNN entry в docs/SPEC_DEVIATIONS.md"
  exit 1
fi

# Gate 3 [C-10]: deviation count в коде vs SPEC_DEVIATIONS.md
if [ -f docs/SPEC_DEVIATIONS.md ]; then
  CODE_DEVS=$(grep -rcE "SPEC DEVIATION DEV-[0-9]+" crates/ 2>/dev/null | grep -v ":0$" | awk -F: '{s+=$2} END {print s+0}')
  DOC_DEVS=$(grep -c "^## DEV-" docs/SPEC_DEVIATIONS.md 2>/dev/null || echo 0)
  if [ "$CODE_DEVS" -gt "$DOC_DEVS" ]; then
    echo "ОТКАЗ [C-10]: $CODE_DEVS SPEC DEVIATION в коде, $DOC_DEVS entries в docs/SPEC_DEVIATIONS.md"
    echo "Каждый DEV-NNN в коде должен иметь entry в SPEC_DEVIATIONS.md"
    exit 1
  fi
fi

# Gate 4 [C-7]: запрет прямого mut на consensus state в node/example crates.
# Контекст ±3 строки: cargo fmt может переместить `// SPEC DEVIATION DEV-N`
# comment на отдельную строку (struct literal multi-line) — hook должен видеть
# deviation marker в окрестности, не только на той же строке.
# Gate 4 scope: production binary code только. cargo example bins
# (crates/*/examples/*.rs) — shakedown demos, не consensus path.
BAD_MUT=$(git diff --cached -U3 -- 'crates/montana-node/**/*.rs' ':!crates/*/examples/*.rs' 2>/dev/null \
  | awk '
    /^\+/ && /\.(insert|remove)\(/ && /(accounts|nodes|candidates|account_table|node_table|candidate_pool)/ {
      lines[NR] = $0
      target_line[NR] = 1
    }
    /SPEC DEVIATION DEV-/ {
      dev_line[NR] = 1
    }
    END {
      for (n in target_line) {
        has_dev = 0
        for (k = n - 3; k <= n + 3; k++) {
          if (dev_line[k]) { has_dev = 1; break }
        }
        if (!has_dev) print lines[n]
      }
    }
  ' \
  || true)
if [ -n "$BAD_MUT" ]; then
  echo "ОТКАЗ [C-7]: прямой mut-доступ к consensus state таблице вне apply_* функции"
  echo "$BAD_MUT"
  echo
  echo "Прямой insert/remove на AccountTable/NodeTable/CandidatePool разрешён только внутри apply_* функций соответствующего crate"
  echo "Если deviation legitimate — добавьте // SPEC DEVIATION DEV-NNN с обоснованием в docs/SPEC_DEVIATIONS.md в пределах ±3 строк от insert/remove"
  exit 1
fi

# Gate 5: четыре обязательные команды (workspace level)
# Этот gate тяжёлый — запускается только если consensus-critical изменены
if [ -n "$CONSENSUS_CHANGED" ]; then
  echo "Pre-commit: проверка fmt..."
  cargo fmt --all -- --check >/dev/null 2>&1 || {
    echo "ОТКАЗ: cargo fmt --all -- --check не прошёл"
    cargo fmt --all -- --check
    exit 1
  }
  echo "Pre-commit: проверка clippy..."
  cargo clippy --all-targets -- -D warnings >/dev/null 2>&1 || {
    echo "ОТКАЗ: cargo clippy не прошёл"
    cargo clippy --all-targets -- -D warnings 2>&1 | tail -30
    exit 1
  }
fi

echo "Pre-commit: все проверки пройдены"
