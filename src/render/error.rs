#[derive(Clone, Debug)]
pub struct RenderError {
	pub message: String,
}

impl RenderError {
	pub fn new(message: &str) -> Self {
		Self { message: String::from(message) }
	}
}

impl From<wasm_bindgen::JsValue> for RenderError {
	fn from(value: wasm_bindgen::JsValue) -> Self {
		RenderError {
			message: match value.as_string() {
				Some(msg) => msg,
				None => String::from("Unknown Error (None)"),
			},
		}
	}
}

impl From<web_sys::Element> for RenderError {
	fn from(el: web_sys::Element) -> Self {
		RenderError {
			message: format!("An error occured for element with id  '{}'!", el.id()),
		}
	}
}
