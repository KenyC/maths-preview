let context;
// TODO: display loading message
// TODO: catch error in rendering


/**
* On document load, assign click handlers to each button and try to load the
* user's origin and destination language preferences if previously set.
*/
$(function() {
  $('#compile-formula').click(runTranslation);
  $('#insert-text').click(insertText);
  // load WASM
  const wasm_bytearray = base64ToArray(wasm_bytecode);
  console.log("Started initialization ...");
  initSync(wasm_bytearray);
  context = init_font();
  console.log("done initialization.");
});


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
* Runs a server-side function to translate the user-selected text and update
* the sidebar UI with the resulting translation.
*/
function runTranslation() {
  console.log("BHDT");
  var canvas = $("#output-canvas")[0]; // [0] is used to access the DOM element from the jQuery object
  
  // Check if the browser supports the canvas element
  if (canvas.getContext) {
    var ctx = canvas.getContext("2d");
    var textarea = document.getElementById('formula_input');
    var formula = textarea.value;

    console.log("Started rendering...");
    render_formula_no_err(context, formula, ctx);
    console.log("done rendering");
  } else {
    console.log("Canvas is not supported in this browser.");
  }
}

/**
 * Runs a server-side function to insert the translated text into the document
 * at the user's cursor or selection.
 */
function insertText() {
  this.disabled = true;
  $('#error').remove();
  google.script.run
    .withSuccessHandler(
     function(returnSuccess, element) {
       element.disabled = false;
     })
    .withFailureHandler(
     function(msg, element) {
       showError(msg, $('#button-bar'));
       element.disabled = false;
     })
    .withUserObject(this)
    .insertText($('#translated-text').val());
}

/**
 * Inserts a div that contains an error message after a given element.
 *
 * @param {string} msg The error message to display.
 * @param {DOMElement} element The element after which to display the error.
 */
function showError(msg, element) {
  const div = $('<div id="error" class="error">' + msg + '</div>');
  $(element).after(div);
}
