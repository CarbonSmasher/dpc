use std::collections::HashMap;

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

#[derive(Debug, Clone)]
pub struct LIRInstruction {
	pub kind: LIRInstrKind,
}

impl LIRInstruction {
	pub fn new(kind: LIRInstrKind) -> Self {
		Self { kind }
	}
}

#[derive(Debug, Clone)]
pub enum LIRInstrKind {
	SetScore(MutableValue, Value),
	AddScore(MutableValue, Value),
	SubScore(MutableValue, Value),
	MulScore(MutableValue, Value),
	DivScore(MutableValue, Value),
	ModScore(MutableValue, Value),
	SwapScore(MutableValue, MutableValue),
}
