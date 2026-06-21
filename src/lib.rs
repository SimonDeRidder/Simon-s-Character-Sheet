mod config;
mod domain;
mod render;
mod utils;

use config::{CONFIG, Config};
use serde::ser::SerializeStruct as _;
use wasm_bindgen::prelude::wasm_bindgen;

use domain::{general_info::GeneralInfo, stats::Stats};

use crate::domain::equipment::Equipment;
// use render::context_menu::remove_all_context_menus;

// Called when the wasm module is instantiated
#[wasm_bindgen(start)]
pub fn main() {
	// First, set panic hook, this should happen once during initialisation
	utils::set_panic_hook();
	any_spawner::Executor::init_wasm_bindgen().expect("Failed to init_wasm_bindgen");
}

#[wasm_bindgen]
pub struct Character {
	startup: leptos::prelude::RwSignal<bool>, // Temporary, to delay effects until after JS global variables are initiated
	general_info: GeneralInfo,
	stats: Stats,
	equipment: Equipment,
}

#[wasm_bindgen]
#[allow(clippy::new_without_default)]
impl Character {
	pub fn new() -> Self {
		// build character
		Character {
			startup: leptos::prelude::RwSignal::new(true),
			general_info: GeneralInfo::default(),
			stats: Stats::default(),
			equipment: Equipment::default(),
		}
	}
}

impl serde::Serialize for Character {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		let mut serde_state = serializer.serialize_struct("Character", false as usize + 1)?;
		serde_state.serialize_field("config", &CONFIG)?;
		serde_state.serialize_field("general_info", &self.general_info)?;
		serde_state.serialize_field("stats", &self.stats)?;
		serde_state.serialize_field("equipment", &self.equipment)?;
		serde_state.end()
	}
}

impl<'de> serde::Deserialize<'de> for Character {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		enum Field {
			ConfigFld,
			GeneralInfoFld,
			StatsFld,
			EquipmentFld,
			Ignore,
		}
		struct FieldVisitor;

		impl serde::de::Visitor<'_> for FieldVisitor {
			type Value = Field;
			fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
				std::fmt::Formatter::write_str(formatter, "field identifier")
			}

			fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
			where
				E: serde::de::Error,
			{
				match value {
					"config" => Ok(Field::ConfigFld),
					"general_info" => Ok(Field::GeneralInfoFld),
					"stats" => Ok(Field::StatsFld),
					"equipment" => Ok(Field::EquipmentFld),
					_ => Ok(Field::Ignore),
				}
			}
		}

		impl<'de> serde::Deserialize<'de> for Field {
			fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
			where
				D: serde::Deserializer<'de>,
			{
				serde::Deserializer::deserialize_identifier(deserializer, FieldVisitor)
			}
		}

		struct Visitor {}

		impl<'de> serde::de::Visitor<'de> for Visitor {
			type Value = Character;
			fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
				std::fmt::Formatter::write_str(formatter, "struct Character")
			}

			fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
			where
				A: serde::de::MapAccess<'de>,
			{
				let mut config_field: Option<Config> = None;
				let mut general_info_field: Option<GeneralInfo> = None;
				let mut stats_field: Option<Stats> = None;
				let mut equipment_field: Option<Equipment> = None;
				while let Some(key) = serde::de::MapAccess::next_key::<Field>(&mut map)? {
					match key {
						Field::ConfigFld => {
							if Option::is_some(&config_field) {
								return Err(<A::Error as serde::de::Error>::duplicate_field("config"));
							}
							config_field = Some(serde::de::MapAccess::next_value::<Config>(&mut map)?);
						},
						Field::GeneralInfoFld => {
							if Option::is_some(&general_info_field) {
								return Err(<A::Error as serde::de::Error>::duplicate_field("general_info"));
							}
							general_info_field =
								Some(serde::de::MapAccess::next_value::<GeneralInfo>(&mut map)?);
						},
						Field::StatsFld => {
							if Option::is_some(&stats_field) {
								return Err(<A::Error as serde::de::Error>::duplicate_field("stats"));
							}
							stats_field = Some(serde::de::MapAccess::next_value::<Stats>(&mut map)?);
						},
						Field::EquipmentFld => {
							if Option::is_some(&equipment_field) {
								return Err(<A::Error as serde::de::Error>::duplicate_field("equipment"));
							}
							equipment_field = Some(serde::de::MapAccess::next_value::<Equipment>(&mut map)?);
						},
						_ => {
							let _ = serde::de::MapAccess::next_value::<serde::de::IgnoredAny>(&mut map)?;
						},
					}
				}
				let general_info = match general_info_field {
					Some(general_info) => Ok(general_info),
					None => Err(serde::de::Error::missing_field("general_info")),
				}?;
				let stats = match stats_field {
					Some(stats) => Ok(stats),
					None => Err(serde::de::Error::missing_field("stats")),
				}?;
				let equipment = match equipment_field {
					Some(equipment) => Ok(equipment),
					None => Err(serde::de::Error::missing_field("equipment")),
				}?;
				Ok(Character {
					startup: leptos::prelude::RwSignal::new(true),
					general_info,
					stats,
					equipment,
				})
			}
		}

		serde::Deserializer::deserialize_struct(
			deserializer,
			"Character",
			&["general_info", "stats", "equipment"],
			Visitor {},
		)
	}
}
