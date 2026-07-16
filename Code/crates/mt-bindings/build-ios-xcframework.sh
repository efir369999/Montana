#!/bin/bash
# Пересборка MontanaBindings.xcframework (только устройство arm64) с --features network.
# SSOT заголовка — cbindgen (montana_ffi.h) + ручной mt_bindings.h. Запуск из crates/mt-bindings.
set -euo pipefail
cd "$(dirname "$0")"
ROOT="../.."
OUT_XC="/Users/kh./Python/Ничто/Montana/App/Montana-Messenger-iOS/Frameworks/MontanaBindings.xcframework"

# Заголовок include/montana_ffi.h ведётся вручную (патчами): cbindgen 0.29.4 регрессит
# opaque-typedef (WakeRegistry). Регенерацию НЕ запускаем автоматически.

echo "[1/3] cargo build device (только устройство, без симулятора) (aarch64-apple-ios)"
( cd "$ROOT" && rustup run 1.92.0 cargo rustc -p mt-bindings --features network --release --target aarch64-apple-ios --crate-type staticlib )

echo "[2/3] headers dir + modulemap"
HDIR="$(mktemp -d)"
cp include/montana_ffi.h include/mt_bindings.h "$HDIR/"
cat > "$HDIR/module.modulemap" <<'EOF'
module MontanaBindings {
    header "mt_bindings.h"
    header "montana_ffi.h"
    export *
}
EOF

echo "[3/3] create xcframework → $OUT_XC"
rm -rf "$OUT_XC"
xcodebuild -create-xcframework \
  -library "$ROOT/target/aarch64-apple-ios/release/libmt_bindings.a" -headers "$HDIR" \
  -output "$OUT_XC"
echo "ГОТОВО xcframework"
