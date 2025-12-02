WASM_TARGET_DIR?=target/

all: desktop web

desktop:
	cargo build

web:
	mkdir -p www
	cargo build --lib --target wasm32-unknown-unknown
	wasm-bindgen ${WASM_TARGET_DIR}/wasm32-unknown-unknown/debug/maths_preview.wasm --out-dir www/ --target web
	cp -f src/web/static/* www/
