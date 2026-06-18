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

# Use the JS loader that matches the exact miniquad version cargo compiled the
# wasm against. macroquad's bundled mq_js_bundle.js targets a different miniquad
# build (and bundles audio/net plugins this game doesn't import), which traps at
# runtime; miniquad's own gl.js is the correct, version-matched glue.
MQ_VER="$(awk '/name = "miniquad"/{getline; print}' Cargo.lock | sed 's/version = "//;s/"//')"
GL_JS="$(find "${CARGO_HOME:-$HOME/.cargo}/registry/src" -path "*miniquad-${MQ_VER}/js/gl.js" 2>/dev/null | sort | tail -n1 || true)"
if [ -n "${GL_JS}" ]; then
  cp "${GL_JS}" web/mq_js_bundle.js
  echo "    refreshed web/mq_js_bundle.js from miniquad ${MQ_VER} gl.js"
else
  echo "    WARNING: could not find miniquad ${MQ_VER} gl.js; keeping existing web/mq_js_bundle.js"
fi

echo "==> Done. Serve it with any static server, e.g.:"
echo "    (cd web && python3 -m http.server 8080)   # then open http://localhost:8080"
