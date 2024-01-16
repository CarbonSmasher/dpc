use rustc_hash::FxHashSet;

use crate::common::mc::modifier::Modifier;
use crate::common::val::{MutableScoreValue, ScoreValue};
use crate::lir::LIRInstrKind;
use crate::util::GetSetOwned;

pub mod merge;
pub mod null;
pub mod simplify;

/// The aspects of the game that a modifier can modify. Can be used
/// to model modifier and command dependencies and relationships
/// to perform certain optimizations
#[derive(Eq, PartialEq, Hash, Clone, Debug)]
pub enum ModifierContext {
	/// Every context. Used as a fallback for unknown context
	Everything,
	/// Who is running the command
	Executor,
	/// Position in the world
	Position,
}

/// Newtype thing for some traits
#[derive(Eq, PartialEq, Hash, Clone, Debug)]
pub struct Dependency(pub ModifierContext);

impl Dependency {
	pub fn to_modified(self) -> Modified {
		Modified(self.0)
	}
}

/// Newtype thing for some traits
#[derive(Eq, PartialEq, Hash, Clone, Debug)]
pub struct Modified(pub ModifierContext);

impl Modified {
	pub fn to_dep(self) -> Dependency {
		Dependency(self.0)
	}
}

impl GetSetOwned<Dependency> for Modifier {
	fn append_set(&self, set: &mut FxHashSet<Dependency>) {
		match self {
			Modifier::As(tgt) => {
				set.insert(Dependency(ModifierContext::Executor));
				if tgt.relies_on_position() {
					set.insert(Dependency(ModifierContext::Position));
				}
			}
			Modifier::At(..) => {
				set.insert(Dependency(ModifierContext::Position));
			}
			_ => {
				set.insert(Dependency(ModifierContext::Everything));
			}
		}
	}
}

impl GetSetOwned<Modified> for Modifier {
	fn append_set(&self, set: &mut FxHashSet<Modified>) {
		match self {
			Modifier::As(..) => {
				set.insert(Modified(ModifierContext::Executor));
			}
			Modifier::At(..) => {
				set.insert(Modified(ModifierContext::Position));
			}
			_ => {
				set.insert(Modified(ModifierContext::Everything));
			}
		}
	}
}

impl GetSetOwned<Dependency> for LIRInstrKind {
	fn append_set(&self, set: &mut FxHashSet<Dependency>) {
		match self {
			LIRInstrKind::AddScore(l, r)
			| LIRInstrKind::SubScore(l, r)
			| LIRInstrKind::MulScore(l, r)
			| LIRInstrKind::DivScore(l, r)
			| LIRInstrKind::ModScore(l, r)
			| LIRInstrKind::MinScore(l, r)
			| LIRInstrKind::MaxScore(l, r) => {
				l.append_set(set);
				r.append_set(set);
			}
			_ => {
				set.insert(Dependency(ModifierContext::Everything));
			}
		}
	}
}

impl GetSetOwned<Dependency> for ScoreValue {
	fn append_set(&self, set: &mut FxHashSet<Dependency>) {
		match self {
			ScoreValue::Constant(..) => {}
			ScoreValue::Mutable(val) => val.append_set(set),
		}
	}
}

impl GetSetOwned<Dependency> for MutableScoreValue {
	fn append_set(&self, set: &mut FxHashSet<Dependency>) {
		match self {
			MutableScoreValue::Arg(..)
			| MutableScoreValue::Reg(..)
			| MutableScoreValue::CallArg(..)
			| MutableScoreValue::CallReturnValue(..)
			| MutableScoreValue::ReturnValue(..) => {}
			_ => {
				set.insert(Dependency(ModifierContext::Everything));
			}
		}
	}
}
