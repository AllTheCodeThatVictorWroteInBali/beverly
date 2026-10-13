#!/bin/sh
set -eu
cd "$(dirname "$0")"
export CARGO_BUILD_JOBS="${CARGO_BUILD_JOBS:-2}"
if [ "${SKIP_AGENT_BUILD:-false}" != "true" ]; then
    bash ../agent-inference/build.sh
    CARGO_TARGET_DIR="${AGENT_UI_TARGET_DIR:-../demos/target}" trunk build --release --public-url "${TRUNK_PUBLIC_URL:-/assets/agent-demo/}"
    mkdir -p dist/inference/pkg dist/assets
    cp ../agent-inference/worker.js ../agent-inference/model.json dist/inference/
    cp ../agent-inference/pkg/beverly_agent_inference.js ../agent-inference/pkg/beverly_agent_inference_bg.wasm dist/inference/pkg/
    cp ../../apps/beverly-agent/assets/beverly_logo_black.png dist/assets/
fi
test -f dist/index.html
test -f dist/inference/pkg/beverly_agent_inference_bg.wasm
cp shell.js dist/shell.js
cp ../agent-inference/worker.js ../agent-inference/model.json dist/inference/
find dist/snippets -name bridge.js -exec cp bridge.js {} \;
python3 refresh-assets.py
rm -rf ../../assets/agent-demo
mkdir -p ../../assets/agent-demo
cp -R dist/. ../../assets/agent-demo/