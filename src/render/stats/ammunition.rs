use std::ops::Not;

use crate::{
	Character,
	config::{AmmunitionDefinition, AmmunitionIcon, AmmunitionVariant, CONFIG},
	domain::equipment::Ammunition,
	render::{error::RenderError, interface::handle_event},
};
use leptos::{
	leptos_dom::logging::{console_error, console_log},
	prelude::{
		ClassAttribute as _, CollectView as _, CustomAttribute as _, ElementChild as _, GlobalAttributes as _,
		OnAttribute as _, PropAttribute as _, Read as _, ReadUntracked as _, Set as _, StyleAttribute as _,
		Update as _, Write as _,
	},
};

const AMMUNITION_IMAGE_GRID_ASCPECT_RATIO: f32 = 1.8234904;

pub fn render_ammunition(character: &Character, stats_page: web_sys::HtmlElement) -> Result<(), RenderError> {
	let ammunitions = character.equipment.ammunition;
	console_log(">> Rendering ammunition pane");
	leptos::mount::mount_to(stats_page, move || {
		leptos::html::div()
			.id("ammunition_pane")
			.class("pane")
			.style(
				"top: 87.2%;left: 4.55%;width: 30.85%;height: 9.95%;display: flex;justify-content: space-between",
			)
			.child((create_ammunition_pane(ammunitions, 0), create_ammunition_pane(ammunitions, 1)))
	})
	.forget();
	Ok(())
}

fn create_ammunition_pane(
	ammunitions_list: leptos::prelude::RwSignal<Vec<leptos::prelude::RwSignal<Ammunition>>>,
	index: usize,
) -> impl leptos::IntoView + use<> {
	let ammunition = move || ammunitions_list.read().get(index).copied();
	let ammo_name = leptos::prelude::Memo::new(move |_| {
		ammunition().map_or(String::new(), move |ammo_signal| {
			let ammo = ammo_signal.read();
			let base_name = CONFIG
				.ammunition_definitions
				.iter()
				.position(|el: &AmmunitionDefinition| el.id.eq(&ammo.id))
				.map_or("", |ind| CONFIG.ammunition_definitions[ind].name);
			let variant_prefix = ammo.variant_id.as_ref().map(|variant_id| {
				CONFIG
					.ammunition_variants
					.iter()
					.position(|el: &AmmunitionVariant| el.id.eq(variant_id))
					.map_or("", |ind| CONFIG.ammunition_variants[ind].name)
			});
			variant_prefix.map_or(String::new(), |prefix| String::from(prefix) + " ") + base_name
		})
	});
	let ammo_name_id = format!("ammunition_name_{}", index);
	let ammo_total_id = format!("ammunition_total_{}", index);
	leptos::html::div()
		.class("pane simple")
		.style("height: 100%;width: 47.5%;position: relative")
		.child((
			leptos::html::select()
				.id(ammo_name_id.clone())
				.class(move || {if ammunition().is_none() {"inputfield-regular hide-arrow print-underline"} else {"inputfield-regular hide-arrow"}})
				.style("top: 10%;left: 8%;width: 62%;height: 15%;text-align: left;font-size: 75%;font-size: max(69%, min(125%, calc(-22% + 1700%/attr(data-chars type(<number>), 21))))")
				.attr("data-chars", move || ammo_name.read().len())
				.title("Select the type of ammunition you want to use.")
				.child((
					leptos::html::option()
						.value("")
						.child("")
						.prop("selected", move || ammunition().is_none()),
					CONFIG.ammunition_definitions.iter().map(|ammo_def| {
						leptos::html::option()
							.value(ammo_def.id)
							.child(ammo_def.name)
							.prop("selected", move || ammunition().is_some_and(|ammo| ammo_def.id.eq(&ammo.read().id) && ammo.read().variant_id.is_none()))
					}).collect_view(),
					CONFIG.ammunition_variants.iter().flat_map(|ammo_variant| {
						CONFIG.ammunition_definitions.iter().map(|ammo_def| {
							leptos::html::option()
								.value(String::from(ammo_def.id) + ":::::" + ammo_variant.id)
								.child(String::from(ammo_variant.name) + " "  + ammo_def.name)
								.prop("selected", move || ammunition().is_some_and(|ammo| ammo_def.id.eq(&ammo.read().id) && ammo.read().variant_id.as_ref().is_some_and(|variant_id| variant_id.eq(ammo_variant.id))))
						}).collect_view()
					}).collect_view()
				))
				.on(leptos::ev::change, move |event| {
					let new_id_raw = leptos::prelude::event_target_value(&event);
					let mut new_id_raw_split = new_id_raw.split(":::::");
					let new_id = new_id_raw_split.next().unwrap().to_owned(); // ok, split always has at least 1 element
					let new_variant_id = new_id_raw_split.next().and_then(|v_id| v_id.is_empty().not().then(|| v_id.to_owned()));
					let new_id_opt = match new_id.is_empty() {
						false => Some(new_id),
						true => None,
					};
					let old_ammo = ammunition();
					let old_ammo_id = old_ammo.map(|ammo| ammo.read_untracked().id.clone());
					let old_ammo_variant_id = old_ammo.and_then(|ammo| ammo.read_untracked().variant_id.clone());
					if old_ammo_id != new_id_opt || old_ammo_variant_id != new_variant_id {
						match new_id_opt {
							None => {ammunitions_list.write().remove(index);},
							Some(id) => {
								if let Some(ammo_def) = CONFIG.ammunition_definitions.iter().position(|el: &AmmunitionDefinition| el.id.eq(&id)).map(|ind| &CONFIG.ammunition_definitions[ind]) {
									let new_value = Ammunition { id, total: ammo_def.default_amount, used: 0, icon: ammo_def.icon.clone(), variant_id: new_variant_id.clone() };
									match ammunition() {
										None => {
											// could end up in other ammo slot, so change value to empty first
											leptos::prelude::event_target::<web_sys::HtmlInputElement>(&event).set_value("");
											ammunitions_list.write().push(leptos::prelude::RwSignal::new(new_value))
										},
										Some(ammo) => {ammo.set(new_value)}
									};
								}
							}
						}
					}
				}),
			leptos::html::label()
				.class("textlabel dimmed-text")
				.style("top: 2.5%;left: 11%;font-size: 80%")
				.attr("for", ammo_name_id)
				.child("NAME"),
			leptos::html::input()
				.id(ammo_total_id.clone())
				.class(move || {
					String::from("inputfield-regular")
						+ {
						if ammunition().is_none_or(|ammo| ammo.read().total == 0) {
							" print-underline"
						} else {
							""
						}
					}
				})
				.style("top: 10%;right: 8%;width: 20%;height: 15%")
				.r#type("number")
				.prop("value", move || ammunition().map_or(String::new(), |ammo| match ammo.read().total {0=>String::new(),num=>num.to_string()}))
				.attr("data-chars", move || ammunition().map_or(1, |ammo| ammo.read().total.to_string().len()))
				.title("The total amount in your (long-term) possession.")
				.on(leptos::ev::input, move |event| {
					let new_value = leptos::prelude::event_target_value(&event);
					if new_value.is_empty()
						&& <wasm_bindgen::JsValue as Into<web_sys::InputEvent>>::into(event.clone().into())
							.input_type()
							.starts_with("delete")
					{
						// new_value is also empty if a non-number character was entered,
						// hence we only set to default if it's empty after a delete type event (backspace, delete, ...)
						if let Some(ammo) = ammunition() {
							ammo.update(|ammo| ammo.total = 0)
						};
						return;
					}
					match new_value.parse::<u8>() {
						Ok(value) => {
							if let Some(ammo) = ammunition() {
								ammo.update(|ammo| ammo.total = value)
							};
						},
						Err(parse_err) => {
							console_error(format!("Could not parse as int: {}", parse_err).as_str());
							leptos::prelude::event_target::<web_sys::HtmlInputElement>(&event)
								.set_value(&ammunition().map_or(String::new(), |ammo| match ammo.read().total {0=>String::new(), n => n.to_string()}));
						},
					}
				})
				.on(leptos::ev::change, move |_| {
					handle_event(String::from("Ammo") + match index {0=> "Left",_=>"Right"} + "Display_Amount_change");
				}),
			leptos::html::label()
				.class("textlabel dimmed-text")
				.style("top: 2.5%;right: 8.5%;font-size: 80%")
				.attr("for", ammo_total_id)
				.child("TOTAL"),
			leptos::html::button()
				.class("button hide-in-print")
				.style("left: 8%;top: 25%;width: 23%;height: 11.5%")
				.title("Expend a single unit of ammunition.")
				.child("Shoot")
				.on(leptos::ev::click, move |_| {
					if let Some(ammo) = ammunition() {
						ammo.update(|amm| amm.shoot());
					};
				}),
			leptos::html::button()
				.class("button hide-in-print")
				.style("left: 38.5%;top: 25%;width: 23%;height: 11.5%")
				.title("Pick up expended ammunition, filling back to the total amount.")
				.child("Pick up")
				.on(leptos::ev::click, move |_| {
					if let Some(ammo) = ammunition() {
						ammo.update(|amm| amm.pick_up());
					};
				}),
			leptos::html::button()
				.class("button hide-in-print")
				.style("right: 8%;top: 25%;width: 23%;height: 11.5%")
				.title("Leave expended ammunition, reducing the total amount.")
				.child("Lose")
				.on(leptos::ev::click, move |_| {
					if let Some(ammo) = ammunition() {
						ammo.update(|amm| amm.lose());
					};
					handle_event(String::from("Ammo") + match index {0=> "Left",_=>"Right"} + "Display_Amount_change");
				}),
			leptos::html::div()
				.style("display:flex;position: absolute;top: 36%;left: 8%;width: 84%;height: 51%;flex-wrap: wrap;place-content: center")
				.child(
					move || {
						let the_ammo = ammunition().map(|ammo| ammo.read());
						let (checked_image, unchecked_image, image_aspect) = match the_ammo.as_ref().map_or(&AmmunitionIcon::Arrow, |ammo| &ammo.icon) {
								AmmunitionIcon::Arrow => ("arrow_checked", "arrow_unchecked", 0.3333333),
								AmmunitionIcon::Axe => ("axe_checked", "axe_unchecked", 0.5),
								AmmunitionIcon::Bullet => ("bullet_checked", "bullet_unchecked", 0.8333333),
								AmmunitionIcon::Dagger => ("dagger_checked", "dagger_unchecked", 0.3333333),
								AmmunitionIcon::Flask => ("flask_checked", "flask_unchecked", 0.3333333),
								AmmunitionIcon::Hammer => ("hammer_checked", "hammer_unchecked", 0.5),
								AmmunitionIcon::Spear => ("spear_checked", "spear_unchecked", 0.2),
								AmmunitionIcon::Vial => ("vial_checked", "vial_unchecked", 0.5333333),
							};
						let current_total = the_ammo.as_ref().map_or(0, |ammo| ammo.total);
						let current_unused = current_total.saturating_sub(the_ammo.as_ref().map_or(0, |ammo| ammo.used));

						let grid_width = 100.0;
						let grid_height_rescaled = (grid_width * image_aspect) / AMMUNITION_IMAGE_GRID_ASCPECT_RATIO;
						let mut size = 0.0;
						let early_stop = current_total.isqrt(); // assumes grid_height_rescaled <= grid_width
						for num_rows in 1..(early_stop+1) {
							let num_columns = current_total.div_ceil(num_rows);
							let s = (grid_width / num_columns as f32).min(grid_height_rescaled / num_rows as f32);
							if s > size {
								size = s;
							}
						}

						(0..current_total).map(
							|ind| {
								leptos::html::img()
									.style(move || format!("flex: 0 0 auto;object-fit: contain;width: {}%;height: auto", size))
									.src(move || format!("img/ammunition/{}.svg", {if ind >= current_unused {checked_image} else {unchecked_image}}))
							}
						).collect_view()
					}
				),
			leptos::html::span()
				.class("textlabel-bold")
				.style("top: 87.5%;left: 26%;font-size: 92%;")
				.child("AMMUNITION"),
		))
}
