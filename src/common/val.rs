use std::fmt::Debug;

use anyhow::{bail, Context};

use super::function::{FunctionSignature, ReturnType};
use super::mc::{FullDataLocation, Score};
use super::{ty, Identifier, RegisterList, ResourceLocation};

use super::ty::{ArraySize, DataType, DataTypeContents, NBTTypeContents, ScoreTypeContents};

#[derive(Clone, PartialEq)]
pub enum Value {
	Mutable(MutableValue),
	Constant(DataTypeContents),
}

impl Value {
	pub fn get_ty(&self, regs: &RegisterList, sig: &FunctionSignature) -> anyhow::Result<DataType> {
		let out = match self {
			Self::Constant(contents) => contents.get_ty(),
			Self::Mutable(val) => val.get_ty(regs, sig)?,
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

	pub fn is_value_eq(&self, other: &Self) -> bool {
		matches!((self, other), (Self::Constant(l), Self::Constant(r)) if l.is_value_eq(r))
			|| matches!((self, other), (Self::Mutable(l), Self::Mutable(r)) if l.is_same_val(r))
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
	Property(Box<MutableValue>, String),
	Index(Box<MutableValue>, ArraySize),
	Arg(u16),
	CallArg(u16, ResourceLocation, DataType),
	ReturnValue(u16),
	CallReturnValue(u16, ResourceLocation, DataType),
}

impl MutableValue {
	pub fn get_ty(&self, regs: &RegisterList, sig: &FunctionSignature) -> anyhow::Result<DataType> {
		let out = match self {
			Self::Register(id) => {
				let reg = regs
					.get(id)
					.with_context(|| format!("Failed to get register ${id}"))?;
				reg.ty.clone()
			}
			Self::Score(..) => DataType::Score(ty::ScoreType::Score),
			// TODO: Deduce actual type for property and index
			Self::Data(..) | Self::Property(..) | Self::Index(..) => {
				DataType::NBT(ty::NBTType::Any)
			}
			Self::Arg(num) => sig
				.params
				.get(*num as usize)
				.context("Argument index out of range")?
				.clone(),
			Self::ReturnValue(num) => match &sig.ret {
				ReturnType::Standard(tys) => tys
					.get(*num as usize)
					.context("Return value index out of range")?
					.clone(),
				ReturnType::Void => bail!("Function does not return a value"),
			},
			Self::CallArg(_, _, ty) | Self::CallReturnValue(_, _, ty) => ty.clone(),
		};

		Ok(out)
	}

	pub fn get_used_regs(&self) -> Vec<&Identifier> {
		match self {
			Self::Register(reg) => vec![&reg],
			Self::Property(val, ..) | Self::Index(val, ..) => val.get_used_regs(),
			_ => Vec::new(),
		}
	}

	pub fn get_used_regs_mut(&mut self) -> Vec<&mut Identifier> {
		match self {
			Self::Register(reg) => vec![reg],
			Self::Property(val, ..) | Self::Index(val, ..) => val.get_used_regs_mut(),
			_ => Vec::new(),
		}
	}

	pub fn is_same_val(&self, other: &Self) -> bool {
		matches!((self, other), (Self::Register(left), Self::Register(right)) if left == right)
			|| matches!((self, other), (Self::Score(left), Self::Score(right)) if left.is_value_eq(right))
			|| matches!((self, other), (Self::Data(left), Self::Data(right)) if left.is_value_eq(right))
			|| matches!((self, other), (Self::Arg(left), Self::Arg(right)) if left == right)
			|| matches!((self, other), (Self::Property(lv, lp), Self::Property(rv, rp)) if lv.is_same_val(rv) && lp == rp)
			|| matches!((self, other), (Self::Index(lv, li), Self::Index(rv, ri)) if lv.is_same_val(rv) && li == ri)
			|| matches!((self, other), (Self::CallArg(la, lf, ..), Self::CallArg(ra, rf, ..)) if la == ra && lf == rf)
			|| matches!((self, other), (Self::ReturnValue(left), Self::ReturnValue(right)) if left == right)
			|| matches!((self, other), (Self::CallReturnValue(la, lf, ..), Self::CallReturnValue(ra, rf, ..)) if la == ra && lf == rf)
	}

	pub fn to_mutable_score_value(self) -> anyhow::Result<MutableScoreValue> {
		match self {
			Self::Register(reg) => Ok(MutableScoreValue::Reg(reg)),
			Self::Score(score) => Ok(MutableScoreValue::Score(score)),
			Self::Arg(arg) => Ok(MutableScoreValue::Arg(arg)),
			Self::CallArg(arg, func, ty @ DataType::Score(..)) => {
				Ok(MutableScoreValue::CallArg(arg, func, ty))
			}
			Self::ReturnValue(ret) => Ok(MutableScoreValue::ReturnValue(ret)),
			Self::CallReturnValue(ret, func, ty @ DataType::Score(..)) => {
				Ok(MutableScoreValue::CallReturnValue(ret, func, ty))
			}
			_ => bail!("Value cannot be converted to a score value"),
		}
	}

	pub fn to_mutable_nbt_value(self) -> anyhow::Result<MutableNBTValue> {
		match self {
			Self::Register(reg) => Ok(MutableNBTValue::Reg(reg)),
			Self::Data(data) => Ok(MutableNBTValue::Data(data)),
			Self::Property(val, prop) => Ok(MutableNBTValue::Property(
				Box::new(val.to_mutable_nbt_value()?),
				prop,
			)),
			Self::Index(val, idx) => Ok(MutableNBTValue::Index(
				Box::new(val.to_mutable_nbt_value()?),
				idx,
			)),
			Self::Arg(arg) => Ok(MutableNBTValue::Arg(arg)),
			Self::CallArg(arg, func, ty @ DataType::NBT(..)) => {
				Ok(MutableNBTValue::CallArg(arg, func, ty))
			}
			Self::ReturnValue(ret) => Ok(MutableNBTValue::ReturnValue(ret)),
			Self::CallReturnValue(ret, func, ty @ DataType::NBT(..)) => {
				Ok(MutableNBTValue::CallReturnValue(ret, func, ty))
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
			Self::Property(val, prop) => format!("{val:?}.{prop}"),
			Self::Index(val, idx) => format!("{val:?}[{idx}]"),
			Self::Arg(num) => format!("&{num}"),
			Self::CallArg(idx, ..) => format!("&&{idx}"),
			Self::ReturnValue(idx) => format!("*{idx}"),
			Self::CallReturnValue(idx, ..) => format!("**{idx}"),
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
	ReturnValue(u16),
	CallReturnValue(u16, ResourceLocation, DataType),
}

impl MutableScoreValue {
	pub fn get_used_regs(&self) -> Vec<&Identifier> {
		match self {
			Self::Score(..) => Vec::new(),
			Self::Reg(reg) => vec![reg],
			Self::Arg(..)
			| Self::CallArg(..)
			| Self::ReturnValue(..)
			| Self::CallReturnValue(..) => Vec::new(),
		}
	}

	pub fn get_used_regs_mut(&mut self) -> Vec<&mut Identifier> {
		match self {
			Self::Score(..) => Vec::new(),
			Self::Reg(reg) => vec![reg],
			Self::Arg(..)
			| Self::CallArg(..)
			| Self::ReturnValue(..)
			| Self::CallReturnValue(..) => Vec::new(),
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
			Self::ReturnValue(idx) => format!("*{idx}"),
			Self::CallReturnValue(idx, ..) => format!("**{idx}"),
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
	Property(Box<MutableNBTValue>, String),
	Index(Box<MutableNBTValue>, ArraySize),
	Arg(u16),
	CallArg(u16, ResourceLocation, DataType),
	ReturnValue(u16),
	CallReturnValue(u16, ResourceLocation, DataType),
}

impl MutableNBTValue {
	pub fn get_used_regs(&self) -> Vec<&Identifier> {
		match self {
			Self::Data(..) => Vec::new(),
			Self::Property(val, ..) | Self::Index(val, ..) => val.get_used_regs(),
			Self::Reg(reg) => vec![reg],
			Self::Arg(..)
			| Self::CallArg(..)
			| Self::ReturnValue(..)
			| Self::CallReturnValue(..) => Vec::new(),
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
			Self::Property(val, prop) => format!("{val:?}.{prop}"),
			Self::Index(val, idx) => format!("{val:?}[{idx}]"),
			Self::Arg(idx) => format!("&{idx}"),
			Self::CallArg(idx, ..) => format!("&&{idx}"),
			Self::ReturnValue(idx) => format!("*{idx}"),
			Self::CallReturnValue(idx, ..) => format!("**{idx}"),
		};
		write!(f, "{text}")
	}
}
