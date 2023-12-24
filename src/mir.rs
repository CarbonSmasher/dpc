use std::iter;
use std::{collections::HashMap, fmt::Debug};

use crate::common::block::{Block, BlockAllocator, BlockID};
use crate::common::condition::Condition;
use crate::common::function::{
	CallInterface, FunctionAnnotations, FunctionInterface, FunctionSignature,
};
use crate::common::mc::block::{CloneData, FillBiomeData, FillData, SetBlockData};
use crate::common::mc::entity::{AttributeType, EffectDuration, UUID};
use crate::common::mc::item::ItemData;
use crate::common::mc::pos::{Angle, DoubleCoordinates, DoubleCoordinates2D, IntCoordinates};
use crate::common::mc::scoreboard_and_teams::Criterion;
use crate::common::mc::time::{Time, TimePreset, TimeQuery};
use crate::common::mc::{
	DatapackListMode, DatapackOrder, DatapackPriority, Difficulty, EntityTarget, Gamemode,
	Location, Weather, XPValue,
};
use crate::common::ty::{DataType, NBTCompoundTypeContents};
use crate::common::{val::MutableValue, val::Value, DeclareBinding, Identifier, ResourceLocation};

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

	/// Get the data and block of a function with an ID
	pub fn get_fn(&self, id: &ResourceLocation) -> Option<(&FunctionInterface, &BlockID)> {
		self.functions.get_key_value(&FunctionInterface {
			id: id.clone(),
			sig: FunctionSignature::new(),
			annotations: FunctionAnnotations::new(),
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
	AddXP {
		target: EntityTarget,
		amount: i32,
		value: XPValue,
	},
	GetXP {
		target: EntityTarget,
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
	SetBlock {
		data: SetBlockData,
	},
	Fill {
		data: FillData,
	},
	Clone {
		data: CloneData,
	},
	SetWeather {
		weather: Weather,
		duration: Option<Time>,
	},
	AddTime {
		time: Time,
	},
	SetTime {
		time: Time,
	},
	SetTimePreset {
		time: TimePreset,
	},
	GetTime {
		query: TimeQuery,
	},
	AddTag {
		target: EntityTarget,
		tag: Identifier,
	},
	RemoveTag {
		target: EntityTarget,
		tag: Identifier,
	},
	ListTags {
		target: EntityTarget,
	},
	RideMount {
		target: EntityTarget,
		vehicle: EntityTarget,
	},
	RideDismount {
		target: EntityTarget,
	},
	FillBiome {
		data: FillBiomeData,
	},
	Spectate {
		target: EntityTarget,
		spectator: EntityTarget,
	},
	SpectateStop,
	SetGamemode {
		target: EntityTarget,
		gamemode: Gamemode,
	},
	DefaultGamemode {
		gamemode: Gamemode,
	},
	TeleportToEntity {
		source: EntityTarget,
		dest: EntityTarget,
	},
	TeleportToLocation {
		source: EntityTarget,
		dest: DoubleCoordinates,
	},
	TeleportWithRotation {
		source: EntityTarget,
		dest: DoubleCoordinates,
		rotation: DoubleCoordinates2D,
	},
	TeleportFacingLocation {
		source: EntityTarget,
		dest: DoubleCoordinates,
		facing: DoubleCoordinates,
	},
	TeleportFacingEntity {
		source: EntityTarget,
		dest: DoubleCoordinates,
		facing: EntityTarget,
	},
	GiveItem {
		target: EntityTarget,
		item: ItemData,
		amount: u32,
	},
	AddScoreboardObjective {
		objective: String,
		criterion: Criterion,
		display_name: Option<String>,
	},
	RemoveScoreboardObjective {
		objective: String,
	},
	ListScoreboardObjectives,
	TriggerAdd {
		objective: String,
		amount: i32,
	},
	TriggerSet {
		objective: String,
		amount: i32,
	},
	GetAttribute {
		target: EntityTarget,
		attribute: ResourceLocation,
		scale: f64,
	},
	GetAttributeBase {
		target: EntityTarget,
		attribute: ResourceLocation,
		scale: f64,
	},
	SetAttributeBase {
		target: EntityTarget,
		attribute: ResourceLocation,
		value: f64,
	},
	AddAttributeModifier {
		target: EntityTarget,
		attribute: ResourceLocation,
		uuid: UUID,
		name: String,
		value: f64,
		ty: AttributeType,
	},
	RemoveAttributeModifier {
		target: EntityTarget,
		attribute: ResourceLocation,
		uuid: UUID,
	},
	GetAttributeModifier {
		target: EntityTarget,
		attribute: ResourceLocation,
		uuid: UUID,
		scale: f64,
	},
	DisableDatapack {
		pack: String,
	},
	EnableDatapack {
		pack: String,
	},
	SetDatapackPriority {
		pack: String,
		priority: DatapackPriority,
	},
	SetDatapackOrder {
		pack: String,
		order: DatapackOrder,
		existing: String,
	},
	ListDatapacks {
		mode: DatapackListMode,
	},
	ListPlayerUUIDs,
	SummonEntity {
		entity: ResourceLocation,
		pos: DoubleCoordinates,
		nbt: NBTCompoundTypeContents,
	},
	SetWorldSpawn {
		pos: IntCoordinates,
		angle: Angle,
	},
	ClearItems {
		targets: Vec<EntityTarget>,
		item: Option<ItemData>,
		max_count: Option<u32>,
	},
	SetSpawnpoint {
		targets: Vec<EntityTarget>,
		pos: IntCoordinates,
		angle: Angle,
	},
	SpreadPlayers {
		center: DoubleCoordinates2D,
		spread_distance: f32,
		max_range: f32,
		max_height: Option<f32>,
		respect_teams: bool,
		target: EntityTarget,
	},
	ClearEffect {
		target: EntityTarget,
		effect: Option<ResourceLocation>,
	},
	GiveEffect {
		target: EntityTarget,
		effect: ResourceLocation,
		duration: EffectDuration,
		amplifier: u8,
		hide_particles: bool,
	},
	ReturnValue {
		index: u16,
		value: Value,
	},
	NoOp,
	Command {
		command: String,
	},
	Comment {
		comment: String,
	},
	SetGameruleBool {
		rule: String,
		value: bool,
	},
	SetGameruleInt {
		rule: String,
		value: i32,
	},
	GetGamerule {
		rule: String,
	},
	Locate {
		location_type: Location,
		location: ResourceLocation,
	},
	As {
		target: EntityTarget,
		body: Box<MIRInstrKind>,
	},
	At {
		target: EntityTarget,
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
			Self::Get { value } => format!("get {value:?}"),
			Self::Merge { left, right } => format!("merge {left:?}, {right:?}"),
			Self::Push { left, right } => format!("push {left:?}, {right:?}"),
			Self::PushFront { left, right } => format!("pushf {left:?}, {right:?}"),
			Self::Insert { left, right, index } => format!("ins {left:?}, {right:?}, {index}"),
			Self::Use { val } => format!("use {val:?}"),
			Self::Call { call } => format!("call {call:?}"),
			Self::CallExtern { func } => format!("callx {func}"),
			Self::If { condition, body } => format!("if {condition:?} then {body:?}"),
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
			Self::SetBlock { data } => format!("sb {data:?}"),
			Self::Fill { data } => format!("fill {data:?}"),
			Self::Clone { data } => format!("cln {data:?}"),
			Self::SetWeather { weather, duration } => format!("setw {weather} {duration:?}"),
			Self::AddTime { time } => format!("tima {time:?}"),
			Self::SetTime { time } => format!("tims {time:?}"),
			Self::SetTimePreset { time } => format!("timp {time:?}"),
			Self::GetTime { query } => format!("timg {query:?}"),
			Self::AddTag { target, tag } => format!("taga {target:?} {tag}"),
			Self::RemoveTag { target, tag } => format!("tagr {target:?} {tag}"),
			Self::ListTags { target } => format!("tagl {target:?}"),
			Self::RideMount { target, vehicle } => format!("mnt {target:?} {vehicle:?}"),
			Self::RideDismount { target } => format!("dmnt {target:?}"),
			Self::FillBiome { data } => format!("fillb {data:?}"),
			Self::Spectate { target, spectator } => format!("spec {target:?} {spectator:?}"),
			Self::SpectateStop => "specs".into(),
			Self::SetGamemode { target, gamemode } => format!("setgm {target:?} {gamemode}"),
			Self::DefaultGamemode { gamemode } => format!("dgm {gamemode}"),
			Self::TeleportToEntity { source, dest } => format!("tpe {source:?} {dest:?}"),
			Self::TeleportToLocation { source, dest } => format!("tpl {source:?} {dest:?}"),
			Self::TeleportWithRotation {
				source,
				dest,
				rotation,
			} => format!("tpr {source:?} {dest:?} {rotation:?}"),
			Self::TeleportFacingLocation {
				source,
				dest,
				facing,
			} => {
				format!("tpfl {source:?} {dest:?} {facing:?}")
			}
			Self::TeleportFacingEntity {
				source,
				dest,
				facing,
			} => {
				format!("tpfe {source:?} {dest:?} {facing:?}")
			}
			Self::GiveItem {
				target,
				item,
				amount,
			} => format!("itmg {target:?} {item:?} {amount}"),
			Self::AddScoreboardObjective {
				objective,
				criterion,
				display_name,
			} => {
				format!("sboa {objective} {criterion:?} {display_name:?}")
			}
			Self::RemoveScoreboardObjective { objective } => format!("sbor {objective}"),
			Self::ListScoreboardObjectives => "sbol".into(),
			Self::TriggerAdd { objective, amount } => format!("trga {objective}, {amount}"),
			Self::TriggerSet { objective, amount } => format!("trgs {objective}, {amount}"),
			Self::GetAttribute {
				target,
				attribute,
				scale,
			} => format!("attrg {target:?} {attribute} {scale}"),
			Self::GetAttributeBase {
				target,
				attribute,
				scale,
			} => format!("attrgb {target:?} {attribute} {scale}"),
			Self::SetAttributeBase {
				target,
				attribute,
				value,
			} => format!("attrs {target:?} {attribute} {value}"),
			Self::AddAttributeModifier {
				target,
				attribute,
				uuid,
				name,
				value,
				ty,
			} => {
				format!("attrma {target:?} {attribute} {uuid:?} {name} {value} {ty:?}")
			}
			Self::RemoveAttributeModifier {
				target,
				attribute,
				uuid,
			} => {
				format!("attrmr {target:?} {attribute} {uuid:?}")
			}
			Self::GetAttributeModifier {
				target,
				attribute,
				uuid,
				scale,
			} => {
				format!("attrmg {target:?} {attribute} {uuid:?} {scale}")
			}
			Self::DisableDatapack { pack } => format!("dpd {pack}"),
			Self::EnableDatapack { pack } => format!("dpe {pack}"),
			Self::SetDatapackPriority { pack, priority } => format!("dpp {pack} {priority:?}"),
			Self::SetDatapackOrder {
				pack,
				order,
				existing,
			} => {
				format!("dpo {pack} {order:?} {existing}")
			}
			Self::ListDatapacks { mode } => format!("dpl {mode:?}"),
			Self::ListPlayerUUIDs => "lspu".into(),
			Self::SummonEntity { entity, pos, nbt } => format!("smn {entity} {pos:?} {nbt:?}"),
			Self::SetWorldSpawn { pos, angle } => format!("sws {pos:?} {angle:?}"),
			Self::SetSpawnpoint {
				targets,
				pos,
				angle,
			} => {
				format!("ssp {targets:?} {pos:?} {angle:?}")
			}
			Self::ClearItems {
				targets,
				item,
				max_count,
			} => {
				format!("itmc {targets:?} {item:?} {max_count:?}")
			}
			Self::SpreadPlayers {
				center,
				spread_distance,
				max_range,
				max_height,
				respect_teams,
				target,
			} => format!("spd {center:?} {spread_distance} {max_range} {max_height:?} {respect_teams} {target:?}"),
			Self::Remove { val } => format!("rm {val:?}"),
			Self::AddXP { target, amount, value } => format!("xpa {target:?} {amount} {value}"),
			Self::GetXP { target, value } => format!("xpg {target:?} {value}"),
			Self::ClearEffect { target, effect } => format!("effc {target:?} {effect:?}"),
			Self::GiveEffect {
				target,
				effect,
				duration,
				amplifier,
				hide_particles
			} => {
				format!("effg {target:?} {effect} {duration:?} {amplifier} {hide_particles}")
			}
			Self::ReturnValue { index, value } => format!("retv {index} {value:?}"),
			Self::NoOp => "noop".into(),
			Self::Command { command } => format!("cmd {command}"),
			Self::Comment { comment } => format!("cmt {comment}"),
			Self::SetGameruleBool {
				rule,
				value,
			} => format!("grsb {rule} {value}"),
			Self::SetGameruleInt {
				rule,
				value,
			} => format!("grsi {rule} {value}"),
			Self::GetGamerule { rule } => format!("grg {rule}"),
			Self::Locate { location_type, location } => format!("loc {location_type:?} {location}"),
			Self::As { target, body } => format!("as {target:?}: {body:?}"),
			Self::At { target, body } => format!("at {target:?}: {body:?}"),
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
			Self::Remove { val } => val.get_used_regs(),
			Self::ReturnValue { value, .. } => value.get_used_regs(),
			Self::If { condition, body } => {
				[condition.get_used_regs(), body.get_used_regs()].concat()
			}
			Self::As { body, .. } | Self::At { body, .. } => body.get_used_regs(),
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
			Self::If { condition, body } => {
				for reg in condition.iter_used_regs_mut() {
					f(reg);
				}
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

	pub fn replace_mut_vals<F: Fn(&mut MutableValue) -> ()>(&mut self, f: F) {
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
			| Self::Get { value: val }
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
			_ => {}
		}
	}
}
