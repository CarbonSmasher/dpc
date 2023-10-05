pub mod block;
pub mod target_selector;
pub mod mc;
pub mod modifier;
pub mod ty;

use std::{fmt::Debug, hash::Hash, sync::Arc};

use anyhow::{bail, Context};
use dashmap::DashMap;

use self::mc::Score;

use self::ty::{DataType, DataTypeContents, ScoreTypeContents};

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

	pub fn to_score_value(self) -> anyhow::Result<ScoreValue> {
		let out = match self {
			Self::Constant(val) => {
				if let DataTypeContents::Score(score) = val {
					ScoreValue::Constant(score)
				} else {
					bail!("Expected value to be a score");
				}
			}
			Self::Mutable(val) => ScoreValue::Mutable(val.to_mutable_score_value()),
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
		};

		Ok(out)
	}

	pub fn get_used_regs(&self) -> Vec<&Identifier> {
		match self {
			Self::Register(reg) => vec![&reg],
		}
	}

	pub fn is_same_val(&self, other: &Self) -> bool {
		matches!((self, other), (Self::Register(left), Self::Register(right)) if left == right)
	}

	pub fn to_mutable_score_value(self) -> MutableScoreValue {
		match self {
			Self::Register(reg) => MutableScoreValue::Reg(reg),
		}
	}
}

impl Debug for MutableValue {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let text = match self {
			Self::Register(reg) => format!("${reg}"),
		};
		write!(f, "{text}")
	}
}

#[derive(Clone)]
pub enum DeclareBinding {
	Value(Value),
	Cast(DataType, MutableValue),
	Index {
		ty: DataType,
		val: Value,
		index: Value,
	},
}

impl DeclareBinding {
	pub fn get_ty(&self, regs: &RegisterList) -> anyhow::Result<DataType> {
		let out = match self {
			Self::Value(val) => val.get_ty(regs)?,
			Self::Cast(ty, ..) => ty.clone(),
			Self::Index { ty, .. } => ty.clone(),
		};

		Ok(out)
	}

	pub fn get_used_regs(&self) -> Vec<&Identifier> {
		match self {
			Self::Value(val) => val.get_used_regs(),
			Self::Cast(_, val) => val.get_used_regs(),
			Self::Index { val, index, .. } => [val.get_used_regs(), index.get_used_regs()].concat(),
		}
	}
}

impl Debug for DeclareBinding {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let text = match self {
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

#[derive(Debug, Clone)]
pub enum ScoreValue {
	Constant(ScoreTypeContents),
	Mutable(MutableScoreValue),
}

impl ScoreValue {
	pub fn get_used_regs(&self) -> Vec<&Identifier> {
		match self {
			Self::Constant(..) => Vec::new(),
			ScoreValue::Mutable(val) => val.get_used_regs(),
		}
	}
}

#[derive(Debug, Clone)]
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
}

#[derive(Debug, Clone, Eq)]
pub struct FunctionInterface {
	pub id: ResourceLocation,
	pub sig: FunctionSignature,
}

impl FunctionInterface {
	pub fn new(id: ResourceLocation) -> Self {
		Self::with_signature(id, FunctionSignature::new())
	}

	pub fn with_signature(id: ResourceLocation, sig: FunctionSignature) -> Self {
		Self { id, sig }
	}
}

impl Hash for FunctionInterface {
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
		self.id.hash(state)
	}
}

impl PartialEq for FunctionInterface {
	fn eq(&self, other: &Self) -> bool {
		self.id.eq(&other.id)
	}
}

pub type FunctionArgs = Vec<DataType>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionSignature {
	pub args: FunctionArgs,
	pub ret: ReturnType,
}

impl FunctionSignature {
	pub fn new() -> Self {
		Self::with_all(FunctionArgs::new(), ReturnType::Void)
	}

	pub fn with_args(args: FunctionArgs) -> Self {
		Self::with_all(args, ReturnType::Void)
	}

	pub fn with_ret(ret: ReturnType) -> Self {
		Self::with_all(FunctionArgs::new(), ret)
	}

	pub fn with_all(args: FunctionArgs, ret: ReturnType) -> Self {
		Self { args, ret }
	}
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReturnType {
	Void,
	Standard(DataType),
}
