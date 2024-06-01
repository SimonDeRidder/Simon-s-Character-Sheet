pub mod abilities;

use self::abilities::Abilities;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Stats {
	pub abilities: Abilities,
}

impl Stats {
	pub fn default() -> Self {
		Self { abilities: Abilities::default() }
	}
}
