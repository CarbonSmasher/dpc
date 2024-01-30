use anyhow::{bail, Context};
use derivative::Derivative;

use super::{
	function::{FunctionSignature, ReturnType},
	ty::DataType,
	val::ArgRetIndex,
	Identifier, RegisterList, ResourceLocation,
};
use std::fmt::Debug;

pub trait GetUsedRegs {
	fn append_used_regs<'this>(&'this self, regs: &mut Vec<&'this Identifier>);

	fn get_used_regs(&self) -> Vec<&Identifier> {
		let mut out = Vec::new();
		self.append_used_regs(&mut out);
		out
	}
}

/// Local values, like regs and return values, for LIR to use
#[derive(Clone, PartialEq, Eq, Derivative)]
#[derivative(Hash)]
pub enum Local {
	Reg(Identifier),
	Arg(ArgRetIndex),
	CallArg(
		ArgRetIndex,
		ResourceLocation,
		#[derivative(Hash = "ignore")] DataType,
	),
	ReturnValue(ArgRetIndex),
	CallReturnValue(
		ArgRetIndex,
		ResourceLocation,
		#[derivative(Hash = "ignore")] DataType,
	),
}

impl Local {
	pub fn get_used_regs_mut(&mut self) -> Vec<&mut Identifier> {
		match self {
			Self::Reg(reg) => vec![reg],
			Self::Arg(..)
			| Self::CallArg(..)
			| Self::ReturnValue(..)
			| Self::CallReturnValue(..) => Vec::new(),
		}
	}

	pub fn get_ty(&self, regs: &RegisterList, sig: &FunctionSignature) -> anyhow::Result<DataType> {
		let out = match self {
			Self::Reg(id) => {
				let reg = regs
					.get(id)
					.with_context(|| format!("Failed to get register ${id}"))?;
				reg.ty.clone()
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
}

impl GetUsedRegs for Local {
	fn append_used_regs<'this>(&'this self, regs: &mut Vec<&'this Identifier>) {
		if let Self::Reg(reg) = self {
			regs.push(reg);
		}
	}
}

impl Debug for Local {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let text = match self {
			Self::Reg(reg) => format!("%{reg}"),
			Self::Arg(idx) => format!("&{idx}"),
			Self::CallArg(idx, ..) => format!("&&{idx}"),
			Self::ReturnValue(idx) => format!("*{idx}"),
			Self::CallReturnValue(idx, ..) => format!("**{idx}"),
		};
		write!(f, "{text}")
	}
}

pub trait GetUsedLocals {
	fn append_used_locals<'this>(&'this self, locals: &mut Vec<&'this Local>);

	fn get_used_locals(&self) -> Vec<&Local> {
		let mut out = Vec::new();
		self.append_used_locals(&mut out);
		out
	}
}
