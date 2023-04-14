use base64::Engine;
use wasm_bindgen::JsCast;
use web_sys::{Element, Document, HtmlAnchorElement, Blob, Url};



#[derive(Clone)]
pub struct ErrorBar(Element);

impl ErrorBar {
	pub fn new(element: Element) -> Self {
		Self(element)
	}

	pub fn set_text(&self, text : &str) {
		self.0.set_class_name("");
		self.0.set_text_content(Some(text));
	}

	pub fn hide(&self) {
		self.0.set_class_name("invisible");
	}
}


pub fn initiate_download_file(document : &Document, file : &Blob, name : Option<&str>) -> Result<(), wasm_bindgen::JsValue>
{
	let link = 
		document
		.create_element("a").unwrap()
		.dyn_into::<HtmlAnchorElement>()?
	;
	link.set_class_name("invisible");
	link.set_id("downloadlink");
	link.set_attribute("download", name.unwrap_or("out"))?;
	link.set_attribute("href", &Url::create_object_url_with_blob(file).unwrap())?;
	document.body().unwrap().append_child(&link)?;


	link.click();

	document.default_view().unwrap().set_timeout_with_str("kill_download_link()")?;
	// link.parent_node().unwrap().remove_child(&link)?;
	Ok(())
}

// pub fn create_url_from_byte_slice(bytes : &[u8]) -> String {
// 	// base64::engine::general_purpose::STANDARD_NO_PAD::encode(bytes)

// 	const URL_PREFIX : &[u8] = b"data:application/octet-stream;base64,";
// 	const URL_PREFIX_LENGTH : usize = URL_PREFIX.len();

// 	let mut url : Vec<u8> = vec![0x0; bytes.len() * 4 / 3 + 4 + URL_PREFIX.len()];
// 	url[.. URL_PREFIX_LENGTH].copy_from_slice(URL_PREFIX);

// 	let n_bytes_written = base64::engine::general_purpose::STANDARD_NO_PAD.encode_slice(bytes, &mut url[URL_PREFIX_LENGTH ..]).unwrap();
// 	url.truncate(URL_PREFIX_LENGTH + n_bytes_written);
	


// 	String::from_utf8(url).unwrap()
// }
