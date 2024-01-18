use std::{collections::HashMap, fmt::Debug};

use crate::common::{ty::NBTCompoundTypeContents, Identifier};
use crate::common::{ResourceLocation, ResourceLocationTag};

use super::pos::IntCoordinates;

#[derive(Clone, PartialEq, Eq)]
pub struct SetBlockData {
	pub pos: IntCoordinates,
	pub block: BlockData,
	pub mode: SetBlockMode,
}

impl Debug for SetBlockData {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{:?} {:?} {:?}", self.pos, self.block, self.mode)
	}
}

#[derive(Clone, PartialEq, Eq)]
pub struct FillData {
	pub start: IntCoordinates,
	pub end: IntCoordinates,
	pub block: BlockData,
	pub mode: FillMode,
}

impl Debug for FillData {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(
			f,
			"{:?} {:?} {:?} {:?}",
			self.start, self.end, self.block, self.mode
		)
	}
}

#[derive(Clone, PartialEq, Eq)]
pub struct CloneData {
	pub source_dimension: Option<ResourceLocation>,
	pub start: IntCoordinates,
	pub end: IntCoordinates,
	pub target_dimension: Option<ResourceLocation>,
	pub destination: IntCoordinates,
	pub mask_mode: CloneMaskMode,
	pub mode: CloneMode,
}

impl Debug for CloneData {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		if let Some(source) = &self.source_dimension {
			write!(f, "{source} ")?;
		}

		write!(f, "{:?} {:?}", self.start, self.end)?;

		if let Some(target) = &self.target_dimension {
			write!(f, "{target}")?;
		}

		write!(
			f,
			"{:?} {:?} {:?}",
			self.destination, self.mask_mode, self.mode
		)?;

		Ok(())
	}
}

#[derive(Clone, PartialEq, Eq)]
pub struct FillBiomeData {
	pub start: IntCoordinates,
	pub end: IntCoordinates,
	pub biome: ResourceLocation,
	pub replace: Option<ResourceLocation>,
}

impl Debug for FillBiomeData {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{:?} {:?} {:?}", self.start, self.end, self.biome)?;
		if let Some(repl) = &self.replace {
			write!(f, " repl {repl:?}")?;
		}
		Ok(())
	}
}

#[derive(Clone, PartialEq, Eq)]
pub struct BlockData {
	pub block: ResourceLocation,
	pub props: BlockProperties,
}

impl BlockData {
	pub fn new(block: ResourceLocation, props: BlockProperties) -> Self {
		Self { block, props }
	}
}

impl Debug for BlockData {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}{:?}", self.block, self.props)
	}
}

#[derive(Clone, PartialEq, Eq)]
pub struct BlockFilter {
	pub block: ResourceLocationTag,
	pub props: BlockProperties,
}

impl Debug for BlockFilter {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}{:?}", self.block, self.props)
	}
}

#[derive(Clone, PartialEq, Eq)]
pub struct BlockProperties {
	pub data: NBTCompoundTypeContents,
	pub states: BlockStates,
}

impl BlockProperties {
	pub fn new() -> Self {
		Self {
			data: NBTCompoundTypeContents::new(),
			states: BlockStates::new(HashMap::new()),
		}
	}
}

impl Default for BlockProperties {
	fn default() -> Self {
		Self::new()
	}
}

impl Debug for BlockProperties {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{:?} {:?}", self.data, self.states)
	}
}

#[derive(Clone, PartialEq, Eq)]
pub enum SetBlockMode {
	Destroy,
	Keep,
	Replace,
}

impl Debug for SetBlockMode {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Destroy => write!(f, "destroy"),
			Self::Keep => write!(f, "keep"),
			Self::Replace => write!(f, "replace"),
		}
	}
}

impl SetBlockMode {
	pub fn parse(string: &str) -> Option<Self> {
		match string {
			"destroy" => Some(Self::Destroy),
			"keep" => Some(Self::Keep),
			"replace" => Some(Self::Replace),
			_ => None,
		}
	}
}

#[derive(Clone, PartialEq, Eq)]
pub enum FillMode {
	Destroy,
	Hollow,
	Keep,
	Outline,
	Replace(Option<BlockFilter>),
}

impl Debug for FillMode {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Destroy => write!(f, "destroy"),
			Self::Hollow => write!(f, "hollow"),
			Self::Keep => write!(f, "keep"),
			Self::Outline => write!(f, "outline"),
			Self::Replace(filter) => {
				if let Some(filter) = filter {
					write!(f, "replace {filter:?}")
				} else {
					write!(f, "replace")
				}
			}
		}
	}
}

#[derive(Clone, PartialEq, Eq)]
pub enum CloneMaskMode {
	Replace,
	Masked,
	Filtered(BlockFilter),
}

impl Debug for CloneMaskMode {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Replace => write!(f, "replace"),
			Self::Masked => write!(f, "masked"),
			Self::Filtered(filter) => write!(f, "filtered {filter:?}"),
		}
	}
}

#[derive(Clone, PartialEq, Eq)]
pub enum CloneMode {
	Force,
	Move,
	Normal,
}

impl Debug for CloneMode {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Force => write!(f, "force"),
			Self::Move => write!(f, "move"),
			Self::Normal => write!(f, "normal"),
		}
	}
}

#[derive(Clone, PartialEq, Eq)]
pub struct BlockStates(HashMap<String, BlockStateValue>);

impl BlockStates {
	pub fn new(values: HashMap<String, BlockStateValue>) -> Self {
		Self(values)
	}

	pub fn get(&self) -> &HashMap<String, BlockStateValue> {
		&self.0
	}

	pub fn is_empty(&self) -> bool {
		self.0.is_empty()
	}

	pub fn len(&self) -> usize {
		self.0.len()
	}
}

impl Debug for BlockStates {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		self.0.fmt(f)
	}
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BlockStateValue {
	String(Identifier),
}
