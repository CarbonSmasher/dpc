pub mod block;
pub mod entity;
pub mod instr;
pub mod item;
pub mod modifier;
pub mod pos;
pub mod scoreboard_and_teams;
pub mod time;

use std::fmt::{Debug, Display};

use crate::common::Identifier;

use self::pos::IntCoordinates;

use self::entity::TargetSelector;
use super::ty::ArraySize;
use super::ResourceLocation;

#[derive(Clone, PartialEq)]
pub enum EntityTarget {
	Player(String),
	Selector(TargetSelector),
}

impl EntityTarget {
	pub fn is_blank_this(&self) -> bool {
		matches!(self, EntityTarget::Selector(sel) if sel.is_blank_this())
	}

	pub fn is_value_eq(&self, other: &Self) -> bool {
		matches!((self, other), (Self::Player(l), Self::Player(r)) if l == r)
			|| matches!((self, other), (Self::Selector(l), Self::Selector(r)) if l.is_value_eq(r))
	}

	pub fn relies_on_position(&self) -> bool {
		matches!(self, Self::Selector(tgt) if tgt.relies_on_position())
	}
}

impl Debug for EntityTarget {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Player(player) => write!(f, "{player}"),
			Self::Selector(sel) => write!(f, "{sel:?}"),
		}
	}
}

#[derive(Clone, PartialEq)]
pub struct Score {
	pub holder: EntityTarget,
	pub objective: Identifier,
}

impl Score {
	pub fn new(holder: EntityTarget, objective: Identifier) -> Self {
		Self { holder, objective }
	}

	pub fn is_value_eq(&self, other: &Self) -> bool {
		self.holder.is_value_eq(&other.holder) && self.objective == other.objective
	}
}

impl Debug for Score {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{:?} {}", self.holder, self.objective)
	}
}

#[derive(Clone, PartialEq, Eq, Default)]
pub enum DataPath {
	#[default]
	This,
	String(String),
	Access(Box<DataPath>, String),
	Index(Box<DataPath>, ArraySize),
}

impl Debug for DataPath {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::This => write!(f, "this"),
			Self::String(string) => write!(f, "{string}"),
			Self::Access(path, prop) => write!(f, "{path:?}.{prop}"),
			Self::Index(path, idx) => write!(f, "{path:?}[{idx}]"),
		}
	}
}

#[derive(Debug, Clone, PartialEq)]
pub enum DataLocation {
	Block(IntCoordinates),
	Entity(EntityTarget),
	Storage(ResourceLocation),
}

impl DataLocation {
	pub fn is_value_eq(&self, other: &Self) -> bool {
		matches!((self, other), (Self::Block(l), Self::Block(r)) if l.is_value_eq(r))
			|| matches!((self, other), (Self::Entity(l), Self::Entity(r)) if l.is_value_eq(r))
			|| matches!((self, other), (Self::Storage(l), Self::Storage(r)) if l == r)
	}
}

#[derive(Debug, Clone, PartialEq)]
pub struct FullDataLocation {
	pub loc: DataLocation,
	pub path: DataPath,
}

impl FullDataLocation {
	pub fn is_value_eq(&self, other: &Self) -> bool {
		self.loc.is_value_eq(&other.loc) && self.path == other.path
	}
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum XPValue {
	Points,
	Levels,
}

impl XPValue {
	pub fn parse(string: &str) -> Option<Self> {
		match string {
			"points" => Some(Self::Points),
			"levels" => Some(Self::Levels),
			_ => None,
		}
	}
}

impl Display for XPValue {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(
			f,
			"{}",
			match self {
				Self::Points => "points",
				Self::Levels => "levels",
			}
		)
	}
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Difficulty {
	Peaceful,
	Easy,
	Normal,
	Hard,
}

impl Display for Difficulty {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(
			f,
			"{}",
			match self {
				Self::Peaceful => "peaceful",
				Self::Easy => "easy",
				Self::Normal => "normal",
				Self::Hard => "hard",
			}
		)
	}
}

impl Difficulty {
	pub fn parse(string: &str) -> Option<Self> {
		match string {
			"peaceful" => Some(Self::Peaceful),
			"easy" => Some(Self::Easy),
			"normal" => Some(Self::Normal),
			"hard" => Some(Self::Hard),
			_ => None,
		}
	}
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Gamemode {
	Survival,
	Creative,
	Adventure,
	Spectator,
}

impl Display for Gamemode {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(
			f,
			"{}",
			match self {
				Self::Survival => "survival",
				Self::Creative => "creative",
				Self::Adventure => "adventure",
				Self::Spectator => "spectator",
			}
		)
	}
}

#[derive(Clone, PartialEq, Eq)]
pub enum Heightmap {
	WorldSurface,
	MotionBlocking,
	MotionBlockingNoLeaves,
	OceanFloor,
}

impl Debug for Heightmap {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::WorldSurface => write!(f, "world_surface"),
			Self::MotionBlocking => write!(f, "motion_blocking"),
			Self::MotionBlockingNoLeaves => write!(f, "motion_blocking_no_leaves"),
			Self::OceanFloor => write!(f, "ocean_floor"),
		}
	}
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Weather {
	Clear,
	Rain,
	Thunder,
}

impl Display for Weather {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(
			f,
			"{}",
			match self {
				Self::Clear => "clear",
				Self::Rain => "rain",
				Self::Thunder => "thunder",
			}
		)
	}
}

#[derive(Clone, PartialEq, Eq)]
pub enum DatapackPriority {
	First,
	Last,
}

impl Debug for DatapackPriority {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::First => write!(f, "first"),
			Self::Last => write!(f, "last"),
		}
	}
}

#[derive(Clone, PartialEq, Eq)]
pub enum DatapackOrder {
	Before,
	After,
}

impl Debug for DatapackOrder {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Before => write!(f, "before"),
			Self::After => write!(f, "after"),
		}
	}
}

#[derive(Clone, PartialEq, Eq)]
pub enum DatapackListMode {
	All,
	Available,
	Enabled,
}

impl Debug for DatapackListMode {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::All => write!(f, "all"),
			Self::Available => write!(f, "available"),
			Self::Enabled => write!(f, "enabled"),
		}
	}
}

#[derive(Clone, PartialEq, Eq)]
pub enum Location {
	Structure,
	Biome,
	POI,
}

impl Location {
	pub fn parse(string: &str) -> Option<Self> {
		match string {
			"struct" => Some(Self::Structure),
			"bio" => Some(Self::Biome),
			"poi" => Some(Self::POI),
			_ => None,
		}
	}
}

impl Debug for Location {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(
			f,
			"{}",
			match self {
				Self::Structure => "structure",
				Self::Biome => "biome",
				Self::POI => "poi",
			}
		)
	}
}

#[derive(Clone, PartialEq, Eq)]
pub enum SoundSource {
	Master,
	Music,
	Record,
	Weather,
	Block,
	Hostile,
	Neutral,
	Player,
	Ambient,
	Voice,
}

impl SoundSource {
	pub fn parse(string: &str) -> Option<Self> {
		match string {
			"master" => Some(Self::Master),
			"music" => Some(Self::Music),
			"record" => Some(Self::Record),
			"weather" => Some(Self::Weather),
			"block" => Some(Self::Block),
			"hostile" => Some(Self::Hostile),
			"neutral" => Some(Self::Neutral),
			"player" => Some(Self::Player),
			"ambient" => Some(Self::Ambient),
			"voice" => Some(Self::Voice),
			_ => None,
		}
	}
}

impl Debug for SoundSource {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(
			f,
			"{}",
			match self {
				Self::Master => "master",
				Self::Music => "music",
				Self::Record => "record",
				Self::Weather => "weather",
				Self::Block => "block",
				Self::Hostile => "hostile",
				Self::Neutral => "neutral",
				Self::Player => "player",
				Self::Ambient => "ambient",
				Self::Voice => "voice",
			}
		)
	}
}
