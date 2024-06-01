use std::hash::{Hash, Hasher as _};

use leptos::leptos_dom::logging::console_error;

#[derive(Hash, Clone)]
pub struct Config {
	pub ability_names_with_max: [(&'static str, &'static str, u8); 6], //Vec<(String, String)>, // Abbreviation, full name
	pub ability_improvement_amount: i8, // how many ability increases the character gets per class level ability improvement
	pub icons: IconsConfig,
	pub level_experience_thresholds: [u32; 19], // level thresholds, starting from lvl 1 -> lvl 2 threshold
}

#[derive(Hash, Clone)]
pub struct IconsConfig {
	pub factions: [(&'static str, &'static str); 5],
	pub classes: [(&'static str, &'static str); 13],
	pub adventure_league: [(&'static str, &'static str); 10],
}

impl Config {
	pub const fn get() -> Config {
		Config {
			ability_names_with_max: [
				("Str", "Strength", 20),
				("Dex", "Dexterity", 20),
				("Con", "Constitution", 20),
				("Int", "Intelligence", 20),
				("Wis", "Wisdom", 20),
				("Cha", "Charisma", 20),
			],
			ability_improvement_amount: 2,
			icons: IconsConfig {
				factions: [
					("Emerald Enclave", "emerald_enclave"),
					("Harpers", "harpers"),
					("Lords' Alliance", "lords_alliance"),
					("Order of the Gauntlet", "order_gauntlet"),
					("Zhentarim", "zhentarim"),
				],
				classes: [
					("Artificer", "artificer"),
					("Barbarian", "barbarian"),
					("Bard", "bard"),
					("Cleric", "cleric"),
					("Druid", "druid"),
					("Fighter", "fighter"),
					("Monk", "monk"),
					("Paladin", "paladin"),
					("Ranger", "ranger"),
					("Rogue", "rogue"),
					("Sorcerer", "sorcerer"),
					("Warlock", "warlock"),
					("Wizard", "wizard"),
				],
				adventure_league: [
					("1 Tyranny of Dragons", "tod"),
					("2 Elemental Evil", "ee"),
					("3 Rage of Demons", "rod"),
					("4 Curse of Strahd", "cos"),
					("5 Storm King's Thunder", "skt"),
					("6 Tales from the Yawning Portal", "totyp"),
					("7 Tomb of Annihilation", "toa"),
					("8 Waterdeep Adventures", "wda"),
					("9 Descent into Avernus", "dia"),
					("10 Rime of the Frostmaiden", "rotf"),
				],
			},
			level_experience_thresholds: [
				300, 900, 2700, 6500, 14000, 23000, 34000, 48000, 64000, 85000, 100000, 120000, 140000,
				165000, 195000, 225000, 265000, 305000, 355000,
			],
		}
	}

	pub fn get_hash(&self) -> String {
		let mut hasher = std::hash::DefaultHasher::new();
		std::hash::Hash::hash(&self, &mut hasher);
		format!("{}", hasher.finish())
	}
}

impl serde::Serialize for Config {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		serializer.serialize_str(&self.get_hash())
	}
}

impl<'de> serde::Deserialize<'de> for Config {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		let hash = String::deserialize(deserializer)?;
		let config = Config::get();
		let current_hash = config.get_hash();
		if !hash.is_empty() && (current_hash != hash) {
			console_error(
				format!(
					"Loaded hash: {}, current hash: {}, will attempt to continue loading.",
					hash, current_hash
				)
				.as_str(),
			);
			web_sys::window().unwrap().alert_with_message(
				"Config of the saved file is not the same as the current config. Will attempt to load the file with the new config."
			).unwrap();
		}
		Ok(config)
	}
}

pub static CONFIG: Config = Config::get();
