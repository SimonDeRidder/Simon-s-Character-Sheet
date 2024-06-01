// Interface with the JS code, both effects and callbacks

use std::collections::HashMap;

use gloo_utils::format::JsValueSerdeExt as _;
use leptos::{
	leptos_dom::logging::console_error,
	prelude::{Get as _, GetUntracked as _, ReadUntracked as _, Set as _, Update as _, Write as _},
};
use wasm_bindgen::UnwrapThrowExt;
use wasm_bindgen::prelude::wasm_bindgen;

#[allow(unused_imports)]
use wasm_bindgen::JsValue;

use crate::{
	Character,
	config::CONFIG,
	domain::{
		general_info::ClassLevel,
		stats::abilities::{
			AbilityImprovementValue, AbilityLimitSource, AbilitySource, ImprovementAbilitySource,
			RegularAbilitySource,
		},
		types::{AbilityPart, AbilityValue, Modifier},
	},
	render::{
		stats::{abilities::show_ability_modal, header::show_class_selection_modal},
		utils::SignalFuture,
	},
};

#[wasm_bindgen]
pub struct ClassForExport {
	id: String,
	pub level: u8,
	subclass_id: Option<String>,
	name: String,
}
#[wasm_bindgen]
impl ClassForExport {
	#[wasm_bindgen(getter)]
	pub fn id(&self) -> String {
		self.id.clone()
	}
	#[wasm_bindgen(getter)]
	pub fn subclass_id(&self) -> Option<String> {
		self.subclass_id.clone()
	}
	#[wasm_bindgen(getter)]
	pub fn name(&self) -> String {
		self.name.clone()
	}
}

impl From<&ClassLevel> for ClassForExport {
	fn from(class_level: &ClassLevel) -> Self {
		ClassForExport {
			id: class_level.id.clone(),
			level: class_level.level,
			subclass_id: class_level.subclass_id.clone(),
			name: class_level.name.clone(),
		}
	}
}

#[wasm_bindgen]
extern "C" {
	#[wasm_bindgen(js_namespace = eventManager)]
	fn handle_event(event_type: String);

	pub fn adapter_helper_get_all_class_ids() -> Vec<String>;
	pub fn adapter_helper_get_class(class_id: String) -> Vec<String>;
	pub fn adapter_helper_get_all_subclasses(class_id: String) -> Vec<String>;
	pub fn adapter_helper_get_subclass(class_id: String, subclass_id: String) -> Vec<String>;
	pub fn adapter_helper_get_available_subclasses_for_class(class_id: String, class_level: u8)
	-> Vec<String>;
	pub fn adapter_helper_get_subclass_type_name(class_id: String) -> String;
	pub fn adapter_helper_get_subclass_names(subclass_ids: Vec<String>) -> Vec<String>;
	pub fn adapter_helper_get_backgrounds() -> Vec<String>;
	pub fn adapter_helper_get_background_names() -> Vec<String>;
	pub fn adapter_helper_get_background_option_title(background_id: Option<String>) -> Option<String>;
	pub fn adapter_helper_get_background_options(background_id: Option<String>) -> Option<Vec<String>>;
	pub fn adapter_helper_get_races() -> Vec<String>;
	pub fn adapter_helper_get_race_names() -> Vec<String>;
	pub fn adapter_helper_get_race_variants(race_id: String) -> Vec<String>;
	pub fn adapter_helper_get_race_variant_names(race_id: String) -> Vec<String>;
	pub fn adapter_helper_race_needs_previous_race(race_id: Option<String>) -> bool;

	#[wasm_bindgen(js_name = ApplyClasses, catch)]
	pub async fn apply_classes(
		oldClasses: Vec<ClassForExport>,
		newClasses: Vec<ClassForExport>,
	) -> Result<(), JsValue>;

	#[wasm_bindgen(js_name = UpdateLevelFeatures, catch)]
	pub async fn update_level_features(
		type_: String,
		level: u8,
		classes_to_update: Option<String>,
		old_level: Option<u8>,
		race_id: Option<String>,
	) -> Result<(), JsValue>;

	#[wasm_bindgen(js_name = ApplyBackground)]
	pub async fn apply_background(background_id: Option<String>, old_background_id: Option<String>);

	#[wasm_bindgen(js_name = ApplyRace)]
	pub async fn apply_race(race_id: Option<String>, old_race_id: Option<String>);
}

// === EFFECTS ===
pub fn create_all_effects(character: &Character) {
	// > Stats
	// >> Header
	let character_name = character.general_info.name;
	leptos::prelude::Effect::new(move |_| {
		character_name.get();
		handle_event(String::from("PC_Name_change"));
	});
	let gen_info_clone = character.general_info.clone();
	let startup = character.startup;
	leptos::reactive::effect::Effect::new(move |prev_value: Option<Option<u8>>| {
		let current_value = gen_info_clone.level.get();
		if startup.get() {
			return None;
		}
		let flat_prev = prev_value.unwrap_or_default();
		if flat_prev.is_none_or(|prev_val| current_value != prev_val) {
			leptos::reactive::spawn_local(async move {
				handle_event(String::from("Character_Level_change"));
			});
			if !(flat_prev.is_none() && current_value == 0) {
				leptos::reactive::spawn_local(async move {
					let _ = update_level_features(
						String::from("notclass"),
						std::cmp::max(1, current_value),
						None,
						flat_prev,
						None,
					)
					.await
					.inspect_err(|error| {
						console_error(
							format!("error in update_level_features from level change: {:?}", error).as_str(),
						)
					});
				});
				let old_classes = gen_info_clone.classes.get_untracked();
				if old_classes
					.iter()
					.map(|class_level| class_level.level)
					.sum::<u8>() != current_value
				{
					if current_value > 0 {
						show_class_selection_modal(
							gen_info_clone.classes,
							gen_info_clone.level,
							gen_info_clone.experience,
							i16::from(current_value) - i16::from(flat_prev.unwrap_or_default()),
						);
					} else {
						gen_info_clone.classes.set(Vec::new());
						leptos::reactive::spawn_local(async move {
							let _ = apply_classes(
								old_classes.iter().map(|class_level| class_level.into()).collect(),
								vec![],
							)
							.await
							.inspect_err(|error| {
								console_error(
									format!("error in apply_classes from level change: {:?}", error).as_str(),
								)
							});
						});
					}
				}
			}
		}
		Some(current_value)
	});
	// >> Abilities
	for (abbreviation, ability) in character.stats.abilities.abilities.iter() {
		let abi_value = ability.value;
		let abi_modifier = ability.modifier;
		let abbr = (*abbreviation).clone();
		leptos::reactive::effect::Effect::new(move |_| {
			abi_value.get();
			handle_event(abbr.clone() + "_change");
		});
		let abbr = (*abbreviation).clone();
		leptos::reactive::effect::Effect::new(move |_| {
			abi_modifier.get();
			handle_event(abbr.clone() + "_Mod_change");
		});
	}
}

// === CALLBACKS ==

#[wasm_bindgen]
impl Character {
	pub fn stop_startup(&self) {
		self.startup.set(false);
	}

	// > Stats

	// >> Header

	pub fn get_name(&self) -> String {
		self.general_info.name.get_untracked()
	}

	pub fn get_level(&self) -> u8 {
		self.general_info.level.get_untracked()
	}

	pub fn get_class_level(&self, class_id: String) -> u8 {
		self.general_info.get_class_level(class_id)
	}

	pub fn has_class(&self, class_id: String) -> bool {
		self.general_info.has_class(class_id)
	}

	pub fn get_subclass(&self, class_id: String) -> Option<String> {
		self.general_info.get_subclass(class_id)
	}

	pub fn list_classes(&self) -> Vec<String> {
		self.general_info.list_classes()
	}

	pub fn set_class(&self, class_id: String, name: String, subclass: Option<String>, level: u8) {
		self.general_info.set_class(class_id, name, subclass, level);
	}

	pub fn remove_class(&self, class_id: String) {
		self.general_info.remove_class(class_id);
	}

	pub fn set_subclass(&self, class_id: String, subclass: Option<String>) {
		self.general_info.set_subclass(class_id, subclass);
	}

	pub fn get_primary_class(&self) -> Option<String> {
		self.general_info.get_primary_class()
	}

	pub fn get_background_id(&self) -> Option<String> {
		self.general_info.background.get_untracked()
	}

	pub fn get_background_option(&self) -> Option<String> {
		self.general_info.background_option.get_untracked()
	}

	pub fn get_race_id(&self) -> Option<String> {
		self.general_info.race.get_untracked()
	}

	pub fn get_race_previous(&self) -> Option<String> {
		self.general_info.race_previous.get_untracked()
	}

	// >> Abilities

	pub fn get_ability(&self, abbreviation: String) -> Option<u8> {
		self.stats
			.abilities
			.abilities
			.get(&abbreviation)
			.map(|ability| ability.value.get_untracked())
	}

	pub fn get_ability_modifier(&self, abbreviation: String) -> Option<Modifier> {
		self.stats
			.abilities
			.abilities
			.get(&abbreviation)
			.map(|ability| ability.modifier.get_untracked())
	}

	pub fn has_ability_source(&self, name: String) -> bool {
		for src in self.stats.abilities.sources.read_untracked().iter() {
			if *match src {
				AbilitySource::Regular(rsrc) => &rsrc.title,
				AbilitySource::Improvement(isrc) => &isrc.title,
			} == name
			{
				return true;
			}
		}
		false
	}

	pub fn add_ability_source(
		&self,
		name: String,
		description: String,
		ability_abbreviations: Vec<String>,
		values: Vec<AbilityPart>,
		check_sum: Option<AbilityPart>,
		subset: Option<Vec<String>>,
	) {
		let mut existing = false;
		for (index, src) in self.stats.abilities.sources.read_untracked().iter().enumerate() {
			let name_clone = name.clone();
			let descr_clone = description.clone();
			if *src.get_title() == name {
				// ability already exists, overwrite
				existing = true;
				self.stats.abilities.sources.update(|sources| {
					let _ = std::mem::replace(
						&mut sources[index],
						AbilitySource::Regular(RegularAbilitySource {
							title: name_clone,
							description: descr_clone,
							ability_parts: ability_abbreviations
								.iter()
								.zip(&values)
								.filter_map(|(abbr, val)| {
									if *val == 0 {
										None
									} else {
										Some((abbr.clone(), *val))
									}
								})
								.collect(),
							check_sum,
							subset: subset.clone(),
						}),
					);
				});
				break;
			}
		}
		if !existing {
			self.stats
				.abilities
				.sources
				.write()
				.push(AbilitySource::Regular(RegularAbilitySource {
					title: name,
					description,
					ability_parts: ability_abbreviations
						.iter()
						.zip(&values)
						.filter_map(|(abbr, val)| {
							if *val == 0 {
								None
							} else {
								Some((abbr.clone(), *val))
							}
						})
						.collect(),
					check_sum,
					subset,
				}))
		}
	}

	pub fn add_ability_source_limit(
		&self,
		name: String,
		description: String,
		abiltity_abbreviations: Vec<String>,
		values: Vec<AbilityValue>,
		is_max: bool,
	) {
		let limit_sources = if is_max {
			self.stats.abilities.max_sources
		} else {
			self.stats.abilities.min_sources
		};
		let mut existing = false;
		for (index, src) in limit_sources.read_untracked().iter().enumerate() {
			let name_clone = name.clone();
			if src.title == name {
				// ability already exists, overwrite
				existing = true;
				let descr_clone = if description.is_empty() {
					src.description.clone()
				} else {
					description.clone()
				};
				limit_sources.update(|sources| {
					let _ = std::mem::replace(
						&mut sources[index],
						AbilityLimitSource {
							title: name_clone,
							description: descr_clone,
							ability_parts: abiltity_abbreviations
								.iter()
								.zip(values.clone())
								.filter_map(|(abbr, val)| {
									if val == 0 {
										None
									} else {
										Some((abbr.clone(), val))
									}
								})
								.collect(),
						},
					);
				});
				break;
			}
		}
		if !existing {
			limit_sources.write().push(AbilityLimitSource {
				title: name,
				description,
				ability_parts: abiltity_abbreviations
					.iter()
					.zip(values.clone())
					.filter_map(|(abbr, val)| {
						if val == 0 {
							None
						} else {
							Some((abbr.clone(), val))
						}
					})
					.collect(),
			})
		}
	}

	pub fn update_ability_source_improvements(&self, new_improvement_names: Vec<String>) {
		let new_improvement_titles = new_improvement_names.clone();
		self.stats.abilities.sources.update(move |sources| {
			*sources = (*sources)
				.iter()
				.filter(|source| match source {
					AbilitySource::Regular(_) => true,
					AbilitySource::Improvement(improvement_src) => {
						new_improvement_titles.contains(&improvement_src.title)
					},
				})
				.cloned()
				.collect();
			let added_improvement_titles: Vec<String> = new_improvement_titles
				.iter()
				.filter_map(|title| {
					for source in sources.iter() {
						match source {
							AbilitySource::Regular(_) => {},
							AbilitySource::Improvement(improvement_src) => {
								if improvement_src.title == **title {
									return None;
								}
							},
						}
					}
					Some(title.clone())
				})
				.collect();
			for added_title in added_improvement_titles.iter() {
				sources.push(AbilitySource::Improvement(ImprovementAbilitySource {
					title: added_title.clone(),
					description: format!(
						"{}x +1 to any ability, or a feat.",
						CONFIG.ability_improvement_amount
					),
					ability_improvement: AbilityImprovementValue::AbilityParts(HashMap::new()),
					check_sum: CONFIG.ability_improvement_amount,
				}));
			}
		});
	}

	pub fn remove_ability_source(&self, name: String) {
		let ability_sources_len = self.stats.abilities.sources.read_untracked().len();
		let mut remove_index = ability_sources_len;
		for (index, src) in self.stats.abilities.sources.read_untracked().iter().enumerate() {
			if *match src {
				AbilitySource::Regular(rsrc) => &rsrc.title,
				AbilitySource::Improvement(isrc) => &isrc.title,
			} == name
			{
				remove_index = index;
				break;
			}
		}
		if remove_index >= ability_sources_len {
			console_error(format!("Could find ability source to remove: {}", name).as_str());
		} else {
			self.stats.abilities.sources.write().remove(remove_index);
		}
	}

	pub fn remove_ability_source_limit(&self, name: String, is_max: bool) {
		let source_limit = if is_max {
			self.stats.abilities.max_sources
		} else {
			self.stats.abilities.min_sources
		};
		let ability_sources_limit_len = source_limit.read_untracked().len();
		let mut remove_index = ability_sources_limit_len;
		for (index, src) in source_limit.read_untracked().iter().enumerate() {
			if src.title == name {
				remove_index = index;
				break;
			}
		}
		if remove_index >= ability_sources_limit_len {
			console_error(format!("Could find ability source to remove: {}", name).as_str());
		} else {
			source_limit.write().remove(remove_index);
		}
	}

	pub fn get_abilities_tooltip(&self) -> String {
		self.stats.abilities.tooltip.get_untracked()
	}

	pub async fn show_abilities_dialog(&self) {
		let (finished_signal_r, finished_signal_w) = leptos::prelude::signal(false);
		show_ability_modal(
			self.stats.abilities.sources,
			self.stats.abilities.min_sources,
			self.stats.abilities.max_sources,
			Some(finished_signal_w),
		);
		SignalFuture { signal: finished_signal_r }.await;
	}

	// > Serialisation

	pub fn get_character_json(&self) -> wasm_bindgen::JsValue {
		wasm_bindgen::JsValue::from_serde(self).unwrap()
	}

	pub fn get_character_from_json(json: wasm_bindgen::JsValue) -> Self {
		json.into_serde().unwrap_throw()
	}
}
