import init, { 
	init_font, 
	render_formula_to_canvas_js_err,
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




function triggerDownloadText(filename, content) {
	// Create a Blob with the content
	const blob = new Blob([content], { type: "image/svg+xml" });
	
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
	const formula = formula_input.value;
	const svg_render = render_formula_to_svg(font_context, formula);
	triggerDownloadText("formula.svg", svg_render);
}



function renderFormula() {
	const formula = formula_input.value;
	try {
		render_formula_to_canvas_js_err(font_context, formula, canvas_context);
		error_msg.classList.add("invisible");
		previously_working_formula = formula;
	}
	catch(error) {
		console.log(error);
		error_msg.textContent = error;
		error_msg.classList.remove("invisible");
		render_formula_to_canvas_js_err(font_context, previously_working_formula, canvas_context);
	}
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
