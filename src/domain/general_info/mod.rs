use leptos::prelude::Get as _;
use wasm_bindgen::prelude::wasm_bindgen;

use crate::config::CONFIG;

use super::types::SignalField;

#[derive(serde::Serialize, Clone, Debug)]
pub struct GeneralInfo {
	pub name: SignalField<String>,
	pub player_icon: SignalField<String>,
	pub experience: SignalField<u32>,
	pub classes: SignalField<Vec<ClassLevel>>, // first class is 'primary'
	pub player_name: SignalField<String>,
	pub background: SignalField<Option<String>>,
	pub background_option: SignalField<Option<String>>,
	pub race: SignalField<Option<String>>,
	pub race_previous: SignalField<Option<String>>, // for e.g. Dhampir

	#[serde(skip)]
	pub level: leptos::prelude::Memo<u8>,
	#[serde(skip)]
	pub next_level_experience: leptos::prelude::Memo<u32>,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct ClassLevel {
	pub id: String,
	pub name: String,
	pub subclass_id: Option<String>,
	pub level: u8,
}

impl GeneralInfo {
	pub fn default() -> Self {
		GeneralInfo::new(
			String::from(""),
			String::from("img/icons/blank.svg"),
			0u32,
			Vec::new(),
			String::from(""),
			None,
			None,
			None,
			None,
		)
	}

	#[allow(clippy::too_many_arguments)]
	fn new(
		name: String,
		player_icon: String,
		experience: u32,
		classes: Vec<ClassLevel>,
		player_name: String,
		background: Option<String>,
		background_option: Option<String>,
		race: Option<String>,
		race_previous: Option<String>,
	) -> Self {
		let experience_signal = SignalField::new(experience);
		let level_memo =
			leptos::prelude::Memo::new(move |_| calculate_level_from_experience(experience_signal.get()));
		GeneralInfo {
			name: SignalField::new(name),
			player_icon: SignalField::new(player_icon),
			experience: experience_signal,
			classes: SignalField::new(classes),
			player_name: SignalField::new(player_name),
			background: SignalField::new(background),
			background_option: SignalField::new(background_option),
			race: SignalField::new(race),
			race_previous: SignalField::new(race_previous),

			level: level_memo,
			next_level_experience: leptos::prelude::Memo::new(move |_| {
				get_minimum_experience_for_level(level_memo.get() + 1)
			}),
		}
	}

	pub fn get_class_level(&self, class_id: String) -> u8 {
		for class_level in self.classes.read_untracked().iter() {
			if class_level.id == class_id {
				return class_level.level;
			}
		}
		0
	}

	pub fn has_class(&self, class_id: String) -> bool {
		for class_level in self.classes.read_untracked().iter() {
			if class_level.id == class_id {
				return true;
			}
		}
		false
	}

	pub fn get_subclass(&self, class_id: String) -> Option<String> {
		for class_level in self.classes.read_untracked().iter() {
			if class_level.id == class_id {
				return class_level.subclass_id.clone();
			}
		}
		None
	}

	pub fn list_classes(&self) -> Vec<String> {
		self.classes
			.read_untracked()
			.iter()
			.map(|class_level| class_level.id.clone())
			.collect()
	}

	pub fn set_class(&self, class_id: String, name: String, subclass: Option<String>, level: u8) {
		self.classes.update(|classes| {
			for class_level in classes.iter_mut() {
				if class_level.id == class_id {
					class_level.name = name.clone();
					class_level.subclass_id = subclass.clone();
					class_level.level = level;
					return;
				}
			}
			classes.push(ClassLevel {
				id: class_id.clone(),
				name: name.clone(),
				subclass_id: subclass.clone(),
				level,
			});
		});
	}

	pub fn remove_class(&self, class_id: String) {
		self.classes.update(|classes| {
			classes.retain(|class_level| class_level.id != class_id);
		})
	}

	pub fn set_subclass(&self, class_id: String, subclass: Option<String>) {
		self.classes.update(|classes| {
			for class_level in classes {
				if class_level.id == class_id {
					class_level.subclass_id = subclass.clone();
				}
			}
		});
	}

	// first class is the primary class
	pub fn get_primary_class(&self) -> Option<String> {
		let classes_ = self.classes.read_untracked();
		if classes_.is_empty() {
			None
		} else {
			Some(classes_[0].id.clone())
		}
	}
}

pub fn calculate_level_from_experience(experience: u32) -> u8 {
	if experience == 0 {
		return 0; // special case, before character creation (TODO: remove and make character creation wizard pop up at start)
	}
	match CONFIG.level_experience_thresholds.binary_search(&experience) {
		Ok(index) => (index as u8) + 2, // casting to u8 should be ok, level should never be bigger than 20, let alone 255
		Err(insertion_index) => (insertion_index as u8) + 1,
	}
}

#[wasm_bindgen] // backwards compatibility for loading 0.2 saves
pub fn get_minimum_experience_for_level(level: u8) -> u32 {
	match level {
		0 => 0,
		1 => 1,
		lvl => {
			CONFIG.level_experience_thresholds
				[usize::from(lvl - 2).min(CONFIG.level_experience_thresholds.len() - 1)]
		},
	}
}

impl<'de> serde::Deserialize<'de> for GeneralInfo {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		#[derive(serde::Deserialize)]
		struct SerialisedGeneralInfo {
			name: String,
			player_icon: String,
			experience: u32,
			classes: Vec<ClassLevel>,
			player_name: String,
			background: Option<String>,
			background_option: Option<String>,
			race: Option<String>,
			race_previous: Option<String>,
		}

		serde::Deserialize::deserialize(deserializer).map(move |general_info: SerialisedGeneralInfo| {
			GeneralInfo::new(
				general_info.name,
				general_info.player_icon,
				general_info.experience,
				general_info.classes,
				general_info.player_name,
				general_info.background,
				general_info.background_option,
				general_info.race,
				general_info.race_previous,
			)
		})
	}
}

#[cfg(test)]
mod tests {
	use crate::config::CONFIG;

	use super::{calculate_level_from_experience, get_minimum_experience_for_level};

	#[test]
	fn test_calculate_level_from_experience() {
		let thresholds = CONFIG.level_experience_thresholds;
		let max_level = (thresholds.len() + 1) as u8;
		for (experience, expected_level) in [
			(0, 0),
			(1, 1),
			(thresholds[0] - 1, 1),
			(thresholds[0], 2),
			(thresholds[0] + 1, 2),
			(thresholds[thresholds.len() - 1] - 1, max_level - 1),
			(thresholds[thresholds.len() - 1], max_level),
			(thresholds[thresholds.len() - 1] + 1, max_level),
		] {
			assert_eq!(calculate_level_from_experience(experience), expected_level)
		}
	}

	#[test]
	fn test_get_minimum_experience_for_level() {
		let thresholds = CONFIG.level_experience_thresholds;
		let max_level = (thresholds.len() + 1) as u8;
		for (level, expected_experience) in
			[(0, 0), (1, 1), (2, thresholds[0]), (max_level, thresholds[thresholds.len() - 1])]
		{
			assert_eq!(get_minimum_experience_for_level(level), expected_experience)
		}
	}
}
