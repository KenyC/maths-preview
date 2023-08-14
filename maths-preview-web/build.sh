#!/usr/bin/bash
set -e

if [ -n "$NETLIFY" ]; then
    rustup toolchain install stable --profile minimal --target wasm32-unknown-unknown
    cargo install wasm-bindgen-cli
    MATHS_PREVIEW_RELEASE=true
fi

if [ -n "$MATHS_PREVIEW_RELEASE" ]; then
    CARGO_FLAGS=(--lib --release)
    LOCATION_EXE=target/wasm32-unknown-unknown/release/maths_preview_web.wasm
else 
    CARGO_FLAGS=--lib
    LOCATION_EXE=target/wasm32-unknown-unknown/debug/maths_preview_web.wasm
fi

mkdir -p www
cargo build "${CARGO_FLAGS[@]}" --target wasm32-unknown-unknown
wasm-bindgen --target web --no-typescript --out-dir www "$LOCATION_EXE"
cp static/* www/