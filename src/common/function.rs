use super::{ty::DataType, ResourceLocation};
use super::{Identifier, Value};
use std::fmt::Debug;
use std::hash::Hash;

#[derive(Clone)]
pub struct FunctionInterface {
	pub id: ResourceLocation,
	pub sig: FunctionSignature,
	pub annotations: Vec<FunctionAnnotation>,
}

impl FunctionInterface {
	pub fn new(id: ResourceLocation) -> Self {
		Self::with_signature(id, FunctionSignature::new())
	}

	pub fn with_signature(id: ResourceLocation, sig: FunctionSignature) -> Self {
		Self::with_all(id, sig, Vec::new())
	}

	pub fn with_all(
		id: ResourceLocation,
		sig: FunctionSignature,
		annotations: Vec<FunctionAnnotation>,
	) -> Self {
		Self {
			id,
			sig,
			annotations,
		}
	}
}

impl Debug for FunctionInterface {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}{:?}", self.id, self.sig)
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

impl Eq for FunctionInterface {}

pub type FunctionParams = Vec<DataType>;
pub type FunctionArgs = Vec<Value>;

#[derive(Clone, PartialEq, Eq)]
pub struct FunctionSignature {
	pub params: FunctionParams,
	pub ret: ReturnType,
}

impl FunctionSignature {
	pub fn new() -> Self {
		Self::with_all(FunctionParams::new(), ReturnType::Void)
	}

	pub fn with_params(params: FunctionParams) -> Self {
		Self::with_all(params, ReturnType::Void)
	}

	pub fn with_ret(ret: ReturnType) -> Self {
		Self::with_all(FunctionParams::new(), ret)
	}

	pub fn with_all(params: FunctionParams, ret: ReturnType) -> Self {
		Self { params, ret }
	}
}

impl Debug for FunctionSignature {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "(")?;
		for (i, param) in self.params.iter().enumerate() {
			param.fmt(f)?;
			if i != self.params.len() - 1 {
				write!(f, ",")?;
			}
		}
		write!(f, "): {:?}", self.ret)?;

		Ok(())
	}
}

#[derive(Clone, PartialEq, Eq)]
pub enum ReturnType {
	Void,
	Standard(DataType),
}

impl Debug for ReturnType {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Void => write!(f, "void"),
			Self::Standard(ty) => ty.fmt(f),
		}
	}
}

#[derive(Clone)]
pub struct CallInterface {
	pub function: ResourceLocation,
	pub args: FunctionArgs,
}

impl CallInterface {
	pub fn get_used_regs(&self) -> Vec<&Identifier> {
		let mut out = Vec::new();
		for arg in &self.args {
			out.extend(arg.get_used_regs())
		}
		out
	}

	pub fn iter_used_regs_mut(&mut self) -> impl Iterator<Item = &mut Identifier> {
		self.args
			.iter_mut()
			.map(|x| x.get_used_regs_mut().into_iter())
			.flatten()
	}
}

impl Debug for CallInterface {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}(", self.function)?;
		for (i, arg) in self.args.iter().enumerate() {
			arg.fmt(f)?;
			if i != self.args.len() - 1 {
				write!(f, ",")?;
			}
		}
		write!(f, ")")?;

		Ok(())
	}
}

#[derive(Debug, Clone)]
pub enum FunctionAnnotation {
	NoDiscard,
}