#!/usr/bin/env bash
# scripts/check_doc_drift.sh
#
# Защита от documentation drift между фактическим кодом и audit package.
# Закрывает class ошибок обнаруженных внешним аудитом Claude Opus 4.7
# (отчёт #1 F-1/F-4/F-18, отчёт #2 M2-12/DOC-1/DOC-2): AUDIT.md /
# audit-checklist.md / security-cards.md повторяют устаревшие числа
# (line counts, unsafe block count, domain count).
#
# Стратегия: вычислить факт из исходного кода (wc / grep) и сверить с
# заявлениями в .md файлах. При mismatch — exit 1 с понятной diagnostic.
#
# Запускается:
#   - Локально перед commit (git pre-commit hook опционально)
#   - В CI как gate doc_drift
#
# Использование:
#   ./scripts/check_doc_drift.sh
#
# Exit code: 0 (clean) | 1 (drift detected, listing напечатан в stderr)

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"

EXIT=0
errors=()

err() {
    EXIT=1
    errors+=("$1")
    echo "FAIL: $1" >&2
}

ok() {
    echo "ok:   $1"
}

# ---------- 1. mt-crypto/src/lib.rs line count ----------
ACTUAL_MT_CRYPTO_LOC=$(wc -l < crates/mt-crypto/src/lib.rs | tr -d ' ')

# AUDIT.md заявляет в строке: | `mt-crypto` | ... | **NNN** | ... |
CLAIMED_MT_CRYPTO_LOC=$(grep -E '^\| `mt-crypto` \|' AUDIT.md | grep -oE '\*\*[0-9]+\*\*' | head -1 | tr -d '*')

if [ -z "$CLAIMED_MT_CRYPTO_LOC" ]; then
    err "AUDIT.md не содержит mt-crypto LOC claim в expected format '**NNN**'"
elif [ "$ACTUAL_MT_CRYPTO_LOC" != "$CLAIMED_MT_CRYPTO_LOC" ]; then
    err "AUDIT.md mt-crypto LOC drift: claimed $CLAIMED_MT_CRYPTO_LOC, actual $ACTUAL_MT_CRYPTO_LOC"
else
    ok "AUDIT.md mt-crypto LOC = $ACTUAL_MT_CRYPTO_LOC"
fi

# audit-checklist.md заявляет: Layer 1 Rust shim** **NNN строк** —
# LOC после "shim** **" чтобы не зацепить "Layer 1" prefix
CHECKLIST_MT_CRYPTO_LOC=$(grep -oE 'Rust shim\*\* \*\*[0-9]+ строк' docs/audit-checklist.md | grep -oE '\*\*[0-9]+ строк' | grep -oE '[0-9]+')
if [ -z "$CHECKLIST_MT_CRYPTO_LOC" ]; then
    err "audit-checklist.md не содержит 'Layer 1 Rust shim **NNN строк**' claim"
elif [ "$ACTUAL_MT_CRYPTO_LOC" != "$CHECKLIST_MT_CRYPTO_LOC" ]; then
    err "audit-checklist.md mt-crypto LOC drift: claimed $CHECKLIST_MT_CRYPTO_LOC, actual $ACTUAL_MT_CRYPTO_LOC"
else
    ok "audit-checklist.md mt-crypto LOC = $ACTUAL_MT_CRYPTO_LOC"
fi

# ---------- 2. unsafe blocks в mt-crypto ----------
# Все формы: `unsafe {`, `unsafe fn`, `let _ = unsafe {`, etc.
# Считаем все unsafe keyword tokens (excluding /// doc comments + // comments)
ACTUAL_UNSAFE=$(grep -E '\bunsafe\b' crates/mt-crypto/src/lib.rs | grep -vE '^\s*///|^\s*//[^/]' | wc -l | tr -d ' ')

# AUDIT.md фразирует "Все **N** `unsafe` blocks"
CLAIMED_UNSAFE=$(grep -oE 'Все \*\*[0-9]+\*\* `unsafe` blocks' AUDIT.md | grep -oE '[0-9]+' | head -1)
if [ -z "$CLAIMED_UNSAFE" ]; then
    err "AUDIT.md не содержит 'Все **N** \`unsafe\` blocks' claim"
elif [ "$ACTUAL_UNSAFE" != "$CLAIMED_UNSAFE" ]; then
    err "AUDIT.md unsafe count drift: claimed $CLAIMED_UNSAFE, actual $ACTUAL_UNSAFE"
else
    ok "AUDIT.md unsafe count = $ACTUAL_UNSAFE"
fi

# ---------- 3. Domain separators в mt-codec ----------
ACTUAL_DOMAINS=$(grep -cE '^\s*pub const [A-Z_]+: &\[u8\] = b"mt-' crates/mt-codec/src/lib.rs)

# AUDIT.md заявляет "Domain separators registry (NN domains, ..."
CLAIMED_DOMAINS=$(grep -oE 'Domain separators registry \([0-9]+ domains' AUDIT.md | grep -oE '[0-9]+' | head -1)
if [ -z "$CLAIMED_DOMAINS" ]; then
    err "AUDIT.md не содержит 'Domain separators registry (NN domains' claim"
elif [ "$ACTUAL_DOMAINS" != "$CLAIMED_DOMAINS" ]; then
    err "AUDIT.md domain count drift: claimed $CLAIMED_DOMAINS, actual $ACTUAL_DOMAINS"
else
    ok "AUDIT.md domain count = $ACTUAL_DOMAINS"
fi

# AUDIT.md row "Domain registry sync (NN domains)"
SECONDARY_DOMAINS=$(grep -oE 'Domain registry sync \([0-9]+ domains\)' AUDIT.md | grep -oE '[0-9]+' | head -1)
if [ -n "$SECONDARY_DOMAINS" ] && [ "$ACTUAL_DOMAINS" != "$SECONDARY_DOMAINS" ]; then
    err "AUDIT.md secondary domain count drift: claimed $SECONDARY_DOMAINS, actual $ACTUAL_DOMAINS"
fi

# ---------- 4. mt-crypto-native C wrapper LOC ----------
ACTUAL_C_LOC=$(wc -l < crates/mt-crypto-native/csrc/mt_crypto.c | tr -d ' ')
ACTUAL_H_LOC=$(wc -l < crates/mt-crypto-native/csrc/mt_crypto.h | tr -d ' ')

# AUDIT.md `mt_crypto.c | NNN |` — Layer 2 row
CLAIMED_C_LOC=$(grep -E 'mt_crypto.c\)\s*\|\s*[0-9]+' AUDIT.md | head -1 | grep -oE '\| [0-9]+ \|' | head -1 | grep -oE '[0-9]+')
if [ -n "$CLAIMED_C_LOC" ] && [ "$ACTUAL_C_LOC" != "$CLAIMED_C_LOC" ]; then
    err "AUDIT.md mt_crypto.c LOC drift: claimed $CLAIMED_C_LOC, actual $ACTUAL_C_LOC"
elif [ -n "$CLAIMED_C_LOC" ]; then
    ok "AUDIT.md mt_crypto.c LOC = $ACTUAL_C_LOC"
fi

CLAIMED_H_LOC=$(grep -E 'mt_crypto.h\)\s*\|\s*[0-9]+' AUDIT.md | head -1 | grep -oE '\| [0-9]+ \|' | head -1 | grep -oE '[0-9]+')
if [ -n "$CLAIMED_H_LOC" ] && [ "$ACTUAL_H_LOC" != "$CLAIMED_H_LOC" ]; then
    err "AUDIT.md mt_crypto.h LOC drift: claimed $CLAIMED_H_LOC, actual $ACTUAL_H_LOC"
elif [ -n "$CLAIMED_H_LOC" ]; then
    ok "AUDIT.md mt_crypto.h LOC = $ACTUAL_H_LOC"
fi

# ---------- 5. Total audit surface ----------
RUST_BINDING_LOC=$(wc -l < crates/mt-crypto-native/src/lib.rs | tr -d ' ')
BUILD_RS_LOC=$(wc -l < crates/mt-crypto-native/build.rs | tr -d ' ')
ACTUAL_TOTAL=$((ACTUAL_MT_CRYPTO_LOC + RUST_BINDING_LOC + ACTUAL_C_LOC + ACTUAL_H_LOC + BUILD_RS_LOC))

# AUDIT.md "Total own audit surface (Layer 1 + Layer 2): NNNN lines"
CLAIMED_TOTAL=$(grep -oE 'Total own audit surface \(Layer 1 \+ Layer 2\): [0-9]+ lines' AUDIT.md | grep -oE '[0-9]+ lines' | grep -oE '[0-9]+' | head -1)
if [ -z "$CLAIMED_TOTAL" ]; then
    err "AUDIT.md не содержит 'Total own audit surface (Layer 1 + Layer 2): NNNN lines' claim"
elif [ "$ACTUAL_TOTAL" != "$CLAIMED_TOTAL" ]; then
    err "AUDIT.md total audit surface drift: claimed $CLAIMED_TOTAL, actual $ACTUAL_TOTAL ($ACTUAL_MT_CRYPTO_LOC + $RUST_BINDING_LOC + $ACTUAL_C_LOC + $ACTUAL_H_LOC + $BUILD_RS_LOC)"
else
    ok "AUDIT.md total audit surface = $ACTUAL_TOTAL"
fi

# ---------- 6. mt-account LOC (M3 audit scope) ----------
ACTUAL_MT_ACCOUNT_LOC=$(wc -l < crates/mt-account/src/lib.rs | tr -d ' ')

# AUDIT.md row: | `mt-account` | ... | **NNNN** | ... |
CLAIMED_MT_ACCOUNT_LOC=$(grep -E '^\| `mt-account` \|' AUDIT.md | grep -oE '\*\*[0-9]+\*\*' | head -1 | tr -d '*')

if [ -z "$CLAIMED_MT_ACCOUNT_LOC" ]; then
    err "AUDIT.md не содержит mt-account LOC claim в expected format '**NNNN**'"
elif [ "$ACTUAL_MT_ACCOUNT_LOC" != "$CLAIMED_MT_ACCOUNT_LOC" ]; then
    err "AUDIT.md mt-account LOC drift: claimed $CLAIMED_MT_ACCOUNT_LOC, actual $ACTUAL_MT_ACCOUNT_LOC"
else
    ok "AUDIT.md mt-account LOC = $ACTUAL_MT_ACCOUNT_LOC"
fi

# AUDIT.md TL;DR table: M3 row "mt-account | NNNN | ..."
TLDR_MT_ACCOUNT_LOC=$(grep -E 'M3 apply_proposal layer.*mt-account' AUDIT.md | grep -oE '\| [0-9]+ \|' | head -1 | grep -oE '[0-9]+')
if [ -n "$TLDR_MT_ACCOUNT_LOC" ] && [ "$ACTUAL_MT_ACCOUNT_LOC" != "$TLDR_MT_ACCOUNT_LOC" ]; then
    err "AUDIT.md TL;DR mt-account LOC drift: claimed $TLDR_MT_ACCOUNT_LOC, actual $ACTUAL_MT_ACCOUNT_LOC"
fi

# ---------- 7. mt-account test count ----------
ACTUAL_MT_ACCOUNT_TESTS=$(grep -cE '#\[test\]' crates/mt-account/src/lib.rs)

# AUDIT.md TL;DR заявляет: "NN unit + 35 determinism invariants"
CLAIMED_MT_ACCOUNT_TESTS=$(grep -oE '[0-9]+ unit \+ [0-9]+ determinism invariants' AUDIT.md | head -1 | grep -oE '^[0-9]+')
if [ -z "$CLAIMED_MT_ACCOUNT_TESTS" ]; then
    err "AUDIT.md не содержит mt-account 'NN unit + 35 determinism invariants' claim"
elif [ "$ACTUAL_MT_ACCOUNT_TESTS" != "$CLAIMED_MT_ACCOUNT_TESTS" ]; then
    err "AUDIT.md mt-account unit test count drift: claimed $CLAIMED_MT_ACCOUNT_TESTS, actual $ACTUAL_MT_ACCOUNT_TESTS"
else
    ok "AUDIT.md mt-account unit test count = $ACTUAL_MT_ACCOUNT_TESTS"
fi

# ---------- 8. mt-account determinism_invariants tests ----------
ACTUAL_M3_DETERMINISM=$(grep -cE '#\[test\]' crates/mt-account/tests/determinism_invariants.rs)

# Check that 29 (M3 number after v34 monetary refactor) appears
if ! grep -q "29 determinism invariants" AUDIT.md; then
    err "AUDIT.md не упоминает '29 determinism invariants' для M3"
elif [ "$ACTUAL_M3_DETERMINISM" != "29" ]; then
    err "M3 determinism_invariants count drift: claimed 29, actual $ACTUAL_M3_DETERMINISM"
else
    ok "AUDIT.md M3 determinism invariants = 29 (matches actual $ACTUAL_M3_DETERMINISM)"
fi

# ---------- 9..12: M4+M5 crate LOC + test counts ----------

# M4 mt-lottery
ACTUAL_MT_LOTTERY_LOC=$(wc -l < crates/mt-lottery/src/lib.rs | tr -d ' ')
CLAIMED_MT_LOTTERY_LOC=$(grep -E '^\| `mt-lottery` \|' AUDIT.md | grep -oE '\*\*[0-9]+\*\*' | head -1 | tr -d '*')
if [ -z "$CLAIMED_MT_LOTTERY_LOC" ]; then
    err "AUDIT.md не содержит mt-lottery LOC claim в expected format '**NNNN**'"
elif [ "$ACTUAL_MT_LOTTERY_LOC" != "$CLAIMED_MT_LOTTERY_LOC" ]; then
    err "AUDIT.md mt-lottery LOC drift: claimed $CLAIMED_MT_LOTTERY_LOC, actual $ACTUAL_MT_LOTTERY_LOC"
else
    ok "AUDIT.md mt-lottery LOC = $ACTUAL_MT_LOTTERY_LOC"
fi

# M4 mt-consensus
ACTUAL_MT_CONSENSUS_LOC=$(wc -l < crates/mt-consensus/src/lib.rs | tr -d ' ')
CLAIMED_MT_CONSENSUS_LOC=$(grep -E '^\| `mt-consensus` \|' AUDIT.md | grep -oE '\*\*[0-9]+\*\*' | head -1 | tr -d '*')
if [ -z "$CLAIMED_MT_CONSENSUS_LOC" ]; then
    err "AUDIT.md не содержит mt-consensus LOC claim"
elif [ "$ACTUAL_MT_CONSENSUS_LOC" != "$CLAIMED_MT_CONSENSUS_LOC" ]; then
    err "AUDIT.md mt-consensus LOC drift: claimed $CLAIMED_MT_CONSENSUS_LOC, actual $ACTUAL_MT_CONSENSUS_LOC"
else
    ok "AUDIT.md mt-consensus LOC = $ACTUAL_MT_CONSENSUS_LOC"
fi

# M4 mt-entry
ACTUAL_MT_ENTRY_LOC=$(wc -l < crates/mt-entry/src/lib.rs | tr -d ' ')
CLAIMED_MT_ENTRY_LOC=$(grep -E '^\| `mt-entry` \|' AUDIT.md | grep -oE '\*\*[0-9]+\*\*' | head -1 | tr -d '*')
if [ -z "$CLAIMED_MT_ENTRY_LOC" ]; then
    err "AUDIT.md не содержит mt-entry LOC claim"
elif [ "$ACTUAL_MT_ENTRY_LOC" != "$CLAIMED_MT_ENTRY_LOC" ]; then
    err "AUDIT.md mt-entry LOC drift: claimed $CLAIMED_MT_ENTRY_LOC, actual $ACTUAL_MT_ENTRY_LOC"
else
    ok "AUDIT.md mt-entry LOC = $ACTUAL_MT_ENTRY_LOC"
fi

# M5 mt-store
ACTUAL_MT_STORE_LOC=$(wc -l < crates/mt-store/src/lib.rs | tr -d ' ')
CLAIMED_MT_STORE_LOC=$(grep -E '^\| `mt-store` \|' AUDIT.md | grep -oE '\*\*[0-9]+\*\*' | head -1 | tr -d '*')
if [ -z "$CLAIMED_MT_STORE_LOC" ]; then
    err "AUDIT.md не содержит mt-store LOC claim"
elif [ "$ACTUAL_MT_STORE_LOC" != "$CLAIMED_MT_STORE_LOC" ]; then
    err "AUDIT.md mt-store LOC drift: claimed $CLAIMED_MT_STORE_LOC, actual $ACTUAL_MT_STORE_LOC"
else
    ok "AUDIT.md mt-store LOC = $ACTUAL_MT_STORE_LOC"
fi

# M4 total surface — extract число между ":** " и " lines" (skip M4 milestone digit)
ACTUAL_M4_TOTAL=$((ACTUAL_MT_LOTTERY_LOC + ACTUAL_MT_CONSENSUS_LOC + ACTUAL_MT_ENTRY_LOC))
CLAIMED_M4_TOTAL=$(grep -oE 'Total M4 audit surface:\*\* [0-9]+ lines' AUDIT.md | awk -F'[*: ]+' '{print $(NF-1)}')
if [ -n "$CLAIMED_M4_TOTAL" ] && [ "$ACTUAL_M4_TOTAL" != "$CLAIMED_M4_TOTAL" ]; then
    err "AUDIT.md M4 total surface drift: claimed $CLAIMED_M4_TOTAL, actual $ACTUAL_M4_TOTAL"
elif [ -n "$CLAIMED_M4_TOTAL" ]; then
    ok "AUDIT.md M4 total surface = $ACTUAL_M4_TOTAL"
fi

# M5 total surface
ACTUAL_M5_TOTAL=$ACTUAL_MT_STORE_LOC
CLAIMED_M5_TOTAL=$(grep -oE 'Total M5 audit surface:\*\* [0-9]+ lines' AUDIT.md | awk -F'[*: ]+' '{print $(NF-1)}')
if [ -n "$CLAIMED_M5_TOTAL" ] && [ "$ACTUAL_M5_TOTAL" != "$CLAIMED_M5_TOTAL" ]; then
    err "AUDIT.md M5 total surface drift: claimed $CLAIMED_M5_TOTAL, actual $ACTUAL_M5_TOTAL"
elif [ -n "$CLAIMED_M5_TOTAL" ]; then
    ok "AUDIT.md M5 total surface = $ACTUAL_M5_TOTAL"
fi

# M4 + M5 determinism invariant counts
ACTUAL_M4_LOTTERY_DET=$(grep -cE '#\[test\]' crates/mt-lottery/tests/determinism_invariants.rs)
ACTUAL_M4_CONSENSUS_DET=$(grep -cE '#\[test\]' crates/mt-consensus/tests/determinism_invariants.rs)
ACTUAL_M4_ENTRY_DET=$(grep -cE '#\[test\]' crates/mt-entry/tests/determinism_invariants.rs)
ACTUAL_M5_STORE_DET=$(grep -cE '#\[test\]' crates/mt-store/tests/determinism_invariants.rs)

if [ "$ACTUAL_M4_LOTTERY_DET" != "34" ]; then
    err "mt-lottery determinism_invariants count drift: expected 34, actual $ACTUAL_M4_LOTTERY_DET"
else
    ok "mt-lottery determinism_invariants = 34"
fi
if [ "$ACTUAL_M4_CONSENSUS_DET" != "27" ]; then
    err "mt-consensus determinism_invariants count drift: expected 27, actual $ACTUAL_M4_CONSENSUS_DET"
else
    ok "mt-consensus determinism_invariants = 27"
fi
if [ "$ACTUAL_M4_ENTRY_DET" != "24" ]; then
    err "mt-entry determinism_invariants count drift: expected 24, actual $ACTUAL_M4_ENTRY_DET"
else
    ok "mt-entry determinism_invariants = 24"
fi
if [ "$ACTUAL_M5_STORE_DET" != "17" ]; then
    err "mt-store determinism_invariants count drift: expected 17 (v34 monetary refactor удалила MonetaryState persistence), actual $ACTUAL_M5_STORE_DET"
else
    ok "mt-store determinism_invariants = 17"
fi

# ---------- Summary ----------
echo ""
if [ $EXIT -eq 0 ]; then
    echo "================================================================"
    echo "✅ Documentation drift check PASS — все числа sync с фактическим кодом"
    echo "================================================================"
else
    echo ""
    echo "================================================================"
    echo "❌ Documentation drift check FAIL — найдено ${#errors[@]} расхождений:"
    for e in "${errors[@]}"; do
        echo "   - $e"
    done
    echo ""
    echo "Действие: обновить AUDIT.md / docs/audit-checklist.md / docs/security-cards.md"
    echo "так чтобы числа совпадали с фактическим кодом. Если числа в коде верны —"
    echo "правьте .md файлы. Если в .md правильно (concept change) — обнови код."
    echo "================================================================"
fi

exit $EXIT
