use std::{collections::HashMap, fmt::Debug};

use crate::common::block::{Block as BlockTrait, BlockAllocator, BlockID};
use crate::common::mc::{EntityTarget, XPValue};
use crate::common::ty::DataType;
use crate::common::{DeclareBinding, FunctionInterface, Identifier, MutableValue, Value};

#[derive(Debug, Clone)]
pub struct IR {
	pub functions: HashMap<FunctionInterface, BlockID>,
	pub blocks: BlockAllocator<Block>,
}

impl IR {
	pub fn new() -> Self {
		Self {
			functions: HashMap::new(),
			blocks: BlockAllocator::new(),
		}
	}
}

#[derive(Clone)]
pub struct Block {
	pub contents: Vec<Instruction>,
}

impl Block {
	pub fn new() -> Self {
		Self {
			contents: Vec::new(),
		}
	}
}

impl BlockTrait for Block {
	fn instr_count(&self) -> usize {
		self.contents.len()
	}

	fn get_children(&self) -> Vec<BlockID> {
		Vec::new()
	}
}

impl Debug for Block {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		self.contents.fmt(f)
	}
}

#[derive(Clone)]
pub struct Instruction {
	pub kind: InstrKind,
}

impl Instruction {
	pub fn new(kind: InstrKind) -> Self {
		Self { kind }
	}
}

impl Debug for Instruction {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		self.kind.fmt(f)
	}
}

#[derive(Clone)]
pub enum InstrKind {
	Declare {
		left: Identifier,
		ty: DataType,
		right: DeclareBinding,
	},
	Assign {
		left: MutableValue,
		right: Value,
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
	Pow {
		base: MutableValue,
		exp: u8,
	},
	Use {
		val: MutableValue,
	},
	Say {
		message: String,
	},
	Tell {
		target: EntityTarget,
		message: String,
	},
	Kill {
		target: EntityTarget,
	},
	Reload,
	SetXP {
		target: EntityTarget,
		amount: i32,
		value: XPValue,
	},
}

impl Debug for InstrKind {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let text = match self {
			Self::Declare { left, ty, right } => format!("let {left}: {ty:?} = {right:?}"),
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
			Self::Pow { base, exp } => format!("pow {base:?}, {exp}"),
			Self::Use { val } => format!("use {val:?}"),
			Self::Say { message } => format!("say {message}"),
			Self::Tell { target, message } => format!("tell {target:?} {message}"),
			Self::Kill { target } => format!("kill {target:?}"),
			Self::Reload => "reload".into(),
			Self::SetXP {
				target,
				amount,
				value,
			} => format!("xps {target:?} {amount} {value}"),
		};
		write!(f, "{text}")
	}
}
