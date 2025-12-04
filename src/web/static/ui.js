import init, { 
	init_font, 
	render_formula_to_canvas_js_err,
	render_formula_to_offscreen_canvas_js_err,
	render_formula_to_svg 
} from './maths_preview.js';

let wasm_context;

let previously_working_formula = "";
let canvas;
let canvas_context;
let formula_input;
let save_button;
let format_picker;
let error_msg;
let export_options_link;
let settings_form;

async function run() {
	await init();

	// -- initialize all protagonists
	wasm_context = init_font();
	canvas        = document.getElementById("canvas");
	formula_input = document.getElementById("formula");
	save_button   = document.getElementById("save");
	format_picker = document.getElementById("saveFormat");
	error_msg     = document.getElementById("error");
	settings_form = document.getElementById("settings_form");
	export_options_link   = document.getElementById("export_options_link");
	export_options_dialog = document.getElementById("export_options_dialog");
	canvas_context = canvas.getContext("2d");


	// -- Resize Canvas to maximal width
	resizeCanvas(); // <- this takes care of the first render
	let observer = new ResizeObserver(() => {
		resizeCanvas();
		renderFormula();
	}); 
	observer.observe(canvas);

	// -- Set canvas auto-update
	formula_input.addEventListener("input", renderFormula);
	
	// -- Set save function
	save_button.addEventListener("click", saveFormula);

	// -- Open dialog on export option click
	export_options_link.addEventListener("click", showDialog);

	// -- Dismiss dialog on button click
	document.getElementById("close_dialog").addEventListener("click", closeDialog);

	// -- submit changes in settings on every change
	setSettingsFromContext();
	settings_form.addEventListener("change", sendSettings);

}




function triggerDownload(filename, blob) {
	
	// Generate a URL for the Blob
	const url = URL.createObjectURL(blob);
	
	// Create a download link
	const downloadLink = document.createElement("a");
	downloadLink.href = url;
	downloadLink.download = filename; // Specify the filename
	
	// Trigger a click event on the link
	document.body.appendChild(downloadLink);
	downloadLink.click();
	
	// Clean up
	document.body.removeChild(downloadLink);
	URL.revokeObjectURL(url);
}


function saveFormula() {
	switch(format_picker.value) {
		case "svg":
			saveFormulaToSvg();
			break;
		case "png":
			saveFormulaToPng();
			break;
		default:
			setError(format_picker.value + " not supported");
	}
}

async function saveFormulaToPng() {
	const formula = formula_input.value;
	const offscreen = new OffscreenCanvas(1, 1);
	render_formula_to_offscreen_canvas_js_err(wasm_context, formula, (width, height) => {
		console.log(width, height);
		offscreen.width  = Math.ceil(width);
		offscreen.height = Math.ceil(height);
		return offscreen.getContext("2d");
	});
	const blob = await offscreen.convertToBlob({type : "image/png"});
	triggerDownload("formula.png", blob);
}

function saveFormulaToSvg() {
	const formula = formula_input.value;
	const svg_render = render_formula_to_svg(wasm_context, formula);
	triggerDownload("formula.svg", new Blob([svg_render], { type: "image/svg+xml" }));
}



function renderFormula() {
	const formula = formula_input.value;
	try {
		render_formula_to_canvas_js_err(wasm_context, formula, canvas_context);
		unsetError();
		previously_working_formula = formula;
	}
	catch(error) {
		setError(error);
		render_formula_to_canvas_js_err(wasm_context, previously_working_formula, canvas_context);
	}
}

function showDialog() {
	export_options_dialog.show();
}

function closeDialog() {
	export_options_dialog.close();
}

function sendSettings() {
	const formData = new FormData(settings_form);
	const font_size = String(formData.get("font_size"));
	const glyph_as_text = Boolean(formData.get("glyph_as_text"));

	wasm_context.set_settings_from_js(
		glyph_as_text,
		font_size
	);
	setSettingsFromContext();
	// // Filling in unchecked checkbox
	// if(!("glyph_as_text" in formData)) {
	// 	formData.glyph_as_text = false;
	// }
	// for (const [key, value] of Object.entries(formData)) {
	//   console.log(`${key}: ${value} ; ${typeof(value)}`);
	// }
}

function setSettingsFromContext() {
	fillForm(settings_form, {
		glyph_as_text: wasm_context.glyph_as_text,
		font_size: wasm_context.font_size,
	});
}


function fillForm(form, data) {
	// Iterate over each key in the data object
	for (const key in data) {
		// Ensure the key is a property of the data object itself, not from its prototype
		if (Object.hasOwn(data, key)) {
			const value = data[key];
			
			// Find the form element(s) with the matching 'name'
			const element = form.elements[key];

			if (!element) {
				console.warn(`No form element found with name: ${key}`);
				continue;
			}

			// The 'type' of the element determines how we set its value
			const type = element.type || (element.length > 0 ? element[0].type : null);

			switch (type) {
				case 'checkbox':
					// For checkboxes, set the 'checked' property based on a boolean value
					element.checked = Boolean(value);
					break;

				case 'radio':
					// For radio buttons, we need to find the specific radio in the group
					// that has the matching value and check it.
					// 'element' is actually a RadioNodeList of all radios with that name.
					const radioToSelect = Array.from(element).find(radio => radio.value === value);
					if (radioToSelect) {
						radioToSelect.checked = true;
					}
					break;
				
				// For all other standard input types, we can just set the 'value'
				default:
					element.value = value;
					break;
			}
		}
	}
}


function setError(error) {
	console.log(error);
	error_msg.textContent = error;
	error_msg.classList.remove("invisible");	
}

function unsetError() {
	error_msg.classList.add("invisible");
}

function resizeCanvas(){
	const width  = canvas.clientWidth;
	const target_format = 4. / 3.;
	let height = width / target_format;
	if(height > 0.9 * window.innerHeight) {
		height = 0.9 * window.innerHeight;
	}
	canvas.width  = width;
	canvas.height = height;
}

run();
