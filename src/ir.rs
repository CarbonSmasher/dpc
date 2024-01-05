use std::fmt::Debug;

use rustc_hash::FxHashMap;

use crate::common::block::{Block as BlockTrait, BlockAllocator, BlockID};
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
pub struct IR {
	pub functions: FxHashMap<ResourceLocation, Function>,
	pub blocks: BlockAllocator<Block>,
}

impl IR {
	pub fn new() -> Self {
		Self {
			functions: FxHashMap::default(),
			blocks: BlockAllocator::new(),
		}
	}
}

impl IRType for IR {
	type BlockType = Block;
	type InstrType = Instruction;
	type InstrKindType = InstrKind;

	fn get_fns<'this>(&'this self) -> &'this FxHashMap<ResourceLocation, Function> {
		&self.functions
	}

	fn get_fns_mut<'this>(&'this mut self) -> &'this mut FxHashMap<ResourceLocation, Function> {
		&mut self.functions
	}

	fn get_blocks<'this>(&'this self) -> &'this BlockAllocator<Self::BlockType> {
		&self.blocks
	}

	fn get_blocks_mut<'this>(&'this mut self) -> &'this mut BlockAllocator<Self::BlockType> {
		&mut self.blocks
	}
}

impl Default for IR {
	fn default() -> Self {
		Self::new()
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

impl Default for Block {
	fn default() -> Self {
		Self::new()
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
	Use {
		val: MutableValue,
	},
	If {
		condition: Condition,
		body: Box<InstrKind>,
	},
	Call {
		call: CallInterface,
	},
	CallExtern {
		func: ResourceLocation,
	},
	MC(MinecraftInstr),
	ReturnValue {
		index: u16,
		value: Value,
	},
	Return {
		value: Value,
	},
	ReturnRun {
		body: Box<InstrKind>,
	},
	Command {
		command: String,
	},
	Comment {
		comment: String,
	},
	As {
		target: EntityTarget,
		body: Box<InstrKind>,
	},
	At {
		target: EntityTarget,
		body: Box<InstrKind>,
	},
	StoreResult {
		location: StoreModLocation,
		body: Box<InstrKind>,
	},
	StoreSuccess {
		location: StoreModLocation,
		body: Box<InstrKind>,
	},
	Positioned {
		position: DoubleCoordinates,
		body: Box<InstrKind>,
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
			Self::Use { val } => format!("use {val:?}"),
			Self::Call { call } => format!("call {call:?}"),
			Self::CallExtern { func } => format!("callx {func}"),
			Self::If { condition, body } => format!("if {condition:?}: {body:?}"),
			Self::Remove { val } => format!("rm {val:?}"),
			Self::ReturnValue { index, value } => format!("retv {index} {value:?}"),
			Self::Return { value } => format!("ret {value:?}"),
			Self::ReturnRun { body } => format!("retr {body:?}"),
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
