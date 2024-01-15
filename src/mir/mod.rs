pub mod instr_impls;

use std::fmt::Debug;
use std::hash::BuildHasherDefault;

use rustc_hash::FxHashMap;

use crate::common::block::Block;
use crate::common::condition::Condition;
use crate::common::function::{CallInterface, FunctionInterface};
use crate::common::mc::instr::MinecraftInstr;
use crate::common::mc::modifier::StoreModLocation;
use crate::common::mc::pos::DoubleCoordinates;
use crate::common::mc::EntityTarget;
use crate::common::reg::GetUsedRegs;
use crate::common::ty::{DataType, Double};
use crate::common::{val::MutableValue, val::Value, DeclareBinding, Identifier, ResourceLocation};
use crate::common::{FunctionTrait, IRType};

#[derive(Debug, Clone)]
pub struct MIR {
	pub functions: FxHashMap<ResourceLocation, MIRFunction>,
}

impl MIR {
	pub fn new() -> Self {
		Self {
			functions: FxHashMap::default(),
		}
	}

	pub fn with_capacity(function_capacity: usize) -> Self {
		Self {
			functions: FxHashMap::with_capacity_and_hasher(
				function_capacity,
				BuildHasherDefault::default(),
			),
		}
	}
}

impl IRType for MIR {
	type FunctionType = MIRFunction;

	fn get_fns<'this>(&'this self) -> &'this FxHashMap<ResourceLocation, Self::FunctionType> {
		&self.functions
	}

	fn get_fns_mut<'this>(
		&'this mut self,
	) -> &'this mut FxHashMap<ResourceLocation, Self::FunctionType> {
		&mut self.functions
	}
}

impl Default for MIR {
	fn default() -> Self {
		Self::new()
	}
}

#[derive(Debug, Clone)]
pub struct MIRFunction {
	pub interface: FunctionInterface,
	pub block: MIRBlock,
}

impl FunctionTrait for MIRFunction {
	type BlockType = MIRBlock;

	fn block(&self) -> &Self::BlockType {
		&self.block
	}

	fn block_mut(&mut self) -> &mut Self::BlockType {
		&mut self.block
	}
}

#[derive(Clone)]
pub struct MIRBlock {
	pub contents: Vec<MIRInstruction>,
}

impl MIRBlock {
	pub fn new() -> Self {
		Self::with_contents(Vec::new())
	}

	pub fn with_contents(contents: Vec<MIRInstruction>) -> Self {
		Self { contents }
	}

	pub fn with_capacity(capacity: usize) -> Self {
		Self {
			contents: Vec::with_capacity(capacity),
		}
	}

	pub fn replace_regs<F: Fn(&mut Identifier)>(&mut self, f: &F) {
		for instr in &mut self.contents {
			instr.kind.replace_regs(f);
		}
	}

	pub fn replace_mut_vals<F: Fn(&mut MutableValue)>(&mut self, f: &F) {
		for instr in &mut self.contents {
			instr.kind.replace_mut_vals(f);
		}
	}
}

impl Default for MIRBlock {
	fn default() -> Self {
		Self::new()
	}
}

impl Block for MIRBlock {
	type InstrType = MIRInstruction;
	type InstrKindType = MIRInstrKind;

	fn contents(&self) -> &Vec<Self::InstrType> {
		&self.contents
	}

	fn contents_mut(&mut self) -> &mut Vec<Self::InstrType> {
		&mut self.contents
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

impl GetUsedRegs for MIRInstruction {
	fn append_used_regs<'this>(&'this self, regs: &mut Vec<&'this Identifier>) {
		self.kind.append_used_regs(regs);
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
	GetConst {
		value: i32,
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
	Call {
		call: CallInterface,
	},
	CallExtern {
		func: ResourceLocation,
	},
	If {
		condition: Condition,
		body: Box<MIRBlock>,
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
		body: Box<MIRBlock>,
	},
	NoOp,
	Command {
		command: String,
	},
	Comment {
		comment: String,
	},
	// Modifiers
	As {
		target: EntityTarget,
		body: Box<MIRBlock>,
	},
	At {
		target: EntityTarget,
		body: Box<MIRBlock>,
	},
	StoreResult {
		location: StoreModLocation,
		body: Box<MIRBlock>,
	},
	StoreSuccess {
		location: StoreModLocation,
		body: Box<MIRBlock>,
	},
	Positioned {
		position: DoubleCoordinates,
		body: Box<MIRBlock>,
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
			Self::GetConst { value } => format!("getc {value:?}"),
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
