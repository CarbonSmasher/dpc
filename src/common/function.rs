use super::reg::GetUsedRegs;
use super::val::MutableValue;
use super::{ty::DataType, ResourceLocation};
use super::{Identifier, Value};
use std::fmt::Debug;
use std::hash::Hash;

#[derive(Clone)]
pub struct FunctionInterface {
	pub id: ResourceLocation,
	pub sig: FunctionSignature,
	pub annotations: FunctionAnnotations,
}

impl FunctionInterface {
	pub fn new(id: ResourceLocation) -> Self {
		Self::with_signature(id, FunctionSignature::new())
	}

	pub fn with_signature(id: ResourceLocation, sig: FunctionSignature) -> Self {
		Self::with_all(id, sig, FunctionAnnotations::new())
	}

	pub fn with_all(
		id: ResourceLocation,
		sig: FunctionSignature,
		annotations: FunctionAnnotations,
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

impl Default for FunctionInterface {
	fn default() -> Self {
		Self::new("".into())
	}
}

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

impl Default for FunctionSignature {
	fn default() -> Self {
		Self::new()
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
	Standard(Vec<DataType>),
}

impl Debug for ReturnType {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Void => write!(f, "void"),
			Self::Standard(ty) => ty.fmt(f),
		}
	}
}

#[derive(Clone, PartialEq)]
pub struct CallInterface {
	pub function: ResourceLocation,
	pub args: FunctionArgs,
	pub ret: Vec<MutableValue>,
}

impl CallInterface {
	pub fn iter_used_regs_mut(&mut self) -> impl Iterator<Item = &mut Identifier> {
		let args = self
			.args
			.iter_mut()
			.flat_map(|x| x.get_used_regs_mut().into_iter());
		let ret = self
			.ret
			.iter_mut()
			.flat_map(|x| x.get_used_regs_mut().into_iter());
		args.chain(ret)
	}
}

impl GetUsedRegs for CallInterface {
	fn append_used_regs<'a>(&'a self, regs: &mut Vec<&'a Identifier>) {
		for arg in &self.args {
			arg.append_used_regs(regs);
		}
		for ret in &self.ret {
			ret.append_used_regs(regs);
		}
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
		if !self.ret.is_empty() {
			write!(f, " -> ")?;
		}
		for (i, ret) in self.ret.iter().enumerate() {
			ret.fmt(f)?;
			if i != self.ret.len() - 1 {
				write!(f, ",")?;
			}
		}

		Ok(())
	}
}

#[derive(Debug, Clone)]
pub struct FunctionAnnotations {
	pub preserve: bool,
	pub no_inline: bool,
	pub no_strip: bool,
}

impl FunctionAnnotations {
	pub fn new() -> Self {
		Self {
			preserve: false,
			no_inline: false,
			no_strip: false,
		}
	}
}

impl Default for FunctionAnnotations {
	fn default() -> Self {
		Self::new()
	}
}
