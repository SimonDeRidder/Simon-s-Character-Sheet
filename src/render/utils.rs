use std::sync::Arc;

use leptos::{
	html::ElementChild as _,
	leptos_dom::logging::console_log,
	prelude::{
		ClassAttribute as _, CustomAttribute as _, Get as _, GetUntracked as _, GlobalAttributes as _,
		NodeRefAttribute as _, OnAttribute as _, PropAttribute as _, Set as _, StyleAttribute as _,
	},
};

use wasm_bindgen::JsCast as _;

use super::{
	context_menu::{
		content::{ContextMenuContent, ContextMenuItem},
		create_context_menu,
	},
	error::RenderError,
	get_document, log_and_panic,
};
use crate::{
	config::IconsConfig,
	domain::types::{AbilityValue, Modifier, SignalField},
};

const MODAL_CANVAS_CLASS: &str = "modal-canvas";

pub trait RenderableValue {
	fn render(&self) -> String;
}
impl RenderableValue for Modifier {
	fn render(&self) -> String {
		if *self > 0 {
			format!("+{}", *self)
		} else {
			format!("{}", *self)
		}
	}
}
impl RenderableValue for AbilityValue {
	fn render(&self) -> String {
		format!("{}", *self)
	}
}

pub fn show_modal<T: leptos::prelude::IntoRender>(
	title: &'static str,
	content: T,
	buttons: Vec<(String, leptos::prelude::RwSignal<Option<String>>, Option<leptos::prelude::Memo<bool>>)>,
) where
	<T as leptos::prelude::IntoRender>::Output: 'static + leptos::IntoView,
{
	let modal_id = uuid::Uuid::new_v4().to_string();
	let button_vec = buttons
		.clone()
		.iter()
		.map(|(button_text, button_signal, disable_signal)| {
			let modal_id_button_clone = modal_id.clone();
			let signal_clone = *button_signal;
			let disable_signal_clone = *disable_signal;
			leptos::html::button()
				.class("button")
				.child(button_text.clone())
				.on(leptos::ev::click, move |_| signal_clone.set(Some(modal_id_button_clone.clone())))
				.disabled(move || {
					disable_signal_clone.is_some_and(|disable_signal_val| disable_signal_val.get())
				})
		})
		.collect::<Vec<_>>();
	let modal_window = leptos::html::div()
		.class("modal-window")
		.child(leptos::html::div().class("modal-title").child(title))
		.child(leptos::html::div().class("modal-content").child(content))
		.child(leptos::html::div().class("modal-buttons").child(button_vec))
		.on(leptos::ev::click, |event| event.stop_propagation());
	let modal_id_clone = modal_id.clone();
	let canvas = leptos::html::div()
		.id(modal_id.clone())
		.class(MODAL_CANVAS_CLASS)
		.child(modal_window)
		.on(leptos::ev::click, move |_| {
			remove_modal(modal_id_clone.clone()).unwrap();
		});
	leptos::mount::mount_to_body(|| canvas);
}

pub fn remove_modal(id: String) -> Result<(), RenderError> {
	get_document()
		.ok_or(RenderError::new("Could not get document"))?
		.get_element_by_id(id.as_str())
		.ok_or(RenderError::new(format!("Could not find modal with id {}", id).as_str()))?
		.remove();
	Ok(())
}

pub fn create_number_input<T, S>(
	signal: leptos::prelude::RwSignal<T>,
	additional_style: S,
	disable_signal: Option<leptos::prelude::RwSignal<bool>>,
	show_zero: bool,
	no_spinner: bool,
	underline_if_zero_in_print: bool,
	show_in_print: bool,
) -> leptos::html::HtmlElement<leptos::html::Input, impl leptos::attr::Attribute, ()>
where
	T: std::str::FromStr + std::default::Default + 'static,
	<T as std::str::FromStr>::Err: std::fmt::Display,
	leptos::prelude::RwSignal<T>:
		leptos::prelude::Get + leptos::prelude::Set<Value = T> + leptos::tachys::html::property::IntoProperty,
	<leptos::prelude::RwSignal<T> as leptos::prelude::Get>::Value:
		leptos::attr::AttributeValue + std::fmt::Display + PartialEq<T>,
	S: leptos::tachys::html::style::IntoStyle,
{
	let signal_to_string = move || {
		let value = signal.get();
		if value == T::default() && !show_zero {
			String::new()
		} else {
			value.to_string()
		}
	};
	leptos::html::input()
		.class(move || {
			String::from("inputfield-regular")
				+ {
					if disable_signal.is_some_and(|disable_signal_val| disable_signal_val.get()) {
						" inputfield-disabled"
					} else {
						""
					}
				} + {
				if no_spinner {
					" no-spinner"
				} else {
					""
				}
			} + {
				if underline_if_zero_in_print && (signal.get() == T::default()) {
					" print-underline"
				} else {
					""
				}
			} + {
				if show_in_print {
					""
				} else {
					" hide-in-print"
				}
			}
		})
		.style(additional_style)
		.r#type("number")
		.prop("value", signal_to_string)
		.attr("data-chars", move || std::cmp::max(signal_to_string().len(), 1))
		.disabled(move || disable_signal.is_some_and(|disable_signal_val| disable_signal_val.get()))
		.on(leptos::ev::input, move |event| {
			let new_value = leptos::prelude::event_target_value(&event);
			if new_value.is_empty()
				&& <wasm_bindgen::JsValue as Into<web_sys::InputEvent>>::into(event.clone().into())
					.input_type()
					.starts_with("delete")
			{
				// new_value is also empty if a non-number character was entered,
				// hence we only set to default if it's empty after a delete type event (backspace, delete, ...)
				// see https://w3c.github.io/input-events/#interface-InputEvent-Attributes
				signal.set(T::default());
				return;
			}
			match new_value.parse::<T>() {
				Ok(value) => signal.set(value),
				Err(parse_err) => {
					console_log(format!("Could not parse as int: {}", parse_err).as_str());
					leptos::prelude::event_target::<web_sys::HtmlInputElement>(&event)
						.set_value(signal_to_string().as_str());
				},
			}
		})
}

pub struct SignalFuture<SignalType>
where
	SignalType:
		leptos::prelude::Get<Value = bool> + leptos::prelude::GetUntracked<Value = bool> + Clone + 'static,
{
	pub signal: SignalType,
}

impl<SignalType> std::future::Future for SignalFuture<SignalType>
where
	SignalType:
		leptos::prelude::Get<Value = bool> + leptos::prelude::GetUntracked<Value = bool> + Clone + 'static,
{
	type Output = ();

	fn poll(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Self::Output> {
		match self.signal.try_get_untracked() {
			None | Some(true) => std::task::Poll::Ready(()),
			Some(false) => {
				let sgn_clone = self.signal.clone();
				let waker = cx.waker().clone();
				leptos::prelude::Effect::new(move || {
					if sgn_clone.try_get().is_none_or(|v| v) {
						waker.wake_by_ref();
					}
				});
				std::task::Poll::Pending
			},
		}
	}
}

#[derive(Clone, Copy, Debug)]
pub enum HorizontalReference {
	Left,
	Right,
}

#[derive(Clone, Copy, Debug)]
pub enum VerticalReference {
	Top,
	Bottom,
}

#[derive(Clone, Copy, Debug)]
pub enum DistanceType {
	Pixels,
	// Percentage,
}

#[derive(Clone, Copy, Debug)]
pub struct RelativeHorizontalCoordinate {
	pub reference: HorizontalReference,
	pub distance: f32,
	pub unit: DistanceType,
}

#[derive(Clone, Copy, Debug)]
pub struct RelativeVerticalCoordinate {
	pub reference: VerticalReference,
	pub distance: f32,
	pub unit: DistanceType,
}

#[derive(Clone, Copy, Debug)]
pub enum Orientation {
	BottomRight,
	BottomLeft,
	TopRight,
	TopLeft,
}

#[derive(Clone, Copy, Debug)]
pub struct RelativePosition {
	pub anchor: (RelativeHorizontalCoordinate, RelativeVerticalCoordinate),
	pub orientation: Orientation,
}

struct UnsafeFileReaderWrapper(web_sys::FileReader);
unsafe impl Send for UnsafeFileReaderWrapper {}
unsafe impl Sync for UnsafeFileReaderWrapper {}

async fn set_icon_from_filepicker(target_id: &str, icon_signal: SignalField<String>) {
	let filepicker_element = leptos::html::input()
		.style("visibility: hidden")
		.attr("type", "file")
		.on(leptos::ev::click, move |ev| ev.stop_propagation())
		.on(leptos::ev::change, move |ev| {
			let this_element = leptos::prelude::event_target::<wasm_bindgen::JsValue>(&ev)
				.dyn_into::<web_sys::HtmlInputElement>()
				.unwrap();
			let file_list = match this_element.files() {
				Some(file_list) => file_list,
				None => {
					this_element.remove();
					return;
				},
			};
			let file = match file_list.get(0) {
				Some(file) => file,
				None => {
					this_element.remove();
					return;
				},
			};
			let file_reader = Arc::new(UnsafeFileReaderWrapper(web_sys::FileReader::new().unwrap()));
			let file_reader_clone = file_reader.clone();
			file_reader.0.set_onload(Some(
				&wasm_bindgen::closure::Closure::once_into_js(move |_event: web_sys::ProgressEvent| {
					icon_signal.set(file_reader_clone.0.result().unwrap().as_string().unwrap());
				})
				.into(),
			));
			file_reader.0.read_as_data_url(&file).unwrap();
		});
	let document = get_document().unwrap();
	let parent_el = document.get_element_by_id(target_id).unwrap().dyn_into().unwrap();

	let filepicker_node_ref = leptos::prelude::NodeRef::<leptos::html::Input>::new();
	let filepicker_node_element = filepicker_element.node_ref(filepicker_node_ref);

	leptos::mount::mount_to(parent_el, || filepicker_node_element).forget();

	filepicker_node_ref
		.get_untracked()
		.expect("filepicker failed to mount")
		.click();
}

async fn reset_icon(icon_signal: SignalField<String>) {
	icon_signal.set(String::from("img/icons/blank.svg"));
}

async fn empty_icon(icon_signal: SignalField<String>) {
	icon_signal.set(String::new());
}

async fn set_icon_url(icon_signal: SignalField<String>, url: String) {
	icon_signal.set(url);
}

pub fn make_icon_context_menu(
	parent_id: &'static str,
	icon_signal: SignalField<String>,
	mouse_event: &web_sys::MouseEvent,
	include_factions_classes: bool,
	config: IconsConfig,
) {
	let mut root_menu = ContextMenuContent::new();
	let mut faction_symbol_menu = ContextMenuContent::new();
	let mut faction_icon_menu = ContextMenuContent::new();
	let mut faction_banner_menu = ContextMenuContent::new();
	let mut class_icon_menu = ContextMenuContent::new();
	let mut adventure_league_icon_menu = ContextMenuContent::new();

	// Default options
	root_menu.push(ContextMenuItem::new(
		"Set any image file as this icon",
		leptos::prelude::Action::new(move |_| set_icon_from_filepicker(parent_id, icon_signal)),
	));
	root_menu.push(ContextMenuItem::new(
		"Reset this icon",
		leptos::prelude::Action::new(move |_| reset_icon(icon_signal)),
	));
	root_menu.push(ContextMenuItem::new(
		"Empty this icon",
		leptos::prelude::Action::new(move |_| empty_icon(icon_signal)),
	));

	// Faction icons, symbols, banners; Class and AL season icons
	if include_factions_classes {
		// Factions
		for (faction_name, faction_icon_filename) in config.factions {
			faction_symbol_menu.push(ContextMenuItem::new(
				faction_name,
				leptos::prelude::Action::new(move |_| {
					set_icon_url(
						icon_signal,
						String::from("img/icons/factions/symbol/") + faction_icon_filename + ".svg",
					)
				}),
			));
			faction_banner_menu.push(ContextMenuItem::new(
				faction_name,
				leptos::prelude::Action::new(move |_| {
					set_icon_url(
						icon_signal,
						String::from("img/icons/factions/banner/") + faction_icon_filename + ".svg",
					)
				}),
			));
			faction_icon_menu.push(ContextMenuItem::new(
				faction_name,
				leptos::prelude::Action::new(move |_| {
					set_icon_url(
						icon_signal,
						String::from("img/icons/factions/icon/") + faction_icon_filename + ".svg",
					)
				}),
			));
		}
		root_menu.push(ContextMenuItem::new_separator());
		root_menu.push(ContextMenuItem::new_submenu("Set faction symbol", faction_symbol_menu));
		root_menu.push(ContextMenuItem::new_submenu("Set faction banner", faction_banner_menu));
		root_menu.push(ContextMenuItem::new_submenu("Set faction icon", faction_icon_menu));

		// Classes
		for (class_name, class_icon_filename) in config.classes {
			class_icon_menu.push(ContextMenuItem::new(
				class_name,
				leptos::prelude::Action::new(move |_| {
					set_icon_url(
						icon_signal,
						String::from("img/icons/classes/") + class_icon_filename + ".svg",
					)
				}),
			));
		}
		root_menu.push(ContextMenuItem::new_separator());
		root_menu.push(ContextMenuItem::new_submenu("Set class icon", class_icon_menu));

		// The AL seasons
		for (al_name, al_icon_filename) in config.adventure_league {
			adventure_league_icon_menu.push(ContextMenuItem::new(
				al_name,
				leptos::prelude::Action::new(move |_| {
					set_icon_url(
						icon_signal,
						String::from("img/icons/adventure_league/") + al_icon_filename + ".svg",
					)
				}),
			));
		}
		root_menu.push(ContextMenuItem::new_separator());
		root_menu.push(ContextMenuItem::new_submenu(
			"Set Adventure League season icon",
			adventure_league_icon_menu,
		));
	}

	match create_context_menu(&root_menu, mouse_event) {
		Ok(_) => (),
		Err(err) => log_and_panic(err.reason),
	};
}

pub fn trash_bin_icon() -> impl leptos::IntoView {
	leptos::svg::svg().attr("viewBox", "0 0 16 16").child(leptos::svg::path().attr(
		"d",
		"M 2.5 1 A 1 1 0 0 0 1.5 2 v1a1 1 0 0 0 1 1H3v9a2 2 0 0 0 2 2h6a2 2 0 0 0 2-2V4h.5a1 1 0 0 0 1-1V2a1 1 0 0 0-1-1H10a1 1 0 0 0-1-1H7a1 1 0 0 0-1 1zm3 4a.5.5 0 0 1 .5.5v7a.5.5 0 0 1-1 0v-7a.5.5 0 0 1 .5-.5M8 5a.5.5 0 0 1 .5.5v7a.5.5 0 0 1-1 0v-7A.5.5 0 0 1 8 5m3 .5v7a.5.5 0 0 1-1 0v-7a.5.5 0 0 1 1 0"
	))
}
