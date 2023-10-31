use std::{collections::HashMap, fmt::Debug};

use crate::common::block::{Block, BlockAllocator, BlockID};
use crate::common::function::{CallInterface, FunctionInterface, FunctionSignature};
use crate::common::mc::{EntityTarget, XPValue};
use crate::common::ty::DataType;
use crate::common::{DeclareBinding, Identifier, MutableValue, ResourceLocation, Value};

#[derive(Debug, Clone)]
pub struct MIR {
	pub functions: HashMap<FunctionInterface, BlockID>,
	pub blocks: BlockAllocator<MIRBlock>,
}

impl MIR {
	pub fn new() -> Self {
		Self {
			functions: HashMap::new(),
			blocks: BlockAllocator::new(),
		}
	}

	pub fn with_capacity(function_capacity: usize, block_capacity: usize) -> Self {
		Self {
			functions: HashMap::with_capacity(function_capacity),
			blocks: BlockAllocator::with_capacity(block_capacity),
		}
	}

	/// Get the block of a function with an ID
	pub fn get_fn(&self, id: &ResourceLocation) -> Option<&BlockID> {
		self.functions.get(&FunctionInterface {
			id: id.clone(),
			sig: FunctionSignature::new(),
			annotations: Vec::new(),
		})
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

	pub fn with_capacity(capacity: usize) -> Self {
		Self {
			contents: Vec::with_capacity(capacity),
		}
	}
}

impl Block for MIRBlock {
	fn instr_count(&self) -> usize {
		self.contents.len()
	}

	fn get_children(&self) -> Vec<BlockID> {
		Vec::new()
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
	Pow {
		base: MutableValue,
		exp: u8,
	},
	Get {
		value: MutableValue,
	},
	Merge {
		left: MutableValue,
		right: Value,
	},
	Push {
		left: MutableValue,
		right: Value,
	},
	PushFront {
		left: MutableValue,
		right: Value,
	},
	Insert {
		left: MutableValue,
		right: Value,
		index: i32,
	},
	Use {
		val: MutableValue,
	},
	Call {
		call: CallInterface,
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
			Self::Pow { base, exp } => format!("pow {base:?}, {exp}"),
			Self::Get { value } => format!("get {value:?}"),
			Self::Merge { left, right } => format!("merge {left:?}, {right:?}"),
			Self::Push { left, right } => format!("push {left:?}, {right:?}"),
			Self::PushFront { left, right } => format!("pushf {left:?}, {right:?}"),
			Self::Insert { left, right, index } => format!("ins {left:?}, {right:?}, {index}"),
			Self::Use { val } => format!("use {val:?}"),
			Self::Call { call } => format!("call {call:?}"),
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

impl MIRInstrKind {
	pub fn get_used_regs(&self) -> Vec<&Identifier> {
		match self {
			Self::Assign { left, right } => [left.get_used_regs(), right.get_used_regs()].concat(),
			Self::Add { left, right }
			| Self::Sub { left, right }
			| Self::Mul { left, right }
			| Self::Div { left, right }
			| Self::Mod { left, right }
			| Self::Min { left, right }
			| Self::Max { left, right }
			| Self::Merge { left, right }
			| Self::Push { left, right }
			| Self::PushFront { left, right }
			| Self::Insert { left, right, .. } => [left.get_used_regs(), right.get_used_regs()].concat(),
			Self::Swap { left, right } => [left.get_used_regs(), right.get_used_regs()].concat(),
			Self::Abs { val } => val.get_used_regs(),
			Self::Pow { base, .. } => base.get_used_regs(),
			Self::Get { value } => value.get_used_regs(),
			Self::Use { val } => val.get_used_regs(),
			Self::Call { call } => call.get_used_regs(),
			_ => Vec::new(),
		}
	}

	pub fn replace_regs<F: Fn(&mut Identifier) -> ()>(&mut self, f: F) {
		match self {
			Self::Declare { left, .. } => {
				f(left);
			}
			Self::Assign { left, right } => {
				let right_regs: Box<dyn Iterator<Item = _>> = match right {
					DeclareBinding::Null => Box::new(std::iter::empty()),
					DeclareBinding::Value(val) => Box::new(val.get_used_regs_mut().into_iter()),
					DeclareBinding::Cast(_, val) => Box::new(val.get_used_regs_mut().into_iter()),
					DeclareBinding::Index { val, index, .. } => Box::new(
						val.get_used_regs_mut()
							.into_iter()
							.chain(index.get_used_regs_mut()),
					),
				};
				for reg in left.get_used_regs_mut().into_iter().chain(right_regs) {
					f(reg);
				}
			}
			Self::Add { left, right }
			| Self::Sub { left, right }
			| Self::Mul { left, right }
			| Self::Div { left, right }
			| Self::Mod { left, right }
			| Self::Min { left, right }
			| Self::Max { left, right }
			| Self::Merge { left, right }
			| Self::Push { left, right }
			| Self::PushFront { left, right }
			| Self::Insert { left, right, .. } => {
				for reg in left
					.get_used_regs_mut()
					.into_iter()
					.chain(right.get_used_regs_mut())
				{
					f(reg);
				}
			}
			Self::Swap { left, right } => {
				for reg in left
					.get_used_regs_mut()
					.into_iter()
					.chain(right.get_used_regs_mut())
				{
					f(reg);
				}
			}
			Self::Abs { val }
			| Self::Pow { base: val, .. }
			| Self::Get { value: val }
			| Self::Use { val } => {
				for reg in val.get_used_regs_mut() {
					f(reg);
				}
			}
			Self::Call { call } => {
				for reg in call.iter_used_regs_mut() {
					f(reg);
				}
			}
			_ => {}
		}
	}
}
