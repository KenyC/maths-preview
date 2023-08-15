import string

with open("src/sidebar_template.html") as f:
	template = string.Template(f.read())


with open("src/setupui.js") as f:
	setupui = f.read()

with open("build/wasmbytecode.js") as f:
	wasmbytecode = f.read()

with open("build/initwasm.js") as f:
	initwasm = f.read()



with open("build/sidebar.html", "w") as f:
	f.write(template.substitute(
		setupui      = setupui,
		wasmbytecode = wasmbytecode,
		initwasm     = initwasm,
	))

