#[derive(Clone)]
pub struct ContextMenuContent(Vec<ContextMenuItem>);

#[derive(Clone)]
pub enum ContextMenuItem {
	RegularItem {
		text: &'static str,
		action: leptos::prelude::Action<(), ()>,
	},
	SubmenuItem {
		text: &'static str,
		sub_menu: ContextMenuContent,
	},
	Separator,
}

impl ContextMenuContent {
	pub fn new() -> Self {
		Self(Vec::new())
	}

	pub fn push(&mut self, item: ContextMenuItem) {
		self.0.push(item)
	}

	pub(super) fn iter(&self) -> ContextMenuIterator<'_> {
		ContextMenuIterator { content: self, index: 0 }
	}
}

impl ContextMenuItem {
	pub fn new(text: &'static str, action: leptos::prelude::Action<(), ()>) -> Self {
		Self::RegularItem { text, action }
	}

	pub fn new_submenu(text: &'static str, sub_menu: ContextMenuContent) -> Self {
		Self::SubmenuItem { text, sub_menu }
	}

	pub fn new_separator() -> Self {
		Self::Separator {}
	}
}

pub(super) struct ContextMenuIterator<'a> {
	content: &'a ContextMenuContent,
	index: usize,
}

impl<'a> Iterator for ContextMenuIterator<'a> {
	type Item = &'a ContextMenuItem;

	fn next(&mut self) -> Option<Self::Item> {
		if self.index >= self.content.0.len() {
			return None;
		}
		self.index += 1;
		Some(self.content.0.get(self.index - 1).unwrap())
	}
}
