use std::fmt::{Debug, Display};

use num_traits::Num;

use crate::common::Identifier;
use crate::linker::codegen::t::macros::cgwrite;
use crate::linker::codegen::Codegen;
use crate::linker::codegen::CodegenBlockCx;

use super::target_selector::TargetSelector;
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

#[derive(Clone)]
pub enum Coordinates<T> {
	XYZ(AbsOrRelCoord<T>, AbsOrRelCoord<T>, AbsOrRelCoord<T>),
	Local(T, T, T),
}

impl<T: Num> Coordinates<T> {
	pub fn are_zero(&self) -> bool {
		matches!(self, Self::XYZ(x, y, z) if x.is_relative_zero() && y.is_relative_zero() && z.is_relative_zero())
			|| matches!(self, Self::Local(a, b, c) if a.is_zero() && b.is_zero() && c.is_zero())
	}
}

impl<T: Num + PartialEq + Eq> Coordinates<T> {
	pub fn is_value_eq(&self, other: &Self) -> bool {
		matches!((self, other), (Self::XYZ(x1, y1, z1), Self::XYZ(x2, y2, z2)) if x1.is_value_eq(x2) && y1.is_value_eq(y2) && z1.is_value_eq(z2))
			|| matches!((self, other), (Self::Local(x1, y1, z1), Self::Local(x2, y2, z2)) if x1 == x2 && y1 == y2 && z1 == z2)
	}
}

impl<T: Debug + Num> Debug for Coordinates<T> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::XYZ(x, y, z) => {
				write!(f, "{x:?} {y:?} {z:?}")?;
			}
			Self::Local(a, b, c) => {
				write!(f, "^")?;
				if !a.is_zero() {
					write!(f, "{a:?}")?;
				}
				write!(f, " ^")?;
				if !b.is_zero() {
					write!(f, "{b:?}")?;
				}
				write!(f, " ^")?;
				if !c.is_zero() {
					write!(f, "{c:?}")?;
				}
			}
		}
		Ok(())
	}
}

impl<T: Debug + Num> Codegen for Coordinates<T> {
	fn gen_writer<F>(&self, f: &mut F, cbcx: &mut CodegenBlockCx) -> anyhow::Result<()>
	where
		F: std::fmt::Write,
	{
		let _ = cbcx;
		write!(f, "{self:?}")?;
		Ok(())
	}
}

pub type DoubleCoordinates = Coordinates<f64>;
pub type IntCoordinates = Coordinates<i64>;

#[derive(Clone)]
pub struct Rotation<T>(AbsOrRelCoord<T>, AbsOrRelCoord<T>);

impl<T: Num> Rotation<T> {
	pub fn are_zero(&self) -> bool {
		self.0.is_relative_zero() && self.1.is_relative_zero()
	}
}

impl<T: Num + PartialEq + Eq> Rotation<T> {
	pub fn is_value_eq(&self, other: &Self) -> bool {
		self.0.is_value_eq(&other.0) && self.1.is_value_eq(&other.1)
	}
}

impl<T: Debug + Num> Debug for Rotation<T> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{:?} {:?}", self.0, self.1)
	}
}

impl<T: Debug + Num> Codegen for Rotation<T> {
	fn gen_writer<F>(&self, f: &mut F, cbcx: &mut CodegenBlockCx) -> anyhow::Result<()>
	where
		F: std::fmt::Write,
	{
		let _ = cbcx;
		write!(f, "{self:?}")?;
		Ok(())
	}
}

pub type DoubleRotation = Rotation<f64>;
pub type IntRotation = Rotation<i64>;

#[derive(Clone)]
pub enum AbsOrRelCoord<T> {
	Abs(T),
	Rel(T),
}

impl<T: Num> AbsOrRelCoord<T> {
	/// Checks if this coordinate is relatively zero. Absolute zero will return false
	pub fn is_relative_zero(&self) -> bool {
		matches!(self, Self::Rel(val) if val.is_zero())
	}
}

impl<T: Num + PartialEq + Eq> AbsOrRelCoord<T> {
	pub fn is_value_eq(&self, other: &Self) -> bool {
		matches!((self, other), (Self::Abs(l), Self::Abs(r)) if l == r)
			|| matches!((self, other), (Self::Rel(l), Self::Rel(r)) if l == r)
	}
}

impl<T: Debug + Num> Debug for AbsOrRelCoord<T> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Abs(val) => write!(f, "{val:?}")?,
			Self::Rel(val) => {
				write!(f, "~")?;
				if !val.is_zero() {
					write!(f, "{val:?}")?;
				}
			}
		}

		Ok(())
	}
}

impl<T: Debug + Num> Codegen for AbsOrRelCoord<T> {
	fn gen_writer<F>(&self, f: &mut F, cbcx: &mut CodegenBlockCx) -> anyhow::Result<()>
	where
		F: std::fmt::Write,
	{
		let _ = cbcx;
		write!(f, "{self:?}")?;

		Ok(())
	}
}

#[derive(Debug, Clone)]
pub enum Axis {
	X,
	Y,
	Z,
}

impl Axis {
	pub fn codegen_str(&self) -> &'static str {
		match self {
			Self::X => "x",
			Self::Y => "y",
			Self::Z => "z",
		}
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