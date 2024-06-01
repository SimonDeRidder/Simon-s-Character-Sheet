pub type AbilityPart = i8;
pub type AbilityValue = u8;
pub type Modifier = i8;

// a wrapper around RwSignal that serialises outside reactive context (for saving to file)
pub struct SignalField<T: 'static>(leptos::prelude::RwSignal<T>);

impl<T> SignalField<T>
where
	T: Clone + Send + Sync,
{
	pub fn new(value: T) -> Self {
		Self(leptos::prelude::RwSignal::new(value))
	}

	pub fn get(&self) -> T {
		leptos::prelude::Get::get(&self.0)
	}

	pub fn get_untracked(&self) -> T {
		leptos::prelude::GetUntracked::get_untracked(&self.0)
	}

	pub fn set(&self, new_value: T) {
		leptos::prelude::Set::set(&self.0, new_value)
	}

	pub fn update(&self, func: impl FnOnce(&mut T)) {
		leptos::prelude::Update::update(&self.0, func)
	}

	pub fn read(&self) -> leptos::prelude::guards::ReadGuard<T, leptos::prelude::guards::Plain<T>> {
		leptos::prelude::Read::read(&self.0)
	}

	pub fn read_untracked(&self) -> leptos::prelude::guards::ReadGuard<T, leptos::prelude::guards::Plain<T>> {
		leptos::prelude::ReadUntracked::read_untracked(&self.0)
	}
}

impl<T> Clone for SignalField<T> {
	fn clone(&self) -> Self {
		*self
	}
}

impl<T> Copy for SignalField<T> {}

impl serde::Serialize for SignalField<String> {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		serializer.serialize_str(self.get_untracked().as_str())
	}
}

impl<'de> serde::Deserialize<'de> for SignalField<String> {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		struct SignalFieldVisitor;
		impl serde::de::Visitor<'_> for SignalFieldVisitor {
			type Value = SignalField<String>;

			fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
				formatter.write_str("String")
			}

			fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
			where
				E: serde::de::Error,
			{
				Ok(Self::Value::new(String::from(v)))
			}

			fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
			where
				E: serde::de::Error,
			{
				Ok(Self::Value::new(v))
			}
		}
		deserializer.deserialize_string(SignalFieldVisitor {})
	}
}

impl<T> serde::Serialize for SignalField<Option<T>>
where
	T: serde::Serialize + Clone + Send + Sync,
{
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		match self.get_untracked() {
			None => serializer.serialize_none(),
			Some(val) => serializer.serialize_some(&val),
		}
	}
}

impl<'de> serde::Deserialize<'de> for SignalField<Option<String>> {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		struct StringVisitor;
		impl serde::de::Visitor<'_> for StringVisitor {
			type Value = String;

			fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
				formatter.write_str("String")
			}

			fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
			where
				E: serde::de::Error,
			{
				Ok(Self::Value::from(v))
			}

			fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
			where
				E: serde::de::Error,
			{
				Ok(v)
			}
		}

		struct SignalFieldVisitor;
		impl<'de2> serde::de::Visitor<'de2> for SignalFieldVisitor {
			type Value = SignalField<Option<String>>;

			fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
				formatter.write_str("Option<String>")
			}

			fn visit_none<E>(self) -> Result<Self::Value, E>
			where
				E: serde::de::Error,
			{
				Ok(Self::Value::new(None))
			}

			fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
			where
				D: serde::Deserializer<'de2>,
			{
				Ok(Self::Value::new(Some(deserializer.deserialize_string(StringVisitor {})?)))
			}

			fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
			where
				E: serde::de::Error,
			{
				Ok(Self::Value::new(Some(String::from(v))))
			}

			fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
			where
				E: serde::de::Error,
			{
				Ok(Self::Value::new(Some(v)))
			}
		}

		deserializer.deserialize_option(SignalFieldVisitor {})
	}
}

impl serde::Serialize for SignalField<u8> {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		serializer.serialize_u8(self.get_untracked())
	}
}

impl<'de> serde::Deserialize<'de> for SignalField<u8> {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		struct SignalFieldVisitor;
		impl serde::de::Visitor<'_> for SignalFieldVisitor {
			type Value = SignalField<u8>;

			fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
				formatter.write_str("String")
			}

			fn visit_u8<E>(self, v: u8) -> Result<Self::Value, E>
			where
				E: serde::de::Error,
			{
				Ok(Self::Value::new(v))
			}
		}
		deserializer.deserialize_u8(SignalFieldVisitor {})
	}
}

impl serde::Serialize for SignalField<u32> {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		serializer.serialize_u32(self.get_untracked())
	}
}

impl<'de> serde::Deserialize<'de> for SignalField<u32> {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		struct SignalFieldVisitor;
		impl serde::de::Visitor<'_> for SignalFieldVisitor {
			type Value = SignalField<u32>;

			fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
				formatter.write_str("String")
			}

			fn visit_u32<E>(self, v: u32) -> Result<Self::Value, E>
			where
				E: serde::de::Error,
			{
				Ok(Self::Value::new(v))
			}
		}
		deserializer.deserialize_u32(SignalFieldVisitor {})
	}
}

impl<SomeSerialisableType: serde::Serialize + Sync + Send> serde::Serialize
	for SignalField<Vec<SomeSerialisableType>>
{
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		let value_vec = leptos::prelude::ReadUntracked::read_untracked(&self.0);
		let mut seq = serializer.serialize_seq(Some(value_vec.len()))?;
		for element in value_vec.iter() {
			serde::ser::SerializeSeq::serialize_element(&mut seq, element)?;
		}
		serde::ser::SerializeSeq::end(seq)
	}
}
