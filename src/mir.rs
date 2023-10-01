use std::{collections::HashMap, fmt::Debug};

use crate::common::{
	ty::DataType, DeclareBinding, FunctionInterface, Identifier, MutableValue, Value,
};

#[derive(Debug, Clone)]
pub struct MIR {
	pub functions: HashMap<FunctionInterface, MIRBlock>,
}

impl MIR {
	pub fn new() -> Self {
		Self {
			functions: HashMap::new(),
		}
	}
}

#[derive(Clone)]
pub struct MIRBlock {
	pub contents: Vec<MIRInstruction>,
}

impl MIRBlock {
	pub fn new() -> Self {
		Self {
			contents: Vec::new(),
		}
	}
}

impl Debug for MIRBlock {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		self.contents.fmt(f)
	}
}

#[derive(Clone)]
pub struct MIRInstruction {
	pub kind: MIRInstrKind,
}

impl MIRInstruction {
	pub fn new(kind: MIRInstrKind) -> Self {
		Self { kind }
	}
}

impl Debug for MIRInstruction {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		self.kind.fmt(f)
	}
}

#[derive(Clone)]
pub enum MIRInstrKind {
	Declare {
		left: Identifier,
		ty: DataType,
	},
	Assign {
		left: MutableValue,
		right: DeclareBinding,
	},
	Add {
		left: MutableValue,
		right: Value,
	},
	Sub {
		left: MutableValue,
		right: Value,
	},
	Mul {
		left: MutableValue,
		right: Value,
	},
	Div {
		left: MutableValue,
		right: Value,
	},
	Mod {
		left: MutableValue,
		right: Value,
	},
	Min {
		left: MutableValue,
		right: Value,
	},
	Max {
		left: MutableValue,
		right: Value,
	},
	Swap {
		left: MutableValue,
		right: MutableValue,
	},
	Abs {
		val: MutableValue,
	},
	Use {
		val: MutableValue,
	},
}

impl Debug for MIRInstrKind {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let text = match self {
			Self::Declare { left, ty } => format!("let {left}: {ty:?}"),
			Self::Assign { left, right } => format!("{left:?} = {right:?}"),
			Self::Add { left, right } => format!("add {left:?}, {right:?}"),
			Self::Sub { left, right } => format!("sub {left:?}, {right:?}"),
			Self::Mul { left, right } => format!("mul {left:?}, {right:?}"),
			Self::Div { left, right } => format!("div {left:?}, {right:?}"),
			Self::Mod { left, right } => format!("mod {left:?}, {right:?}"),
			Self::Min { left, right } => format!("min {left:?}, {right:?}"),
			Self::Max { left, right } => format!("max {left:?}, {right:?}"),
			Self::Swap { left, right } => format!("swp {left:?}, {right:?}"),
			Self::Abs { val } => format!("abs {val:?}"),
			Self::Use { val } => format!("use {val:?}"),
		};
		write!(f, "{text}")
	}
}

impl MIRInstrKind {
	pub fn get_used_regs(&self) -> Vec<&Identifier> {
		match self {
			Self::Declare { .. } => Vec::new(),
			Self::Assign { left, right } => [left.get_used_regs(), right.get_used_regs()].concat(),
			Self::Add { left, right }
			| Self::Sub { left, right }
			| Self::Mul { left, right }
			| Self::Div { left, right }
			| Self::Mod { left, right }
			| Self::Min { left, right }
			| Self::Max { left, right } => [left.get_used_regs(), right.get_used_regs()].concat(),
			Self::Swap { left, right } => [left.get_used_regs(), right.get_used_regs()].concat(),
			Self::Abs { val } => val.get_used_regs(),
			Self::Use { val } => val.get_used_regs(),
		}
	}
}
