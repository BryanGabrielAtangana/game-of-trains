#!/usr/bin/env bash
# Build the WebAssembly client and assemble the deployable web/ directory.
#
# Usage: ./scripts/build-web.sh
# Output: web/train-client.wasm (+ the already-vendored mq_js_bundle.js)
set -euo pipefail

cd "$(dirname "$0")/.."

echo "==> Building train-client for wasm32 (release)"
cargo build -p train-client --target wasm32-unknown-unknown --release

echo "==> Copying artifacts into web/"
cp target/wasm32-unknown-unknown/release/train-client.wasm web/train-client.wasm

# Keep the JS glue in sync with the macroquad version cargo resolved.
MQ_JS="$(find "${CARGO_HOME:-$HOME/.cargo}/registry/src" -path '*macroquad-*/js/mq_js_bundle.js' 2>/dev/null | sort | tail -n1 || true)"
if [ -n "${MQ_JS}" ]; then
  cp "${MQ_JS}" web/mq_js_bundle.js
  echo "    refreshed web/mq_js_bundle.js from ${MQ_JS}"
fi

echo "==> Done. Serve it with any static server, e.g.:"
echo "    (cd web && python3 -m http.server 8080)   # then open http://localhost:8080"
