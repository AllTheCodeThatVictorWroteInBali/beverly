#!/bin/sh
set -eu
cd "$(dirname "$0")"
export CARGO_BUILD_JOBS="${CARGO_BUILD_JOBS:-2}"
cargo build --locked --release --target wasm32-unknown-unknown
bindgen="${WASM_BINDGEN:-$(command -v wasm-bindgen || true)}"
if [ -z "$bindgen" ] && [ -x "$HOME/Library/Caches/dev.trunkrs.trunk/wasm-bindgen-0.2.129/wasm-bindgen" ]; then
    bindgen="$HOME/Library/Caches/dev.trunkrs.trunk/wasm-bindgen-0.2.129/wasm-bindgen"
fi
if [ -z "$bindgen" ]; then
    cargo install wasm-bindgen-cli --version 0.2.129 --locked
    bindgen="${CARGO_HOME:-$HOME/.cargo}/bin/wasm-bindgen"
fi
"$bindgen" target/wasm32-unknown-unknown/release/beverly_agent_inference.wasm --target web --out-dir pkg