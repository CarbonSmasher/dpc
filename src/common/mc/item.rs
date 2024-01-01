use crate::common::ty::NBTCompoundTypeContents;
use crate::common::ResourceLocation;
use crate::output::codegen::t::macros::cgwrite;
use crate::output::codegen::Codegen;
use std::fmt::Debug;

use super::pos::IntCoordinates;
use super::EntityTarget;

#[derive(Clone, PartialEq, Eq)]
pub struct ItemData {
	pub item: ResourceLocation,
	pub nbt: NBTCompoundTypeContents,
}

impl Debug for ItemData {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{:?} {:?}", self.item, self.nbt)
	}
}

#[derive(Debug, Clone, PartialEq)]
pub enum LootSource {
	Fish {
		table: ResourceLocation,
		pos: IntCoordinates,
		tool: LootTool,
	},
	Loot {
		table: ResourceLocation,
	},
	Kill {
		target: EntityTarget,
	},
	Mine {
		pos: IntCoordinates,
		tool: LootTool,
	},
}

#[derive(Debug, Clone, PartialEq)]
pub enum LootTool {
	Item(ItemData),
	Mainhand,
	Offhand,
}

impl Codegen for LootTool {
	fn gen_writer<F>(
		&self,
		f: &mut F,
		cbcx: &mut crate::output::codegen::CodegenBlockCx,
	) -> anyhow::Result<()>
	where
		F: std::fmt::Write,
	{
		match self {
			Self::Item(itm) => itm.gen_writer(f, cbcx)?,
			Self::Mainhand => write!(f, "mainhand")?,
			Self::Offhand => write!(f, "offhand")?,
		}

		Ok(())
	}
}

impl Codegen for LootSource {
	fn gen_writer<F>(
		&self,
		f: &mut F,
		cbcx: &mut crate::output::codegen::CodegenBlockCx,
	) -> anyhow::Result<()>
	where
		F: std::fmt::Write,
	{
		match self {
			Self::Fish { table, pos, tool } => {
				cgwrite!(f, cbcx, "fish ", table, " ", pos, " ", tool)?
			}
			Self::Loot { table } => write!(f, "loot {table}")?,
			Self::Kill { target } => cgwrite!(f, cbcx, "kill ", target)?,
			Self::Mine { pos, tool } => cgwrite!(f, cbcx, "mine", pos, " ", tool)?,
		}

		Ok(())
	}
}

#[derive(Debug, Clone, PartialEq)]
pub enum ItemModifyLocation {
	Block(IntCoordinates),
	Entity(EntityTarget),
}

impl Codegen for ItemModifyLocation {
	fn gen_writer<F>(
		&self,
		f: &mut F,
		cbcx: &mut crate::output::codegen::CodegenBlockCx,
	) -> anyhow::Result<()>
	where
		F: std::fmt::Write,
	{
		match self {
			Self::Block(pos) => cgwrite!(f, cbcx, "block ", pos),
			Self::Entity(target) => cgwrite!(f, cbcx, "entity ", target),
		}
	}
}
