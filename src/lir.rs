use std::fmt::Debug;
use std::hash::BuildHasherDefault;

use rustc_hash::FxHashMap;

use crate::common::block::Block;
use crate::common::function::FunctionInterface;
use crate::common::mc::instr::MinecraftInstr;
use crate::common::mc::modifier::{IfModCondition, IfScoreCondition, IfScoreRangeEnd, Modifier};
use crate::common::reg::{GetUsedLocals, GetUsedRegs, Local};
use crate::common::ty::Double;
use crate::common::val::{ArgRetIndex, MutableNBTValue, MutableScoreValue, NBTValue, ScoreValue};
use crate::common::{FunctionTrait, IRType, Identifier, RegisterList, ResourceLocation};
use crate::passes::util::AnalysisResult;

#[derive(Debug, Clone)]
pub struct LIR {
	pub functions: FxHashMap<ResourceLocation, LIRFunction>,
}

impl LIR {
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

impl IRType for LIR {
	type FunctionType = LIRFunction;

	fn get_fns<'this>(&'this self) -> &'this FxHashMap<ResourceLocation, Self::FunctionType> {
		&self.functions
	}

	fn get_fns_mut<'this>(
		&'this mut self,
	) -> &'this mut FxHashMap<ResourceLocation, Self::FunctionType> {
		&mut self.functions
	}
}

impl Default for LIR {
	fn default() -> Self {
		Self::new()
	}
}

#[derive(Debug, Clone)]
pub struct LIRFunction {
	pub interface: FunctionInterface,
	pub block: LIRBlock,
	pub parent: Option<ResourceLocation>,
}

impl FunctionTrait for LIRFunction {
	type BlockType = LIRBlock;

	fn block(&self) -> &Self::BlockType {
		&self.block
	}

	fn block_mut(&mut self) -> &mut Self::BlockType {
		&mut self.block
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
	type InstrType = LIRInstruction;
	type InstrKindType = LIRInstrKind;

	fn contents(&self) -> &Vec<Self::InstrType> {
		&self.contents
	}

	fn contents_mut(&mut self) -> &mut Vec<Self::InstrType> {
		&mut self.contents
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
}

impl GetUsedRegs for LIRInstruction {
	fn append_used_regs<'a>(&'a self, regs: &mut Vec<&'a Identifier>) {
		self.kind.append_used_regs(regs);
		for modi in &self.modifiers {
			modi.append_used_regs(regs);
		}
	}
}

impl GetUsedLocals for LIRInstruction {
	fn append_used_locals<'a>(&'a self, locals: &mut Vec<&'a Local>) {
		self.kind.append_used_locals(locals);
		for modi in &self.modifiers {
			modi.append_used_locals(locals);
		}
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
	GetConst(i32),
	PushData(MutableNBTValue, NBTValue),
	PushFrontData(MutableNBTValue, NBTValue),
	InsertData(MutableNBTValue, NBTValue, i32),
	RemoveData(MutableNBTValue),
	Use(Local),
	NoOp,
	Call(ResourceLocation, Vec<Local>),
	ReturnValue(i32),
	ReturnFail,
	ReturnRun(Box<LIRInstruction>),
	Command(String),
	Comment(String),
	MC(MinecraftInstr),
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
			Self::GetConst(value) => format!("getc {value:?}"),
			Self::PushData(left, right) => format!("pushd {left:?} {right:?}"),
			Self::PushFrontData(left, right) => format!("pushfd {left:?} {right:?}"),
			Self::InsertData(left, right, i) => format!("insd {left:?} {right:?} {i}"),
			Self::RemoveData(val) => format!("rmd {val:?}"),
			Self::Use(val) => format!("use {val:?}"),
			Self::NoOp => "no".into(),
			Self::Call(fun, ..) => format!("call {fun}"),
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

impl GetUsedRegs for LIRInstrKind {
	fn append_used_regs<'a>(&'a self, regs: &mut Vec<&'a Identifier>) {
		match self {
			LIRInstrKind::SetScore(left, right)
			| LIRInstrKind::AddScore(left, right)
			| LIRInstrKind::SubScore(left, right)
			| LIRInstrKind::MulScore(left, right)
			| LIRInstrKind::DivScore(left, right)
			| LIRInstrKind::ModScore(left, right)
			| LIRInstrKind::MinScore(left, right)
			| LIRInstrKind::MaxScore(left, right) => {
				left.append_used_regs(regs);
				right.append_used_regs(regs);
			}
			LIRInstrKind::SetData(left, right)
			| LIRInstrKind::MergeData(left, right)
			| LIRInstrKind::PushData(left, right)
			| LIRInstrKind::PushFrontData(left, right)
			| LIRInstrKind::InsertData(left, right, ..) => {
				left.append_used_regs(regs);
				right.append_used_regs(regs);
			}
			LIRInstrKind::SwapScore(left, right) => {
				left.append_used_regs(regs);
				right.append_used_regs(regs);
			}
			LIRInstrKind::GetScore(score) | LIRInstrKind::ResetScore(score) => {
				score.append_used_regs(regs);
			}
			LIRInstrKind::GetData(data, ..) | LIRInstrKind::RemoveData(data) => {
				data.append_used_regs(regs);
			}
			LIRInstrKind::Use(val) => val.append_used_regs(regs),
			LIRInstrKind::Call(_, locals) => {
				for loc in locals {
					loc.append_used_regs(regs);
				}
			}
			LIRInstrKind::NoOp
			| LIRInstrKind::GetConst(..)
			| LIRInstrKind::ReturnValue(..)
			| LIRInstrKind::ReturnFail
			| LIRInstrKind::Command(..)
			| LIRInstrKind::Comment(..)
			| LIRInstrKind::MC(..) => {}
			LIRInstrKind::ReturnRun(body) => body.append_used_regs(regs),
		}
	}
}

impl GetUsedLocals for LIRInstrKind {
	fn append_used_locals<'a>(&'a self, locals: &mut Vec<&'a Local>) {
		match self {
			LIRInstrKind::SetScore(left, right)
			| LIRInstrKind::AddScore(left, right)
			| LIRInstrKind::SubScore(left, right)
			| LIRInstrKind::MulScore(left, right)
			| LIRInstrKind::DivScore(left, right)
			| LIRInstrKind::ModScore(left, right)
			| LIRInstrKind::MinScore(left, right)
			| LIRInstrKind::MaxScore(left, right) => {
				left.append_used_locals(locals);
				right.append_used_locals(locals);
			}
			LIRInstrKind::SetData(left, right)
			| LIRInstrKind::MergeData(left, right)
			| LIRInstrKind::PushData(left, right)
			| LIRInstrKind::PushFrontData(left, right)
			| LIRInstrKind::InsertData(left, right, ..) => {
				left.append_used_locals(locals);
				right.append_used_locals(locals);
			}
			LIRInstrKind::SwapScore(left, right) => {
				left.append_used_locals(locals);
				right.append_used_locals(locals);
			}
			LIRInstrKind::GetScore(score) | LIRInstrKind::ResetScore(score) => {
				score.append_used_locals(locals);
			}
			LIRInstrKind::GetData(data, ..) | LIRInstrKind::RemoveData(data) => {
				data.append_used_locals(locals);
			}
			LIRInstrKind::Use(val) => locals.push(val),
			LIRInstrKind::Call(_, locs) => {
				locals.extend(locs);
			}
			LIRInstrKind::NoOp
			| LIRInstrKind::GetConst(..)
			| LIRInstrKind::ReturnValue(..)
			| LIRInstrKind::ReturnFail
			| LIRInstrKind::Command(..)
			| LIRInstrKind::Comment(..)
			| LIRInstrKind::MC(..) => {}
			LIRInstrKind::ReturnRun(body) => body.append_used_locals(locals),
		}
	}
}

impl LIRInstrKind {
	pub fn get_simple_sb_op_lhs(&self) -> Option<&MutableScoreValue> {
		match self {
			LIRInstrKind::SetScore(left, ..)
			| LIRInstrKind::AddScore(left, ..)
			| LIRInstrKind::SubScore(left, ..)
			| LIRInstrKind::MulScore(left, ..)
			| LIRInstrKind::DivScore(left, ..)
			| LIRInstrKind::ModScore(left, ..)
			| LIRInstrKind::MinScore(left, ..)
			| LIRInstrKind::MaxScore(left, ..) => Some(left),
			_ => None,
		}
	}

	pub fn get_op_rhs_mut(&mut self) -> Option<&mut Local> {
		match self {
			LIRInstrKind::SetScore(_, ScoreValue::Mutable(MutableScoreValue::Local(right)))
			| LIRInstrKind::AddScore(_, ScoreValue::Mutable(MutableScoreValue::Local(right)))
			| LIRInstrKind::SubScore(_, ScoreValue::Mutable(MutableScoreValue::Local(right)))
			| LIRInstrKind::MulScore(_, ScoreValue::Mutable(MutableScoreValue::Local(right)))
			| LIRInstrKind::DivScore(_, ScoreValue::Mutable(MutableScoreValue::Local(right)))
			| LIRInstrKind::ModScore(_, ScoreValue::Mutable(MutableScoreValue::Local(right)))
			| LIRInstrKind::MinScore(_, ScoreValue::Mutable(MutableScoreValue::Local(right)))
			| LIRInstrKind::MaxScore(_, ScoreValue::Mutable(MutableScoreValue::Local(right))) => Some(right),
			LIRInstrKind::SetData(_, NBTValue::Mutable(MutableNBTValue::Local(right)))
			| LIRInstrKind::MergeData(_, NBTValue::Mutable(MutableNBTValue::Local(right)))
			| LIRInstrKind::InsertData(_, NBTValue::Mutable(MutableNBTValue::Local(right)), ..)
			| LIRInstrKind::PushData(_, NBTValue::Mutable(MutableNBTValue::Local(right)))
			| LIRInstrKind::PushFrontData(_, NBTValue::Mutable(MutableNBTValue::Local(right))) => {
				Some(right)
			}
			_ => None,
		}
	}

	pub fn replace_mut_score_vals<'a, F: Fn(&'a mut MutableScoreValue)>(&'a mut self, f: &F) {
		match self {
			LIRInstrKind::SetScore(left, right)
			| LIRInstrKind::AddScore(left, right)
			| LIRInstrKind::SubScore(left, right)
			| LIRInstrKind::MulScore(left, right)
			| LIRInstrKind::DivScore(left, right)
			| LIRInstrKind::ModScore(left, right)
			| LIRInstrKind::MinScore(left, right)
			| LIRInstrKind::MaxScore(left, right) => {
				f(left);
				if let ScoreValue::Mutable(right) = right {
					f(right);
				}
			}
			LIRInstrKind::ResetScore(val) => {
				f(val);
			}
			_ => {}
		}
	}

	pub fn get_read_score_arg(&self) -> Option<&ArgRetIndex> {
		match self {
			LIRInstrKind::SetScore(_, right)
			| LIRInstrKind::AddScore(_, right)
			| LIRInstrKind::SubScore(_, right)
			| LIRInstrKind::MulScore(_, right)
			| LIRInstrKind::DivScore(_, right)
			| LIRInstrKind::ModScore(_, right)
			| LIRInstrKind::MinScore(_, right)
			| LIRInstrKind::MaxScore(_, right) => {
				if let ScoreValue::Mutable(MutableScoreValue::Local(Local::Arg(arg))) = right {
					Some(arg)
				} else {
					None
				}
			}
			LIRInstrKind::ResetScore(MutableScoreValue::Local(Local::Arg(arg))) => Some(arg),
			_ => None,
		}
	}

	pub fn get_modified_locs(&self) -> AnalysisResult<&Local> {
		match self {
			LIRInstrKind::SetScore(left, _)
			| LIRInstrKind::AddScore(left, _)
			| LIRInstrKind::SubScore(left, _)
			| LIRInstrKind::MulScore(left, _)
			| LIRInstrKind::DivScore(left, _)
			| LIRInstrKind::ModScore(left, _)
			| LIRInstrKind::MinScore(left, _)
			| LIRInstrKind::MaxScore(left, _)
			| LIRInstrKind::ResetScore(left) => {
				if let MutableScoreValue::Local(loc) = left {
					AnalysisResult::known(loc)
				} else {
					AnalysisResult::known_empty()
				}
			}
			LIRInstrKind::SwapScore(l, r) => {
				let mut out = Vec::new();
				if let MutableScoreValue::Local(l) = l {
					out.push(l);
				}
				if let MutableScoreValue::Local(r) = r {
					out.push(r);
				}
				AnalysisResult::Known(out)
			}
			LIRInstrKind::SetData(left, _) | LIRInstrKind::RemoveData(left) => {
				if let MutableNBTValue::Local(loc) = left {
					AnalysisResult::known(loc)
				} else {
					AnalysisResult::known_empty()
				}
			}
			LIRInstrKind::Use(loc) => AnalysisResult::known(loc),
			LIRInstrKind::Call(..) => AnalysisResult::Unknown,
			_ => AnalysisResult::known_empty(),
		}
	}
}

impl LIRInstruction {
	pub fn replace_mut_score_vals<'a, F: Fn(&'a mut MutableScoreValue)>(&'a mut self, f: &F) {
		self.kind.replace_mut_score_vals(f);
		for modi in &mut self.modifiers {
			match modi {
				Modifier::If { condition, .. } => match condition.as_mut() {
					IfModCondition::Score(cond) => match cond {
						IfScoreCondition::Single { left, right } => {
							if let ScoreValue::Mutable(left) = left {
								f(left);
							}
							if let ScoreValue::Mutable(right) = right {
								f(right);
							}
						}
						IfScoreCondition::Range { score, left, right } => {
							if let ScoreValue::Mutable(score) = score {
								f(score);
							}
							if let IfScoreRangeEnd::Fixed {
								value: ScoreValue::Mutable(left),
								..
							} = left
							{
								f(left);
							}
							if let IfScoreRangeEnd::Fixed {
								value: ScoreValue::Mutable(right),
								..
							} = right
							{
								f(right);
							}
						}
					},
					_ => {}
				},
				_ => {}
			}
		}
	}
}
