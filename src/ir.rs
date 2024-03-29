use std::fmt::Debug;

use rustc_hash::FxHashMap;

use crate::common::block::Block as BlockTrait;
use crate::common::condition::Condition;
use crate::common::function::{CallInterface, FunctionInterface};
use crate::common::mc::instr::MinecraftInstr;
use crate::common::mc::modifier::MIRModifier;
use crate::common::ty::{DataType, Double};
use crate::common::val::ArgRetIndex;
use crate::common::{val::MutableValue, val::Value, DeclareBinding, Identifier, ResourceLocation};
use crate::common::{FunctionTrait, IRType};

#[derive(Debug, Clone)]
pub struct IR {
	pub functions: FxHashMap<ResourceLocation, IRFunction>,
}

impl IR {
	pub fn new() -> Self {
		Self {
			functions: FxHashMap::default(),
		}
	}
}

impl IRType for IR {
	type FunctionType = IRFunction;

	fn get_fns<'this>(&'this self) -> &'this FxHashMap<ResourceLocation, Self::FunctionType> {
		&self.functions
	}

	fn get_fns_mut<'this>(
		&'this mut self,
	) -> &'this mut FxHashMap<ResourceLocation, Self::FunctionType> {
		&mut self.functions
	}
}

impl Default for IR {
	fn default() -> Self {
		Self::new()
	}
}

#[derive(Debug, Clone)]
pub struct IRFunction {
	pub interface: FunctionInterface,
	pub block: Block,
}

impl FunctionTrait for IRFunction {
	type BlockType = Block;

	fn block(&self) -> &Self::BlockType {
		&self.block
	}

	fn block_mut(&mut self) -> &mut Self::BlockType {
		&mut self.block
	}
}

#[derive(Clone, PartialEq)]
pub struct Block {
	pub contents: Vec<Instruction>,
}

impl Block {
	pub fn new() -> Self {
		Self::with_contents(Vec::new())
	}

	pub fn with_contents(contents: Vec<Instruction>) -> Self {
		Self { contents }
	}

	pub fn from_single(instr: InstrKind) -> Self {
		Self::with_contents(vec![Instruction::new(instr)])
	}
}

impl Default for Block {
	fn default() -> Self {
		Self::new()
	}
}

impl BlockTrait for Block {
	type InstrType = Instruction;
	type InstrKindType = InstrKind;

	fn contents(&self) -> &Vec<Self::InstrType> {
		&self.contents
	}

	fn contents_mut(&mut self) -> &mut Vec<Self::InstrType> {
		&mut self.contents
	}
}

impl Debug for Block {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		self.contents.fmt(f)
	}
}

#[derive(Clone, PartialEq)]
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

#[derive(Clone, PartialEq)]
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
	// Binops
	Not {
		value: MutableValue,
	},
	And {
		left: MutableValue,
		right: Value,
	},
	Or {
		left: MutableValue,
		right: Value,
	},
	Xor {
		left: MutableValue,
		right: Value,
	},
	Use {
		val: MutableValue,
	},
	If {
		condition: Condition,
		body: Box<Block>,
	},
	IfElse {
		condition: Condition,
		first: Box<Block>,
		second: Box<Block>,
	},
	Call {
		call: CallInterface,
	},
	CallExtern {
		func: ResourceLocation,
	},
	MC(MinecraftInstr),
	ReturnValue {
		index: ArgRetIndex,
		value: Value,
	},
	Return {
		value: Value,
	},
	ReturnRun {
		body: Box<Block>,
	},
	Command {
		command: String,
	},
	Comment {
		comment: String,
	},
	Modify {
		modifier: MIRModifier,
		body: Box<Block>,
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
			Self::Get { value, scale } => format!("get {value:?} {scale}"),
			Self::Merge { left, right } => format!("merge {left:?}, {right:?}"),
			Self::Push { left, right } => format!("push {left:?}, {right:?}"),
			Self::PushFront { left, right } => format!("pushf {left:?}, {right:?}"),
			Self::Insert { left, right, index } => format!("ins {left:?}, {right:?}, {index}"),
			Self::Not { value } => format!("not {value:?}"),
			Self::And { left, right } => format!("and {left:?}, {right:?}"),
			Self::Or { left, right } => format!("or {left:?}, {right:?}"),
			Self::Xor { left, right } => format!("xor {left:?}, {right:?}"),
			Self::Use { val } => format!("use {val:?}"),
			Self::Call { call } => format!("call {call:?}"),
			Self::CallExtern { func } => format!("callx {func}"),
			Self::If { condition, body } => format!("if {condition:?}: {body:?}"),
			Self::IfElse {
				condition,
				first,
				second,
			} => format!("if {condition:?}: {first:?} else {second:?}"),
			Self::Remove { val } => format!("rm {val:?}"),
			Self::ReturnValue { index, value } => format!("retv {index} {value:?}"),
			Self::Return { value } => format!("ret {value:?}"),
			Self::ReturnRun { body } => format!("retr {body:?}"),
			Self::Command { command } => format!("cmd {command}"),
			Self::Comment { comment } => format!("cmt {comment}"),
			Self::Modify { modifier, body } => format!("mdf {modifier:?}: {body:?}"),
			Self::MC(instr) => format!("{instr:?}"),
		};
		write!(f, "{text}")
	}
}
