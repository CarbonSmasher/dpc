use std::{collections::HashMap, fmt::Debug};

use crate::common::block::{Block as BlockTrait, BlockAllocator, BlockID};
use crate::common::function::{CallInterface, FunctionInterface};
use crate::common::mc::{Difficulty, EntityTarget, XPValue};
use crate::common::ty::DataType;
use crate::common::{DeclareBinding, Identifier, MutableValue, ResourceLocation, Value};

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
	Me {
		message: String,
	},
	TeamMessage {
		message: String,
	},
	ListPlayers,
	StopServer,
	BanPlayers {
		targets: Vec<EntityTarget>,
		reason: Option<String>,
	},
	BanIP {
		target: String,
		reason: Option<String>,
	},
	PardonPlayers {
		targets: Vec<EntityTarget>,
	},
	PardonIP {
		target: String,
	},
	Banlist,
	Op {
		targets: Vec<EntityTarget>,
	},
	Deop {
		targets: Vec<EntityTarget>,
	},
	WhitelistAdd {
		targets: Vec<EntityTarget>,
	},
	WhitelistRemove {
		targets: Vec<EntityTarget>,
	},
	WhitelistOn,
	WhitelistOff,
	WhitelistReload,
	WhitelistList,
	Publish,
	Kick {
		targets: Vec<EntityTarget>,
		reason: Option<String>,
	},
	Kill {
		target: EntityTarget,
	},
	SetXP {
		target: EntityTarget,
		amount: i32,
		value: XPValue,
	},
	Enchant {
		target: EntityTarget,
		enchantment: ResourceLocation,
		level: i32,
	},
	Seed,
	GetDifficulty,
	SetDifficulty {
		difficulty: Difficulty,
	},
	Reload,
	StopSound,
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
			Self::Get { value } => format!("get {value:?}"),
			Self::Merge { left, right } => format!("merge {left:?}, {right:?}"),
			Self::Push { left, right } => format!("push {left:?}, {right:?}"),
			Self::PushFront { left, right } => format!("pushf {left:?}, {right:?}"),
			Self::Insert { left, right, index } => format!("ins {left:?}, {right:?}, {index}"),
			Self::Use { val } => format!("use {val:?}"),
			Self::Call { call } => format!("call {call:?}"),
			Self::Say { message } => format!("say {message}"),
			Self::Tell { target, message } => format!("tell {target:?} {message}"),
			Self::Me { message } => format!("me {message}"),
			Self::TeamMessage { message } => format!("tm {message}"),
			Self::Banlist => "banl".into(),
			Self::BanPlayers { targets, reason } => format!("ban {targets:?} {reason:?}"),
			Self::BanIP { target, reason } => format!("bani {target} {reason:?}"),
			Self::PardonPlayers { targets } => format!("par {targets:?}"),
			Self::PardonIP { target } => format!("pari {target}"),
			Self::Op { targets } => format!("op {targets:?}"),
			Self::Deop { targets } => format!("deop {targets:?}"),
			Self::WhitelistAdd { targets } => format!("wla {targets:?}"),
			Self::WhitelistRemove { targets } => format!("wlr {targets:?}"),
			Self::WhitelistOn => "wlon".into(),
			Self::WhitelistOff => "wloff".into(),
			Self::WhitelistReload => "wlrl".into(),
			Self::WhitelistList => "wll".into(),
			Self::Kick { targets, reason } => format!("kick {targets:?} {reason:?}"),
			Self::ListPlayers => "lsp".into(),
			Self::Publish => "pub".into(),
			Self::Kill { target } => format!("kill {target:?}"),
			Self::Reload => "rl".into(),
			Self::Seed => "seed".into(),
			Self::StopServer => "stop".into(),
			Self::StopSound => "stops".into(),
			Self::GetDifficulty => "diffg".into(),
			Self::SetDifficulty { difficulty } => format!("diffs {difficulty}"),
			Self::Enchant {
				target,
				enchantment,
				level,
			} => format!("ench {target:?} {enchantment} {level}"),
			Self::SetXP {
				target,
				amount,
				value,
			} => format!("xps {target:?} {amount} {value}"),
		};
		write!(f, "{text}")
	}
}
