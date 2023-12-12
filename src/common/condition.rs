use super::{val::Value, Identifier};
use std::fmt::Debug;

/// Condition for if and other IR instructions
#[derive(Clone, PartialEq)]
pub enum Condition {
	// TODO: More conditions
	Not(Box<Condition>),
	Equal(Value, Value),
	Exists(Value),
	GreaterThan(Value, Value),
	GreaterThanOrEqual(Value, Value),
	LessThan(Value, Value),
	LessThanOrEqual(Value, Value),
	Bool(Value),
}

impl Condition {
	pub fn get_used_regs(&self) -> Vec<&Identifier> {
		match self {
			Self::Equal(l, r)
			| Self::GreaterThan(l, r)
			| Self::GreaterThanOrEqual(l, r)
			| Self::LessThan(l, r)
			| Self::LessThanOrEqual(l, r) => [l.get_used_regs(), r.get_used_regs()].concat(),
			Self::Exists(val) | Self::Bool(val) => val.get_used_regs(),
			Self::Not(condition) => condition.get_used_regs(),
		}
	}

	pub fn iter_used_regs_mut(&mut self) -> Box<dyn Iterator<Item = &mut Identifier> + '_> {
		match self {
			Self::Equal(l, r)
			| Self::GreaterThan(l, r)
			| Self::GreaterThanOrEqual(l, r)
			| Self::LessThan(l, r)
			| Self::LessThanOrEqual(l, r) => Box::new(
				l.get_used_regs_mut()
					.into_iter()
					.chain(r.get_used_regs_mut()),
			),
			Self::Exists(val) | Self::Bool(val) => Box::new(val.get_used_regs_mut().into_iter()),
			Self::Not(condition) => condition.iter_used_regs_mut(),
		}
	}
}

impl Debug for Condition {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Equal(l, r) => write!(f, "{l:?} == {r:?}"),
			Self::Exists(val) => write!(f, "exists {val:?}"),
			Self::Not(condition) => write!(f, "not {condition:?}"),
			Self::GreaterThan(l, r) => write!(f, "{l:?} > {r:?}"),
			Self::GreaterThanOrEqual(l, r) => write!(f, "{l:?} >= {r:?}"),
			Self::LessThan(l, r) => write!(f, "{l:?} < {r:?}"),
			Self::LessThanOrEqual(l, r) => write!(f, "{l:?} <= {r:?}"),
			Self::Bool(val) => write!(f, "bool {val:?}"),
		}
	}
}
