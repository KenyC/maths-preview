

all: out/sidebar.html


out/sidebar.html: scripts/fill_template.py src/sidebar_template.html src/setupui.js out/wasmbytecode.js  out/initwasm.js
	python3 scripts/fill_template.py


out/wasmbytecode.js: out/lib.wasm scripts/convert_wasm_to_base64.py
	python3 scripts/convert_wasm_to_base64.py $< $@


create_symlinks:
	ln -sr out/lib.wasm        ../../maths-preview-web/www/maths_preview_web_bg.wasm
	ln -sr out/initwasm.js     ../../maths-preview-web/www/maths_preview_web.js