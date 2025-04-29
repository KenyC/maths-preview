import init, { 
	init_font, 
	render_formula_to_canvas_js_err,
	render_formula_to_offscreen_canvas_js_err,
	render_formula_to_svg 
} from './maths_preview_web.js';

let previously_working_formula = "";
let canvas;
let canvas_context;
let font_context;
let formula_input;
let save_button;
let format_picker;
let error_msg;

async function run() {
	await init();

	// -- initialize all protagonists
	font_context = init_font();
	canvas        = document.getElementById("canvas");
	formula_input = document.getElementById("formula");
	save_button   = document.getElementById("save");
	format_picker = document.getElementById("saveFormat");
	error_msg     = document.getElementById("error");
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
	save_button.addEventListener("click", saveFormula)

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
	render_formula_to_offscreen_canvas_js_err(font_context, formula, (width, height) => {
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
	const svg_render = render_formula_to_svg(font_context, formula);
	triggerDownload("formula.svg", new Blob([svg_render], { type: "image/svg+xml" }));
}



function renderFormula() {
	const formula = formula_input.value;
	try {
		render_formula_to_canvas_js_err(font_context, formula, canvas_context);
		unsetError();
		previously_working_formula = formula;
	}
	catch(error) {
		setError(error);
		render_formula_to_canvas_js_err(font_context, previously_working_formula, canvas_context);
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
