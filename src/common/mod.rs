pub mod block;
pub mod condition;
pub mod function;
pub mod mc;
pub mod op;
pub mod range;
pub mod reg;
pub mod ty;
pub mod val;

use std::{fmt::Debug, sync::Arc};

use rustc_hash::FxHashMap;

use self::block::BlockAllocator;
use self::condition::Condition;
use self::function::{Function, FunctionSignature};
use self::reg::GetUsedRegs;
use self::ty::DataType;
use self::val::{MutableValue, Value};

#[derive(Clone, PartialEq)]
pub enum DeclareBinding {
	Null,
	Value(Value),
	Cast(DataType, MutableValue),
	Index {
		ty: DataType,
		val: Value,
		index: Value,
	},
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
			Self::Index { ty, .. } => Some(ty.clone()),
			Self::Condition(..) => Some(DataType::Score(ty::ScoreType::Bool)),
		};

		Ok(out)
	}
}

impl GetUsedRegs for DeclareBinding {
	fn append_used_regs<'a>(&'a self, regs: &mut Vec<&'a Identifier>) {
		match self {
			Self::Null => {}
			Self::Value(val) => val.append_used_regs(regs),
			Self::Cast(_, val) => val.append_used_regs(regs),
			Self::Index { val, index, .. } => {
				val.append_used_regs(regs);
				index.append_used_regs(regs);
			}
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
			Self::Index { val, index, ty } => format!("idx {ty:?} {val:?} {index:?}"),
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

/// Trait used to implement helper functions for the different
/// stages of IR we use
pub trait IRType {
	type BlockType;
	type InstrType;
	type InstrKindType;

	// Interface functions for trait methods
	fn get_fns<'this>(&'this self) -> &'this FxHashMap<ResourceLocation, Function>;
	fn get_fns_mut<'this>(&'this mut self) -> &'this mut FxHashMap<ResourceLocation, Function>;
	fn get_blocks<'this>(&'this self) -> &'this BlockAllocator<Self::BlockType>;
	fn get_blocks_mut<'this>(&'this mut self) -> &'this mut BlockAllocator<Self::BlockType>;

	// Iteration methods
	fn iter_fns(&self) -> std::collections::hash_map::Iter<ResourceLocation, Function> {
		self.get_fns().iter()
	}

	fn iter_fns_mut(&mut self) -> std::collections::hash_map::IterMut<ResourceLocation, Function> {
		self.get_fns_mut().iter_mut()
	}
}
