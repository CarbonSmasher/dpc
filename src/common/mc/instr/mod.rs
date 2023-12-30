use std::fmt::Debug;

use crate::common::{ty::NBTCompoundTypeContents, Identifier, ResourceLocation};

use super::block::{CloneData, FillBiomeData, FillData, SetBlockData};
use super::entity::{AttributeType, EffectDuration, UUID};
use super::item::ItemData;
use super::pos::{Angle, DoubleCoordinates, DoubleCoordinates2D, IntCoordinates};
use super::scoreboard_and_teams::Criterion;
use super::time::{Time, TimePreset, TimeQuery};
use super::{
	DatapackListMode, DatapackOrder, DatapackPriority, Difficulty, EntityTarget, Gamemode,
	Location, Weather, XPValue,
};

#[derive(Clone, PartialEq)]
pub enum MinecraftInstr {
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
}

impl Debug for MinecraftInstr {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let text = match self {
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
		};
		write!(f, "{text}")
	}
}