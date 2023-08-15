let font_context;
let canvas_context;
let formula_input;
let insert_button;
// TODO: display loading message
// TODO: catch error in rendering


/**
 * Converts a base64 string to an array
 * 
 */
function base64ToArray(base64String) {
    const binaryString = atob(base64String);
    const array = new Uint8Array(binaryString.length);

    for (let i = 0; i < binaryString.length; i++) {
        array[i] = binaryString.charCodeAt(i);
    }

    return array;
}


/**
* Render formula in input text to the canvas below
* 
*/
function renderFormula() {
  let formula = formula_input.value;

  console.log("Started rendering...");
  try {
    render_formula_to_canvas_js_err(font_context, formula, canvas_context);
    console.log("done rendering");
  }
  catch(error) {
    console.log(error);
  }
}

/**
 * Runs a server-side function to insert the rendered image into the document
 * at the user's cursor or selection.
 */
function insertSvgImage() {
  this.disabled = true;
  // $('#error').remove();
  google.script.run
    .withSuccessHandler(
     function(returnSuccess, element) {
       element.disabled = false;
     })
    .withFailureHandler(
     function(msg, element) {
       // showError(msg, $('#button-bar'));
       element.disabled = false;
     })
    .withUserObject(this)
    .insertSvgImage(formula_input.value);
}

/**
 * Inserts a div that contains an error message after a given element.
 *
 * @param {string} msg The error message to display.
 * @param {DOMElement} element The element after which to display the error.
 */
// function showError(msg, element) {
//   const div = $('<div id="error" class="error">' + msg + '</div>');
//   $(element).after(div);
// }

document.addEventListener("DOMContentLoaded", onLoad);

function onLoad() {
  // Load WASM
  const wasm_bytearray = base64ToArray(wasm_bytecode);
  console.log("Started initialization ...");
  initSync(wasm_bytearray);
  font_context = init_font();
  console.log("done initialization.");


  // Get HTML elements
  insert_button = document.getElementById('insert-button');
  formula_input = document.getElementById('formula-input');
  let canvas = document.getElementById("output-canvas");
  if(canvas.getContext) {
    canvas_context = canvas.getContext("2d");
  }
  else {
    console.log("Canvas not supported in this browser.");
    return;
  }

  formula_input.addEventListener("input", renderFormula);
  renderFormula();

  insert_button.addEventListener("click", insertSvgImage);

  // document.getElementById('insert-text').addEventListener('click', insertText);
  // .addEventListener('click', renderFormula);
  // .addEventListener('click', renderFormula);
}