pub mod block;
pub mod condition;
pub mod cost;
pub mod function;
pub mod mc;
pub mod op;
pub mod range;
pub mod reg;
pub mod ty;
pub mod val;

use std::{fmt::Debug, sync::Arc};

use rustc_hash::FxHashMap;

use self::block::Block;
use self::condition::Condition;
use self::function::FunctionSignature;
use self::reg::GetUsedRegs;
use self::ty::DataType;
use self::val::{MutableValue, Value};

#[derive(Clone, PartialEq)]
pub enum DeclareBinding {
	Null,
	Value(Value),
	Cast(DataType, MutableValue),
	Condition(Condition),
}

impl DeclareBinding {
	pub fn get_ty(
		&self,
		regs: &RegisterList,
		sig: &FunctionSignature,
	) -> anyhow::Result<Option<DataType>> {
		let out = match self {
			Self::Null => None,
			Self::Value(val) => Some(val.get_ty(regs, sig)?),
			Self::Cast(ty, ..) => Some(ty.clone()),
			Self::Condition(..) => Some(DataType::Score(ty::ScoreType::Bool)),
		};

		Ok(out)
	}

	pub fn has_side_effects(&self) -> bool {
		match self {
			Self::Condition(cond) => cond.has_side_effects(),
			_ => false,
		}
	}
}

impl GetUsedRegs for DeclareBinding {
	fn append_used_regs<'a>(&'a self, regs: &mut Vec<&'a Identifier>) {
		match self {
			Self::Null => {}
			Self::Value(val) => val.append_used_regs(regs),
			Self::Cast(_, val) => val.append_used_regs(regs),
			Self::Condition(cond) => cond.append_used_regs(regs),
		}
	}
}

impl Debug for DeclareBinding {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let text = match self {
			Self::Null => "null".to_string(),
			Self::Value(val) => format!("{val:?}"),
			Self::Cast(ty, val) => format!("cast {ty:?} {val:?}"),
			Self::Condition(cond) => format!("cond {cond:?}"),
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

pub type RegisterList = FxHashMap<Identifier, Register>;

pub type ResourceLocation = Identifier;
pub type ResourceLocationTag = Identifier;

pub trait FunctionTrait {
	type BlockType: Block;

	fn block(&self) -> &Self::BlockType;
	fn block_mut(&mut self) -> &mut Self::BlockType;
}

/// Trait used to implement helper functions for the different
/// stages of IR we use
pub trait IRType {
	type FunctionType: FunctionTrait;

	// Interface functions for trait methods
	fn get_fns<'this>(&'this self) -> &'this FxHashMap<ResourceLocation, Self::FunctionType>;
	fn get_fns_mut<'this>(
		&'this mut self,
	) -> &'this mut FxHashMap<ResourceLocation, Self::FunctionType>;

	// Iteration methods
	fn iter_fns(&self) -> std::collections::hash_map::Iter<ResourceLocation, Self::FunctionType> {
		self.get_fns().iter()
	}

	fn iter_fns_mut(
		&mut self,
	) -> std::collections::hash_map::IterMut<ResourceLocation, Self::FunctionType> {
		self.get_fns_mut().iter_mut()
	}

	fn instr_count(&self) -> usize {
		self.iter_fns()
			.fold(0, |sum, x| sum + x.1.block().instr_count())
	}
}
