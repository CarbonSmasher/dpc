pub mod block;
pub mod function;
pub mod mc;
pub mod modifier;
pub mod range;
pub mod target_selector;
pub mod ty;

use std::{fmt::Debug, sync::Arc};

use anyhow::{bail, Context};
use dashmap::DashMap;

use self::mc::{FullDataLocation, Score};

use self::ty::{DataType, DataTypeContents, NBTTypeContents, ScoreTypeContents};

#[derive(Clone)]
pub enum Value {
	Mutable(MutableValue),
	Constant(DataTypeContents),
}

impl Value {
	pub fn get_ty(&self, regs: &RegisterList) -> anyhow::Result<DataType> {
		let out = match self {
			Self::Constant(contents) => contents.get_ty(),
			Self::Mutable(val) => val.get_ty(regs)?,
		};

		Ok(out)
	}

	pub fn get_used_regs(&self) -> Vec<&Identifier> {
		match self {
			Self::Mutable(val) => val.get_used_regs(),
			_ => Vec::new(),
		}
	}

	pub fn get_used_regs_mut(&mut self) -> Vec<&mut Identifier> {
		match self {
			Self::Mutable(val) => val.get_used_regs_mut(),
			_ => Vec::new(),
		}
	}

	pub fn to_score_value(self) -> anyhow::Result<ScoreValue> {
		let out = match self {
			Self::Constant(val) => {
				if let DataTypeContents::Score(score) = val {
					ScoreValue::Constant(score)
				} else {
					bail!("Expected value to be a score");
				}
			}
			Self::Mutable(val) => ScoreValue::Mutable(val.to_mutable_score_value()?),
		};

		Ok(out)
	}

	pub fn to_nbt_value(self) -> anyhow::Result<NBTValue> {
		let out = match self {
			Self::Constant(val) => {
				if let DataTypeContents::NBT(nbt) = val {
					NBTValue::Constant(nbt)
				} else {
					bail!("Expected value to be NBT");
				}
			}
			Self::Mutable(val) => NBTValue::Mutable(val.to_mutable_nbt_value()?),
		};

		Ok(out)
	}
}

impl Debug for Value {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let text = match self {
			Self::Constant(val) => format!("{val:?}"),
			Self::Mutable(val) => format!("{val:?}"),
		};
		write!(f, "{text}")
	}
}

#[derive(Clone)]
pub enum MutableValue {
	Register(Identifier),
	Score(Score),
	Data(FullDataLocation),
}

impl MutableValue {
	pub fn get_ty(&self, regs: &RegisterList) -> anyhow::Result<DataType> {
		let out = match self {
			Self::Register(id) => {
				let reg = regs
					.get(id)
					.with_context(|| format!("Failed to get register ${id}"))?;
				reg.ty.clone()
			}
			Self::Score(..) => DataType::Score(ty::ScoreType::Score),
			Self::Data(..) => DataType::NBT(ty::NBTType::Any),
		};

		Ok(out)
	}

	pub fn get_used_regs(&self) -> Vec<&Identifier> {
		match self {
			Self::Register(reg) => vec![&reg],
			_ => Vec::new(),
		}
	}

	pub fn get_used_regs_mut(&mut self) -> Vec<&mut Identifier> {
		match self {
			Self::Register(reg) => vec![reg],
			_ => Vec::new(),
		}
	}

	pub fn is_same_val(&self, other: &Self) -> bool {
		matches!((self, other), (Self::Register(left), Self::Register(right)) if left == right)
	}

	pub fn to_mutable_score_value(self) -> anyhow::Result<MutableScoreValue> {
		match self {
			Self::Register(reg) => Ok(MutableScoreValue::Reg(reg)),
			Self::Score(score) => Ok(MutableScoreValue::Score(score)),
			_ => bail!("Value cannot be converted to a score value"),
		}
	}

	pub fn to_mutable_nbt_value(self) -> anyhow::Result<MutableNBTValue> {
		match self {
			Self::Register(reg) => Ok(MutableNBTValue::Reg(reg)),
			Self::Data(data) => Ok(MutableNBTValue::Data(data)),
			_ => bail!("Value cannot be converted to a NBT value"),
		}
	}
}

impl Debug for MutableValue {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let text = match self {
			Self::Register(reg) => format!("${reg}"),
			Self::Score(score) => format!("${score:?}"),
			Self::Data(data) => format!("${data:?}"),
		};
		write!(f, "{text}")
	}
}

#[derive(Clone)]
pub enum DeclareBinding {
	Null,
	Value(Value),
	Cast(DataType, MutableValue),
	Index {
		ty: DataType,
		val: Value,
		index: Value,
	},
}

impl DeclareBinding {
	pub fn get_ty(&self, regs: &RegisterList) -> anyhow::Result<Option<DataType>> {
		let out = match self {
			Self::Null => None,
			Self::Value(val) => Some(val.get_ty(regs)?),
			Self::Cast(ty, ..) => Some(ty.clone()),
			Self::Index { ty, .. } => Some(ty.clone()),
		};

		Ok(out)
	}

	pub fn get_used_regs(&self) -> Vec<&Identifier> {
		match self {
			Self::Null => Vec::new(),
			Self::Value(val) => val.get_used_regs(),
			Self::Cast(_, val) => val.get_used_regs(),
			Self::Index { val, index, .. } => [val.get_used_regs(), index.get_used_regs()].concat(),
		}
	}
}

impl Debug for DeclareBinding {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let text = match self {
			Self::Null => "null".to_string(),
			Self::Value(val) => format!("{val:?}"),
			Self::Cast(ty, val) => format!("cast {ty:?} {val:?}"),
			Self::Index { val, index, ty } => format!("idx {ty:?} {val:?} {index:?}"),
		};
		write!(f, "{text}")
	}
}

pub type Identifier = Arc<str>;

#[derive(Debug, Clone)]
pub struct Register {
	pub id: Identifier,
	pub ty: DataType,
}

pub type RegisterList = DashMap<Identifier, Register>;

pub type ResourceLocation = Identifier;
pub type ResourceLocationTag = Identifier;

#[derive(Clone)]
pub enum ScoreValue {
	Constant(ScoreTypeContents),
	Mutable(MutableScoreValue),
}

impl ScoreValue {
	pub fn get_used_regs(&self) -> Vec<&Identifier> {
		match self {
			Self::Constant(..) => Vec::new(),
			Self::Mutable(val) => val.get_used_regs(),
		}
	}

	pub fn get_used_regs_mut(&mut self) -> Vec<&mut Identifier> {
		match self {
			Self::Constant(..) => Vec::new(),
			Self::Mutable(val) => val.get_used_regs_mut(),
		}
	}

	pub fn is_value_eq(&self, other: &Self) -> bool {
		matches!((self, other), (Self::Constant(l), Self::Constant(r)) if l.is_value_eq(r))
			|| matches!((self, other), (Self::Mutable(l), Self::Mutable(r)) if l.is_value_eq(r))
	}
}

impl Debug for ScoreValue {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let text = match self {
			Self::Constant(val) => format!("{val:?}"),
			Self::Mutable(val) => format!("{val:?}"),
		};
		write!(f, "{text}")
	}
}

#[derive(Clone)]
pub enum MutableScoreValue {
	Score(Score),
	Reg(Identifier),
}

impl MutableScoreValue {
	pub fn get_used_regs(&self) -> Vec<&Identifier> {
		match self {
			Self::Score(..) => Vec::new(),
			Self::Reg(reg) => vec![reg],
		}
	}

	pub fn get_used_regs_mut(&mut self) -> Vec<&mut Identifier> {
		match self {
			Self::Score(..) => Vec::new(),
			Self::Reg(reg) => vec![reg],
		}
	}

	pub fn is_value_eq(&self, other: &Self) -> bool {
		matches!((self, other), (Self::Score(l), Self::Score(r)) if l.is_value_eq(r))
			|| matches!((self, other), (Self::Reg(l), Self::Reg(r)) if l == r)
	}
}

impl Debug for MutableScoreValue {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let text = match self {
			Self::Score(score) => format!("{score:?}"),
			Self::Reg(reg) => format!("${reg}"),
		};
		write!(f, "{text}")
	}
}

#[derive(Clone)]
pub enum NBTValue {
	Constant(NBTTypeContents),
	Mutable(MutableNBTValue),
}

impl NBTValue {
	pub fn get_used_regs(&self) -> Vec<&Identifier> {
		match self {
			Self::Constant(..) => Vec::new(),
			Self::Mutable(val) => val.get_used_regs(),
		}
	}

	pub fn is_value_eq(&self, other: &Self) -> bool {
		matches!((self, other), (Self::Constant(l), Self::Constant(r)) if l.is_value_eq(r))
			|| matches!((self, other), (Self::Mutable(l), Self::Mutable(r)) if l.is_value_eq(r))
	}
}

impl Debug for NBTValue {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let text = match self {
			Self::Constant(val) => format!("{val:?}"),
			Self::Mutable(val) => format!("{val:?}"),
		};
		write!(f, "{text}")
	}
}

#[derive(Clone)]
pub enum MutableNBTValue {
	Data(FullDataLocation),
	Reg(Identifier),
}

impl MutableNBTValue {
	pub fn get_used_regs(&self) -> Vec<&Identifier> {
		match self {
			Self::Data(..) => Vec::new(),
			Self::Reg(reg) => vec![reg],
		}
	}

	pub fn is_value_eq(&self, other: &Self) -> bool {
		matches!((self, other), (Self::Data(l), Self::Data(r)) if l.is_value_eq(r))
			|| matches!((self, other), (Self::Reg(l), Self::Reg(r)) if l == r)
	}
}

impl Debug for MutableNBTValue {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let text = match self {
			Self::Data(data) => format!("{data:?}"),
			Self::Reg(reg) => format!("${reg}"),
		};
		write!(f, "{text}")
	}
}
