use std::{collections::HashMap, fmt::Debug};

use crate::common::block::{Block, BlockAllocator, BlockID};
use crate::common::function::FunctionInterface;
use crate::common::mc::block::{CloneData, FillData, SetBlockData};
use crate::common::mc::modifier::Modifier;
use crate::common::mc::time::{Time, TimePreset, TimeQuery};
use crate::common::mc::{Difficulty, EntityTarget, Weather, XPValue};
use crate::common::ty::ArraySize;
use crate::common::val::{MutableNBTValue, MutableScoreValue, MutableValue, NBTValue, ScoreValue};
use crate::common::{Identifier, RegisterList, ResourceLocation};

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
	SetScore(MutableScoreValue, ScoreValue),
	AddScore(MutableScoreValue, ScoreValue),
	SubScore(MutableScoreValue, ScoreValue),
	MulScore(MutableScoreValue, ScoreValue),
	DivScore(MutableScoreValue, ScoreValue),
	ModScore(MutableScoreValue, ScoreValue),
	MinScore(MutableScoreValue, ScoreValue),
	MaxScore(MutableScoreValue, ScoreValue),
	SwapScore(MutableScoreValue, MutableScoreValue),
	SetData(MutableNBTValue, NBTValue),
	MergeData(MutableNBTValue, NBTValue),
	GetScore(MutableScoreValue),
	GetData(MutableNBTValue),
	PushData(MutableNBTValue, NBTValue),
	PushFrontData(MutableNBTValue, NBTValue),
	InsertData(MutableNBTValue, NBTValue, i32),
	ConstIndexToScore {
		score: MutableScoreValue,
		value: NBTValue,
		index: ArraySize,
	},
	Use(MutableValue),
	NoOp,
	Call(ResourceLocation),
	// Chat
	Say(String),
	Tell(EntityTarget, String),
	Me(String),
	TeamMessage(String),
	// Multiplayer
	ListPlayers,
	StopServer,
	BanPlayers(Vec<EntityTarget>, Option<String>),
	BanIP(String, Option<String>),
	PardonPlayers(Vec<EntityTarget>),
	PardonIP(String),
	Banlist,
	Op(Vec<EntityTarget>),
	Deop(Vec<EntityTarget>),
	WhitelistAdd(Vec<EntityTarget>),
	WhitelistRemove(Vec<EntityTarget>),
	WhitelistOn,
	WhitelistOff,
	WhitelistReload,
	WhitelistList,
	Kick(Vec<EntityTarget>, Option<String>),
	Publish,
	// Entities
	Kill(EntityTarget),
	SetXP(EntityTarget, i32, XPValue),
	AddTag(EntityTarget, Identifier),
	RemoveTag(EntityTarget, Identifier),
	ListTags(EntityTarget),
	RideMount(EntityTarget, EntityTarget),
	RideDismount(EntityTarget),
	// Items
	Enchant(EntityTarget, ResourceLocation, i32),
	// Blocks
	SetBlock(SetBlockData),
	Fill(FillData),
	Clone(CloneData),
	// World
	Seed,
	GetDifficulty,
	SetDifficulty(Difficulty),
	SetWeather(Weather, Option<Time>),
	AddTime(Time),
	SetTime(Time),
	SetTimePreset(TimePreset),
	GetTime(TimeQuery),
	// Misc
	Reload,
	StopSound,
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
			LIRInstrKind::GetData(data) => data.get_used_regs(),
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
			Self::MergeData(left, right) => format!("mrgd {left:?} {right:?}"),
			Self::GetScore(val) => format!("gets {val:?}"),
			Self::GetData(val) => format!("getd {val:?}"),
			Self::PushData(left, right) => format!("pushd {left:?} {right:?}"),
			Self::PushFrontData(left, right) => format!("pushfd {left:?} {right:?}"),
			Self::InsertData(left, right, i) => format!("insd {left:?} {right:?} {i}"),
			Self::ConstIndexToScore {
				score,
				value,
				index,
			} => format!("idxcs {score:?} {value:?} {index}"),
			Self::Use(val) => format!("use {val:?}"),
			Self::NoOp => "no".into(),
			Self::Call(fun) => format!("call {fun}"),
			Self::Say(text) => format!("say {text}"),
			Self::Tell(target, text) => format!("tell {target:?} {text}"),
			Self::Me(text) => format!("me {text}"),
			Self::TeamMessage(text) => format!("tm {text}"),
			Self::Kill(target) => format!("kill {target:?}"),
			Self::Reload => "reload".into(),
			Self::SetXP(target, amount, value) => format!("xps {target:?} {amount} {value}"),
			Self::AddTag(target, tag) => format!("taga {target:?} {tag}"),
			Self::RemoveTag(target, tag) => format!("tagr {target:?} {tag}"),
			Self::ListTags(target) => format!("tagl {target:?}"),
			Self::RideMount(target, vehicle) => format!("mnt {target:?} {vehicle:?}"),
			Self::RideDismount(target) => format!("dmnt {target:?}"),
			Self::ListPlayers => "lsp".into(),
			Self::StopSound => "stops".into(),
			Self::StopServer => "stop".into(),
			Self::BanPlayers(targets, reason) => format!("ban {targets:?} {reason:?}"),
			Self::BanIP(target, reason) => format!("bani {target} {reason:?}"),
			Self::PardonPlayers(targets) => format!("par {targets:?}"),
			Self::PardonIP(target) => format!("pari {target}"),
			Self::Banlist => "banl".into(),
			Self::Op(targets) => format!("op {targets:?}"),
			Self::Deop(targets) => format!("deop {targets:?}"),
			Self::WhitelistAdd(targets) => format!("wla {targets:?}"),
			Self::WhitelistRemove(targets) => format!("wlr {targets:?}"),
			Self::WhitelistOn => "wlon".into(),
			Self::WhitelistOff => "wloff".into(),
			Self::WhitelistReload => "wlrl".into(),
			Self::WhitelistList => "wll".into(),
			Self::Kick(targets, reason) => format!("kick {targets:?} {reason:?}"),
			Self::Publish => "pub".into(),
			Self::SetBlock(data) => format!("sb {data:?}"),
			Self::Fill(data) => format!("fill {data:?}"),
			Self::Clone(data) => format!("cln {data:?}"),
			Self::Seed => "seed".into(),
			Self::GetDifficulty => "diffg".into(),
			Self::SetDifficulty(diff) => format!("diffs {diff}"),
			Self::SetWeather(weather, duration) => format!("setw {weather} {duration:?}"),
			Self::AddTime(time) => format!("addt {time:?}"),
			Self::SetTime(time) => format!("sett {time:?}"),
			Self::SetTimePreset(time) => format!("settp {time:?}"),
			Self::GetTime(query) => format!("gett {query:?}"),
			Self::Enchant(target, ench, lvl) => format!("ench {target:?} {ench} {lvl}"),
		};
		write!(f, "{text}")
	}
}
