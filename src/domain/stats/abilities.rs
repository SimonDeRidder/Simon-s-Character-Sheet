use leptos::prelude::{Get as _, GetUntracked as _, Read as _};
use serde::ser::SerializeSeq as _;
use std::collections::HashMap;

use crate::config::CONFIG;
use crate::domain::types::{AbilityPart, AbilityValue, Modifier};
use crate::utils::convert_text_to_bold;

const DEFAULT_BASE_ABILITY_SCORE: AbilityPart = 10;

#[derive(Clone)]
pub struct Abilities {
	pub tooltip: leptos::prelude::Memo<String>,
	pub abilities: HashMap<String, Ability>,
	pub sources: leptos::prelude::RwSignal<Vec<AbilitySource>>,
	pub min_sources: leptos::prelude::RwSignal<Vec<AbilityLimitSource>>,
	pub max_sources: leptos::prelude::RwSignal<Vec<AbilityLimitSource>>,
}
impl Abilities {
	pub fn default() -> Self {
		Self::from_sources(&get_default_sources(), &Vec::new(), &Vec::new())
	}

	fn from_sources(
		sources: &Vec<AbilitySource>,
		min_sources: &Vec<AbilityLimitSource>,
		max_sources: &Vec<AbilityLimitSource>,
	) -> Self {
		let sources_signal = leptos::prelude::RwSignal::new((*sources).clone());
		let min_sources_signal = leptos::prelude::RwSignal::new((*min_sources).clone());
		let max_sources_signal = leptos::prelude::RwSignal::new((*max_sources).clone());
		let mut abilities_list: HashMap<String, Ability> =
			HashMap::with_capacity(CONFIG.ability_names_with_max.len());
		for (ability_abbr, ability_name, base_ability_max) in CONFIG.ability_names_with_max {
			abilities_list.insert(
				String::from(ability_abbr),
				Ability::new(
					ability_abbr,
					ability_name,
					sources_signal,
					base_ability_max,
					min_sources_signal,
					max_sources_signal,
				),
			);
		}
		Self {
			tooltip: construct_tooltip_from_sources(sources_signal, min_sources_signal, max_sources_signal),
			abilities: abilities_list,
			sources: sources_signal,
			min_sources: min_sources_signal,
			max_sources: max_sources_signal,
		}
	}
}
impl serde::Serialize for Abilities {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		let sources = self.sources.get_untracked();
		let min_sources = self.min_sources.get_untracked();
		let max_sources = self.max_sources.get_untracked();
		let mut sources_serialiser =
			serializer.serialize_seq(Some(sources.len() + min_sources.len() + max_sources.len()))?;

		for src in sources {
			sources_serialiser.serialize_element(&match src {
				AbilitySource::Regular(reg_src) => SerialisedAbilitySource {
					type_: SerialisedAbilitySourceType::Regular,
					title: reg_src.title,
					description: reg_src.description,
					ability_parts: Some(
						reg_src
							.ability_parts
							.iter()
							.map(|(k, v)| (k.clone(), i16::from(*v)))
							.collect(),
					),
					feat: false,
					check_sum: reg_src.check_sum,
					subset: reg_src.subset,
				},
				AbilitySource::Improvement(imp_src) => SerialisedAbilitySource {
					type_: SerialisedAbilitySourceType::Improvement,
					title: imp_src.title,
					description: imp_src.description,
					ability_parts: match &imp_src.ability_improvement {
						AbilityImprovementValue::AbilityParts(abi_parts) => Some(
							abi_parts
								.iter()
								.map(|(k, v)| (k.clone(), i16::from(*v)))
								.collect(),
						),
						_ => None,
					},
					feat: (imp_src.ability_improvement == AbilityImprovementValue::Feat),
					check_sum: Some(imp_src.check_sum),
					subset: None,
				},
			})?;
		}
		for min_src in min_sources {
			sources_serialiser.serialize_element(&SerialisedAbilitySource {
				type_: SerialisedAbilitySourceType::Minimum,
				title: min_src.title,
				description: min_src.description,
				ability_parts: Some(
					min_src
						.ability_parts
						.iter()
						.map(|(k, v)| (k.clone(), i16::from(*v)))
						.collect(),
				),
				feat: false,
				check_sum: None,
				subset: None,
			})?;
		}
		for max_src in max_sources {
			sources_serialiser.serialize_element(&SerialisedAbilitySource {
				type_: SerialisedAbilitySourceType::Maximum,
				title: max_src.title,
				description: max_src.description,
				ability_parts: Some(
					max_src
						.ability_parts
						.iter()
						.map(|(k, v)| (k.clone(), i16::from(*v)))
						.collect(),
				),
				feat: false,
				check_sum: None,
				subset: None,
			})?;
		}
		sources_serialiser.end()
	}
}
impl<'de> serde::Deserialize<'de> for Abilities {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		serde::Deserialize::deserialize(deserializer).map(move |sources_vec: Vec<SerialisedAbilitySource>| {
			let mut sources = Vec::new();
			let mut min_sources = Vec::new();
			let mut max_sources = Vec::new();
			for source in sources_vec {
				match source.type_ {
					SerialisedAbilitySourceType::Regular => {
						let ability_parts = source.cast_ability_parts_to_part();
						sources.push(AbilitySource::Regular(RegularAbilitySource {
							title: source.title,
							description: source.description,
							ability_parts,
							check_sum: source.check_sum,
							subset: source.subset,
						}));
					},
					SerialisedAbilitySourceType::Improvement => {
						let improvement = match source.feat {
							true => AbilityImprovementValue::Feat,
							false => {
								AbilityImprovementValue::AbilityParts(source.cast_ability_parts_to_part())
							},
						};
						sources.push(AbilitySource::Improvement(ImprovementAbilitySource {
							title: source.title,
							description: source.description,
							ability_improvement: improvement,
							check_sum: source
								.check_sum
								.expect("ImprovementAbilitySource must have check_sum"),
						}));
					},
					SerialisedAbilitySourceType::Minimum => {
						let ability_parts = source.cast_ability_parts_to_value();
						min_sources.push(AbilityLimitSource {
							title: source.title,
							description: source.description,
							ability_parts,
						});
					},
					SerialisedAbilitySourceType::Maximum => {
						let ability_parts = source.cast_ability_parts_to_value();
						max_sources.push(AbilityLimitSource {
							title: source.title,
							description: source.description,
							ability_parts,
						});
					},
				}
			}
			Abilities::from_sources(&sources, &min_sources, &max_sources)
		})
	}
}

#[derive(Clone)]
pub enum AbilitySource {
	Regular(RegularAbilitySource),
	Improvement(ImprovementAbilitySource),
}
impl AbilitySource {
	pub fn get_title(&self) -> &String {
		match self {
			AbilitySource::Regular(reg_src) => &reg_src.title,
			AbilitySource::Improvement(imp_src) => &imp_src.title,
		}
	}

	pub fn get_description(&self) -> &String {
		match self {
			AbilitySource::Regular(reg_src) => &reg_src.description,
			AbilitySource::Improvement(imp_src) => &imp_src.description,
		}
	}

	pub fn get_check_sum(&self) -> Option<AbilityPart> {
		match self {
			AbilitySource::Regular(reg_src) => reg_src.check_sum,
			AbilitySource::Improvement(imp_src) => Some(imp_src.check_sum),
		}
	}
}

#[derive(Clone)]
pub struct RegularAbilitySource {
	pub title: String,                               // Name of the source
	pub description: String,                         // Brief description pertinent to scores
	pub ability_parts: HashMap<String, AbilityPart>, // map from ability abbreviation (e.g. "Str") to a modifier value
	pub check_sum: Option<AbilityPart>,              // Total value that the ability parts should sum to
	pub subset: Option<Vec<String>>,                 // The subset of abilities (abbreviations) that can be edited
}

#[derive(Clone, PartialEq)]
pub enum AbilityImprovementValue {
	AbilityParts(HashMap<String, AbilityPart>),
	Feat,
}

#[derive(Clone)]
pub struct ImprovementAbilitySource {
	pub title: String,                                // Name of the source
	pub description: String,                          // Brief description pertinent to scores
	pub ability_improvement: AbilityImprovementValue, // map from ability abbreviation (e.g. "Str") to a modifier value, or feat
	pub check_sum: AbilityPart,                       // Total value that the ability parts should sum to
}

#[derive(Clone)]
pub struct AbilityLimitSource {
	pub title: String,
	pub description: String,
	pub ability_parts: HashMap<String, AbilityValue>,
}

#[derive(serde::Serialize, serde::Deserialize)]
enum SerialisedAbilitySourceType {
	Regular,
	Improvement,
	Minimum,
	Maximum,
}
#[derive(serde::Serialize, serde::Deserialize)]
struct SerialisedAbilitySource {
	type_: SerialisedAbilitySourceType,
	title: String,
	description: String,
	#[serde(skip_serializing_if = "Option::is_none")]
	#[serde(default)]
	ability_parts: Option<HashMap<String, i16>>,
	#[serde(skip_serializing_if = "std::ops::Not::not")]
	#[serde(default)]
	feat: bool,
	#[serde(skip_serializing_if = "Option::is_none")]
	#[serde(default)]
	check_sum: Option<AbilityPart>,
	#[serde(skip_serializing_if = "Option::is_none")]
	#[serde(default)]
	subset: Option<Vec<String>>,
}
impl SerialisedAbilitySource {
	fn cast_ability_parts_to_part(&self) -> HashMap<String, AbilityPart> {
		self.ability_parts
			.as_ref()
			.expect("Ability source must have ability_parts")
			.iter()
			.map(|(k, v)| {
				(
					k.clone(),
					AbilityPart::try_from(*v).unwrap_or(if *v < 0 {
						AbilityPart::MIN
					} else {
						AbilityPart::MAX
					}),
				)
			})
			.collect()
	}

	fn cast_ability_parts_to_value(&self) -> HashMap<String, AbilityValue> {
		self.ability_parts
			.as_ref()
			.expect("Ability source must have ability_parts")
			.iter()
			.map(|(k, v)| {
				(
					k.clone(),
					AbilityValue::try_from(*v).unwrap_or(if *v <= 0 {
						0
					} else {
						AbilityValue::MAX
					}),
				)
			})
			.collect()
	}
}

#[derive(Clone)]
pub struct Ability {
	pub name: String,
	pub value: leptos::prelude::Memo<AbilityValue>,
	pub modifier: leptos::prelude::Memo<Modifier>,
}

impl Ability {
	pub fn new(
		abbreviation: &'static str,
		name: &'static str,
		ability_sources: leptos::prelude::RwSignal<Vec<AbilitySource>>,
		base_ability_max: u8,
		ability_min_sources: leptos::prelude::RwSignal<Vec<AbilityLimitSource>>,
		ability_max_sources: leptos::prelude::RwSignal<Vec<AbilityLimitSource>>,
	) -> Self {
		let value_memo = leptos::prelude::Memo::new(move |_| {
			let mut total_val: AbilityPart = 0;
			for src in ability_sources.read().iter() {
				if let Some(ability_parts) = match src {
					AbilitySource::Regular(ability_source) => Some(&ability_source.ability_parts),
					AbilitySource::Improvement(ability_improvement) => {
						match &ability_improvement.ability_improvement {
							AbilityImprovementValue::AbilityParts(ability_parts) => Some(ability_parts),
							AbilityImprovementValue::Feat => None,
						}
					},
				} {
					for (source_abbr, part) in ability_parts.iter() {
						if source_abbr.as_str() == abbreviation {
							total_val = total_val.checked_add(*part).unwrap_or(if total_val < 0 {
								AbilityPart::MIN
							} else {
								AbilityPart::MAX
							});
						}
					}
				};
			}
			let mut total_min = AbilityValue::MIN;
			for min_src in ability_min_sources.read().iter() {
				for (source_abbr, part) in min_src.ability_parts.iter() {
					if source_abbr.as_str() == abbreviation {
						total_min = total_min.max(*part);
					}
				}
			}
			let mut total_max = AbilityValue::from(base_ability_max);
			for max_src in ability_max_sources.read().iter() {
				for (source_abbr, part) in max_src.ability_parts.iter() {
					if source_abbr.as_str() == abbreviation {
						total_max = total_max.max(*part);
					}
				}
			}
			AbilityValue::try_from(total_val)
				.unwrap_or(AbilityValue::from(0))
				.max(total_min)
				.min(total_max)
		});
		Ability {
			name: String::from(name),
			value: value_memo,
			modifier: leptos::prelude::Memo::new(move |_| calc_ability_modifier(value_memo.get())),
		}
	}
}

fn get_default_sources() -> Vec<AbilitySource> {
	vec![AbilitySource::Regular(RegularAbilitySource {
		title: String::from("base"),
		description: String::from("The starting abilities, either a standard array or rolled."),
		ability_parts: CONFIG
			.ability_names_with_max
			.iter()
			.map(|a| (String::from(a.0), DEFAULT_BASE_ABILITY_SCORE))
			.collect(),
		check_sum: None,
		subset: None,
	})]
}

fn calc_ability_modifier(value: AbilityValue) -> Modifier {
	Modifier::try_from(value / 2)
		.map(|v| v - 5)
		.unwrap_or(Modifier::MAX)
}

fn construct_tooltip_from_sources(
	sources: leptos::prelude::RwSignal<Vec<AbilitySource>>,
	min_sources: leptos::prelude::RwSignal<Vec<AbilityLimitSource>>,
	max_sources: leptos::prelude::RwSignal<Vec<AbilityLimitSource>>,
) -> leptos::prelude::Memo<String> {
	leptos::prelude::Memo::new(move |_| {
		let mut titles: Vec<String> = sources
			.read()
			.iter()
			.map(|src| match src {
				AbilitySource::Regular(rsrc) => rsrc.title.clone(),
				AbilitySource::Improvement(isrc) => isrc.title.clone(),
			})
			.collect();
		let mut descriptions: HashMap<String, String> = sources
			.read()
			.iter()
			.map(|src| match src {
				AbilitySource::Regular(rsrc) => (rsrc.title.clone(), rsrc.description.clone()),
				AbilitySource::Improvement(isrc) => (isrc.title.clone(), isrc.description.clone()),
			})
			.collect();
		for source in min_sources.read().iter().chain(max_sources.read().iter()) {
			if !descriptions.contains_key(&source.title) {
				titles.push(source.title.clone());
				descriptions.insert(source.title.clone(), source.description.clone());
			}
		}
		convert_text_to_bold(&String::from("Ability Score sources:\n\n"))
			+ &titles
				.iter()
				.map(|title| {
					String::from("• ")
						+ &convert_text_to_bold(&(title.clone() + "\n"))
						+ "  " + descriptions.get(title).unwrap()
				})
				.collect::<Vec<String>>()
				.join("\n")
	})
}

#[cfg(test)]
mod tests {
	use leptos::prelude::GetUntracked;

	use crate::config::CONFIG;

	use super::{Abilities, AbilitySource, RegularAbilitySource, calc_ability_modifier};

	#[test]
	fn test_calc_ability_from_sources() {
		let sources_input = [[18, 1, 10, 10, 10, 10], [3, -2, 0, 0, 23, 0]];
		let expected = [20, 0, 10, 10, 20, 10];
		let sources = sources_input
			.iter()
			.enumerate()
			.map(|(index, inputs)| {
				AbilitySource::Regular(RegularAbilitySource {
					title: format!("test_{}", index),
					description: format!("test_description_{}", index),
					ability_parts: CONFIG
						.ability_names_with_max
						.iter()
						.zip(inputs.iter())
						.map(|((abi, _, _), input)| (String::from(*abi), *input))
						.collect(),
					check_sum: None,
					subset: None,
				})
			})
			.collect();

		let abilities = Abilities::from_sources(&sources, &Vec::new(), &Vec::new());

		for ((abi, _, _), expected_val) in CONFIG.ability_names_with_max.iter().zip(expected.iter()) {
			assert_eq!(
				abilities
					.abilities
					.get(&String::from(*abi))
					.unwrap()
					.value
					.get_untracked(),
				*expected_val
			);
		}
	}

	#[test]
	fn test_calc_ability_modifier() {
		let test_table = [(1, -5), (2, -4), (9, -1), (10, 0), (11, 0), (12, 1), (30, 10)];

		for (ability, modifier) in test_table {
			assert_eq!(calc_ability_modifier(ability), modifier);
		}
	}
}
