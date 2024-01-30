use std::fmt::Debug;

use anyhow::{bail, Context};

use crate::common::function::FunctionSignature;
use crate::common::reg::{GetUsedLocals, GetUsedRegs, Local};
use crate::common::ty::{DataType, Double, NBTType, NBTTypeContents};
use crate::common::val::{MutableNBTValue, MutableScoreValue, MutableValue, ScoreValue};
use crate::common::{RegisterList, ResourceLocationTag};

use super::block::BlockFilter;
use super::pos::{DoubleCoordinates, DoubleCoordinates2D};
use super::{EntityTarget, FullDataLocation, Heightmap, IntCoordinates, Score};

use super::{Identifier, ResourceLocation};

/// A modifier to the context of a command
#[derive(Clone)]
pub enum Modifier {
	StoreResult(StoreModLocation),
	StoreSuccess(StoreModLocation),
	If {
		condition: Box<IfModCondition>,
		negate: bool,
	},
	Anchored(AnchorLocation),
	Align(AlignAxes),
	As(EntityTarget),
	At(EntityTarget),
	In(ResourceLocation),
	On(EntityRelation),
	Positioned(DoubleCoordinates),
	PositionedAs(EntityTarget),
	PositionedOver(Heightmap),
	Rotated(DoubleCoordinates2D),
	RotatedAs(EntityTarget),
	FacingPosition(DoubleCoordinates),
	FacingEntity(EntityTarget, AnchorLocation),
	Summon(ResourceLocation),
}

impl Modifier {
	/// Checks if this modifier has any side effects that aren't applied to
	/// the command it is modifying
	pub fn has_extra_side_efects(&self) -> bool {
		matches!(
			self,
			Self::StoreResult(..) | Self::StoreSuccess(..) | Self::Summon(..)
		)
	}
}

impl GetUsedRegs for Modifier {
	fn append_used_regs<'a>(&'a self, regs: &mut Vec<&'a Identifier>) {
		match self {
			Modifier::StoreResult(loc) | Modifier::StoreSuccess(loc) => loc.append_used_regs(regs),
			Modifier::If { condition, .. } => condition.append_used_regs(regs),
			_ => {}
		}
	}
}

impl GetUsedLocals for Modifier {
	fn append_used_locals<'a>(&'a self, locals: &mut Vec<&'a Local>) {
		match self {
			Modifier::StoreResult(loc) | Modifier::StoreSuccess(loc) => {
				loc.append_used_locals(locals)
			}
			Modifier::If { condition, .. } => condition.append_used_locals(locals),
			_ => {}
		}
	}
}

impl Debug for Modifier {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::StoreResult(loc) => write!(f, "str {loc:?}"),
			Self::StoreSuccess(loc) => write!(f, "sts {loc:?}"),
			Self::If { condition, negate } => {
				if *negate {
					write!(f, "if !{condition:?}")
				} else {
					write!(f, "if {condition:?}")
				}
			}
			Self::Anchored(loc) => write!(f, "anc {loc:?}"),
			Self::Align(axes) => write!(f, "aln {axes:?}"),
			Self::As(target) => write!(f, "as {target:?}"),
			Self::At(target) => write!(f, "at {target:?}"),
			Self::In(dim) => write!(f, "in {dim}"),
			Self::On(rel) => write!(f, "on {rel:?}"),
			Self::Positioned(coords) => write!(f, "pos {coords:?}"),
			Self::PositionedAs(target) => write!(f, "pose {target:?}"),
			Self::PositionedOver(hm) => write!(f, "poso {hm:?}"),
			Self::Rotated(rot) => write!(f, "rot {rot:?}"),
			Self::RotatedAs(target) => write!(f, "rote {target:?}"),
			Self::FacingPosition(coords) => write!(f, "facp {coords:?}"),
			Self::FacingEntity(target, anchor) => write!(f, "face {target:?} {anchor:?}"),
			Self::Summon(entity) => write!(f, "summon {entity}"),
		}
	}
}

/// A modifier to the context of a command
#[derive(Clone, PartialEq)]
pub enum MIRModifier {
	StoreResult(StoreModLocation),
	StoreSuccess(StoreModLocation),
	Anchored(AnchorLocation),
	Align(AlignAxes),
	As(EntityTarget),
	At(EntityTarget),
	In(ResourceLocation),
	On(EntityRelation),
	Positioned(DoubleCoordinates),
	PositionedAs(EntityTarget),
	PositionedOver(Heightmap),
	Rotated(DoubleCoordinates2D),
	RotatedAs(EntityTarget),
	FacingPosition(DoubleCoordinates),
	FacingEntity(EntityTarget, AnchorLocation),
	Summon(ResourceLocation),
}

impl MIRModifier {
	/// Checks if this modifier has any side effects that aren't applied to
	/// the command it is modifying
	pub fn has_extra_side_efects(&self) -> bool {
		matches!(
			self,
			Self::StoreResult(..) | Self::StoreSuccess(..) | Self::Summon(..)
		)
	}

	pub fn replace_regs<F: Fn(&mut Identifier)>(&mut self, f: &F) {
		match self {
			Self::StoreResult(loc) | Self::StoreSuccess(loc) => loc.replace_regs(f),
			_ => {}
		}
	}
}

impl GetUsedRegs for MIRModifier {
	fn append_used_regs<'a>(&'a self, regs: &mut Vec<&'a Identifier>) {
		match self {
			Self::StoreResult(loc) | Self::StoreSuccess(loc) => loc.append_used_regs(regs),
			_ => {}
		}
	}
}

impl Debug for MIRModifier {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::StoreResult(loc) => write!(f, "str {loc:?}"),
			Self::StoreSuccess(loc) => write!(f, "sts {loc:?}"),
			Self::Anchored(loc) => write!(f, "anc {loc:?}"),
			Self::Align(axes) => write!(f, "aln {axes:?}"),
			Self::As(target) => write!(f, "as {target:?}"),
			Self::At(target) => write!(f, "at {target:?}"),
			Self::In(dim) => write!(f, "in {dim}"),
			Self::On(rel) => write!(f, "on {rel:?}"),
			Self::Positioned(coords) => write!(f, "pos {coords:?}"),
			Self::PositionedAs(target) => write!(f, "pose {target:?}"),
			Self::PositionedOver(hm) => write!(f, "poso {hm:?}"),
			Self::Rotated(rot) => write!(f, "rot {rot:?}"),
			Self::RotatedAs(target) => write!(f, "rote {target:?}"),
			Self::FacingPosition(coords) => write!(f, "facp {coords:?}"),
			Self::FacingEntity(target, anchor) => write!(f, "face {target:?} {anchor:?}"),
			Self::Summon(entity) => write!(f, "summon {entity}"),
		}
	}
}

#[derive(Clone, PartialEq)]
pub enum StoreModLocation {
	Local(Local, Double),
	Score(Score),
	Data(FullDataLocation, StoreDataType, Double),
	Bossbar(ResourceLocation, StoreBossbarMode),
}

impl StoreModLocation {
	pub fn from_mut_val(
		val: MutableValue,
		regs: &RegisterList,
		sig: &FunctionSignature,
	) -> anyhow::Result<Self> {
		match val.get_ty(regs, sig)? {
			DataType::Score(..) => Self::from_mut_score_val(&val.to_mutable_score_value()?),
			DataType::NBT(ty) => {
				let ty = StoreDataType::from_nbt_ty(&ty).context("Unsupported type")?;
				Self::from_mut_nbt_val(&val.to_mutable_nbt_value()?, ty, 1.0)
			}
			_ => bail!("Unsupported type"),
		}
	}
	pub fn from_mut_score_val(val: &MutableScoreValue) -> anyhow::Result<Self> {
		match val {
			MutableScoreValue::Local(loc) => Ok(Self::Local(loc.clone(), 1.0)),
			MutableScoreValue::Score(score) => Ok(Self::Score(score.clone())),
		}
	}

	// TODO: Support storing in idx and prop
	pub fn from_mut_nbt_val(
		val: &MutableNBTValue,
		ty: StoreDataType,
		scale: Double,
	) -> anyhow::Result<Self> {
		match val {
			MutableNBTValue::Local(loc) => Ok(Self::Local(loc.clone(), scale)),
			MutableNBTValue::Data(data) => Ok(Self::Data(data.clone(), ty, scale)),
			_ => bail!("Unsupported storage location"),
		}
	}

	pub fn replace_regs<F: Fn(&mut Identifier)>(&mut self, f: F) {
		match self {
			Self::Local(loc, ..) => {
				for reg in loc.get_used_regs_mut() {
					f(reg);
				}
			}
			_ => {}
		}
	}
}

impl GetUsedRegs for StoreModLocation {
	fn append_used_regs<'a>(&'a self, regs: &mut Vec<&'a Identifier>) {
		match self {
			Self::Local(loc, ..) => loc.append_used_regs(regs),
			Self::Score(..) | Self::Data(..) | Self::Bossbar(..) => {}
		}
	}
}

impl GetUsedLocals for StoreModLocation {
	fn append_used_locals<'a>(&'a self, locals: &mut Vec<&'a Local>) {
		match self {
			Self::Local(loc, ..) => locals.push(loc),
			Self::Score(..) | Self::Data(..) | Self::Bossbar(..) => {}
		}
	}
}

impl Debug for StoreModLocation {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Local(loc, scale) => write!(f, "{loc:?} {scale}"),
			Self::Score(score) => write!(f, "{score:?}"),
			Self::Data(data, ty, scale) => write!(f, "{data:?} {ty:?} {scale}"),
			Self::Bossbar(bar, mode) => write!(f, "bb {bar} {mode:?}"),
		}
	}
}

#[derive(Clone, PartialEq)]
pub enum StoreDataType {
	Byte,
	Short,
	Int,
	Long,
	Float,
	Double,
}

impl Debug for StoreDataType {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Byte => write!(f, "byte"),
			Self::Short => write!(f, "short"),
			Self::Int => write!(f, "int"),
			Self::Long => write!(f, "long"),
			Self::Float => write!(f, "float"),
			Self::Double => write!(f, "double"),
		}
	}
}

impl StoreDataType {
	pub fn from_nbt_ty(ty: &NBTType) -> Option<Self> {
		match ty {
			NBTType::Byte => Some(Self::Byte),
			NBTType::Short => Some(Self::Short),
			NBTType::Int => Some(Self::Int),
			NBTType::Long => Some(Self::Long),
			NBTType::Float => Some(Self::Float),
			NBTType::Double => Some(Self::Double),
			_ => None,
		}
	}
}

#[derive(Clone, PartialEq)]
pub enum StoreBossbarMode {
	Value,
	Max,
}

impl Debug for StoreBossbarMode {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Value => write!(f, "value"),
			Self::Max => write!(f, "max"),
		}
	}
}

#[derive(Clone, PartialEq)]
pub enum AnchorLocation {
	Eyes,
	Feet,
}

impl Debug for AnchorLocation {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Eyes => write!(f, "eyes"),
			Self::Feet => write!(f, "feet"),
		}
	}
}

#[derive(Debug, Clone, PartialEq)]
pub enum EntityRelation {
	Attacker,
	Controller,
	Leasher,
	Origin,
	Owner,
	Passengers,
	Target,
	Vehicle,
}

#[derive(Clone, PartialEq)]
pub struct AlignAxes {
	pub x: bool,
	pub y: bool,
	pub z: bool,
}

impl Debug for AlignAxes {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		if self.x {
			write!(f, "x")?;
		}
		if self.y {
			write!(f, "y")?;
		}
		if self.z {
			write!(f, "z")?;
		}
		Ok(())
	}
}

#[derive(Clone)]
pub enum IfModCondition {
	Score(IfScoreCondition),
	Entity(EntityTarget),
	Predicate(ResourceLocation),
	Function(ResourceLocationTag, Vec<Local>),
	Biome(IntCoordinates, ResourceLocationTag),
	Dimension(ResourceLocation),
	Loaded(IntCoordinates),
	DataExists(MutableNBTValue),
	DataEquals(MutableNBTValue, NBTTypeContents),
	Block(IntCoordinates, BlockFilter),
	Const(bool),
}

impl GetUsedRegs for IfModCondition {
	fn append_used_regs<'a>(&'a self, regs: &mut Vec<&'a Identifier>) {
		match self {
			Self::Score(cond) => match cond {
				IfScoreCondition::Single { left, right } => {
					left.append_used_regs(regs);
					right.append_used_regs(regs);
				}
				IfScoreCondition::Range { score, left, right } => {
					score.append_used_regs(regs);
					left.append_used_regs(regs);
					right.append_used_regs(regs);
				}
			},
			Self::DataExists(val) | Self::DataEquals(val, ..) => val.append_used_regs(regs),
			Self::Function(_, locals) => {
				for loc in locals {
					loc.append_used_regs(regs);
				}
			}
			Self::Entity(..)
			| Self::Predicate(..)
			| Self::Biome(..)
			| Self::Dimension(..)
			| Self::Loaded(..)
			| Self::Block(..)
			| Self::Const(..) => {}
		}
	}
}

impl GetUsedLocals for IfModCondition {
	fn append_used_locals<'a>(&'a self, locals: &mut Vec<&'a Local>) {
		match self {
			Self::Score(cond) => match cond {
				IfScoreCondition::Single { left, right } => {
					left.append_used_locals(locals);
					right.append_used_locals(locals);
				}
				IfScoreCondition::Range { score, left, right } => {
					score.append_used_locals(locals);
					left.append_used_locals(locals);
					right.append_used_locals(locals);
				}
			},
			Self::DataExists(val) | Self::DataEquals(val, ..) => val.append_used_locals(locals),
			Self::Function(_, locs) => locals.extend(locs),
			Self::Entity(..)
			| Self::Predicate(..)
			| Self::Biome(..)
			| Self::Dimension(..)
			| Self::Loaded(..)
			| Self::Block(..)
			| Self::Const(..) => {}
		}
	}
}

impl Debug for IfModCondition {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Score(condition) => write!(f, "sco {condition:?}"),
			Self::Entity(target) => write!(f, "ent {target:?}"),
			Self::Predicate(pred) => write!(f, "pred {pred}"),
			Self::Function(fun, ..) => write!(f, "func {fun}"),
			Self::Biome(pos, biome) => write!(f, "bio {pos:?} {biome}"),
			Self::Dimension(dim) => write!(f, "dim {dim}"),
			Self::Loaded(pos) => write!(f, "load {pos:?}"),
			Self::DataExists(loc) => write!(f, "data {loc:?}"),
			Self::DataEquals(l, r) => write!(f, "deq {l:?} {r:?}"),
			Self::Block(loc, block) => write!(f, "blo {loc:?} {block:?}"),
			Self::Const(val) => write!(f, "const {val}"),
		}
	}
}

#[derive(Clone)]
pub enum IfScoreCondition {
	Single {
		left: ScoreValue,
		right: ScoreValue,
	},
	Range {
		score: ScoreValue,
		left: IfScoreRangeEnd,
		right: IfScoreRangeEnd,
	},
}

impl Debug for IfScoreCondition {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Single { left, right } => write!(f, "{left:?} = {right:?}"),
			Self::Range { score, left, right } => write!(f, "{score:?} {left:?}..{right:?}"),
		}
	}
}

#[derive(Clone)]
pub enum IfScoreRangeEnd {
	Infinite,
	Fixed { value: ScoreValue, inclusive: bool },
}

impl GetUsedRegs for IfScoreRangeEnd {
	fn append_used_regs<'a>(&'a self, regs: &mut Vec<&'a Identifier>) {
		match self {
			Self::Infinite => {}
			Self::Fixed { value, .. } => value.append_used_regs(regs),
		}
	}
}

impl GetUsedLocals for IfScoreRangeEnd {
	fn append_used_locals<'a>(&'a self, locals: &mut Vec<&'a Local>) {
		match self {
			Self::Infinite => {}
			Self::Fixed { value, .. } => value.append_used_locals(locals),
		}
	}
}

impl Debug for IfScoreRangeEnd {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		if let Self::Fixed { value, inclusive } = self {
			if *inclusive {
				write!(f, "=")?;
			}
			write!(f, "{value:?}")?;
		}
		Ok(())
	}
}
