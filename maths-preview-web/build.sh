#!/usr/bin/bash

if [ -n "$NETLIFY" ]; then
    rustup toolchain install stable --profile minimal --target wasm32-unknown-unknown
fi

mkdir -p www
cargo build --lib --target wasm32-unknown-unknown
wasm-bindgen --target web --no-typescript --out-dir www target/wasm32-unknown-unknown/debug/maths_preview_web.wasm
cp static/* www/