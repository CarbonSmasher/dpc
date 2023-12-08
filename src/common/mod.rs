pub mod block;
pub mod condition;
pub mod function;
pub mod mc;
pub mod range;
pub mod ty;
pub mod val;

use std::{fmt::Debug, sync::Arc};

use dashmap::DashMap;

use self::ty::DataType;
use self::val::{MutableValue, Value};

#[derive(Clone, PartialEq, Eq)]
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
