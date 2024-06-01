use std::collections::HashMap;

use leptos::leptos_dom::logging::console_log;
use leptos::prelude::{IntoAny, Set, Write, WriteSignal};
use leptos::{
	html::ElementChild as _,
	prelude::{
		ClassAttribute as _, CollectView as _, Dispose as _, Get as _, GetUntracked as _,
		GlobalAttributes as _, OnAttribute as _, Read as _, ReadUntracked as _, StyleAttribute as _,
		Update as _,
	},
};

use crate::config::CONFIG;
use crate::domain::stats::abilities::{
	AbilityImprovementValue, AbilityLimitSource, AbilitySource, ImprovementAbilitySource, RegularAbilitySource,
};
use crate::domain::types::{AbilityPart, AbilityValue};
use crate::render::utils::{create_number_input, remove_modal};
use crate::{
	Character,
	domain::stats::abilities::Ability,
	render::{
		error::RenderError,
		utils::{RenderableValue as _, show_modal},
	},
};

type InteractiveAbilitySources = leptos::prelude::RwSignal<
	Vec<(String, String, Vec<leptos::prelude::RwSignal<i8>>, Option<leptos::prelude::RwSignal<bool>>)>,
>;

pub fn render_abilities(character: &Character, stats_page: web_sys::HtmlElement) -> Result<(), RenderError> {
	console_log(">> Rendering abilities pane");
	let abilities = &character.stats.abilities.abilities;
	let abilities_len = abilities.len();
	let abilities_tooltip = character.stats.abilities.tooltip;
	let abilities_sources = character.stats.abilities.sources;
	let abilities_min_sources = character.stats.abilities.min_sources;
	let abilities_max_sources = character.stats.abilities.max_sources;
	let ability_pane = leptos::html::div()
		.id("character_abilities")
		.class("pane grey-bg-tb-fancy abilities")
		.style(format!(
			"top: 16.55%;left: 3.8%;width: 10.15%;padding-top: 1%;padding-bottom: 1%;height: {}%",
			if abilities_len <= 6 {
				55.6
			} else {
				64.65
			}
		))
		.title(move || abilities_tooltip.get())
		.child(
			// Add individual ability elements
			CONFIG
				.ability_names_with_max
				.iter()
				.map(|(abbr, _, _)| {
					let abbreviation = String::from(*abbr);
					create_ability_pane(&abbreviation, abilities.get(&abbreviation).unwrap())
				})
				.collect::<Vec<_>>(),
		)
		.on(leptos::ev::click, move |_| {
			show_ability_modal(abilities_sources, abilities_min_sources, abilities_max_sources, None)
		});
	leptos::mount::mount_to(stats_page, || ability_pane).forget();
	Ok(())
}

fn create_ability_pane(abbreviation: &str, ability: &Ability) -> impl leptos::IntoView + use<> {
	let ability_modifier = ability.modifier;
	let ability_value = ability.value;

	leptos::html::div()
		.id(String::from("wasm_") + abbreviation)
		.class("ability")
		.style("width: 86.5%")
		.child((
			leptos::html::div()
				.class("textlabel-bold")
				.style("top: 8.5%;width: 100%;text-transform: uppercase")
				.child(ability.name.clone()),
			leptos::html::div()
				.class("display-field")
				.style("top: 16%;width: 100%;height: 48%;font-size: 360%")
				.child(move || ability_modifier.get().render()),
			leptos::html::div()
				.class("display-field")
				.style("width: 44%;height: 25%;bottom: 4%;left: 28%;font-size: 220%")
				.child(move || ability_value.get().render()),
		))
}

pub fn show_ability_modal(
	abilities_sources: leptos::prelude::RwSignal<Vec<AbilitySource>>,
	abilities_min_sources: leptos::prelude::RwSignal<Vec<AbilityLimitSource>>,
	abilities_max_sources: leptos::prelude::RwSignal<Vec<AbilityLimitSource>>,
	finished_signal: Option<WriteSignal<bool>>,
) {
	let sources_input_array: InteractiveAbilitySources = leptos::prelude::RwSignal::new(Vec::<(
		String,
		String,
		Vec<leptos::prelude::RwSignal<AbilityPart>>,
		Option<leptos::prelude::RwSignal<bool>>,
	)>::new());
	let mut checksum_array = Vec::<Option<leptos::prelude::Memo<bool>>>::new();
	let mut subset_array = Vec::<Option<Vec<String>>>::new();
	let mut feat_effects = Vec::<leptos::prelude::Effect<_>>::new();
	sources_input_array.update(|input_array| {
		for source in abilities_sources.read_untracked().iter() {
			let source_ability_parts: Vec<leptos::prelude::RwSignal<AbilityPart>> = CONFIG
				.ability_names_with_max
				.iter()
				.map(|(abbr, _, _)| {
					leptos::prelude::RwSignal::new(
						(match source {
							AbilitySource::Regular(reg_src) => reg_src.ability_parts.clone(),
							AbilitySource::Improvement(imp_src) => match &imp_src.ability_improvement {
								AbilityImprovementValue::AbilityParts(parts) => parts.clone(),
								AbilityImprovementValue::Feat => HashMap::new(),
							},
						})
						.get(&String::from(*abbr))
						.map_or(AbilityPart::from(0), |v| *v),
					)
				})
				.collect();
			let feat_flag = match source {
				AbilitySource::Regular(_) => None,
				AbilitySource::Improvement(imp_src) => match imp_src.ability_improvement {
					AbilityImprovementValue::AbilityParts(_) => Some(leptos::prelude::RwSignal::new(false)),
					AbilityImprovementValue::Feat => Some(leptos::prelude::RwSignal::new(true)),
				},
			};
			let source_ability_parts_clone = source_ability_parts.clone();
			input_array.push((
				source.get_title().clone(),
				source.get_description().clone(),
				source_ability_parts.clone(),
				feat_flag,
			));
			checksum_array.push(source.get_check_sum().map(move |total_val| {
				leptos::prelude::Memo::new(move |_| {
					if let Some(feat_flag_bool) = feat_flag
						&& feat_flag_bool.get()
					{
						return true;
					}
					let mut current_total: AbilityPart = 0;
					for current_val in source_ability_parts.iter() {
						current_total =
							current_total
								.checked_add(current_val.get())
								.unwrap_or(if current_total > 0 {
									AbilityPart::MAX
								} else {
									AbilityPart::MIN
								})
					}
					current_total == total_val
				})
			}));
			subset_array.push(match source {
				AbilitySource::Regular(reg_src) => reg_src.subset.clone(),
				AbilitySource::Improvement(_) => None,
			});
			if let Some(feat_flag_signal) = feat_flag {
				feat_effects.push(leptos::prelude::Effect::new(move || {
					if feat_flag_signal.get() {
						for ability_part in source_ability_parts_clone.iter() {
							ability_part.set(0);
						}
					}
				}));
			}
		}
	});
	let mut sources_totals = Vec::new();
	for (index, _) in CONFIG.ability_names_with_max.iter().enumerate() {
		sources_totals.push(leptos::prelude::Memo::new(move |_| {
			let mut total: AbilityPart = 0;
			for (_, _, source_abilities, _) in sources_input_array.read().iter() {
				total = total
					.checked_add(source_abilities.get(index).unwrap().get())
					.unwrap_or(if total < 0 {
						AbilityPart::MIN
					} else {
						AbilityPart::MAX
					});
			}
			AbilityValue::try_from(total).unwrap_or(0)
		}));
	}
	let min_sources_input_array =
		leptos::prelude::RwSignal::new(
			Vec::<(String, String, Vec<leptos::prelude::RwSignal<AbilityValue>>)>::new(),
		);
	min_sources_input_array.update(|input_array| {
		for source in abilities_min_sources.read_untracked().iter() {
			input_array.push((
				source.title.clone(),
				source.description.clone(),
				CONFIG
					.ability_names_with_max
					.iter()
					.map(|(abbr, _, _)| {
						leptos::prelude::RwSignal::new(
							source
								.ability_parts
								.get(&String::from(*abbr))
								.map_or(AbilityValue::from(0), |v| *v),
						)
					})
					.collect(),
			));
		}
	});
	let mut min_sources_totals = Vec::new();
	for (index, _) in CONFIG.ability_names_with_max.iter().enumerate() {
		min_sources_totals.push(leptos::prelude::Memo::new(move |_| {
			let mut total_min = AbilityValue::MIN;
			for (_, _, min_source_abilities) in min_sources_input_array.read().iter() {
				total_min = total_min.max(min_source_abilities.get(index).unwrap().get());
			}
			total_min
		}));
	}
	let max_sources_input_array =
		leptos::prelude::RwSignal::new(
			Vec::<(String, String, Vec<leptos::prelude::RwSignal<AbilityValue>>)>::new(),
		);
	max_sources_input_array.update(|input_array| {
		for source in abilities_max_sources.read_untracked().iter() {
			input_array.push((
				source.title.clone(),
				source.description.clone(),
				CONFIG
					.ability_names_with_max
					.iter()
					.map(|(abbr, _, _)| {
						leptos::prelude::RwSignal::new(
							source
								.ability_parts
								.get(&String::from(*abbr))
								.map_or(AbilityValue::from(0), |v| *v),
						)
					})
					.collect(),
			));
		}
	});
	let mut max_sources_totals = Vec::new();
	for (index, (_, _, ability_base_max)) in CONFIG.ability_names_with_max.iter().enumerate() {
		max_sources_totals.push(leptos::prelude::Memo::new(move |_| {
			let mut total_max = AbilityValue::MIN;
			for (_, _, max_source_abilities) in max_sources_input_array.read().iter() {
				total_max = total_max.max(max_source_abilities.get(index).unwrap().get());
			}
			total_max.max(AbilityValue::from(*ability_base_max))
		}));
	}
	let mut final_totals = Vec::new();
	for (index, _) in CONFIG.ability_names_with_max.iter().enumerate() {
		let sources_totals_clone = sources_totals.clone();
		let min_sources_totals_clone = min_sources_totals.clone();
		let max_sources_totals_clone = max_sources_totals.clone();
		final_totals.push(leptos::prelude::Memo::new(move |_| {
			sources_totals_clone
				.get(index)
				.unwrap()
				.get()
				.max(min_sources_totals_clone.get(index).unwrap().get())
				.min(max_sources_totals_clone.get(index).unwrap().get())
		}));
	}

	let min_sources_totals_clone = min_sources_totals.clone();
	let modal_content = leptos::html::table().child((
		leptos::html::thead().child(
			leptos::html::tr().child((
				leptos::html::th(),
				CONFIG
					.ability_names_with_max
					.iter()
					.map(|(abbr, _, _)| leptos::html::th().child(*abbr))
					.chain([leptos::html::th().child("feat")])
					.collect_view(),
			)),
		),
		leptos::html::tbody().child((
			move || {
				sources_input_array
					.read()
					.iter()
					.zip(checksum_array.iter())
					.zip(subset_array.iter())
					.map(|(((abbr, descr, abi_signals, feat_flag), checksum), subset)| {
						let check_sum = *checksum;
						let abi_signals_clone = abi_signals.clone();
						let feat_flag_clone = *feat_flag;
						leptos::html::tr().child((
							leptos::html::th()
								.child((
									leptos::html::span().child(abbr.clone()),
									leptos::html::br(),
									leptos::html::span()
										.child(descr.clone())
										.style("font-weight:normal;font-size:75%;"),
								))
								.title(descr.clone())
								.style(move || {
									if check_sum.is_none_or(|fulfilled| fulfilled.get()) {
										""
									} else {
										"color:red"
									}
								}),
							abi_signals
								.iter()
								.zip(CONFIG.ability_names_with_max.iter())
								.map(|(abi_signal, (abbr, _, _))| {
									leptos::html::td().child(
										if subset
											.clone()
											.is_none_or(|subs| subs.contains(&String::from(*abbr)))
										{
											create_number_input(
												*abi_signal,
												"width: 3em",
												*feat_flag,
												true,
												false,
												false,
												true,
											)
											.into_any()
										} else {
											leptos::html::div()
												.class("display-field")
												.child(*abi_signal)
												.into_any()
										},
									)
								})
								.collect_view(),
							leptos::html::td().child(match feat_flag_clone {
								None => Vec::new(),
								Some(feat_flag_signal) => vec![
									leptos::html::input()
										.r#type("checkbox")
										.checked(feat_flag_signal)
										.on(leptos::ev::change, move |event| {
											console_log(
												format!(
													"before: feat_flag: {:?}, event value: {:?}",
													feat_flag_signal.get_untracked(),
													leptos::prelude::event_target_checked(&event)
												)
												.as_str(),
											);
											let is_checked = leptos::prelude::event_target_checked(&event);
											feat_flag_signal.set(is_checked);
											if is_checked {
												for signal in abi_signals_clone.iter() {
													signal.set(0);
												}
											}
											console_log(
												format!(
													"after: feat_flag: {:?}, event value: {:?}",
													feat_flag_signal.get_untracked(),
													leptos::prelude::event_target_checked(&event)
												)
												.as_str(),
											);
										}),
								],
							}),
						))
					})
					.collect_view()
			},
			leptos::html::tr().class("table-total").child((
				leptos::html::th().child("Total:"),
				sources_totals
					.iter()
					.map(|total_signal| {
						leptos::html::td()
							.child(leptos::html::div().class("display-field").child(*total_signal))
					})
					.collect_view(),
			)),
			move || {
				if min_sources_input_array.read().is_empty() {
					Vec::new()
				} else {
					vec![
						leptos::html::tr().class("empty-row").child((
							leptos::html::th(),
							CONFIG
								.ability_names_with_max
								.iter()
								.map(|_| leptos::html::td())
								.collect_view(),
						)),
					]
				}
			},
			move || {
				if min_sources_input_array.read().is_empty() {
					Vec::new()
				} else {
					vec![
						leptos::html::tr().child((
							leptos::html::th().child("Minimum:"),
							CONFIG
								.ability_names_with_max
								.iter()
								.map(|_| leptos::html::td())
								.collect_view(),
						)),
					]
				}
			},
			move || {
				if min_sources_input_array.read().is_empty() {
					Vec::new()
				} else {
					vec![
						min_sources_input_array
							.read()
							.iter()
							.map(|(abbr, descr, abi_min_signals)| {
								leptos::html::tr().child((
									leptos::html::th().child(abbr.clone()).title(descr.clone()),
									abi_min_signals
										.iter()
										.map(|abi_min_signal| {
											leptos::html::td().child(
												leptos::html::div()
													.class("display-field")
													.child(*abi_min_signal),
											)
										})
										.collect_view(),
								))
							})
							.collect_view(),
					]
				}
			},
			move || {
				if min_sources_input_array.read().is_empty() {
					Vec::new()
				} else {
					vec![
						leptos::html::tr().class("table-total").child((
							leptos::html::th().child("Total:"),
							min_sources_totals_clone
								.iter()
								.map(|total_signal| {
									leptos::html::td()
										.child(leptos::html::div().class("display-field").child(*total_signal))
								})
								.collect_view(),
						)),
					]
				}
			},
			leptos::html::tr().class("empty-row").child((
				leptos::html::th(),
				CONFIG
					.ability_names_with_max
					.iter()
					.map(|_| leptos::html::td())
					.collect_view(),
			)),
			leptos::html::tr().child((
				leptos::html::th().child("Maximum:"),
				CONFIG
					.ability_names_with_max
					.iter()
					.map(|_| leptos::html::td())
					.collect_view(),
			)),
			leptos::html::tr().child((
				leptos::html::th()
					.child("base")
					.title("The default ability maximum."),
				CONFIG
					.ability_names_with_max
					.iter()
					.map(|(_, _, default_ability_max)| {
						leptos::html::td().child(
							leptos::html::div()
								.class("display-field")
								.child(*default_ability_max),
						)
					})
					.collect_view(),
			)),
			move || {
				max_sources_input_array
					.read()
					.iter()
					.map(|(abbr, descr, abi_max_signals)| {
						leptos::html::tr().child((
							leptos::html::th().child(abbr.clone()).title(descr.clone()),
							abi_max_signals
								.iter()
								.map(|abi_max_signal| {
									leptos::html::td().child(
										leptos::html::div().class("display-field").child(*abi_max_signal),
									)
								})
								.collect_view(),
						))
					})
					.collect_view()
			},
			leptos::html::tr().class("table-total").child((
				leptos::html::th().child("Total:"),
				max_sources_totals
					.iter()
					.map(|total_signal| {
						leptos::html::td()
							.child(leptos::html::div().class("display-field").child(*total_signal))
					})
					.collect_view(),
			)),
			leptos::html::tr().class("empty-row").child((
				leptos::html::th(),
				CONFIG
					.ability_names_with_max
					.iter()
					.map(|_| leptos::html::td())
					.collect_view(),
			)),
			leptos::html::tr().child((
				leptos::html::th().child("Final:"),
				final_totals
					.iter()
					.map(|total_signal| {
						leptos::html::td()
							.child(leptos::html::div().class("display-field").child(*total_signal))
					})
					.collect_view(),
			)),
		)),
	));

	let ok_signal = leptos::prelude::RwSignal::<Option<String>>::new(None);
	let totals_clone = sources_totals.clone();
	let min_totals_clone = min_sources_totals.clone();
	let max_totals_clone = max_sources_totals.clone();
	let final_totals_clone = final_totals.clone();
	let feat_effects_clone = feat_effects.clone();
	leptos::prelude::Effect::new(move || {
		if let Some(modal_id) = ok_signal.get() {
			remove_modal(modal_id).unwrap_or_else(|err| console_log(&err.message));
			ok_signal.dispose();
			modify_ability_sources(sources_input_array, abilities_sources);
			dispose_input_signals(
				sources_input_array,
				&sources_totals.clone(),
				min_sources_input_array,
				&min_sources_totals.clone(),
				max_sources_input_array,
				&max_sources_totals.clone(),
				&final_totals.clone(),
				&feat_effects,
			);
			if let Some(sign) = finished_signal {
				*sign.write() = true
			};
		}
	});
	let cancel_signal = leptos::prelude::RwSignal::<Option<String>>::new(None);
	leptos::prelude::Effect::new(move || {
		if let Some(modal_id) = cancel_signal.get() {
			remove_modal(modal_id).unwrap_or_else(|err| console_log(&err.message));
			dispose_input_signals(
				sources_input_array,
				&totals_clone,
				min_sources_input_array,
				&min_totals_clone,
				max_sources_input_array,
				&max_totals_clone,
				&final_totals_clone,
				&feat_effects_clone,
			);
			if let Some(sign) = finished_signal {
				*sign.write() = true
			};
		}
	});
	show_modal(
		"Edit Abilities",
		modal_content,
		vec![(String::from("OK"), ok_signal, None), (String::from("Cancel"), cancel_signal, None)],
	);
}

fn modify_ability_sources(
	new_ability_sources: InteractiveAbilitySources,
	abilities_sources: leptos::prelude::RwSignal<Vec<AbilitySource>>,
) {
	abilities_sources.update(|current| {
		*current = new_ability_sources
			.read_untracked()
			.iter()
			.zip(current.iter())
			.map(|((key, description, ability_mods, feat_flag), current_ability_source)| {
				match current_ability_source {
					AbilitySource::Regular(current_source) => AbilitySource::Regular(RegularAbilitySource {
						title: key.clone(),
						description: description.clone(),
						ability_parts: CONFIG
							.ability_names_with_max
							.iter()
							.map(|abi_names| abi_names.0)
							.zip(ability_mods.iter())
							.filter_map(|(name, mod_signal)| {
								let new_value = mod_signal.get_untracked();
								if new_value != AbilityPart::from(0) {
									Some((String::from(name), new_value))
								} else {
									None
								}
							})
							.collect(),
						check_sum: current_source.check_sum,
						subset: current_source.subset.clone(),
					}),
					AbilitySource::Improvement(current_source) => {
						AbilitySource::Improvement(ImprovementAbilitySource {
							title: key.clone(),
							description: description.clone(),
							ability_improvement: {
								if feat_flag.is_none_or(|feat_flag_signal| !feat_flag_signal.get_untracked()) {
									AbilityImprovementValue::AbilityParts(
										CONFIG
											.ability_names_with_max
											.iter()
											.map(|abi_names| abi_names.0)
											.zip(ability_mods.iter())
											.filter_map(|(name, mod_signal)| {
												let new_value = mod_signal.get_untracked();
												if new_value != AbilityPart::from(0) {
													Some((String::from(name), new_value))
												} else {
													None
												}
											})
											.collect(),
									)
								} else {
									AbilityImprovementValue::Feat
								}
							},
							check_sum: current_source.check_sum,
						})
					},
				}
			})
			.collect();
	});
}

#[allow(clippy::too_many_arguments)]
fn dispose_input_signals(
	input_signals: InteractiveAbilitySources,
	total_values: &[leptos::prelude::Memo<AbilityValue>],
	min_input_signals: leptos::prelude::RwSignal<
		Vec<(String, String, Vec<leptos::prelude::RwSignal<AbilityValue>>)>,
	>,
	min_total_values: &[leptos::prelude::Memo<AbilityValue>],
	max_input_signals: leptos::prelude::RwSignal<
		Vec<(String, String, Vec<leptos::prelude::RwSignal<AbilityValue>>)>,
	>,
	max_total_values: &[leptos::prelude::Memo<AbilityValue>],
	final_total_values: &[leptos::prelude::Memo<AbilityValue>],
	feat_effects: &[leptos::prelude::Effect<leptos::prelude::LocalStorage>],
) {
	for (_, _, signal_vec, feat_flag) in input_signals.read_untracked().iter() {
		for signal in signal_vec {
			signal.dispose();
		}
		if let Some(feat_flag_signal) = feat_flag {
			feat_flag_signal.dispose();
		}
	}
	input_signals.dispose();
	for total in total_values.iter() {
		total.dispose();
	}
	for (_, _, signal_vec) in max_input_signals.read_untracked().iter() {
		for signal in signal_vec {
			signal.dispose();
		}
	}
	min_input_signals.dispose();
	for total in min_total_values.iter() {
		total.dispose();
	}
	max_input_signals.dispose();
	for total in max_total_values.iter() {
		total.dispose();
	}
	for total in final_total_values.iter() {
		total.dispose();
	}
	for feat_effect in feat_effects.iter() {
		feat_effect.dispose();
	}
}
