#!/usr/bin/env bash
# Build the eDirStat web frontend for production.
#
# Pipeline: cargo (release, wasm32) -> wasm-bindgen (--target web) -> wasm-opt.
# The wasm binary is built with atomics/bulk-memory target features (see the
# cargo target rustflags), so wasm-opt must be told to allow those features.
#
# Requires: wasm-bindgen-cli matching the lockfile's wasm-bindgen version,
# and binaryen (wasm-opt).
#
# Output: `crates/edirstat-gui/dist/`
#         (static files, serve with any web server, but requires CORS setup).
set -euo pipefail

cd "$(dirname "$0")/.."

GUI_CRATE="crates/edirstat-gui"
DIST="$GUI_CRATE/dist"
BIN_NAME="edirstat-web"
WASM_OPT_FEATURES=(
    --enable-bulk-memory
    --enable-threads
    --enable-nontrapping-float-to-int
    --enable-simd
    --enable-multivalue
)

echo "==> Building $BIN_NAME (release, wasm32-unknown-unknown)"
cargo build -p edirstat-gui --bin "$BIN_NAME" --target wasm32-unknown-unknown --release

echo "==> Running wasm-bindgen"
rm -rf "$DIST"
mkdir -p "$DIST"
wasm-bindgen --target web --no-typescript \
    --out-dir "$DIST" \
    "target/wasm32-unknown-unknown/release/$BIN_NAME.wasm"

echo "==> Optimizing with wasm-opt"
wasm-opt "${WASM_OPT_FEATURES[@]}" -O4 -ol 100 -s 100 \
    -o "$DIST/${BIN_NAME}_bg.wasm" \
    "$DIST/${BIN_NAME}_bg.wasm"

cp "$GUI_CRATE/index.html" "$DIST/index.html"

echo "==> Done. Serve with e.g.: python3 -m http.server -d $DIST"
