use std::fmt::{Debug, Display};

use super::{super::ty::NBTCompoundTypeContents, Gamemode};

#[derive(Clone, PartialEq, Eq)]
pub struct TargetSelector {
	pub selector: SelectorType,
	pub params: Vec<SelectorParameter>,
}

impl TargetSelector {
	pub fn new(selector: SelectorType) -> Self {
		Self::with_params(selector, Vec::new())
	}

	pub fn with_params(selector: SelectorType, params: Vec<SelectorParameter>) -> Self {
		Self { selector, params }
	}

	pub fn is_blank_this(&self) -> bool {
		matches!(self.selector, SelectorType::This) && self.params.is_empty()
	}

	pub fn is_value_eq(&self, other: &Self) -> bool {
		self.selector == other.selector && self.params == other.params
	}

	/// Checks if this selector is in single type
	pub fn is_single_type(&self) -> bool {
		self.selector.is_single_type()
			|| matches!(self.selector, SelectorType::AllEntities | SelectorType::AllPlayers if self.params.contains(&SelectorParameter::Limit(1)))
	}

	/// Checks if this selector is in player type
	pub fn is_player_type(&self) -> bool {
		self.selector.is_player_type()
			|| matches!(self.selector, SelectorType::AllEntities if self.params.contains(&SelectorParameter::Type { ty: "player".into(), invert: false }))
	}
}

impl Debug for TargetSelector {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.selector.codegen_str())?;
		if !self.params.is_empty() {
			write!(f, "[")?;
			for (i, param) in self.params.iter().enumerate() {
				write!(f, "{param:?}")?;
				if i != self.params.len() - 1 {
					write!(f, ",")?;
				}
			}
			write!(f, "]")?;
		}
		Ok(())
	}
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SelectorType {
	This,
	NearestPlayer,
	RandomPlayer,
	AllPlayers,
	AllEntities,
}

impl SelectorType {
	pub fn codegen_str(&self) -> &'static str {
		match self {
			Self::This => "@s",
			Self::NearestPlayer => "@p",
			Self::RandomPlayer => "@r",
			Self::AllPlayers => "@a",
			Self::AllEntities => "@e",
		}
	}

	/// Checks if this selector type is in single type
	pub fn is_single_type(&self) -> bool {
		matches!(self, Self::This | Self::NearestPlayer | Self::RandomPlayer)
	}

	/// Checks if this selector type is in player type
	pub fn is_player_type(&self) -> bool {
		matches!(
			self,
			Self::NearestPlayer | Self::AllPlayers | Self::RandomPlayer
		)
	}
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SelectorParameter {
	Type {
		ty: String,
		invert: bool,
	},
	Tag {
		tag: String,
		invert: bool,
	},
	NoTags,
	Predicate {
		predicate: String,
		invert: bool,
	},
	Gamemode {
		gamemode: Gamemode,
		invert: bool,
	},
	Name {
		name: String,
		invert: bool,
	},
	NBT {
		nbt: NBTCompoundTypeContents,
		invert: bool,
	},
	Limit(u32),
	Sort(SelectorSort),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SelectorSort {
	Nearest,
	Furthest,
	Random,
	Arbitrary,
}

impl Display for SelectorSort {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(
			f,
			"{}",
			match self {
				Self::Nearest => "nearest",
				Self::Furthest => "furthest",
				Self::Random => "random",
				Self::Arbitrary => "arbitrary",
			}
		)
	}
}
