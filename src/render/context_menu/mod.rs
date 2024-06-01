pub(crate) mod content;

use std::{
	sync::{LazyLock, Mutex},
	time::Duration,
};

use leptos::{
	IntoView,
	leptos_dom::logging::console_warn,
	prelude::{
		ClassAttribute as _, CustomAttribute as _, Dispose as _, ElementChild as _, Get, GetUntracked as _,
		GlobalAttributes as _, IntoAny, NodeRefAttribute as _, OnAttribute as _, Set as _,
		StyleAttribute as _,
	},
};
use wasm_bindgen::JsCast;

use super::{
	get_document, log_and_panic,
	utils::{
		DistanceType, HorizontalReference, Orientation, RelativeHorizontalCoordinate, RelativePosition,
		RelativeVerticalCoordinate, VerticalReference,
	},
};
use content::{ContextMenuContent, ContextMenuItem};

const COVER_DIV_ID: &str = "context-menu-cover";
const CONTEXT_MENU_CLASS: &str = "context-menu";
const PARENT_ID_ATTRIBUTE: &str = "data-parent-id";
const LEVEL_ATTRIBUTE: &str = "data-level";
const MAGIC_TRIANGLE_DELAY: u64 = 300; // milliseconds
const MAGIC_TRIANGLE_TOLERANCE: f64 = 0.5; // pixels
const MOUSE_LOC_MIN_DIST: f64 = 2.0; // pixels

static CONTEXT_VISIBILITY_SIGNALS: LazyLock<Mutex<Vec<Vec<leptos::prelude::RwSignal<String>>>>> =
	LazyLock::new(|| Mutex::new(Vec::new()));
static MAGIC_TRIANGLE_CORNERS: LazyLock<Mutex<(f64, f64, f64)>> =
	LazyLock::new(|| Mutex::new((-1.0, -1.0, -1.0))); // left, top, bottom
static LAST_MOUSE_LOC: LazyLock<Mutex<(f64, f64)>> = LazyLock::new(|| Mutex::new((-1.0, -1.0)));
static MOUSEOVER_TIMEOUT: LazyLock<Mutex<Option<leptos::leptos_dom::helpers::TimeoutHandle>>> =
	LazyLock::new(|| Mutex::new(None));

#[derive(Debug)]
pub struct ContextMenuCreationError {
	pub reason: &'static str,
}

#[derive(Debug)]
struct ContextMenuRemovalError {}

pub fn create_context_menu(
	content: &ContextMenuContent,
	mouse_event: &web_sys::MouseEvent,
) -> Result<(), ContextMenuCreationError> {
	let page_offset = (mouse_event.page_x() as f32, mouse_event.page_y() as f32);
	let client_offset = (mouse_event.client_x() as f32, mouse_event.client_y() as f32);
	let document = get_document().ok_or(ContextMenuCreationError { reason: "Could not get document" })?;

	// spawn invisible div to capture clicks
	let cover_div = leptos::html::div()
		.id(COVER_DIV_ID)
		.on(leptos::ev::click, move |ev| {
			ev.prevent_default();
			remove_all_context_menus().unwrap();
		})
		.on(leptos::ev::mousemove, |ev| {
			let mut mouse_loc = LAST_MOUSE_LOC.lock().unwrap_or_else(|mut e| {
				console_warn("LAST_MOUSE_LOC is poisoned, clearing");
				**e.get_mut() = (-1.0, -1.0);
				LAST_MOUSE_LOC.clear_poison();
				e.into_inner()
			});
			let current_loc = (ev.client_x() as f64, ev.client_y() as f64);
			if ((current_loc.0 - mouse_loc.0).powi(2) + (current_loc.1 - mouse_loc.1).powi(2)).sqrt()
				> MOUSE_LOC_MIN_DIST
			{
				*mouse_loc = current_loc;
			}
		});
	let cover_ref = leptos::prelude::NodeRef::new();
	let _ = leptos_use::use_event_listener_with_options(
		cover_ref,
		leptos::ev::wheel,
		|ev| {
			let target: web_sys::HtmlElement = leptos::prelude::event_target(&ev);
			if target.id() == COVER_DIV_ID {
				ev.prevent_default();
			}
		},
		leptos_use::UseEventListenerOptions::default().passive(false),
	);
	let _ = leptos_use::use_event_listener_with_options(
		cover_ref,
		leptos::ev::touchmove,
		|ev| {
			let target: web_sys::HtmlElement = leptos::prelude::event_target(&ev);
			if target.id() == COVER_DIV_ID {
				ev.prevent_default();
			}
		},
		leptos_use::UseEventListenerOptions::default().passive(false),
	);
	let cover_div_node = cover_div.node_ref(cover_ref);

	leptos::mount::mount_to_body(|| cover_div_node);

	// get mounted node
	let cover_node = cover_ref.get().expect("Could not get created cover node.");

	// get orientation and anchor point
	let orientation = get_root_orientation(
		&document,
		&page_offset,
		&(cover_node.client_width() as f32, cover_node.client_height() as f32),
		&client_offset,
	)?;

	// create the root level
	create_context_menu_level(content, &document, &None, cover_node.into(), orientation, 0, None)?;
	Ok(())
}

fn remove_all_context_menus() -> Result<(), ContextMenuRemovalError> {
	let document = get_document().ok_or(ContextMenuRemovalError {})?;
	// remove all context menu panes
	let context_menu_elements = document.get_elements_by_class_name(CONTEXT_MENU_CLASS);
	for menu_index in 0..context_menu_elements.length() {
		if let Some(el) = context_menu_elements.item(menu_index) {
			el.remove();
		}
	}
	// remove cover div
	if let Some(el) = document.get_element_by_id(COVER_DIV_ID) {
		el.remove();
	}
	// dispose signals
	let mut all_signals = CONTEXT_VISIBILITY_SIGNALS.lock().unwrap_or_else(|mut e| {
		console_warn("CONTEXT_SIGNALS is poisoned, clearing");
		**e.get_mut() = Vec::new();
		CONTEXT_VISIBILITY_SIGNALS.clear_poison();
		e.into_inner()
	});
	for level_signals in all_signals.iter() {
		for signal in level_signals.iter() {
			signal.dispose();
		}
	}
	*all_signals = Vec::new();
	Ok(())
}

fn hide_higher_level_context_menus(level: usize) {
	let all_signals = CONTEXT_VISIBILITY_SIGNALS.lock().unwrap_or_else(|mut e| {
		console_warn("CONTEXT_SIGNALS is poisoned, clearing");
		**e.get_mut() = Vec::new();
		CONTEXT_VISIBILITY_SIGNALS.clear_poison();
		e.into_inner()
	});
	if all_signals.len() > level + 1 {
		for clear_level in (level + 1)..all_signals.len() {
			for signal in all_signals[clear_level].iter() {
				signal.set(String::from("hidden"));
			}
		}
	}
}

fn get_root_orientation(
	document: &web_sys::Document,
	page_offset: &(f32, f32),
	cover_size: &(f32, f32),
	client_offset: &(f32, f32),
) -> Result<RelativePosition, ContextMenuCreationError> {
	let document_element = document.document_element().ok_or(ContextMenuCreationError {
		reason: "Could not get global documentElement",
	})?;
	let lr = client_offset.0 < ((document_element.client_width() / 2i32) as f32);
	let tb = client_offset.1 < ((document_element.client_height() / 2i32) as f32);
	let anchor_horizontal = if lr {
		RelativeHorizontalCoordinate {
			reference: HorizontalReference::Left,
			distance: page_offset.0,
			unit: DistanceType::Pixels,
		}
	} else {
		RelativeHorizontalCoordinate {
			reference: HorizontalReference::Right,
			distance: cover_size.0 - page_offset.0,
			unit: DistanceType::Pixels,
		}
	};
	let anchor_vertical = if tb {
		RelativeVerticalCoordinate {
			reference: VerticalReference::Top,
			distance: page_offset.1,
			unit: DistanceType::Pixels,
		}
	} else {
		RelativeVerticalCoordinate {
			reference: VerticalReference::Bottom,
			distance: cover_size.1 - page_offset.1,
			unit: DistanceType::Pixels,
		}
	};
	let orientation = match (lr, tb) {
		(true, true) => Orientation::BottomRight,
		(true, false) => Orientation::TopRight,
		(false, true) => Orientation::BottomLeft,
		(false, false) => Orientation::TopLeft,
	};
	Ok(RelativePosition {
		anchor: (anchor_horizontal, anchor_vertical),
		orientation,
	})
}

fn create_context_menu_level(
	content: &ContextMenuContent,
	document: &web_sys::Document,
	parent_id: &Option<String>,
	parent: web_sys::HtmlElement,
	position: RelativePosition,
	level: usize,
	visibility_signal: Option<leptos::prelude::RwSignal<String>>,
) -> Result<
	leptos_use::UseElementBoundingReturn<impl Fn() + Clone + Send + std::marker::Sync>,
	ContextMenuCreationError,
> {
	let orientation = position.orientation;
	let document_element = match document.document_element() {
		Some(el) => Ok(el),
		None => Err(ContextMenuCreationError {
			reason: "Could not get global documentElement",
		}),
	}?;
	let level_id = uuid::Uuid::new_v4().to_string();
	let top_signal = leptos::prelude::RwSignal::new(match position.anchor.1.unit {
		// DistanceType::Percentage => format!("{}%", position.anchor.1.distance),
		DistanceType::Pixels => format!("{}px", position.anchor.1.distance),
	});
	let (overflow_signal_read, overflow_signal_write) = leptos::prelude::signal(String::from("hidden"));
	let (max_height_signal_read, max_height_signal_write) = leptos::prelude::signal(String::from("none"));

	let mut all_context_menu_signals = CONTEXT_VISIBILITY_SIGNALS.lock().unwrap_or_else(|mut e| {
		console_warn("CONTEXT_SIGNALS is poisoned, clearing");
		**e.get_mut() = Vec::new();
		CONTEXT_VISIBILITY_SIGNALS.clear_poison();
		e.into_inner()
	});
	while all_context_menu_signals.len() < (level + 2) {
		all_context_menu_signals.push(Vec::new());
	}

	let pane = leptos::html::div()
		.id(level_id.clone())
		.class(CONTEXT_MENU_CLASS)
		.attr(PARENT_ID_ATTRIBUTE, parent_id.clone().unwrap_or_default())
		.attr(LEVEL_ATTRIBUTE, format!("{}", level))
		.style(move || {
			format!(
				"{}:{}{};{}:{};overflow-y:{};max-height:{};visibility:{}",
				match position.anchor.0.reference {
					HorizontalReference::Left => "left",
					HorizontalReference::Right => "right",
				},
				position.anchor.0.distance,
				match position.anchor.0.unit {
					// DistanceType::Percentage => "%",
					DistanceType::Pixels => "px",
				},
				match position.anchor.1.reference {
					VerticalReference::Top => "top",
					VerticalReference::Bottom => "bottom",
				},
				top_signal.get(),
				overflow_signal_read.get(),
				max_height_signal_read.get(),
				match visibility_signal {
					Some(signal) => signal.get(),
					None => String::from("visible"),
				}
			)
		})
		.on(leptos::ev::click, |event| event.stop_propagation())
		.child(
			leptos::html::ul().child(
				content
					.iter()
					.map(|item| match item {
						ContextMenuItem::RegularItem { text, action } => {
							let action_clone = *action;
							leptos::html::li()
								.child((
									leptos::html::span().class("icon_span"),
									leptos::html::span().class("text_span").child(*text),
									leptos::html::span().class("sub_span"),
								))
								.on(leptos::ev::click, move |_ev| {
									action_clone.dispatch(());
									let _ = remove_all_context_menus();
								})
								.on(leptos::ev::mouseover, move |ev| {
									handle_mouseover(
										move || hide_higher_level_context_menus(level),
										level,
										&ev,
									);
								})
								.into_any()
						},
						ContextMenuItem::SubmenuItem { text, sub_menu } => {
							let sub_el_id = uuid::Uuid::new_v4().to_string();
							let sub_el_id_clone = sub_el_id.clone();
							let submenu_signal = leptos::prelude::RwSignal::new(String::from("visible"));
							all_context_menu_signals[level + 1].push(submenu_signal);
							let sub_menu_clone = sub_menu.clone();
							let sub_menu_clone2 = sub_menu_clone.clone();

							leptos::html::li()
								.id(sub_el_id.clone())
								.child((
									leptos::html::span().class("icon_span"),
									leptos::html::span().class("text_span").child(*text),
									leptos::html::span().class("sub_span").child("›"),
								))
								.style(move || {
									format!(
										"background-color:{}",
										match submenu_signal.get().as_str() {
											"hidden" => "inherit",
											_ => "#aaa",
										}
									)
								})
								.on(leptos::ev::click, move |_ev| {
									process_submenu_activation(
										&sub_menu_clone,
										level,
										sub_el_id.clone(),
										&orientation,
										submenu_signal,
									);
								})
								.on(leptos::ev::mouseover, move |ev| {
									let sub_id_clone = sub_el_id_clone.clone();
									let sub_menu_clone3 = sub_menu_clone2.clone();
									handle_mouseover(
										move || {
											process_submenu_activation(
												&sub_menu_clone3,
												level,
												sub_id_clone.clone(),
												&orientation,
												submenu_signal,
											)
										},
										level,
										&ev,
									);
								})
								.into_any()
						},
						ContextMenuItem::Separator => leptos::html::li()
							.class("separator")
							.on(leptos::ev::mouseover, move |ev| {
								handle_mouseover(move || hide_higher_level_context_menus(level), level, &ev);
							})
							.into_any(),
					})
					.collect::<Vec<_>>(),
			),
		);

	let pane_node_ref = leptos::prelude::NodeRef::<leptos::html::Div>::new();
	let pane_node = pane.node_ref(pane_node_ref);
	leptos::prelude::mount_to(parent, move || pane_node.into_view()).forget();

	let pane_bounding_box = leptos_use::use_element_bounding(pane_node_ref);

	vertical_resize(
		pane_bounding_box.height.get_untracked(),
		pane_bounding_box.y.get_untracked(),
		pane_bounding_box.bottom.get_untracked(),
		document_element,
		position,
		top_signal,
		overflow_signal_write,
		max_height_signal_write,
	);
	Ok(pane_bounding_box)
}

fn show_or_build_sublevel(
	content: &ContextMenuContent,
	parent_id: String,
	orientation: &Orientation,
	parent_level: usize,
	visibility_signal: leptos::prelude::RwSignal<String>,
) -> Result<web_sys::DomRect, ContextMenuCreationError> {
	let document = get_document().ok_or(ContextMenuCreationError {
		reason: "Could not get document for submenu",
	})?;
	let parent_el = document
		.get_element_by_id(&parent_id)
		.ok_or(ContextMenuCreationError {
			reason: "Could not get parent element for submenu",
		})?;
	let context_menu_elements = document.get_elements_by_class_name(CONTEXT_MENU_CLASS);
	for menu_index in 0..context_menu_elements.length() {
		if let Some(el) = context_menu_elements.item(menu_index)
			&& el.get_attribute(PARENT_ID_ATTRIBUTE).unwrap_or(String::new()) == parent_id
		{
			visibility_signal.set(String::from("visible"));
			return Ok(el.get_bounding_client_rect());
		};
	}
	let parent_bounding_rect = parent_el.get_bounding_client_rect();
	let horizontal_anchor = match *orientation {
		Orientation::BottomLeft | Orientation::TopLeft => RelativeHorizontalCoordinate {
			reference: HorizontalReference::Right,
			distance: parent_bounding_rect.x() as f32,
			unit: DistanceType::Pixels,
		},
		Orientation::BottomRight | Orientation::TopRight => RelativeHorizontalCoordinate {
			reference: HorizontalReference::Left,
			distance: parent_bounding_rect.right() as f32,
			unit: DistanceType::Pixels,
		},
	};
	let vertical_anchor = match *orientation {
		Orientation::BottomLeft | Orientation::BottomRight => RelativeVerticalCoordinate {
			reference: VerticalReference::Top,
			distance: parent_bounding_rect.y() as f32,
			unit: DistanceType::Pixels,
		},
		Orientation::TopLeft | Orientation::TopRight => RelativeVerticalCoordinate {
			reference: VerticalReference::Bottom,
			distance: parent_bounding_rect.bottom() as f32,
			unit: DistanceType::Pixels,
		},
	};
	let cover_div = document
		.get_element_by_id(COVER_DIV_ID)
		.ok_or(ContextMenuCreationError {
			reason: "Could not get cover div for submenu",
		})?
		.dyn_into()
		.map_err(|_| ContextMenuCreationError {
			reason: "Could not cast cover div to HtmlElement",
		})?;
	match create_context_menu_level(
		content,
		&document,
		&Some(parent_id),
		cover_div,
		RelativePosition {
			anchor: (horizontal_anchor, vertical_anchor),
			orientation: *orientation,
		},
		parent_level + 1,
		Some(visibility_signal),
	) {
		Ok(interactive_bounding_rect) => web_sys::DomRect::new_with_x_and_y_and_width_and_height(
			interactive_bounding_rect.x.get_untracked(),
			interactive_bounding_rect.y.get_untracked(),
			interactive_bounding_rect.width.get_untracked(),
			interactive_bounding_rect.height.get_untracked(),
		)
		.map_err(|_| ContextMenuCreationError { reason: "Error converting bounding rect" }),
		Err(err) => Err(err),
	}
}

#[allow(clippy::too_many_arguments)]
fn vertical_resize(
	pane_height: f64,
	pane_y: f64,
	pane_bottom: f64,
	document_element: web_sys::Element,
	initial_position: RelativePosition,
	top_signal: leptos::prelude::RwSignal<String>,
	overflow_signal_write: leptos::prelude::WriteSignal<String>,
	max_height_signal_write: leptos::prelude::WriteSignal<String>,
) {
	let view_height = document_element.client_height() as f64;
	if pane_height > view_height {
		// must make scrollable
		let offset = match initial_position.orientation {
			Orientation::BottomLeft | Orientation::BottomRight => pane_y,
			Orientation::TopLeft | Orientation::TopRight => view_height - pane_bottom,
		};
		top_signal.set(String::from("calc(") + &top_signal.get_untracked() + &format!(" - {:.2}px)", offset));
		overflow_signal_write.set(String::from("auto"));
		max_height_signal_write.set(format!("{:.2}px", view_height));
	} else {
		// keep non-scrollable, move up if needed
		let height_overflow = match initial_position.orientation {
			Orientation::BottomLeft | Orientation::BottomRight => pane_y + pane_height - view_height,
			Orientation::TopLeft | Orientation::TopRight => pane_height - pane_bottom,
		};
		if height_overflow > 0.0 {
			top_signal.set(
				String::from("calc(") + &top_signal.get_untracked() + &format!(" - {:.2}px)", height_overflow),
			)
		}
	}
}

fn process_submenu_activation(
	content: &ContextMenuContent,
	level: usize,
	element_id: String,
	orientation: &Orientation,
	submenu_signal: leptos::prelude::RwSignal<String>,
) {
	hide_higher_level_context_menus(level);
	match show_or_build_sublevel(content, element_id, orientation, level, submenu_signal) {
		Ok(rect) => set_magic_triangle_corners(&rect),
		Err(err) => log_and_panic(err.reason),
	}
}

fn handle_mouseover(function: impl Fn() + 'static, level: usize, event: &leptos::ev::MouseEvent) {
	let delay = get_activation_delay(level, event);
	let mut timeout = MOUSEOVER_TIMEOUT.lock().unwrap_or_else(|mut e| {
		console_warn("MOUSEOVER_TIMEOUT is poisoned, clearing");
		**e.get_mut() = None;
		MOUSEOVER_TIMEOUT.clear_poison();
		e.into_inner()
	});
	*timeout = match *timeout {
		Some(to) => {
			to.clear();
			None
		},
		None => None,
	};
	if delay > 0 {
		let delayed_handle = leptos::prelude::set_timeout_with_handle(function, Duration::from_millis(delay));
		match delayed_handle {
			Ok(handle) => *timeout = Some(handle),
			Err(_) => console_warn("Failed to set timeout for submenu show/hide"),
		};
	} else {
		function();
	}
}

fn set_magic_triangle_corners(rect: &web_sys::DomRect) {
	let mut current_corners = MAGIC_TRIANGLE_CORNERS.lock().unwrap_or_else(|mut e| {
		console_warn("MAGIC_TRIANGLE_CORNERS is poisoned, clearing");
		**e.get_mut() = (-1.0, -1.0, -1.0);
		MAGIC_TRIANGLE_CORNERS.clear_poison();
		e.into_inner()
	});
	*current_corners = (rect.left(), rect.top(), rect.bottom());
}

fn get_activation_delay(level: usize, event: &web_sys::MouseEvent) -> u64 {
	let all_signals = CONTEXT_VISIBILITY_SIGNALS.lock().unwrap_or_else(|mut e| {
		console_warn("CONTEXT_SIGNALS is poisoned, clearing");
		**e.get_mut() = Vec::new();
		CONTEXT_VISIBILITY_SIGNALS.clear_poison();
		e.into_inner()
	});
	if all_signals.len() < (level + 2) {
		// no submenus on this level, activate immediately
		return 0;
	}
	let mut any_visible = false;
	for signal in all_signals[level + 1].iter() {
		if signal.get_untracked().as_str() == "visible" {
			any_visible = true;
		}
	}
	if !any_visible {
		// no submenus are visible, activate immediately
		return 0;
	}

	let triangle_corners = MAGIC_TRIANGLE_CORNERS.lock().unwrap_or_else(|mut e| {
		console_warn("MAGIC_TRIANGLE_CORNERS is poisoned, clearing");
		**e.get_mut() = (-1.0, -1.0, -1.0);
		MAGIC_TRIANGLE_CORNERS.clear_poison();
		e.into_inner()
	});
	if triangle_corners.0 == -1.0 {
		return 0;
	}

	let last_mouse_loc = LAST_MOUSE_LOC.lock().unwrap_or_else(|mut e| {
		console_warn("LAST_MOUSE_LOC is poisoned, clearing");
		**e.get_mut() = (-1.0, -1.0);
		LAST_MOUSE_LOC.clear_poison();
		e.into_inner()
	});
	if last_mouse_loc.0 == -1.0 {
		console_warn("active submenu but last mouse loc not set");
		return 0;
	}

	let current_mouse_loc = (event.client_x() as f64, event.client_y() as f64);
	if (current_mouse_loc.0 > triangle_corners.0) || (current_mouse_loc.0 < last_mouse_loc.0) {
		// definitely outside magic triangle
		return 0;
	}
	let top_slope = (triangle_corners.1 - last_mouse_loc.1) / (triangle_corners.0 - last_mouse_loc.0);
	let bottom_slope = (triangle_corners.2 - last_mouse_loc.1) / (triangle_corners.0 - last_mouse_loc.0);
	if (current_mouse_loc.1
		> (top_slope * (current_mouse_loc.0 + MAGIC_TRIANGLE_TOLERANCE - last_mouse_loc.0) + last_mouse_loc.1
			- MAGIC_TRIANGLE_TOLERANCE))
		&& (current_mouse_loc.1
			< (bottom_slope * (current_mouse_loc.0 + MAGIC_TRIANGLE_TOLERANCE - last_mouse_loc.0)
				+ last_mouse_loc.1
				+ MAGIC_TRIANGLE_TOLERANCE))
	{
		// inside magic triangle
		return MAGIC_TRIANGLE_DELAY;
	}
	// outside magic triangle
	0
}
