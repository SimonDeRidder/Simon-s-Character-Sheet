use wasm_bindgen::{JsValue, prelude::wasm_bindgen};
use web_sys::{Document, console};

mod stats;

pub(crate) mod context_menu;

mod error;
mod interface;
mod utils;

use crate::Character;

use self::{interface::create_all_effects, stats::render_stats_page};

#[wasm_bindgen]
pub fn render_all(character: &Character) {
	let document = match get_document() {
		Some(doc) => doc,
		None => log_and_panic("Could not get global document!"),
	};
	match render_stats_page(character, &document) {
		Ok(_) => (),
		Err(err) => {
			log_error(err.message.as_str());
		},
	};

	// set up interface
	create_all_effects(character);
}

fn get_document() -> Option<Document> {
	web_sys::window().and_then(|window| window.document())
}

fn log_and_panic(message: &str) -> ! {
	log_error(message);
	panic!("{}", message)
}

fn log_error(message: &str) {
	console::error_1(&JsValue::from_str(message));
}
