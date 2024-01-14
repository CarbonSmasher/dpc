use super::mc::pos::IntCoordinates;
use super::mc::EntityTarget;
use super::reg::GetUsedRegs;
use super::val::{MutableValue, Value};
use super::{Identifier, ResourceLocation, ResourceLocationTag};
use std::fmt::Debug;
use std::iter;

/// Condition for if and other IR instructions
#[derive(Clone, PartialEq)]
pub enum Condition {
	Not(Box<Condition>),
	And(Box<Condition>, Box<Condition>),
	Equal(Value, Value),
	Exists(Value),
	GreaterThan(Value, Value),
	GreaterThanOrEqual(Value, Value),
	LessThan(Value, Value),
	LessThanOrEqual(Value, Value),
	Bool(Value),
	NotBool(Value),
	Entity(EntityTarget),
	Predicate(ResourceLocation),
	Biome(IntCoordinates, ResourceLocationTag),
	Loaded(IntCoordinates),
	Dimension(ResourceLocation),
}

impl Condition {
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
			Self::Exists(val) | Self::Bool(val) | Self::NotBool(val) => {
				Box::new(val.get_used_regs_mut().into_iter())
			}
			Self::Not(condition) => condition.iter_used_regs_mut(),
			Self::And(l, r) => Box::new(l.iter_used_regs_mut().chain(r.iter_used_regs_mut())),
			Self::Entity(..)
			| Self::Predicate(..)
			| Self::Biome(..)
			| Self::Loaded(..)
			| Self::Dimension(..) => Box::new(iter::empty()),
		}
	}

	pub fn iter_mut_vals(&mut self) -> Box<dyn Iterator<Item = &mut MutableValue> + '_> {
		match self {
			Self::Equal(l, r)
			| Self::GreaterThan(l, r)
			| Self::GreaterThanOrEqual(l, r)
			| Self::LessThan(l, r)
			| Self::LessThanOrEqual(l, r) => Box::new(l.iter_mut_val().into_iter().chain(r.iter_mut_val())),
			Self::Exists(val) | Self::Bool(val) | Self::NotBool(val) => {
				Box::new(val.iter_mut_val().into_iter())
			}
			Self::Not(condition) => condition.iter_mut_vals(),
			Self::And(l, r) => Box::new(l.iter_mut_vals().chain(r.iter_mut_vals())),
			Self::Entity(..)
			| Self::Predicate(..)
			| Self::Biome(..)
			| Self::Loaded(..)
			| Self::Dimension(..) => Box::new(iter::empty()),
		}
	}
}

impl GetUsedRegs for Condition {
	fn append_used_regs<'a>(&'a self, regs: &mut Vec<&'a Identifier>) {
		match self {
			Self::Equal(l, r)
			| Self::GreaterThan(l, r)
			| Self::GreaterThanOrEqual(l, r)
			| Self::LessThan(l, r)
			| Self::LessThanOrEqual(l, r) => {
				l.append_used_regs(regs);
				r.append_used_regs(regs);
			}
			Self::Exists(val) | Self::Bool(val) | Self::NotBool(val) => {
				val.append_used_regs(regs);
			}
			Self::Not(condition) => {
				condition.append_used_regs(regs);
			}
			Self::And(l, r) => {
				l.append_used_regs(regs);
				r.append_used_regs(regs);
			}
			Self::Entity(..)
			| Self::Predicate(..)
			| Self::Biome(..)
			| Self::Loaded(..)
			| Self::Dimension(..) => {}
		}
	}
}

impl Debug for Condition {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Equal(l, r) => write!(f, "{l:?} == {r:?}"),
			Self::Exists(val) => write!(f, "exi {val:?}"),
			Self::Not(condition) => write!(f, "not {condition:?}"),
			Self::And(l, r) => write!(f, "and {l:?} {r:?}"),
			Self::GreaterThan(l, r) => write!(f, "{l:?} > {r:?}"),
			Self::GreaterThanOrEqual(l, r) => write!(f, "{l:?} >= {r:?}"),
			Self::LessThan(l, r) => write!(f, "{l:?} < {r:?}"),
			Self::LessThanOrEqual(l, r) => write!(f, "{l:?} <= {r:?}"),
			Self::Bool(val) => write!(f, "bool {val:?}"),
			Self::NotBool(val) => write!(f, "nbool {val:?}"),
			Self::Entity(ent) => write!(f, "ent {ent:?}"),
			Self::Predicate(pred) => write!(f, "pred {pred:?}"),
			Self::Biome(loc, biome) => write!(f, "bio {loc:?} {biome}"),
			Self::Loaded(loc) => write!(f, "load {loc:?}"),
			Self::Dimension(dim) => write!(f, "dim {dim}"),
		}
	}
}
