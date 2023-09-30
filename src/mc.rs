use crate::common::Identifier;

#[derive(Debug, Clone)]
pub enum TargetSelector {
	Player(String),
}

impl TargetSelector {
	pub fn codegen_str(&self) -> String {
		match self {
			Self::Player(player) => player.clone(),
		}
	}
}

#[derive(Debug, Clone)]
pub struct Score {
	pub holder: TargetSelector,
	pub objective: Identifier,
}

impl Score {
	pub fn new(holder: TargetSelector, objective: Identifier) -> Self {
		Self { holder, objective }
	}
}

#[derive(Debug, Clone)]
pub enum DataLocation {
	Entity(String),
	Storage(String),
}
