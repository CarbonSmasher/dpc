pub mod block;
pub mod entity;
pub mod modifier;
pub mod pos;

use std::fmt::{Debug, Display};

use crate::common::Identifier;
use crate::linker::codegen::t::macros::cgwrite;
use crate::linker::codegen::Codegen;
use crate::linker::codegen::CodegenBlockCx;

use self::pos::IntCoordinates;

use self::entity::TargetSelector;
use super::ResourceLocation;

#[derive(Clone)]
pub enum EntityTarget {
	Player(String),
	Selector(TargetSelector),
}

impl EntityTarget {
	pub fn is_blank_this(&self) -> bool {
		matches!(self, EntityTarget::Selector(sel) if sel.is_blank_this())
	}

	pub fn is_value_eq(&self, other: &Self) -> bool {
		matches!((self, other), (Self::Player(l), Self::Player(r)) if l == r)
			|| matches!((self, other), (Self::Selector(l), Self::Selector(r)) if l.is_value_eq(r))
	}
}

impl Debug for EntityTarget {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Player(player) => write!(f, "{player}"),
			Self::Selector(sel) => write!(f, "{sel:?}"),
		}
	}
}

#[derive(Clone)]
pub struct Score {
	pub holder: EntityTarget,
	pub objective: Identifier,
}

impl Score {
	pub fn new(holder: EntityTarget, objective: Identifier) -> Self {
		Self { holder, objective }
	}

	pub fn is_value_eq(&self, other: &Self) -> bool {
		self.holder.is_value_eq(&other.holder) && self.objective == other.objective
	}
}

impl Debug for Score {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{:?} {}", self.holder, self.objective)
	}
}

pub type DataPath = String;

#[derive(Debug, Clone)]
pub enum DataLocation {
	Block(IntCoordinates),
	Entity(EntityTarget),
	Storage(ResourceLocation),
}

impl DataLocation {
	pub fn is_value_eq(&self, other: &Self) -> bool {
		matches!((self, other), (Self::Block(l), Self::Block(r)) if l.is_value_eq(r))
			|| matches!((self, other), (Self::Entity(l), Self::Entity(r)) if l.is_value_eq(r))
			|| matches!((self, other), (Self::Storage(l), Self::Storage(r)) if l == r)
	}
}

impl Codegen for DataLocation {
	fn gen_writer<F>(&self, f: &mut F, cbcx: &mut CodegenBlockCx) -> anyhow::Result<()>
	where
		F: std::fmt::Write,
	{
		match self {
			Self::Block(pos) => cgwrite!(f, cbcx, "block ", pos)?,
			Self::Entity(target) => cgwrite!(f, cbcx, "entity ", target)?,
			Self::Storage(loc) => cgwrite!(f, cbcx, "storage ", loc)?,
		}
		Ok(())
	}
}

#[derive(Debug, Clone)]
pub struct FullDataLocation {
	pub loc: DataLocation,
	pub path: DataPath,
}

impl FullDataLocation {
	pub fn is_value_eq(&self, other: &Self) -> bool {
		self.loc.is_value_eq(&other.loc) && self.path == other.path
	}
}

impl Codegen for FullDataLocation {
	fn gen_writer<F>(&self, f: &mut F, cbcx: &mut CodegenBlockCx) -> anyhow::Result<()>
	where
		F: std::fmt::Write,
	{
		cgwrite!(f, cbcx, self.loc, " ", self.path)?;
		Ok(())
	}
}

#[derive(Debug, Clone)]
pub enum XPValue {
	Points,
	Levels,
}

impl Display for XPValue {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(
			f,
			"{}",
			match self {
				Self::Points => "points",
				Self::Levels => "levels",
			}
		)
	}
}

impl Codegen for XPValue {
	fn gen_writer<F>(&self, f: &mut F, cbcx: &mut CodegenBlockCx) -> anyhow::Result<()>
	where
		F: std::fmt::Write,
	{
		let _ = cbcx;
		write!(f, "{self}")?;
		Ok(())
	}
}

#[derive(Debug, Clone)]
pub enum Difficulty {
	Peaceful,
	Easy,
	Normal,
	Hard,
}

impl Display for Difficulty {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(
			f,
			"{}",
			match self {
				Self::Peaceful => "peaceful",
				Self::Easy => "easy",
				Self::Normal => "normal",
				Self::Hard => "hard",
			}
		)
	}
}

impl Codegen for Difficulty {
	fn gen_writer<F>(&self, f: &mut F, cbcx: &mut CodegenBlockCx) -> anyhow::Result<()>
	where
		F: std::fmt::Write,
	{
		let _ = cbcx;
		write!(f, "{self}")?;
		Ok(())
	}
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Gamemode {
	Survival,
	Creative,
	Adventure,
	Spectator,
}

impl Display for Gamemode {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(
			f,
			"{}",
			match self {
				Self::Survival => "survival",
				Self::Creative => "creative",
				Self::Adventure => "adventure",
				Self::Spectator => "spectator",
			}
		)
	}
}

impl Codegen for Gamemode {
	fn gen_writer<F>(&self, f: &mut F, cbcx: &mut CodegenBlockCx) -> anyhow::Result<()>
	where
		F: std::fmt::Write,
	{
		let _ = cbcx;
		write!(f, "{self}")?;
		Ok(())
	}
}

#[derive(Clone)]
pub enum Heightmap {
	WorldSurface,
	MotionBlocking,
	MotionBlockingNoLeaves,
	OceanFloor,
}

impl Debug for Heightmap {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::WorldSurface => write!(f, "world_surface"),
			Self::MotionBlocking => write!(f, "motion_blocking"),
			Self::MotionBlockingNoLeaves => write!(f, "motion_blocking_no_leaves"),
			Self::OceanFloor => write!(f, "ocean_floor"),
		}
	}
}
