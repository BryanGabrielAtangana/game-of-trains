#!/usr/bin/env bash
# Build the Rail Royale WASM client and drop it next to the static web shell.
# No wasm-bindgen / trunk needed: the client is a plain cdylib that talks to JS
# through a tiny hand-written loader (web/index.html).
set -euo pipefail

cd "$(dirname "$0")/.."

echo "› building train-client for wasm32-unknown-unknown (release)…"
cargo build -p train-client --target wasm32-unknown-unknown --release

src="target/wasm32-unknown-unknown/release/train_client.wasm"
dst="web/rail_royale.wasm"
cp "$src" "$dst"
echo "› wrote $dst ($(du -h "$dst" | cut -f1))"
echo "Serve ./web with any static server, e.g.:  python3 -m http.server -d web 8080"
