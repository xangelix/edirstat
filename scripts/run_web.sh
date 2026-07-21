#!/usr/bin/env bash
# Build WASM frontend, package static site with Zola & Vite, and serve locally.
#
# Pipeline:
#   1. ./scripts/build_web.sh (cargo wasm32 -> wasm-bindgen -> wasm-opt -> crates/edirstat-gui/dist)
#   2. web/ packaging (npm install -> copy-assets -> vite build -> zola build -> web/public)
#   3. static-web-server (hosts web/public)
#
# Environment variables:
#   PORT     - Port to listen on (default: 8080)
#   HOST     - Host address to bind to (default: 127.0.0.1)
#   BASE_URL - Base URL override for Zola (default: http://$HOST:$PORT)

set -euo pipefail

cd "$(dirname "$0")/.."

PORT="${PORT:-8080}"
HOST="${HOST:-127.0.0.1}"
BASE_URL="${BASE_URL:-http://${HOST}:${PORT}}"

echo "==> Step 1: Building WASM app (build_web.sh)"
./scripts/build_web.sh

echo "==> Step 2: Packaging web site and viewer"
cd web
npm install
npm run build:assets
zola build --base-url "$BASE_URL"
cd ..

echo "==> Step 3: Hosting web server at $BASE_URL"
exec static-web-server --port "$PORT" --host "$HOST" --root web/public
