use leptos::{
	// leptos_dom::logging::console_log,
	reactive::traits::{GetUntracked, ReadUntracked},
};
use serde::ser::SerializeStruct;

use crate::config::AmmunitionIcon;

// #[derive(/*serde::Deserialize,*/ Clone)]
#[derive(serde::Deserialize, Clone)]
pub struct Equipment {
	pub ammunition: leptos::prelude::RwSignal<Vec<leptos::prelude::RwSignal<Ammunition>>>,
}

impl Equipment {
	pub fn default() -> Self {
		Equipment::new(vec![])
	}

	fn new(ammunition: Vec<Ammunition>) -> Self {
		Equipment {
			ammunition: leptos::prelude::RwSignal::new(
				ammunition
					.iter()
					.map(|ammo| leptos::prelude::RwSignal::new(ammo.clone()))
					.collect(),
			),
		}
	}
}

impl serde::Serialize for Equipment {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		let ammunition_vec = self.ammunition.read_untracked();
		let mut equipment_serialiser = serializer.serialize_struct("Equipment", 1)?;
		equipment_serialiser.serialize_field(
			"ammunition",
			&ammunition_vec
				.iter()
				.map(|ammo| ammo.get_untracked())
				.collect::<Vec<_>>(),
		)?;
		equipment_serialiser.end()
	}
}

// impl<'de> serde::Deserialize<'de> for Equipment {
// 	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
// 	where
// 		D: serde::Deserializer<'de>,
// 	{
// 		console_log("starting Equipment deserialise");
// 		#[derive(serde::Deserialize, Clone)]
// 		pub struct Equipment2 {
// 			pub ammunition: leptos::prelude::RwSignal<Vec<leptos::prelude::RwSignal<Ammunition>>>,
// 		}

// 		let a = serde::Deserialize::deserialize(deserializer)
// 			.map(|res: Equipment2| Equipment { ammunition: res.ammunition.clone() });

// 		console_log("ending Equipment deserialise");
// 		a
// 	}
// }

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct Ammunition {
	pub id: String,
	pub total: u8,
	pub used: u8,
	pub icon: AmmunitionIcon,
	pub variant_id: Option<String>,
}

impl Ammunition {
	pub fn pick_up(&mut self) {
		self.used = 0;
	}

	pub fn lose(&mut self) {
		self.total = self.total.saturating_sub(self.used);
		self.used = 0;
	}

	pub fn shoot(&mut self) {
		self.used += 1;
	}
}
