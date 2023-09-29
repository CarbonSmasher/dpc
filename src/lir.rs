use std::{collections::HashMap, fmt::Debug};

use crate::common::{FunctionInterface, MutableValue, Value};

#[derive(Debug, Clone)]
pub struct LIR {
	pub functions: HashMap<FunctionInterface, LIRBlock>,
}

impl LIR {
	pub fn new() -> Self {
		Self {
			functions: HashMap::new(),
		}
	}
}

#[derive(Debug, Clone)]
pub struct LIRBlock {
	pub contents: Vec<LIRInstruction>,
}

impl LIRBlock {
	pub fn new() -> Self {
		Self {
			contents: Vec::new(),
		}
	}
}

#[derive(Clone)]
pub struct LIRInstruction {
	pub kind: LIRInstrKind,
}

impl LIRInstruction {
	pub fn new(kind: LIRInstrKind) -> Self {
		Self { kind }
	}
}

impl Debug for LIRInstruction {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		self.kind.fmt(f)
	}
}

#[derive(Clone)]
pub enum LIRInstrKind {
	SetScore(MutableValue, Value),
	AddScore(MutableValue, Value),
	SubScore(MutableValue, Value),
	MulScore(MutableValue, Value),
	DivScore(MutableValue, Value),
	ModScore(MutableValue, Value),
	MinScore(MutableValue, Value),
	MaxScore(MutableValue, Value),
	SwapScore(MutableValue, MutableValue),
}

impl Debug for LIRInstrKind {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let text = match self {
			Self::SetScore(left, right) => format!("sets {left:?} {right:?}"),
			Self::AddScore(left, right) => format!("adds {left:?} {right:?}"),
			Self::SubScore(left, right) => format!("subs {left:?} {right:?}"),
			Self::MulScore(left, right) => format!("muls {left:?} {right:?}"),
			Self::DivScore(left, right) => format!("divs {left:?} {right:?}"),
			Self::ModScore(left, right) => format!("mods {left:?} {right:?}"),
			Self::MinScore(left, right) => format!("mins {left:?} {right:?}"),
			Self::MaxScore(left, right) => format!("maxs {left:?} {right:?}"),
			Self::SwapScore(left, right) => format!("swps {left:?} {right:?}"),
		};
		write!(f, "{text}")
	}
}
