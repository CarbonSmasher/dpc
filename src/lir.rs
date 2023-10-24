use std::{collections::HashMap, fmt::Debug};

use crate::common::block::{Block, BlockAllocator, BlockID};
use crate::common::mc::{EntityTarget, XPValue};
use crate::common::modifier::Modifier;
use crate::common::ty::ArraySize;
use crate::common::{FunctionInterface, Identifier, MutableValue, RegisterList, Value};

#[derive(Debug, Clone)]
pub struct LIR {
	pub functions: HashMap<FunctionInterface, BlockID>,
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
				.map(|x| x.get_used_regs().into_iter())
				.flatten()
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
	SetScore(MutableValue, Value),
	AddScore(MutableValue, Value),
	SubScore(MutableValue, Value),
	MulScore(MutableValue, Value),
	DivScore(MutableValue, Value),
	ModScore(MutableValue, Value),
	MinScore(MutableValue, Value),
	MaxScore(MutableValue, Value),
	SwapScore(MutableValue, MutableValue),
	SetData(MutableValue, Value),
	ConstIndexToScore {
		score: MutableValue,
		value: Value,
		index: ArraySize,
	},
	Use(MutableValue),
	NoOp,
	// Commands
	Say(String),
	Tell(EntityTarget, String),
	Kill(EntityTarget),
	Reload,
	SetXP(EntityTarget, i32, XPValue),
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
			| LIRInstrKind::MaxScore(left, right)
			| LIRInstrKind::SetData(left, right) => [left.get_used_regs(), right.get_used_regs()].concat(),
			LIRInstrKind::SwapScore(left, right) => {
				[left.get_used_regs(), right.get_used_regs()].concat()
			}
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
			Self::SetData(left, right) => format!("setd {left:?} {right:?}"),
			Self::ConstIndexToScore {
				score,
				value,
				index,
			} => format!("idxcs {score:?} {value:?} {index}"),
			Self::Use(val) => format!("use {val:?}"),
			Self::NoOp => "no".into(),
			Self::Say(text) => format!("say {text}"),
			Self::Tell(target, text) => format!("tell {target:?} {text}"),
			Self::Kill(target) => format!("kill {target:?}"),
			Self::Reload => "reload".into(),
			Self::SetXP(target, amount, value) => format!("xps {target:?} {amount} {value}"),
		};
		write!(f, "{text}")
	}
}
