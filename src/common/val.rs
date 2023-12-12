use std::fmt::Debug;

use anyhow::{bail, Context};

use super::function::FunctionParams;
use super::mc::{FullDataLocation, Score};
use super::{ty, Identifier, RegisterList, ResourceLocation};

use super::ty::{DataType, DataTypeContents, NBTTypeContents, ScoreTypeContents};

#[derive(Clone, PartialEq)]
pub enum Value {
	Mutable(MutableValue),
	Constant(DataTypeContents),
}

impl Value {
	pub fn get_ty(&self, regs: &RegisterList, params: &FunctionParams) -> anyhow::Result<DataType> {
		let out = match self {
			Self::Constant(contents) => contents.get_ty(),
			Self::Mutable(val) => val.get_ty(regs, params)?,
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

	pub fn iter_mut_val(&mut self) -> Option<&mut MutableValue> {
		match self {
			Self::Constant(..) => None,
			Self::Mutable(val) => Some(val),
		}
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

#[derive(Clone, PartialEq, Eq)]
pub enum MutableValue {
	Register(Identifier),
	Score(Score),
	Data(FullDataLocation),
	Arg(u16),
	CallArg(u16, ResourceLocation, DataType),
}

impl MutableValue {
	pub fn get_ty(&self, regs: &RegisterList, params: &FunctionParams) -> anyhow::Result<DataType> {
		let out = match self {
			Self::Register(id) => {
				let reg = regs
					.get(id)
					.with_context(|| format!("Failed to get register ${id}"))?;
				reg.ty.clone()
			}
			Self::Score(..) => DataType::Score(ty::ScoreType::Score),
			Self::Data(..) => DataType::NBT(ty::NBTType::Any),
			Self::Arg(num) => params
				.get(*num as usize)
				.context("Argument index out of range")?
				.clone(),
			Self::CallArg(_, _, ty) => ty.clone(),
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
			|| matches!((self, other), (Self::Score(left), Self::Score(right)) if left.is_value_eq(right))
			|| matches!((self, other), (Self::Data(left), Self::Data(right)) if left.is_value_eq(right))
			|| matches!((self, other), (Self::Arg(left), Self::Arg(right)) if left == right)
			|| matches!((self, other), (Self::CallArg(la, lf, ..), Self::CallArg(ra, rf, ..)) if la == ra && lf == rf)
	}

	pub fn to_mutable_score_value(self) -> anyhow::Result<MutableScoreValue> {
		match self {
			Self::Register(reg) => Ok(MutableScoreValue::Reg(reg)),
			Self::Score(score) => Ok(MutableScoreValue::Score(score)),
			Self::Arg(arg) => Ok(MutableScoreValue::Arg(arg)),
			Self::CallArg(arg, func, ty @ DataType::Score(..)) => {
				Ok(MutableScoreValue::CallArg(arg, func, ty))
			}
			_ => bail!("Value cannot be converted to a score value"),
		}
	}

	pub fn to_mutable_nbt_value(self) -> anyhow::Result<MutableNBTValue> {
		match self {
			Self::Register(reg) => Ok(MutableNBTValue::Reg(reg)),
			Self::Data(data) => Ok(MutableNBTValue::Data(data)),
			Self::Arg(arg) => Ok(MutableNBTValue::Arg(arg)),
			Self::CallArg(arg, func, ty @ DataType::NBT(..)) => {
				Ok(MutableNBTValue::CallArg(arg, func, ty))
			}
			_ => bail!("Value cannot be converted to a NBT value"),
		}
	}
}

impl Debug for MutableValue {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let text = match self {
			Self::Register(reg) => format!("%{reg}"),
			Self::Score(score) => format!("sco {score:?}"),
			Self::Data(data) => format!("{data:?}"),
			Self::Arg(num) => format!("&{num}"),
			Self::CallArg(idx, ..) => format!("&&{idx}"),
		};
		write!(f, "{text}")
	}
}

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
	Arg(u16),
	CallArg(u16, ResourceLocation, DataType),
}

impl MutableScoreValue {
	pub fn get_used_regs(&self) -> Vec<&Identifier> {
		match self {
			Self::Score(..) => Vec::new(),
			Self::Reg(reg) => vec![reg],
			Self::Arg(..) | Self::CallArg(..) => Vec::new(),
		}
	}

	pub fn get_used_regs_mut(&mut self) -> Vec<&mut Identifier> {
		match self {
			Self::Score(..) => Vec::new(),
			Self::Reg(reg) => vec![reg],
			Self::Arg(..) | Self::CallArg(..) => Vec::new(),
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
			Self::Reg(reg) => format!("%{reg}"),
			Self::Arg(idx) => format!("&{idx}"),
			Self::CallArg(idx, ..) => format!("&&{idx}"),
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
	Arg(u16),
	CallArg(u16, ResourceLocation, DataType),
}

impl MutableNBTValue {
	pub fn get_used_regs(&self) -> Vec<&Identifier> {
		match self {
			Self::Data(..) => Vec::new(),
			Self::Reg(reg) => vec![reg],
			Self::Arg(..) | Self::CallArg(..) => Vec::new(),
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
			Self::Arg(idx) => format!("&{idx}"),
			Self::CallArg(idx, ..) => format!("&&{idx}"),
		};
		write!(f, "{text}")
	}
}
