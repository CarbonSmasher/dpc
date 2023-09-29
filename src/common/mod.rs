pub mod ty;

use std::{collections::HashMap, hash::Hash, sync::Arc};

use anyhow::Context;

use self::ty::{DataType, DataTypeContents};

#[derive(Debug, Clone)]
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
}

#[derive(Debug, Clone)]
pub enum MutableValue {
	Register(Identifier),
}

impl MutableValue {
	pub fn get_ty(&self, regs: &RegisterList) -> anyhow::Result<DataType> {
		let out = match self {
			Self::Register(id) => {
				let reg = regs.get(id).context("Failed to get register")?;
				reg.ty
			}
		};

		Ok(out)
	}

	pub fn get_used_regs(&self) -> Vec<&Identifier> {
		match self {
			Self::Register(reg) => vec![&reg],
		}
	}
}


#[derive(Debug, Clone)]
pub enum DeclareBinding {
	Value(Value),
	Cast(DataType, MutableValue),
}

impl DeclareBinding {
	pub fn get_ty(&self, regs: &RegisterList) -> anyhow::Result<DataType> {
		let out = match self {
			Self::Value(val) => val.get_ty(regs)?,
			Self::Cast(ty, ..) => *ty,
		};

		Ok(out)
	}
}

pub type Identifier = Arc<str>;

#[derive(Debug, Clone)]
pub struct Register {
	pub id: Identifier,
	pub ty: DataType,
}

pub type RegisterList = HashMap<Identifier, Register>;

pub type ResourceLocation = Identifier;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReturnType {
	Void,
	Standard(DataType),
}
