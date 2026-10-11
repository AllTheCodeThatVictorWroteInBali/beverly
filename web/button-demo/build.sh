#!/bin/sh
set -eu

cd "$(dirname "$0")"
CARGO_BUILD_JOBS="${CARGO_BUILD_JOBS:-2}"
export CARGO_BUILD_JOBS
TRUNK_PUBLIC_URL="${TRUNK_PUBLIC_URL:-/assets/button-demo/}"
trunk build --release --public-url "$TRUNK_PUBLIC_URL"
rm -rf ../../docs/book/assets/button-demo
mkdir -p ../../docs/book/assets/button-demo
cp -R dist/. ../../docs/book/assets/button-demo/
