use std::{cmp::max, iter::zip};

use leptos::{
	leptos_dom::logging::{console_error, console_log, console_warn},
	prelude::{
		ClassAttribute as _, CollectView as _, CustomAttribute as _, Dispose as _, ElementChild as _,
		Get as _, GetUntracked as _, GlobalAttributes as _, IntoAny, OnAttribute as _, PropAttribute as _,
		Read as _, ReadUntracked as _, Set as _, StyleAttribute as _, Update as _, Write as _,
	},
};

use crate::{
	Character,
	config::CONFIG,
	domain::{
		general_info::{ClassLevel, calculate_level_from_experience, get_minimum_experience_for_level},
		types::SignalField,
	},
	render::{
		error::RenderError,
		interface::{
			adapter_helper_get_all_class_ids, adapter_helper_get_all_subclasses,
			adapter_helper_get_available_subclasses_for_class, adapter_helper_get_background_names,
			adapter_helper_get_background_option_title, adapter_helper_get_background_options,
			adapter_helper_get_backgrounds, adapter_helper_get_class, adapter_helper_get_race_names,
			adapter_helper_get_race_variant_names, adapter_helper_get_race_variants, adapter_helper_get_races,
			adapter_helper_get_subclass, adapter_helper_get_subclass_names,
			adapter_helper_get_subclass_type_name, adapter_helper_race_needs_previous_race, apply_background,
			apply_classes, apply_race,
		},
		utils::{
			SignalFuture, create_number_input, make_icon_context_menu, remove_modal, show_modal,
			trash_bin_icon,
		},
	},
};

type ClassesInput = leptos::prelude::RwSignal<
	Vec<(leptos::prelude::RwSignal<u8>, leptos::prelude::RwSignal<String>, leptos::prelude::RwSignal<String>)>,
>;
type ClassReference = (String, String, String, Vec<(String, String, String, String)>);

pub fn render_header(character: &Character, stats_page: web_sys::HtmlElement) -> Result<(), RenderError> {
	console_log(">> Rendering header pane");

	let header_pane = leptos::html::div()
		.id("character_header_pane")
		.class("pane header")
		.style("width: 100%;height: 16.4%")
		.child((
			create_icon(character.general_info.player_icon),
			create_character_name(character.general_info.name),
			create_character_level(character.general_info.level, character.general_info.experience),
			create_class_display(
				character.general_info.classes,
				character.general_info.level,
				character.general_info.experience,
			),
			create_player_name(character.general_info.player_name),
			create_background(character.general_info.background, character.general_info.background_option),
			create_race(character.general_info.race, character.general_info.race_previous),
			create_experience_fields(
				character.general_info.level,
				character.general_info.experience,
				character.general_info.next_level_experience,
			),
		));
	leptos::mount::mount_to(stats_page, || header_pane).forget();
	Ok(())
}

fn create_character_name(character_name: SignalField<String>) -> impl leptos::IntoView {
	leptos::html::div().child((
		leptos::html::input()
			.id("character_name")
			.class("inputfield-regular")
			.style("top: 54.0%;left: 9.5%;width: 25.0%;height: 14.0%;font-size: 180%;font-size: max(70%, min(190%, calc(12% + 4190%/attr(data-chars type(<number>), 25))))")
			.attr("data-chars", move || max(character_name.read().len(), 1))
			.attr("type", "text")
			.attr("value", character_name.get_untracked())
			.prop("value", move || character_name.get())
			.on(leptos::ev::change, move |event| {
				character_name.set(leptos::prelude::event_target_value(&event))
			}),
		leptos::html::label()
			.id("character_name_label")
			.class("textlabel dimmed-text")
			.style("top: 68.0%;left: 9.5%")
			.attr("for", "character_name")
			.child("CHARACTER NAME"),
	))
}

fn create_icon(player_icon: SignalField<String>) -> impl leptos::IntoView {
	let icon_config = CONFIG.icons.clone();
	leptos::html::button()
		.id("player_icon")
		.class(move || {
			String::from("inputfield-iconbutton printable-image")
				+ if player_icon.read() == String::from("img/icons/blank.svg") {
					" hide-in-print"
				} else {
					""
				}
		})
		.attr("title", "Click here to change the Header Icon.")
		.style(move || {
			format!(
				"top: 53.35%;left: 6.5%;width: 2.7%;height: 17.45%;background-image: url('{}')",
				player_icon.get()
			)
		})
		.on(leptos::ev::click, move |event| {
			make_icon_context_menu("player_icon", player_icon, &event, true, icon_config.clone());
		})
}

fn create_class_display(
	class_levels: SignalField<Vec<ClassLevel>>,
	char_level: leptos::prelude::Memo<u8>,
	experience: SignalField<u32>,
) -> impl leptos::IntoView {
	let class_text = leptos::prelude::Memo::new(move |_| {
		let classes = class_levels.read();
		match classes.len() {
			0 => String::from("<Click here to select a class>"),
			1 => classes[0].name.clone(),
			_ => {
				let mut full_str = String::new();
				let mut first = true;
				for class in classes.iter() {
					if !first {
						full_str += ", "
					}
					full_str += format!("{} [{}]", class.name, class.level).as_str();
					first = false;
				}
				full_str
			},
		}
	});
	leptos::html::div()
		.id("classes")
		.class(move || {
			if class_levels.read().is_empty() {
				"display-field print-underline hide-in-print"
			} else {
				"display-field"
			}
		})
		.style(
			"top: 43%;left: 44.5%;width: 29%;height: 11%;pointer-events: auto;cursor: pointer;font-size: 130%;font-size: max(45%, min(150%, calc(-4% + 5442%/attr(data-chars type(<number>), 41))))",
		)
		.attr("data-chars", move || max(class_text.read().len(), 1))
		.title("Click here to change your class(es).")
		.child(move || class_text.get())
		.on(leptos::ev::click, move |_| show_class_selection_modal(class_levels, char_level, experience, 0))
}

fn create_character_level(
	level: leptos::prelude::Memo<u8>,
	experience: SignalField<u32>,
) -> impl leptos::IntoView {
	let level_input_signal = leptos::prelude::RwSignal::new(level.get_untracked());
	let _ = leptos::prelude::Effect::new(move || {
		let current_level = level.get();
		if current_level != level_input_signal.get_untracked() {
			level_input_signal.set(current_level);
		}
	});
	leptos::html::div().child((
		create_number_input(
			level_input_signal,
			"top: 43%;left: 40.7%;width: 3.3%;height: 11.5%;font-size: 150%",
			None,
			false,
			false,
			true,
			true,
		)
		.id("level")
		.title("The total level.")
		.required(true)
		.min(0)
		.on(leptos::ev::change, move |_| {
			experience.set(get_experience_for_new_level(
				level_input_signal.get_untracked(),
				level.get_untracked(),
				experience.get_untracked(),
			));
		}),
		leptos::html::label()
			.id("level_label")
			.class("textlabel dimmed-text")
			.style("top: 54.5%;left: 41.0%")
			.attr("for", "level")
			.child("LEVEL & CLASS"),
	))
}

fn create_player_name(player_name: SignalField<String>) -> impl leptos::IntoView {
	leptos::html::div().child((
		leptos::html::input()
			.id("player_name")
			.class(move || {
				if player_name.read().is_empty() {
					"inputfield-regular print-underline"
				} else {
					"inputfield-regular"
				}
			})
			.style("text-align: left;top: 43%;left: 73.9%;height: 11.5%;width: 18.4%;font-size: 130%;font-size: max(65%, min(160%, calc(-1% + 3064%/attr(data-chars type(<number>), 23))))")
			.attr("data-chars", move || max(player_name.read().len(), 1))
			.attr("type", "text")
			.attr("value", player_name.get_untracked())
			.prop("value", move || player_name.get())
			.on(leptos::ev::change, move |event| player_name.set(leptos::prelude::event_target_value(&event))),
		leptos::html::label()
			.id("player_name_label")
			.class("textlabel dimmed-text")
			.style("top: 54.5%;left: 73.8%")
			.attr("for", "player_name")
			.child("PLAYER NAME"),
	))
}

fn create_experience_fields(
	level: leptos::prelude::Memo<u8>,
	experience: SignalField<u32>,
	next_level_experience: leptos::prelude::Memo<u32>,
) -> impl leptos::IntoView {
	let experience_input_signal = leptos::prelude::RwSignal::<u32>::new(experience.get_untracked());
	let experience_add_signal = leptos::prelude::RwSignal::<u32>::new(0);
	let react_to_experience_change = move || {
		let new_value = experience_input_signal.get_untracked();
		let current_level = level.get_untracked();
		let new_level = calculate_level_from_experience(new_value);
		if new_level != current_level {
			show_experience_modal(experience, new_value, current_level, new_level, experience_input_signal);
		} else {
			experience.set(new_value);
		}
	};
	let _ = leptos::prelude::Effect::new(move || {
		let current_experience = experience.get();
		if current_experience != experience_input_signal.get_untracked() {
			experience_input_signal.set(current_experience);
		}
	});
	leptos::html::div()
		.id("player_experience_box")
		.style("position: absolute;top: 63.4%;left: 73.9%;width: 18.4%;height: 11.65%")
		.child((
			create_number_input(
				experience_input_signal,
				"position: absolute;top: 0%;left: 0%;width: 34%;height: 100%;font-size: 130%;font-size: max(100%, min(160%, calc(953%/attr(data-chars type(<number>), 7))))",
				None,
				false,
				false,
				true,
				true
			)
			.id("experience")
			.title("The total amount of experience points.")
			.on(leptos::ev::change, move |_| {
				react_to_experience_change();
			}),
			leptos::html::label()
				.id("experience_label")
				.class("textlabel dimmed-text")
				.style("top: 95%;left: 0%")
				.attr("for", "experience")
				.child("EXPERIENCE"),
			leptos::html::button()
				.id("add_experience_button")
				.class("button hide-in-print")
				.style("left: 34%;top: 30%;width: 13.5%;height: 59%")
				.title("Add the experience points on the right to the total to the left.")
				.child("Add:")
				.on(leptos::ev::click, move |_| {
					experience_input_signal
						.set(experience_input_signal.get_untracked() + experience_add_signal.get_untracked());
					react_to_experience_change();
					experience_add_signal.set(0);
				}),
			create_number_input(
				experience_add_signal,
				"color: #0056D3;position: absolute;top: 0%;left: 47%;width: 25%;height: 100%;font-size: 130%;font-size: max(90%, min(150%, calc(25% + 497%/attr(data-chars type(<number>), 5))))",
				None,
				false,
				true,
				false,
				false
			)
			.id("experience_to_add")
			.title("Experience points that are to be added to the total to the left."),
			leptos::html::div()
				.id("next_level_experience")
				.class("display-field dimmed-text")
				.style("top: 0%;right: 0%;width: 29%;height: 100%;font-size: 130%;font-size: max(80%, min(150%, calc(-12% + 807%/attr(data-chars type(<number>), 6))))")
				.attr("data-chars", move || std::cmp::max(next_level_experience.read().to_string().len(), 1))
				.child(next_level_experience),
			leptos::html::label()
				.id("next_level_experience_label")
				.class("textlabel dimmed-text")
				.style("top: 95%;left: 72%")
				.attr("for", "next_level_experience")
				.child("Next Level"),
		))
}

fn create_background(
	background: SignalField<Option<String>>,
	background_option: SignalField<Option<String>>,
) -> impl leptos::IntoView {
	let background_ids = adapter_helper_get_backgrounds();
	let background_names = adapter_helper_get_background_names();
	let background_ids_clone = background_ids.clone();
	let background_names_clone = background_names.clone();
	let background_option_title: leptos::prelude::RwSignal<Option<String>> =
		leptos::prelude::RwSignal::new(None);
	let background_options: leptos::prelude::RwSignal<Option<Vec<String>>> =
		leptos::prelude::RwSignal::new(None);
	leptos::html::div()
		.style("position: absolute;top: 63.5%;left: 40.7%;width: 20.1%;height: 18%")
		.child((
			leptos::html::select()
				.id("background")
				.class(move || {if background.read().as_ref().is_none_or(|b| b.is_empty()) {"inputfield-regular hide-arrow print-underline"} else {"inputfield-regular hide-arrow"}})
				.style("top: 0%;left: 0%;width: 100%;height: 63.9%;text-align: left;font-size: 130%;font-size: max(71%, min(150%, calc(-6% + 3650%/attr(data-chars type(<number>), 28))))")
				.attr("data-chars", move || background.read().as_ref().and_then(|bo| background_ids_clone.iter().position(|el| el.eq(bo)).map(|ind| background_names_clone[ind].len())))
				// .prop("value",  move || background.get())
				.child((
					leptos::html::option()
						.value("")
						.child(""),
					background_ids
						.iter()
						.enumerate()
						.map(|(index, background_id)| {
							leptos::html::option()
								.value(background_id.clone())
								.child(background_names[index].clone())
								.selected(background_id.eq(background.read_untracked().as_ref().unwrap_or(&String::new())))
						})
						.collect_view(),
				))
				.on(leptos::ev::change, move |event| {
					let new_val = leptos::prelude::event_target_value(&event);
					let new_val_opt = match new_val.is_empty() {
						false => Some(new_val),
						true => None,
					};
					let old_background = background.get_untracked();
					if old_background != new_val_opt {
						background.set(new_val_opt.clone());
						background_option.set(None);
					}
					background_option_title
						.set(adapter_helper_get_background_option_title(new_val_opt.clone()));
					background_options.set(adapter_helper_get_background_options(new_val_opt.clone()));
					leptos::reactive::spawn_local(async move {
						apply_background(new_val_opt, old_background).await;
					});
				}),
			leptos::html::label()
				.id("background_label")
				.class("textlabel dimmed-text")
				.style("top: 60.5%;left: 1.5%")
				.attr("for", "background")
				.child("BACKGROUND"),
			leptos::html::select()
				.id("background_option")
				.class("inputfield-regular hide-arrow")
				.title(move || background_option_title.get())
				.style(move || {
					format!(
						"bottom: 0%;right: 0%;width: 58%;height: 35%;font-size: 60%;font-size: max(25%, min(83%, calc(-1% + 2090%/attr(data-chars type(<number>), 33)))){}",
						if background_options.read().is_none() {
							";display: none"
						} else {
							""
						}
					)
				})
				.attr("data-chars", move || background_option.read().as_ref().map_or(1, |bo| max(bo.len(), 1)))
				.child((
					leptos::html::option()
						.value("")
						.child(""),
					move || {
						background_options
							.read()
							.as_ref()
							.unwrap_or(&vec![])
							.iter()
							.map(|background_option_| {
								leptos::html::option()
									.value(background_option_.clone())
									.child(background_option_.clone())
									.selected(background_option_.eq(background_option.read_untracked().as_ref().unwrap_or(&String::new())))
							})
							.collect_view()
					},
				))
				.on(leptos::ev::change,  move |event| {
					let new_value = leptos::prelude::event_target_value(&event);
					let new_value_opt = match new_value.is_empty() {
						false => Some(new_value),
						true => None
					};
					background_option.set(new_value_opt);
				}),
		))
}

fn create_race(
	race: SignalField<Option<String>>,
	race_previous: SignalField<Option<String>>,
) -> impl leptos::IntoView {
	let race_map: Vec<_> =
		zip(adapter_helper_get_races().drain(..), adapter_helper_get_race_names().drain(..))
			.map(|(race_id, race_name)| {
				let mut variants = adapter_helper_get_race_variants(race_id.clone());
				if variants.is_empty() {
					(race_id, race_name, vec![])
				} else {
					let mut variant_names = adapter_helper_get_race_variant_names(race_id.clone());
					(race_id, race_name, zip(variants.drain(..), variant_names.drain(..)).collect())
				}
			})
			.collect();
	let race_map_clone = race_map.clone();
	leptos::html::div().style("position: absolute;top: 63.5%;left: 61%;width: 12.6%;height: 16%")
		.child((
			leptos::html::select()
				.id("race")
				.class(move || {if race.read().as_ref().is_none_or(|r| r.is_empty()) {"inputfield-regular hide-arrow print-underline"} else {"inputfield-regular hide-arrow"}})
				.style("top: 0%;left: 0%;width: 100%;height: 71.9%;text-align: left;font-size: 130%;font-size: max(64%, min(150%, calc(-7% + 2167%/attr(data-chars type(<number>), 16))))")
				.attr("data-chars", move || race.read().as_ref().map(|race_| race_map_clone.iter().filter_map(|(id, name, variants)| {if id.eq(race_) {Some(name.len())} else {variants.iter().filter_map(|(v_id, v_name)| {if v_id.eq(race_) {Some(v_name.len())} else {None}}).next()}}).next()))
				.child((
					leptos::html::option()
						.value("")
						.child(""),
					race_map
						.iter()
						.map(|(race_id, race_name, variants)| {
							if variants.is_empty() {
								leptos::html::option()
									.value(race_id.clone())
									.child(race_name.clone())
									.selected(race_id.eq(race.read_untracked().as_ref().unwrap_or(&String::new())))
									.into_any()
							} else {
								leptos::html::optgroup().label(race_name.clone()).child((
									leptos::html::option()
										.value(race_id.clone())
										.child(race_name.clone())
										.selected(race_id.eq(race.read_untracked().as_ref().unwrap_or(&String::new()))),
									variants.iter().map(|(variant_id, variant_name)| {
										leptos::html::option()
											.value(variant_id.clone())
											.child(variant_name.clone())
											.selected(variant_id.eq(race.read_untracked().as_ref().unwrap_or(&String::new())))
									}).collect_view()
								)).into_any()
							}
						})
						.collect_view(),
				))
				.on(leptos::ev::change, move |event| {
					let race_map_clone = race_map.clone();

					leptos::reactive::spawn_local(async move {
						let new_val = leptos::prelude::event_target_value(&event);
						let new_val_clone = new_val.clone();
						let new_val_opt = match new_val.is_empty() {
							false => Some(new_val),
							true => None,
						};
						let old_race = race.get_untracked();
						if old_race != new_val_opt {
							race.set(new_val_opt.clone());
							if adapter_helper_race_needs_previous_race(new_val_opt.clone()) {
								let previous_race_list = race_map_clone.iter().flat_map(|(race_id, race_name, variants)| {
									let mut race_and_variants = Vec::with_capacity(1 + variants.len());
									if race_id.ne(&new_val_clone) && !adapter_helper_race_needs_previous_race(Some(race_id.clone())) {
										race_and_variants.push((race_id.clone(), race_name.clone()));
										for (variant_id, variant_name) in variants.iter() {
											race_and_variants.push((variant_id.clone(), variant_name.clone()));
										}
									}
									race_and_variants
								}).collect::<Vec<_>>();

								let previous_race_selection_signal = leptos::prelude::RwSignal::new(false);
								let race_name = race_map_clone.iter().filter_map(|(id, name, variants)| {
									if id.eq(&new_val_clone) {
										Some(name)
									} else {
										variants.iter().filter_map(|(v_id, v_name)| {
											if v_id.eq(&new_val_clone) {Some(v_name)} else {None}
										}).next()
									}
								}).next().unwrap();
								show_previous_race_modal(
									race_previous,
									race_name,
									previous_race_selection_signal,
									previous_race_list
								);
								SignalFuture { signal: previous_race_selection_signal }.await;
								previous_race_selection_signal.dispose();
							}
						}
						console_log(format!("old race: {:?}, new race: {:?}", old_race, new_val_opt).as_str());
						leptos::reactive::spawn_local(async move {
							apply_race(new_val_opt, old_race).await;
						});
					})
				}),
			leptos::html::label()
				.id("race_label")
				.class("textlabel dimmed-text")
				.style("top: 68%;left: -1%")
				.attr("for", "race")
				.child("RACE")
		))
}

fn show_previous_race_modal(
	race_previous_signal: SignalField<Option<String>>,
	race_name: &String,
	finished_signal: leptos::prelude::RwSignal<bool>,
	previous_race_list: Vec<(String, String)>,
) {
	let chosen_previous_race = leptos::prelude::RwSignal::new(None);
	let no_previous_race_chosen = leptos::prelude::Memo::new(move |_| chosen_previous_race.get().is_none());

	let modal_content = leptos::html::div().child((
		leptos::html::span().child(format!(
			"The {} race requires that the character has (had) a previous race from which some traits are retained. Please select one from the list below. If cancelled, these traits can be added manually.",
			race_name
		)),
		leptos::html::ul().class("nobullets multicolumn").style("column-width: 17em;font-size: 60%").child(move || {
			previous_race_list
				.iter()
				.enumerate()
				.map(|(ind, (race_id, race_name))| {
					let option_id = format!("race_{}", ind);
					let option_id_clone = option_id.clone();
					let race_id_clone = race_id.clone();
					leptos::html::li().child((
						leptos::html::input()
							.r#type("radio")
							.id(option_id)
							.name("previous_race")
							.value(race_id.clone())
							.on(leptos::ev::change, move |event| {
								if leptos::prelude::event_target_checked(&event) {
									chosen_previous_race.set(Some(race_id_clone.clone()));
								}
							}),
						leptos::html::label()
							.r#for(option_id_clone)
							.child(race_name.clone()),
					))
				})
				.collect_view()
		}),
	));

	let apply_signal = leptos::prelude::RwSignal::<Option<String>>::new(None);
	leptos::prelude::Effect::new(move || {
		if let Some(modal_id) = apply_signal.get() {
			remove_modal(modal_id).unwrap_or_else(|err| console_log(&err.message));
			apply_signal.dispose();
			race_previous_signal.set(chosen_previous_race.get_untracked());
			chosen_previous_race.dispose();
			finished_signal.set(true);
		}
	});
	let cancel_signal = leptos::prelude::RwSignal::<Option<String>>::new(None);
	leptos::prelude::Effect::new(move || {
		if let Some(modal_id) = cancel_signal.get() {
			remove_modal(modal_id).unwrap_or_else(|err| console_log(&err.message));
			cancel_signal.dispose();
			race_previous_signal.set(None);
			chosen_previous_race.dispose();
			finished_signal.set(true);
		}
	});

	show_modal(
		"Select a previous race",
		modal_content,
		vec![
			(String::from("Apply"), apply_signal, Some(no_previous_race_chosen)),
			(String::from("Cancel"), cancel_signal, None),
		],
	);
}

fn show_experience_modal(
	experience: SignalField<u32>,
	new_value: u32,
	current_level: u8,
	new_level: u8,
	experience_input_signal: leptos::prelude::RwSignal<u32>,
) {
	let modal_content =
		leptos::html::div().child(format!(
			"The character has {} experience points, corresponding to level {}. The current level is {}. What would you like to do?", new_value, new_level, current_level
		));

	let level_up_signal = leptos::prelude::RwSignal::<Option<String>>::new(None);
	leptos::prelude::Effect::new(move || {
		if let Some(modal_id) = level_up_signal.get() {
			remove_modal(modal_id).unwrap_or_else(|err| console_log(&err.message));
			level_up_signal.dispose();
			experience.set(new_value);
		}
	});
	let cancel_signal = leptos::prelude::RwSignal::<Option<String>>::new(None);
	leptos::prelude::Effect::new(move || {
		if let Some(modal_id) = cancel_signal.get() {
			remove_modal(modal_id).unwrap_or_else(|err| console_log(&err.message));
			cancel_signal.dispose();
			experience_input_signal.set(experience.get_untracked());
		}
	});
	show_modal(
		"Level and Experience Points do not match.",
		modal_content,
		vec![
			(format!("Change level to {}", new_level), level_up_signal, None),
			(format!("Set experience back to {}", experience.get_untracked()), cancel_signal, None),
		],
	);
}

fn dispose_class_selection_signals(classes_input: ClassesInput, total_level: leptos::prelude::Memo<u8>) {
	total_level.dispose();
	match classes_input.try_read_untracked() {
		None => {
			console_warn("tried to read classes_input for disposal, but failed");
		},
		Some(class_input_vec) => {
			for class_input in class_input_vec.iter() {
				class_input.0.dispose();
				class_input.1.dispose();
				class_input.2.dispose();
			}
		},
	};
	classes_input.dispose();
}

pub fn show_class_selection_modal(
	class_levels: SignalField<Vec<ClassLevel>>,
	char_level: leptos::prelude::Memo<u8>,
	experience: SignalField<u32>,
	level_change: i16,
) {
	let all_class_ids = adapter_helper_get_all_class_ids();
	let mut all_classes = Vec::with_capacity(all_class_ids.len());
	{
		for class_id in all_class_ids.iter() {
			let class_props = adapter_helper_get_class(class_id.clone());
			let subclass_ids = adapter_helper_get_all_subclasses(class_id.clone());
			let mut subclasses = Vec::with_capacity(subclass_ids.len());
			for subclass_id in subclass_ids.iter() {
				let subclass_props = adapter_helper_get_subclass(class_id.clone(), subclass_id.clone());
				subclasses.push((
					subclass_id.clone(),
					subclass_props[0].clone(), // subclass name
					subclass_props[1].clone(), // fullname
					subclass_props[2].clone(), // source
				));
			}
			all_classes.push((class_id.clone(), class_props[0].clone(), class_props[1].clone(), subclasses));
			// id, name, source, subclasses
		}
	}
	let all_classes_clone = all_classes.clone();

	let classes_input = ClassesInput::new(Vec::new());
	if class_levels.read_untracked().is_empty() {
		classes_input.update(|inputs| {
			inputs.push((
				leptos::prelude::RwSignal::new(1),
				leptos::prelude::RwSignal::new(String::new()),
				leptos::prelude::RwSignal::new(String::new()),
			));
		});
	} else {
		classes_input.update(|inputs| {
			let mut first = true;
			for class_level in class_levels.read_untracked().iter() {
				let level_corr = if first {
					u8::try_from(max(0, i16::from(class_level.level) + level_change)).unwrap()
				} else {
					class_level.level
				};
				inputs.push((
					leptos::prelude::RwSignal::new(level_corr),
					leptos::prelude::RwSignal::new(class_level.id.clone()),
					leptos::prelude::RwSignal::new(
						class_level
							.subclass_id
							.as_ref()
							.map_or_else(String::new, |sub_id| sub_id.clone()),
					),
				));
				first = false;
			}
		});
	}
	let total_level = leptos::prelude::Memo::new(move |_| {
		let mut total = 0;
		for (class_level, _, _) in classes_input.read().iter() {
			total += class_level.get()
		}
		total
	});

	let modal_content = leptos::html::div().child((
		leptos::html::span().child(
			"Please select the level, class and (optionally) subclass for your character. If you want to multiclass, you can add additional rows. The first row is considered the 'primary class' and provides proficiencies."
		),
		leptos::html::table().style("justify-self:center").child((
			leptos::html::thead().child((
				leptos::html::th(),
				leptos::html::th().child("Level").style("min-width:2.5em"),
				leptos::html::th().child("Class").style("min-width:4em"),
				leptos::html::th().child("Source").style("min-width:3em"),
				leptos::html::th().child("Subclass").style("min-width:4em"),
				leptos::html::th().child("Source").style("min-width:3em"),
				leptos::html::th()
			)),
			leptos::html::tbody().child((
				move || {
					let mut class_rows = Vec::new();
					let mut first = true;
					for (index, (level, class, subclass)) in classes_input.read().iter().enumerate() {
						let all_classes_clone = all_classes.clone();
						let all_classes_clone_2 = all_classes.clone();
						let all_classes_clone_3 = all_classes.clone();
						let all_classes_clone_4 = all_classes.clone();
						let level_clone = *level;
						let class_clone = *class;
						let subclass_clone = *subclass;
						class_rows.push(leptos::html::tr().child((
							leptos::html::td(),
							leptos::html::td().child(create_number_input(*level, "width: 3em", None, true, false, false, true)),
							move || {
								let current_classes: Vec<String> = classes_input.read().iter().map(|row| row.1.get()).filter(|class_id| *class_id != class_clone.get()).collect();
								let mut class_list: Vec<_> = all_classes_clone.iter().map(|(class_id, class_name, _, _)| (class_id.clone(), class_name.clone())).filter(|(class_id, _)| !current_classes.contains(class_id)).collect();
								class_list.insert(0, (String::new(), String::new()));

								leptos::html::td().child(
										leptos::html::select()
										.class("inputfield-regular")
										.style("width:-webkit-fill-available;width:-moz-available;width:stretch")
										.child(
											class_list
											.iter()
											.map(|(class_id, class_name)| {leptos::html::option().value(class_id.clone()).child(class_name.clone())})
											.collect_view()
										)
										.on(leptos::ev::change, move |event| {let new_val = leptos::prelude::event_target_value(&event);class_clone.set(new_val.clone());subclass_clone.set(String::new());if !new_val.is_empty() && (level_clone.get_untracked() == 0) {level_clone.set(1)}})
								)
							},
							leptos::html::td().child(move || {
								let current_class_id = class_clone.get();
								for (class_id, _, class_source, _) in all_classes_clone_2.iter() {
									if class_id.eq(&current_class_id) {
										return class_source.clone()
									}
								}
								String::new()
							}),
							leptos::html::td().child(
								move || {
									let current_class_id = class_clone.get();
									let mut subclasses_list = vec![(String::new(), String::new())];
									for (class_id, _, _, subclasses) in all_classes_clone_3.iter() {
										if class_id.eq(&current_class_id) {
											subclasses_list.extend(subclasses.iter().map(|(subclass_id, subclass_name, _, _)| (subclass_id.clone(), subclass_name.clone())));
										}
									}
									leptos::html::select().class("inputfield-regular").style("min-width:3em;width:-webkit-fill-available;width:-moz-available;width:stretch").child(
										subclasses_list.iter().map(|(subclass_id, subclass_name)| {leptos::html::option().value(subclass_id.clone()).child(subclass_name.clone())}).collect_view()
									).on(leptos::ev::change, move |event| {subclass_clone.set(leptos::prelude::event_target_value(&event))})
								}
							),
							leptos::html::td().child(move || {
								let current_class_id = class_clone.get();
								match subclass_clone.get().as_str() {
									"" => String::new(),
									current_subclass_id => {
										for (class_id, _, _, subclasses) in all_classes_clone_4.iter() {
											if class_id.eq(&current_class_id) {
												for (subclass_id, _, _, subclass_source) in subclasses.iter() {
													if subclass_id.as_str() == current_subclass_id {
														return subclass_source.clone()
													}
												}
											}
										}
										String::new()
									},
								}
							}),
							leptos::html::td().child(move || {if !first {Some(
								leptos::html::button().class("button").style("width:30%").child(trash_bin_icon()).on(leptos::ev::click, move |_| {level_clone.dispose();class_clone.dispose();subclass_clone.dispose();classes_input.write().remove(index);})
							)} else {None}})
						)));
						first = false;
					}
					class_rows
				},
				leptos::html::tr().child((
					leptos::html::td(),
					leptos::html::td(),
					leptos::html::td(),
					leptos::html::td(),
					leptos::html::td(),
					leptos::html::td(),
					leptos::html::td().child(
						leptos::html::button().class("button").style("align:right").child("Add a new row").on(
							leptos::ev::click,
							move |_| classes_input.write().push((
								leptos::prelude::RwSignal::new(0),
								leptos::prelude::RwSignal::new(String::new()),
								leptos::prelude::RwSignal::new(String::new())
							))
						)
					)
				)),
				leptos::html::tr().child((
					leptos::html::td().child(leptos::html::span().style("font-weight:bold").child("Total:")),
					leptos::html::td().child(leptos::html::div().class("display-field").child(move || total_level.get())),
					leptos::html::td(),
					leptos::html::td(),
					leptos::html::td(),
					leptos::html::td(),
					leptos::html::td()
				))
			))
		))
	));

	let apply_signal = leptos::prelude::RwSignal::<Option<String>>::new(None);
	leptos::prelude::Effect::new(move || {
		if let Some(modal_id) = apply_signal.get() {
			let old_classes = class_levels.get_untracked().clone();
			remove_modal(modal_id).unwrap_or_else(|err| console_log(&err.message));
			apply_signal.dispose();
			let all_classes_clone_2 = all_classes_clone.clone();

			leptos::reactive::spawn_local(async move {
				apply_class_selection(
					class_levels,
					classes_input,
					&all_classes_clone_2,
					old_classes,
					total_level,
					char_level,
					experience,
				)
				.await;
				dispose_class_selection_signals(classes_input, total_level);
			})
		}
	});
	let cancel_signal = leptos::prelude::RwSignal::<Option<String>>::new(None);
	leptos::prelude::Effect::new(move || {
		if let Some(modal_id) = cancel_signal.get() {
			remove_modal(modal_id).unwrap_or_else(|err| console_log(&err.message));
			cancel_signal.dispose();
			dispose_class_selection_signals(classes_input, total_level);
		}
	});

	show_modal(
		"Select the class(es) for your character",
		modal_content,
		vec![(String::from("Apply"), apply_signal, None), (String::from("Cancel"), cancel_signal, None)],
	);
}

async fn apply_class_selection(
	class_levels: SignalField<Vec<ClassLevel>>,
	classes_input: ClassesInput,
	all_classes: &[ClassReference],
	old_classes: Vec<ClassLevel>,
	total_level: leptos::prelude::Memo<u8>,
	char_level: leptos::prelude::Memo<u8>,
	experience: SignalField<u32>,
) {
	// check if a subclass should be chosen
	for (class_level_signal, class_id_signal, subclass_id_signal) in classes_input.read_untracked().iter() {
		let old_class = old_classes
			.iter()
			.find(|&oc| class_id_signal.read_untracked().eq(&oc.id));
		let old_subclass = old_class.map(|oc| oc.subclass_id.clone());
		// don't ask a subclass if there isn't/wasn't one already
		if (!subclass_id_signal.read_untracked().is_empty()) || old_subclass.is_some() {
			continue;
		}
		let subclasses = adapter_helper_get_available_subclasses_for_class(
			class_id_signal.get_untracked(),
			class_level_signal.get_untracked(),
		);
		if subclasses.is_empty() {
			continue;
		}

		let subclass_selection_signal = leptos::prelude::RwSignal::new(false);
		// assumes all class_ids are in all_classes
		let class_name = all_classes
			.iter()
			.filter_map(|(cl_id, cl_name, _, _)| {
				if class_id_signal.read_untracked().eq(cl_id) {
					Some(cl_name.clone())
				} else {
					None
				}
			})
			.next()
			.unwrap();

		show_subclass_selection_modal(
			class_id_signal.get_untracked(),
			class_name,
			subclass_selection_signal,
			*subclass_id_signal,
			subclasses,
		);
		SignalFuture { signal: subclass_selection_signal }.await;
		subclass_selection_signal.dispose();
	}

	// assumes class_id and subclass_id are either empty or exist in all_classes
	class_levels.set(
		classes_input
			.read_untracked()
			.iter()
			.filter_map(|(class_level, class_id, subclass_id)| {
				let class_id_ = class_id.get_untracked();
				let class_level_ = class_level.get_untracked();
				if class_id_.is_empty() || (class_level_ == 0) {
					return None;
				}
				//id, name, source, subclasses
				let (class_name, subclasses) = all_classes
					.iter()
					.filter(|(id_, _, _, _)| *id_ == class_id_)
					.map(|(_, name_, _, subclasses)| (name_, subclasses))
					.next()
					.unwrap();
				let subclass_id_ = subclass_id.get_untracked();
				let subclass_id_option = if subclass_id_.is_empty() {
					None
				} else {
					Some(subclass_id_)
				};
				Some(ClassLevel {
					id: class_id.get_untracked(),
					name: match &subclass_id_option {
						None => class_name.clone(),
						Some(subclass_id_) => subclasses
							.iter()
							.filter(|(id_, _, _, _)| *id_ == *subclass_id_)
							.map(|(_, _, fullname, _)| fullname)
							.next()
							.unwrap()
							.clone(),
					},
					subclass_id: subclass_id_option,
					level: class_level_,
				})
			})
			.collect(),
	);

	experience.set(get_experience_for_new_level(
		total_level.get_untracked(),
		char_level.get_untracked(),
		experience.get_untracked(),
	));

	let _ = apply_classes(
		old_classes.iter().map(|class_level| class_level.into()).collect(),
		class_levels
			.get_untracked()
			.iter()
			.map(|class_level| class_level.into())
			.collect(),
	)
	.await
	.inspect_err(|error| {
		console_error(format!("error in apply_classes from class selection: {:?}", error).as_str())
	});
}

fn show_subclass_selection_modal(
	class_id: String,
	class_name: String,
	finished_signal: leptos::prelude::RwSignal<bool>,
	subclass_id_signal: leptos::prelude::RwSignal<String>,
	subclass_ids: Vec<String>,
) {
	let chosen_subclass = leptos::prelude::RwSignal::new(None);
	let no_subclass_chosen = leptos::prelude::Memo::new(move |_| chosen_subclass.get().is_none());
	let subclass_ids_clone = subclass_ids.clone();

	let subclass_type_name = adapter_helper_get_subclass_type_name(class_id);
	let subclass_names = adapter_helper_get_subclass_names(subclass_ids);
	let modal_content = leptos::html::div().child((
		leptos::html::span().child(format!(
			"The {} class has a high enough level to add a {}. You may select one from the list below.",
			class_name, subclass_type_name
		)),
		leptos::html::ul().class("nobullets multicolumn").child(move || {
			subclass_ids_clone
				.iter()
				.enumerate()
				.map(|(ind, subclass_id)| {
					let subclass_id_clone = subclass_id.clone();
					let option_id = format!("subclass_{}", ind);
					let option_id_clone = option_id.clone();
					leptos::html::li().child((
						leptos::html::input()
							.r#type("radio")
							.id(option_id)
							.name("subclass")
							.value(subclass_id.clone())
							.on(leptos::ev::change, move |event| {
								if leptos::prelude::event_target_checked(&event) {
									chosen_subclass.set(Some(subclass_id_clone.clone()));
								}
							}),
						leptos::html::label()
							.r#for(option_id_clone)
							.child(subclass_names[ind].clone()),
					))
				})
				.collect_view()
		}),
	));

	let apply_signal = leptos::prelude::RwSignal::<Option<String>>::new(None);
	leptos::prelude::Effect::new(move || {
		if let Some(modal_id) = apply_signal.get() {
			remove_modal(modal_id).unwrap_or_else(|err| console_log(&err.message));
			apply_signal.dispose();
			subclass_id_signal.set(chosen_subclass.get_untracked().unwrap_or_default());
			chosen_subclass.dispose();
			finished_signal.set(true);
		}
	});
	let cancel_signal = leptos::prelude::RwSignal::<Option<String>>::new(None);
	leptos::prelude::Effect::new(move || {
		if let Some(modal_id) = cancel_signal.get() {
			remove_modal(modal_id).unwrap_or_else(|err| console_log(&err.message));
			cancel_signal.dispose();
			chosen_subclass.dispose();
			finished_signal.set(true);
		}
	});

	show_modal(
		"Select a subclass",
		modal_content,
		vec![
			(String::from("Apply"), apply_signal, Some(no_subclass_chosen)),
			(String::from("Cancel"), cancel_signal, None),
		],
	);
}

fn get_experience_for_new_level(new_level: u8, old_level: u8, current_experience: u32) -> u32 {
	if new_level > old_level {
		std::cmp::max(current_experience, get_minimum_experience_for_level(new_level))
	} else if new_level < old_level {
		std::cmp::min(current_experience, get_minimum_experience_for_level(new_level + 1) - 1)
	} else {
		current_experience
	}
}
