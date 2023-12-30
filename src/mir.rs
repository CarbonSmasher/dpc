use std::iter;
use std::{collections::HashMap, fmt::Debug};

use crate::common::block::{Block, BlockAllocator, BlockID};
use crate::common::condition::Condition;
use crate::common::function::{CallInterface, Function};
use crate::common::mc::instr::MinecraftInstr;
use crate::common::mc::modifier::StoreModLocation;
use crate::common::mc::pos::DoubleCoordinates;
use crate::common::mc::EntityTarget;
use crate::common::ty::{DataType, Double};
use crate::common::IRType;
use crate::common::{val::MutableValue, val::Value, DeclareBinding, Identifier, ResourceLocation};

#[derive(Debug, Clone)]
pub struct MIR {
	pub functions: HashMap<ResourceLocation, Function>,
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
}

impl IRType for MIR {
	type BlockType = MIRBlock;
	type InstrType = MIRInstruction;
	type InstrKindType = MIRInstrKind;

	fn get_fns<'this>(&'this self) -> &'this HashMap<ResourceLocation, Function> {
		&self.functions
	}

	fn get_fns_mut<'this>(&'this mut self) -> &'this mut HashMap<ResourceLocation, Function> {
		&mut self.functions
	}

	fn get_blocks<'this>(&'this self) -> &'this BlockAllocator<Self::BlockType> {
		&self.blocks
	}

	fn get_blocks_mut<'this>(&'this mut self) -> &'this mut BlockAllocator<Self::BlockType> {
		&mut self.blocks
	}
}

impl Default for MIR {
	fn default() -> Self {
		Self::new()
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

impl Default for MIRBlock {
	fn default() -> Self {
		Self::new()
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
	Remove {
		val: MutableValue,
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
		scale: Double,
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
	CallExtern {
		func: ResourceLocation,
	},
	If {
		condition: Condition,
		body: Box<MIRInstrKind>,
	},
	// Game instructions
	MC(MinecraftInstr),
	ReturnValue {
		index: u16,
		value: Value,
	},
	Return {
		value: Value,
	},
	ReturnRun {
		body: Box<MIRInstrKind>,
	},
	NoOp,
	Command {
		command: String,
	},
	Comment {
		comment: String,
	},
	As {
		target: EntityTarget,
		body: Box<MIRInstrKind>,
	},
	At {
		target: EntityTarget,
		body: Box<MIRInstrKind>,
	},
	StoreResult {
		location: StoreModLocation,
		body: Box<MIRInstrKind>,
	},
	StoreSuccess {
		location: StoreModLocation,
		body: Box<MIRInstrKind>,
	},
	Positioned {
		position: DoubleCoordinates,
		body: Box<MIRInstrKind>,
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
			Self::Get { value, scale } => format!("get {value:?} {scale}"),
			Self::Merge { left, right } => format!("merge {left:?}, {right:?}"),
			Self::Push { left, right } => format!("push {left:?}, {right:?}"),
			Self::PushFront { left, right } => format!("pushf {left:?}, {right:?}"),
			Self::Insert { left, right, index } => format!("ins {left:?}, {right:?}, {index}"),
			Self::Use { val } => format!("use {val:?}"),
			Self::Call { call } => format!("call {call:?}"),
			Self::CallExtern { func } => format!("callx {func}"),
			Self::If { condition, body } => format!("if {condition:?} then {body:?}"),
			Self::Remove { val } => format!("rm {val:?}"),
			Self::ReturnValue { index, value } => format!("retv {index} {value:?}"),
			Self::Return { value } => format!("ret {value:?}"),
			Self::ReturnRun { body } => format!("retr {body:?}"),
			Self::NoOp => "noop".into(),
			Self::Command { command } => format!("cmd {command}"),
			Self::Comment { comment } => format!("cmt {comment}"),
			Self::As { target, body } => format!("as {target:?}: {body:?}"),
			Self::At { target, body } => format!("at {target:?}: {body:?}"),
			Self::StoreResult { location, body } => format!("str {location:?}: {body:?}"),
			Self::StoreSuccess { location, body } => format!("sts {location:?}: {body:?}"),
			Self::Positioned { position, body } => format!("pos {position:?}: {body:?}"),
			Self::MC(instr) => format!("{instr:?}"),
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
			Self::Get { value, .. } => value.get_used_regs(),
			Self::Use { val } => val.get_used_regs(),
			Self::Call { call } => call.get_used_regs(),
			Self::Remove { val } => val.get_used_regs(),
			Self::ReturnValue { value, .. } => value.get_used_regs(),
			Self::If { condition, body } => {
				[condition.get_used_regs(), body.get_used_regs()].concat()
			}
			Self::As { body, .. } | Self::At { body, .. } | Self::Positioned { body, .. } => {
				body.get_used_regs()
			}
			Self::StoreResult { location, body } | Self::StoreSuccess { location, body } => {
				[location.get_used_regs(), body.get_used_regs()].concat()
			}
			_ => Vec::new(),
		}
	}

	pub fn replace_regs<F: Fn(&mut Identifier)>(&mut self, f: F) {
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
			| Self::Get { value: val, .. }
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
			Self::If { condition, body } => {
				for reg in condition.iter_used_regs_mut() {
					f(reg);
				}
				body.replace_regs(f);
			}
			Self::As { body, .. } | Self::At { body, .. } | Self::Positioned { body, .. } => {
				body.replace_regs(f);
			}
			Self::StoreResult { location, body } | Self::StoreSuccess { location, body } => {
				location.replace_regs(&f);
				body.replace_regs(f);
			}
			Self::ReturnValue { value, .. } => {
				for reg in value.get_used_regs_mut() {
					f(reg);
				}
			}
			_ => {}
		}
	}

	pub fn replace_mut_vals<F: Fn(&mut MutableValue)>(&mut self, f: F) {
		match self {
			Self::Assign { left, right } => {
				let right_regs: Box<dyn Iterator<Item = _>> = match right {
					DeclareBinding::Null => Box::new(std::iter::empty()),
					DeclareBinding::Value(val) => Box::new(val.iter_mut_val().into_iter()),
					DeclareBinding::Cast(_, val) => Box::new(iter::once(val)),
					DeclareBinding::Index { val, index, .. } => {
						Box::new(val.iter_mut_val().into_iter().chain(index.iter_mut_val()))
					}
				};
				for reg in iter::once(left).chain(right_regs) {
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
				for reg in iter::once(left).chain(right.iter_mut_val()) {
					f(reg);
				}
			}
			Self::Swap { left, right } => {
				f(left);
				f(right);
			}
			Self::Abs { val }
			| Self::Pow { base: val, .. }
			| Self::Get { value: val, .. }
			| Self::Use { val }
			| Self::ReturnValue {
				value: Value::Mutable(val),
				..
			} => {
				f(val);
			}
			Self::Call { call } => {
				for val in &mut call.args {
					if let Value::Mutable(val) = val {
						f(val);
					}
				}
			}
			Self::If { condition, body } => {
				for val in condition.iter_mut_vals() {
					f(val);
				}
				body.replace_mut_vals(f);
			}
			Self::As { body, .. } | Self::At { body, .. } | Self::Positioned { body, .. } => {
				body.replace_mut_vals(f);
			}
			_ => {}
		}
	}
}
