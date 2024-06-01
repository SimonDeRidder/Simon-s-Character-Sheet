pub mod abilities;
pub mod header;

use leptos::leptos_dom::logging::console_log;
use wasm_bindgen::JsCast as _;

use crate::Character;

use self::abilities::render_abilities;
use self::header::render_header;

use super::error::RenderError;

pub fn render_stats_page(character: &Character, document: &web_sys::Document) -> Result<(), RenderError> {
	// try to render all, only propagate errors at the end
	console_log("> Rendering stats page");

	let stats_page = match document.get_element_by_id("CSfront") {
		Some(element) => Ok(element),
		None => Err(RenderError::new("Could not find stats page CSfront!")),
	}?
	.dyn_into::<web_sys::HtmlElement>()?;
	let stats_page_clone = stats_page.clone();
	let results: Vec<Result<(), RenderError>> =
		vec![render_header(character, stats_page), render_abilities(character, stats_page_clone)];

	for res in results {
		res?;
	}
	Ok(())
}
