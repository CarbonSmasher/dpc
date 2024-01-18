use num_traits::Num;
use rustc_hash::FxHashSet;

use crate::common::mc::instr::MinecraftInstr;
use crate::common::mc::item::{ItemModifyLocation, LootSource};
use crate::common::mc::modifier::Modifier;
use crate::common::mc::pos::{Coordinates, Coordinates2D};
use crate::common::mc::{DataLocation, EntityTarget};
use crate::common::val::{MutableNBTValue, MutableScoreValue, NBTValue, ScoreValue};
use crate::lir::LIRInstrKind;
use crate::util::GetSetOwned;

pub mod merge;
pub mod null;
pub mod simplify;

/// The aspects of the game that a modifier can modify. Can be used
/// to model modifier and command dependencies and relationships
/// to perform certain optimizations
#[derive(Eq, PartialEq, Hash, Clone, Debug)]
pub enum ModifierContext {
	/// Every context. Used as a fallback for unknown context
	Everything,
	/// How many times the modifiers afterwards are run
	Repetition,
	/// Who is running the command
	Executor,
	/// Position in the world
	Position,
	/// Rotation in the world
	Rotation,
	/// The execution dimension
	Dimension,
}

/// Newtype thing for some traits
#[derive(Eq, PartialEq, Hash, Clone, Debug)]
pub struct Dependency(pub ModifierContext);

impl Dependency {
	pub fn to_modified(self) -> Modified {
		Modified(self.0)
	}
}

/// Newtype thing for some traits
#[derive(Eq, PartialEq, Hash, Clone, Debug)]
pub struct Modified(pub ModifierContext);

impl Modified {
	pub fn to_dep(self) -> Dependency {
		Dependency(self.0)
	}
}

impl GetSetOwned<Dependency> for Modifier {
	fn append_set(&self, set: &mut FxHashSet<Dependency>) {
		match self {
			Modifier::As(tgt) => {
				tgt.append_set(set);
			}
			Modifier::PositionedAs(tgt) | Modifier::RotatedAs(tgt) => {
				tgt.append_set(set);
				set.insert(Dependency(ModifierContext::Rotation));
			}
			Modifier::On(..) => {
				set.insert(Dependency(ModifierContext::Executor));
			}
			Modifier::At(..) | Modifier::Anchored(..) => {
				set.insert(Dependency(ModifierContext::Position));
				set.insert(Dependency(ModifierContext::Rotation));
			}
			Modifier::Positioned(pos) => {
				pos.append_set(set);
			}
			Modifier::Align(..) => {
				set.insert(Dependency(ModifierContext::Position));
			}
			Modifier::FacingEntity(..)
			| Modifier::FacingPosition(..)
			| Modifier::PositionedOver(..) => {
				set.insert(Dependency(ModifierContext::Position));
			}
			Modifier::If { .. } => {
				set.insert(Dependency(ModifierContext::Everything));
			}
			Modifier::In(..) => {}
			_ => {
				set.insert(Dependency(ModifierContext::Everything));
			}
		}
	}
}

impl GetSetOwned<Modified> for Modifier {
	fn append_set(&self, set: &mut FxHashSet<Modified>) {
		match self {
			Modifier::As(tgt) => {
				set.insert(Modified(ModifierContext::Executor));
				if let EntityTarget::Selector(sel) = tgt {
					if !sel.is_single_type() {
						set.insert(Modified(ModifierContext::Repetition));
					}
				}
			}
			Modifier::At(..) | Modifier::Align(..) | Modifier::Anchored(..) | Modifier::On(..) => {
				set.insert(Modified(ModifierContext::Position));
				set.insert(Modified(ModifierContext::Rotation));
			}
			Modifier::In(..) => {
				set.insert(Modified(ModifierContext::Position));
				set.insert(Modified(ModifierContext::Rotation));
				set.insert(Modified(ModifierContext::Dimension));
			}
			Modifier::PositionedAs(..)
			| Modifier::Positioned(..)
			| Modifier::PositionedOver(..) => {
				set.insert(Modified(ModifierContext::Position));
			}
			Modifier::RotatedAs(..)
			| Modifier::FacingEntity(..)
			| Modifier::FacingPosition(..)
			| Modifier::Rotated(..) => {
				set.insert(Modified(ModifierContext::Position));
			}
			_ => {
				set.insert(Modified(ModifierContext::Everything));
			}
		}
	}
}

impl GetSetOwned<Dependency> for LIRInstrKind {
	fn append_set(&self, set: &mut FxHashSet<Dependency>) {
		let mut depend_repetition = true;
		match self {
			LIRInstrKind::AddScore(l, r)
			| LIRInstrKind::SubScore(l, r)
			| LIRInstrKind::MulScore(l, r)
			| LIRInstrKind::DivScore(l, r) => {
				l.append_set(set);
				r.append_set(set);
			}
			LIRInstrKind::SwapScore(l, r) => {
				l.append_set(set);
				r.append_set(set);
			}
			LIRInstrKind::MinScore(l, r)
			| LIRInstrKind::MaxScore(l, r)
			| LIRInstrKind::SetScore(l, r)
			| LIRInstrKind::ModScore(l, r) => {
				l.append_set(set);
				r.append_set(set);
				// Repeated assignments to the same value wont change anything
				depend_repetition = false;
			}
			LIRInstrKind::SetData(l, r) => {
				l.append_set(set);
				r.append_set(set);
				// Repeated assignments to the same value wont change anything
				depend_repetition = false;
			}
			LIRInstrKind::ResetScore(v) => {
				v.append_set(set);
				depend_repetition = false;
			}
			LIRInstrKind::RemoveData(v) => {
				v.append_set(set);
				// We can't remove the repetition dependency because
				// we might be removing from a list index which will change
			}
			LIRInstrKind::GetConst(..)
			| LIRInstrKind::Comment(..)
			| LIRInstrKind::NoOp
			| LIRInstrKind::ReturnValue(..)
			| LIRInstrKind::Use(..) => {}
			LIRInstrKind::GetData(v, _) => {
				v.append_set(set);
			}
			LIRInstrKind::GetScore(v) => {
				v.append_set(set);
			}
			LIRInstrKind::InsertData(l, r, _)
			| LIRInstrKind::PushData(l, r)
			| LIRInstrKind::PushFrontData(l, r)
			| LIRInstrKind::MergeData(l, r) => {
				l.append_set(set);
				r.append_set(set);
			}
			LIRInstrKind::Call(..)
			| LIRInstrKind::Command(..)
			| LIRInstrKind::ReturnFail
			| LIRInstrKind::ReturnRun(..) => {
				set.insert(Dependency(ModifierContext::Everything));
				depend_repetition = false;
			}
			LIRInstrKind::MC(instr) => {
				instr.append_set(set);
				// The Minecraft instr will handle this
				depend_repetition = false;
			}
		}

		if depend_repetition {
			set.insert(Dependency(ModifierContext::Repetition));
		}
	}
}

impl GetSetOwned<Dependency> for MinecraftInstr {
	fn append_set(&self, set: &mut FxHashSet<Dependency>) {
		let mut depend_repetition = true;
		match self {
			MinecraftInstr::WhitelistReload
			| MinecraftInstr::Say { .. }
			| MinecraftInstr::GetTime { .. }
			| MinecraftInstr::GetGamerule { .. }
			| MinecraftInstr::WorldBorderAdd { .. }
			| MinecraftInstr::Reload
			| MinecraftInstr::AddScoreboardObjective { .. }
			| MinecraftInstr::BanIP { .. }
			| MinecraftInstr::PardonIP { .. }
			| MinecraftInstr::RemoveScoreboardObjective { .. }
			| MinecraftInstr::SetDatapackOrder { .. }
			| MinecraftInstr::SetDatapackPriority { .. }
			| MinecraftInstr::AddTime { .. }
			| MinecraftInstr::StopServer
			| MinecraftInstr::StopSound
			| MinecraftInstr::ListDatapacks { .. }
			| MinecraftInstr::WorldBorderSet { .. } => {}
			MinecraftInstr::Banlist
			| MinecraftInstr::GetDifficulty
			| MinecraftInstr::WhitelistList
			| MinecraftInstr::WhitelistOff
			| MinecraftInstr::WhitelistOn
			| MinecraftInstr::Seed
			| MinecraftInstr::WorldBorderGet
			| MinecraftInstr::DisableDatapack { .. }
			| MinecraftInstr::EnableDatapack { .. }
			| MinecraftInstr::WorldBorderBuffer { .. }
			| MinecraftInstr::WorldBorderDamage { .. }
			| MinecraftInstr::WorldBorderWarningDistance { .. }
			| MinecraftInstr::WorldBorderWarningTime { .. }
			| MinecraftInstr::DefaultGamemode { .. }
			| MinecraftInstr::ListPlayerUUIDs
			| MinecraftInstr::ListPlayers
			| MinecraftInstr::ListScoreboardObjectives
			| MinecraftInstr::SetGameruleBool { .. }
			| MinecraftInstr::SetGameruleInt { .. }
			| MinecraftInstr::SetTime { .. }
			| MinecraftInstr::SetTimePreset { .. }
			| MinecraftInstr::SetDifficulty { .. } => {
				depend_repetition = false;
			}
			MinecraftInstr::AddTag { target, .. }
			| MinecraftInstr::Kill { target }
			| MinecraftInstr::AddXP { target, .. }
			| MinecraftInstr::SetXP { target, .. }
			| MinecraftInstr::GetXP { target, .. }
			| MinecraftInstr::ListTags { target }
			| MinecraftInstr::Tell { target, .. }
			| MinecraftInstr::RemoveTag { target, .. }
			| MinecraftInstr::GiveEffect { target, .. }
			| MinecraftInstr::AddAttributeModifier { target, .. }
			| MinecraftInstr::GetAttribute { target, .. }
			| MinecraftInstr::GetAttributeBase { target, .. }
			| MinecraftInstr::GetAttributeModifier { target, .. }
			| MinecraftInstr::SetAttributeBase { target, .. }
			| MinecraftInstr::RemoveAttributeModifier { target, .. }
			| MinecraftInstr::ClearEffect { target, .. }
			| MinecraftInstr::Enchant { target, .. }
			| MinecraftInstr::RideDismount { target }
			| MinecraftInstr::SetGamemode { target, .. }
			| MinecraftInstr::GiveItem { target, .. } => {
				target.append_set(set);
			}
			MinecraftInstr::BanPlayers { targets, .. }
			| MinecraftInstr::PardonPlayers { targets }
			| MinecraftInstr::ClearItems { targets, .. }
			| MinecraftInstr::Op { targets }
			| MinecraftInstr::Deop { targets }
			| MinecraftInstr::Kick { targets, .. }
			| MinecraftInstr::WhitelistAdd { targets }
			| MinecraftInstr::WhitelistRemove { targets } => {
				for tgt in targets {
					tgt.append_set(set);
				}
			}
			MinecraftInstr::SetSpawnpoint { targets, pos, .. } => {
				for tgt in targets {
					tgt.append_set(set);
				}
				pos.append_set(set);
			}
			MinecraftInstr::SetWorldSpawn { pos, .. }
			| MinecraftInstr::WorldBorderCenter { pos } => {
				pos.append_set(set);
			}
			MinecraftInstr::SummonEntity { pos, .. } => {
				pos.append_set(set);
			}
			MinecraftInstr::TeleportToLocation { source, dest } => {
				source.append_set(set);
				dest.append_set(set);
			}
			MinecraftInstr::TeleportWithRotation {
				source,
				dest,
				rotation,
			} => {
				source.append_set(set);
				dest.append_set(set);
				rotation.append_set(set);
			}
			MinecraftInstr::TeleportFacingEntity {
				source,
				dest,
				facing,
			} => {
				source.append_set(set);
				dest.append_set(set);
				facing.append_set(set);
			}
			MinecraftInstr::TeleportFacingLocation {
				source,
				dest,
				facing,
			} => {
				source.append_set(set);
				dest.append_set(set);
				facing.append_set(set);
			}
			MinecraftInstr::Locate { .. } => {
				set.insert(Dependency(ModifierContext::Position));
			}
			MinecraftInstr::Fill { data } => {
				data.start.append_set(set);
				data.end.append_set(set);
			}
			MinecraftInstr::FillBiome { data } => {
				data.start.append_set(set);
				data.end.append_set(set);
			}
			MinecraftInstr::ItemModify { location, .. }
			| MinecraftInstr::ItemReplaceWith { location, .. } => {
				location.append_set(set);
			}
			MinecraftInstr::ItemReplaceFrom { dest, source, .. } => {
				dest.append_set(set);
				source.append_set(set);
			}
			MinecraftInstr::Clone { data } => {
				data.start.append_set(set);
				data.end.append_set(set);
				data.destination.append_set(set);
				set.insert(Dependency(ModifierContext::Position));
			}
			MinecraftInstr::SpreadPlayers { center, target, .. } => {
				center.append_set(set);
				target.append_set(set);
			}
			MinecraftInstr::TeamMessage { .. }
			| MinecraftInstr::Me { .. }
			| MinecraftInstr::SpectateStop
			| MinecraftInstr::TriggerAdd { .. }
			| MinecraftInstr::TriggerSet { .. } => {
				set.insert(Dependency(ModifierContext::Executor));
			}
			MinecraftInstr::LootGive {
				player: target,
				source,
			}
			| MinecraftInstr::LootReplaceEntity { target, source, .. } => {
				target.append_set(set);
				source.append_set(set);
			}
			MinecraftInstr::LootInsert { pos, source }
			| MinecraftInstr::LootReplaceBlock { pos, source, .. } => {
				pos.append_set(set);
				source.append_set(set);
			}
			MinecraftInstr::LootSpawn { pos, source } => {
				pos.append_set(set);
				source.append_set(set);
			}
			MinecraftInstr::PlaceFeature { pos, .. }
			| MinecraftInstr::PlaceJigsaw { pos, .. }
			| MinecraftInstr::PlaceStructure { pos, .. } => {
				pos.append_set(set);
			}
			MinecraftInstr::PlaySound { target, pos, .. } => {
				target.append_set(set);
				pos.append_set(set);
			}
			MinecraftInstr::RideMount { target, vehicle }
			| MinecraftInstr::Spectate {
				target,
				spectator: vehicle,
			}
			| MinecraftInstr::TeleportToEntity {
				source: target,
				dest: vehicle,
			} => {
				target.append_set(set);
				vehicle.append_set(set);
			}
			MinecraftInstr::SetBlock { data } => {
				data.pos.append_set(set);
			}
			MinecraftInstr::SetWeather { .. } => {
				set.insert(Dependency(ModifierContext::Dimension));
			}
		}

		if depend_repetition {
			set.insert(Dependency(ModifierContext::Repetition));
		}
	}
}

impl GetSetOwned<Dependency> for ScoreValue {
	fn append_set(&self, set: &mut FxHashSet<Dependency>) {
		match self {
			ScoreValue::Constant(..) => {}
			ScoreValue::Mutable(val) => val.append_set(set),
		}
	}
}

impl GetSetOwned<Dependency> for MutableScoreValue {
	fn append_set(&self, set: &mut FxHashSet<Dependency>) {
		match self {
			MutableScoreValue::Arg(..)
			| MutableScoreValue::Reg(..)
			| MutableScoreValue::CallArg(..)
			| MutableScoreValue::CallReturnValue(..)
			| MutableScoreValue::ReturnValue(..) => {}
			MutableScoreValue::Score(sco) => {
				sco.holder.append_set(set);
			}
		}
	}
}

impl GetSetOwned<Dependency> for NBTValue {
	fn append_set(&self, set: &mut FxHashSet<Dependency>) {
		match self {
			NBTValue::Constant(..) => {}
			NBTValue::Mutable(val) => val.append_set(set),
		}
	}
}

impl GetSetOwned<Dependency> for MutableNBTValue {
	fn append_set(&self, set: &mut FxHashSet<Dependency>) {
		match self {
			MutableNBTValue::Arg(..)
			| MutableNBTValue::Reg(..)
			| MutableNBTValue::CallArg(..)
			| MutableNBTValue::CallReturnValue(..)
			| MutableNBTValue::ReturnValue(..) => {}
			MutableNBTValue::Index(val, ..) | MutableNBTValue::Property(val, ..) => {
				val.append_set(set);
			}
			MutableNBTValue::Data(loc) => match &loc.loc {
				DataLocation::Block(pos) => pos.append_set(set),
				DataLocation::Entity(tgt) => tgt.append_set(set),
				DataLocation::Storage(..) => {}
			},
		}
	}
}

impl GetSetOwned<Dependency> for EntityTarget {
	fn append_set(&self, set: &mut FxHashSet<Dependency>) {
		if self.relies_on_executor() {
			set.insert(Dependency(ModifierContext::Executor));
		}
		if self.relies_on_position() {
			set.insert(Dependency(ModifierContext::Position));
		}
	}
}

impl<T: Num> GetSetOwned<Dependency> for Coordinates<T> {
	fn append_set(&self, set: &mut FxHashSet<Dependency>) {
		set.insert(Dependency(ModifierContext::Dimension));
		match self {
			Self::Local(..) => {
				set.insert(Dependency(ModifierContext::Position));
				set.insert(Dependency(ModifierContext::Rotation));
			}
			Self::XYZ(x, y, z) => {
				if x.is_rel() || y.is_rel() || z.is_rel() {
					set.insert(Dependency(ModifierContext::Position));
				}
			}
		}
	}
}

impl<T: Num> GetSetOwned<Dependency> for Coordinates2D<T> {
	fn append_set(&self, set: &mut FxHashSet<Dependency>) {
		set.insert(Dependency(ModifierContext::Dimension));
		if self.0.is_rel() || self.1.is_rel() {
			set.insert(Dependency(ModifierContext::Position));
		}
	}
}

impl GetSetOwned<Dependency> for ItemModifyLocation {
	fn append_set(&self, set: &mut FxHashSet<Dependency>) {
		match self {
			ItemModifyLocation::Block(pos) => pos.append_set(set),
			ItemModifyLocation::Entity(tgt) => tgt.append_set(set),
		}
	}
}

impl GetSetOwned<Dependency> for LootSource {
	fn append_set(&self, set: &mut FxHashSet<Dependency>) {
		match self {
			Self::Fish { pos, .. } | Self::Mine { pos, .. } => {
				pos.append_set(set);
			}
			Self::Kill { target } => {
				target.append_set(set);
			}
			Self::Loot { .. } => {}
		}
	}
}
