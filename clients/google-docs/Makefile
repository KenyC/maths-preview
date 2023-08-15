

all: build/sidebar.html


build/sidebar.html: scripts/fill_template.py src/sidebar_template.html src/setupui.js build/wasmbytecode.js  build/initwasm.js
	python3 scripts/fill_template.py


build/wasmbytecode.js: build/lib.wasm scripts/convert_wasm_to_base64.py
	python3 scripts/convert_wasm_to_base64.py $< $@


build/lib.wasm:
	ln -sr ../../maths-preview-web/www/maths_preview_web_bg.wasm build/lib.wasm

build/initwasm.js:
	ln -sr ../../maths-preview-web/www/maths_preview_web.js      build/initwasm.js

symlinks: build/lib.wasm build/initwasm.js