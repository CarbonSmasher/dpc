use std::{collections::HashMap, fmt::Debug};

use crate::common::block::{Block, BlockAllocator, BlockID};
use crate::common::function::Function;
use crate::common::mc::instr::MinecraftInstr;
use crate::common::mc::modifier::Modifier;
use crate::common::ty::{ArraySize, Double};
use crate::common::val::{MutableNBTValue, MutableScoreValue, MutableValue, NBTValue, ScoreValue};
use crate::common::{Identifier, RegisterList, ResourceLocation};

#[derive(Debug, Clone)]
pub struct LIR {
	pub functions: HashMap<ResourceLocation, Function>,
	pub blocks: BlockAllocator<LIRBlock>,
}

impl LIR {
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

impl Default for LIR {
	fn default() -> Self {
		Self::new()
	}
}

#[derive(Clone)]
pub struct LIRBlock {
	pub contents: Vec<LIRInstruction>,
	pub regs: RegisterList,
}

impl LIRBlock {
	pub fn new(regs: RegisterList) -> Self {
		Self {
			contents: Vec::new(),
			regs,
		}
	}
}

impl Block for LIRBlock {
	fn instr_count(&self) -> usize {
		self.contents.len()
	}

	fn get_children(&self) -> Vec<BlockID> {
		Vec::new()
	}
}

impl Debug for LIRBlock {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		self.contents.fmt(f)
	}
}

#[derive(Clone)]
pub struct LIRInstruction {
	pub kind: LIRInstrKind,
	pub modifiers: Vec<Modifier>,
}

impl LIRInstruction {
	pub fn new(kind: LIRInstrKind) -> Self {
		Self::with_modifiers(kind, Vec::new())
	}

	pub fn with_modifiers(kind: LIRInstrKind, modifiers: Vec<Modifier>) -> Self {
		Self { kind, modifiers }
	}

	pub fn get_used_regs(&self) -> Vec<&Identifier> {
		[
			self.kind.get_used_regs(),
			self.modifiers
				.iter()
				.flat_map(|x| x.get_used_regs().into_iter())
				.collect(),
		]
		.concat()
	}
}

impl Debug for LIRInstruction {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		if !self.modifiers.is_empty() {
			write!(f, "@")?;
			for (i, modifier) in self.modifiers.iter().enumerate() {
				write!(f, "{:?}", modifier)?;
				if i != self.modifiers.len() - 1 {
					write!(f, " ")?;
				}
			}
			write!(f, ": ")?;
		}
		write!(f, "{:?}", self.kind)
	}
}

#[derive(Clone)]
pub enum LIRInstrKind {
	// Basic
	SetScore(MutableScoreValue, ScoreValue),
	AddScore(MutableScoreValue, ScoreValue),
	SubScore(MutableScoreValue, ScoreValue),
	MulScore(MutableScoreValue, ScoreValue),
	DivScore(MutableScoreValue, ScoreValue),
	ModScore(MutableScoreValue, ScoreValue),
	MinScore(MutableScoreValue, ScoreValue),
	MaxScore(MutableScoreValue, ScoreValue),
	SwapScore(MutableScoreValue, MutableScoreValue),
	ResetScore(MutableScoreValue),
	SetData(MutableNBTValue, NBTValue),
	MergeData(MutableNBTValue, NBTValue),
	GetScore(MutableScoreValue),
	GetData(MutableNBTValue, Double),
	PushData(MutableNBTValue, NBTValue),
	PushFrontData(MutableNBTValue, NBTValue),
	InsertData(MutableNBTValue, NBTValue, i32),
	RemoveData(MutableNBTValue),
	ConstIndexToScore {
		score: MutableScoreValue,
		value: NBTValue,
		index: ArraySize,
	},
	Use(MutableValue),
	NoOp,
	Call(ResourceLocation),
	ReturnValue(i32),
	ReturnFail,
	ReturnRun(Box<LIRInstruction>),
	Command(String),
	Comment(String),
	MC(MinecraftInstr),
}

impl LIRInstrKind {
	pub fn get_used_regs(&self) -> Vec<&Identifier> {
		match self {
			LIRInstrKind::SetScore(left, right)
			| LIRInstrKind::AddScore(left, right)
			| LIRInstrKind::SubScore(left, right)
			| LIRInstrKind::MulScore(left, right)
			| LIRInstrKind::DivScore(left, right)
			| LIRInstrKind::ModScore(left, right)
			| LIRInstrKind::MinScore(left, right)
			| LIRInstrKind::MaxScore(left, right) => [left.get_used_regs(), right.get_used_regs()].concat(),
			LIRInstrKind::SetData(left, right)
			| LIRInstrKind::MergeData(left, right)
			| LIRInstrKind::PushData(left, right)
			| LIRInstrKind::PushFrontData(left, right)
			| LIRInstrKind::InsertData(left, right, ..) => {
				[left.get_used_regs(), right.get_used_regs()].concat()
			}
			LIRInstrKind::SwapScore(left, right) => {
				[left.get_used_regs(), right.get_used_regs()].concat()
			}
			LIRInstrKind::GetScore(score) => score.get_used_regs(),
			LIRInstrKind::GetData(data, ..) => data.get_used_regs(),
			LIRInstrKind::ConstIndexToScore { score, value, .. } => {
				[score.get_used_regs(), value.get_used_regs()].concat()
			}
			LIRInstrKind::Use(val) => val.get_used_regs(),
			_ => Vec::new(),
		}
	}
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
			Self::ResetScore(val) => format!("rms {val:?}"),
			Self::SetData(left, right) => format!("setd {left:?} {right:?}"),
			Self::MergeData(left, right) => format!("mrgd {left:?} {right:?}"),
			Self::GetScore(val) => format!("gets {val:?}"),
			Self::GetData(val, scale) => format!("getd {val:?} {scale}"),
			Self::PushData(left, right) => format!("pushd {left:?} {right:?}"),
			Self::PushFrontData(left, right) => format!("pushfd {left:?} {right:?}"),
			Self::InsertData(left, right, i) => format!("insd {left:?} {right:?} {i}"),
			Self::RemoveData(val) => format!("rmd {val:?}"),
			Self::ConstIndexToScore {
				score,
				value,
				index,
			} => format!("idxcs {score:?} {value:?} {index}"),
			Self::Use(val) => format!("use {val:?}"),
			Self::NoOp => "no".into(),
			Self::Call(fun) => format!("call {fun}"),
			Self::ReturnValue(val) => format!("retv {val}"),
			Self::ReturnFail => "retf".into(),
			Self::ReturnRun(cmd) => format!("retr {cmd:?}"),
			Self::Command(cmd) => format!("cmd {cmd}"),
			Self::Comment(cmt) => format!("cmt {cmt}"),
			Self::MC(instr) => format!("{instr:?}"),
		};
		write!(f, "{text}")
	}
}
