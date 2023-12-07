use super::{val::Value, Identifier};
use std::fmt::Debug;

/// Condition for if and other IR instructions
#[derive(Clone)]
pub enum Condition {
	// TODO: More conditions
	Equal(Value, Value),
	Exists(Value),
}

impl Condition {
	pub fn get_used_regs(&self) -> Vec<&Identifier> {
		match self {
			Self::Equal(l, r) => [l.get_used_regs(), r.get_used_regs()].concat(),
			Self::Exists(val) => val.get_used_regs(),
		}
	}
}

impl Debug for Condition {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Equal(l, r) => write!(f, "{l:?} == {r:?}"),
			Self::Exists(val) => write!(f, "exists({val:?})"),
		}
	}
}
