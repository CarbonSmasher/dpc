use std::{collections::HashMap, fmt::Debug};

use crate::common::{ty::NBTCompoundTypeContents, Identifier};
use crate::common::{ResourceLocation, ResourceLocationTag};
use crate::linker::codegen::t::macros::cgwrite;
use crate::linker::codegen::Codegen;

use super::pos::IntCoordinates;

#[derive(Clone)]
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

#[derive(Clone)]
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

#[derive(Clone)]
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

impl Codegen for BlockData {
	fn gen_writer<F>(
		&self,
		f: &mut F,
		cbcx: &mut crate::linker::codegen::CodegenBlockCx,
	) -> anyhow::Result<()>
	where
		F: std::fmt::Write,
	{
		write!(f, "{}", self.block)?;
		cgwrite!(f, cbcx, self.props)?;
		Ok(())
	}
}

#[derive(Clone)]
pub struct BlockFilter {
	pub block: ResourceLocationTag,
	pub props: BlockProperties,
}

impl Debug for BlockFilter {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}{:?}", self.block, self.props)
	}
}

#[derive(Clone)]
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

impl Debug for BlockProperties {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{:?} {:?}", self.data, self.states)
	}
}

impl Codegen for BlockProperties {
	fn gen_writer<F>(
		&self,
		f: &mut F,
		cbcx: &mut crate::linker::codegen::CodegenBlockCx,
	) -> anyhow::Result<()>
	where
		F: std::fmt::Write,
	{
		if !self.states.is_empty() {
			cgwrite!(f, cbcx, self.states)?;
		}

		if !self.data.is_empty() {
			cgwrite!(f, cbcx, self.data.get_literal_str())?;
		}

		Ok(())
	}
}

#[derive(Clone)]
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

impl Codegen for SetBlockMode {
	fn gen_writer<F>(
		&self,
		f: &mut F,
		cbcx: &mut crate::linker::codegen::CodegenBlockCx,
	) -> anyhow::Result<()>
	where
		F: std::fmt::Write,
	{
		let _ = cbcx;
		write!(f, "{self:?}")?;
		Ok(())
	}
}

#[derive(Clone)]
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

impl Codegen for FillMode {
	fn gen_writer<F>(
		&self,
		f: &mut F,
		cbcx: &mut crate::linker::codegen::CodegenBlockCx,
	) -> anyhow::Result<()>
	where
		F: std::fmt::Write,
	{
		let _ = cbcx;
		write!(f, "{self:?}")?;
		Ok(())
	}
}

#[derive(Debug, Clone)]
pub struct BlockStates(HashMap<String, BlockStateValue>);

impl BlockStates {
	pub fn new(values: HashMap<String, BlockStateValue>) -> Self {
		Self(values)
	}

	pub fn is_empty(&self) -> bool {
		self.0.is_empty()
	}
}

impl Codegen for BlockStates {
	fn gen_writer<F>(
		&self,
		f: &mut F,
		cbcx: &mut crate::linker::codegen::CodegenBlockCx,
	) -> anyhow::Result<()>
	where
		F: std::fmt::Write,
	{
		let _ = cbcx;
		write!(f, "[")?;

		for (i, (k, v)) in self.0.iter().enumerate() {
			write!(f, "\"{k}\"=")?;
			match v {
				BlockStateValue::String(string) => write!(f, "{string}")?,
			}
			if i != self.0.len() - 1 {
				write!(f, ",")?;
			}
		}

		write!(f, "]")?;
		Ok(())
	}
}

#[derive(Debug, Clone)]
pub enum BlockStateValue {
	String(Identifier),
}
