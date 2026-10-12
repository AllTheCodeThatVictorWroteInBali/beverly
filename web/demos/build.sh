#!/bin/sh
set -eu

cd "$(dirname "$0")"
export CARGO_BUILD_JOBS="${CARGO_BUILD_JOBS:-2}"
TRUNK_PUBLIC_URL="${TRUNK_PUBLIC_URL:-/assets/demos/}"
if [ "${SKIP_TRUNK_BUILD:-false}" != "true" ]; then
    trunk build --release --public-url "$TRUNK_PUBLIC_URL"
fi
test -f dist/index.html
mkdir -p dist/assets/avatars
cp ../../assets/avatars/profile.png dist/assets/avatars/
rm -rf ../../docs/book/assets/demos
mkdir -p ../../docs/book/assets/demos
cp -R dist/. ../../docs/book/assets/demos/