use crate::common::function::Function;
use crate::common::{val::MutableValue, DeclareBinding};
use crate::ir::{InstrKind, IR};
use crate::mir::{MIRBlock, MIRInstrKind, MIRInstruction, MIR};

use anyhow::{anyhow, Context};

/// Lower IR to MIR
pub fn lower_ir(mut ir: IR) -> anyhow::Result<MIR> {
	let mut mir = MIR::with_capacity(ir.functions.len(), ir.blocks.count());

	for (func_id, func) in ir.functions {
		let block = ir
			.blocks
			.remove(&func.block)
			.ok_or(anyhow!("Block does not exist"))?;
		let mut mir_block = MIRBlock::with_capacity(block.contents.len());

		for ir_instr in block.contents {
			let (prelude, instr) =
				lower_kind(ir_instr.kind).context("Failed to lower instruction")?;
			mir_block.contents.extend(prelude);
			mir_block.contents.push(MIRInstruction::new(instr));
		}

		let block = mir.blocks.add(mir_block);
		mir.functions.insert(
			func_id,
			Function {
				interface: func.interface,
				block,
			},
		);
	}

	Ok(mir)
}

macro_rules! lower {
	($kind:ident) => {
		MIRInstrKind::$kind
	};

	($kind:ident, $($arg:ident),+) => {
		MIRInstrKind::$kind {$($arg),+}
	};

	($kind:expr) => {
		$kind
	}
}

fn lower_kind(kind: InstrKind) -> anyhow::Result<(Vec<MIRInstruction>, MIRInstrKind)> {
	let mut prelude = Vec::new();
	let kind = match kind {
		InstrKind::Declare { left, ty, right } => {
			let left_clone = left.clone();
			prelude.push(MIRInstruction::new(lower!(Declare, left, ty)));
			lower!(MIRInstrKind::Assign {
				left: MutableValue::Register(left_clone),
				right,
			})
		}
		InstrKind::Assign { left, right } => {
			lower!(MIRInstrKind::Assign {
				left,
				right: DeclareBinding::Value(right),
			})
		}
		InstrKind::Add { left, right } => lower!(Add, left, right),
		InstrKind::Sub { left, right } => lower!(Sub, left, right),
		InstrKind::Mul { left, right } => lower!(Mul, left, right),
		InstrKind::Div { left, right } => lower!(Div, left, right),
		InstrKind::Mod { left, right } => lower!(Mod, left, right),
		InstrKind::Min { left, right } => lower!(Min, left, right),
		InstrKind::Max { left, right } => lower!(Max, left, right),
		InstrKind::Swap { left, right } => lower!(Swap, left, right),
		InstrKind::Remove { val } => lower!(Remove, val),
		InstrKind::Abs { val } => lower!(Abs, val),
		InstrKind::Pow { base, exp } => lower!(Pow, base, exp),
		InstrKind::Get { value, scale } => lower!(Get, value, scale),
		InstrKind::Merge { left, right } => lower!(Merge, left, right),
		InstrKind::Push { left, right } => lower!(Push, left, right),
		InstrKind::PushFront { left, right } => lower!(PushFront, left, right),
		InstrKind::Insert { left, right, index } => {
			lower!(Insert, left, right, index)
		}
		InstrKind::Use { val } => lower!(Use, val),
		InstrKind::Call { call } => lower!(Call, call),
		InstrKind::CallExtern { func } => lower!(CallExtern, func),
		InstrKind::Say { message } => lower!(Say, message),
		InstrKind::Tell { target, message } => {
			lower!(Tell, target, message)
		}
		InstrKind::Me { message } => {
			lower!(Me, message)
		}
		InstrKind::TeamMessage { message } => {
			lower!(TeamMessage, message)
		}
		InstrKind::BanPlayers { targets, reason } => {
			lower!(BanPlayers, targets, reason)
		}
		InstrKind::BanIP { target, reason } => {
			lower!(BanIP, target, reason)
		}
		InstrKind::PardonPlayers { targets } => {
			lower!(PardonPlayers, targets)
		}
		InstrKind::PardonIP { target } => {
			lower!(PardonIP, target)
		}
		InstrKind::Op { targets } => {
			lower!(Op, targets)
		}
		InstrKind::Deop { targets } => {
			lower!(Deop, targets)
		}
		InstrKind::WhitelistAdd { targets } => {
			lower!(WhitelistAdd, targets)
		}
		InstrKind::WhitelistRemove { targets } => {
			lower!(WhitelistRemove, targets)
		}
		InstrKind::Kick { targets, reason } => {
			lower!(Kick, targets, reason)
		}
		InstrKind::SetDifficulty { difficulty } => {
			lower!(SetDifficulty, difficulty)
		}
		InstrKind::ListPlayers => lower!(ListPlayers),
		InstrKind::Seed => lower!(Seed),
		InstrKind::Banlist => lower!(Banlist),
		InstrKind::WhitelistList => lower!(WhitelistList),
		InstrKind::WhitelistOn => lower!(WhitelistOn),
		InstrKind::WhitelistOff => lower!(WhitelistOff),
		InstrKind::WhitelistReload => lower!(WhitelistReload),
		InstrKind::StopServer => lower!(StopServer),
		InstrKind::StopSound => lower!(StopSound),
		InstrKind::GetDifficulty => lower!(GetDifficulty),
		InstrKind::Publish => lower!(Publish),
		InstrKind::Enchant {
			target,
			enchantment,
			level,
		} => {
			lower!(Enchant, target, enchantment, level)
		}
		InstrKind::Kill { target } => lower!(Kill, target),
		InstrKind::Reload => lower!(Reload),
		InstrKind::SetXP {
			target,
			amount,
			value,
		} => lower!(SetXP, target, amount, value),
		InstrKind::AddXP {
			target,
			amount,
			value,
		} => lower!(AddXP, target, amount, value),
		InstrKind::GetXP { target, value } => lower!(GetXP, target, value),
		InstrKind::SetBlock { data } => lower!(SetBlock, data),
		InstrKind::Fill { data } => lower!(Fill, data),
		InstrKind::Clone { data } => lower!(Clone, data),
		InstrKind::SetWeather { weather, duration } => {
			lower!(SetWeather, weather, duration)
		}
		InstrKind::AddTime { time } => lower!(AddTime, time),
		InstrKind::SetTime { time } => lower!(SetTime, time),
		InstrKind::SetTimePreset { time } => lower!(SetTimePreset, time),
		InstrKind::GetTime { query } => lower!(GetTime, query),
		InstrKind::AddTag { target, tag } => lower!(AddTag, target, tag),
		InstrKind::RemoveTag { target, tag } => lower!(RemoveTag, target, tag),
		InstrKind::ListTags { target } => lower!(ListTags, target),
		InstrKind::RideMount { target, vehicle } => {
			lower!(RideMount, target, vehicle)
		}
		InstrKind::RideDismount { target } => lower!(RideDismount, target),
		InstrKind::FillBiome { data } => lower!(FillBiome, data),
		InstrKind::Spectate { target, spectator } => {
			lower!(Spectate, target, spectator)
		}
		InstrKind::SpectateStop => lower!(SpectateStop),
		InstrKind::SetGamemode { target, gamemode } => {
			lower!(SetGamemode, target, gamemode)
		}
		InstrKind::DefaultGamemode { gamemode } => {
			lower!(DefaultGamemode, gamemode)
		}
		InstrKind::TeleportToEntity { source, dest } => {
			lower!(TeleportToEntity, source, dest)
		}
		InstrKind::TeleportToLocation { source, dest } => {
			lower!(TeleportToLocation, source, dest)
		}
		InstrKind::TeleportWithRotation {
			source,
			dest,
			rotation,
		} => {
			lower!(TeleportWithRotation, source, dest, rotation)
		}
		InstrKind::TeleportFacingLocation {
			source,
			dest,
			facing,
		} => {
			lower!(TeleportFacingLocation, source, dest, facing)
		}
		InstrKind::TeleportFacingEntity {
			source,
			dest,
			facing,
		} => {
			lower!(TeleportFacingEntity, source, dest, facing)
		}
		InstrKind::GiveItem {
			target,
			item,
			amount,
		} => {
			lower!(GiveItem, target, item, amount)
		}
		InstrKind::AddScoreboardObjective {
			objective,
			criterion,
			display_name,
		} => {
			lower!(AddScoreboardObjective, objective, criterion, display_name)
		}
		InstrKind::RemoveScoreboardObjective { objective } => {
			lower!(RemoveScoreboardObjective, objective)
		}
		InstrKind::ListScoreboardObjectives => {
			lower!(ListScoreboardObjectives)
		}
		InstrKind::TriggerAdd { objective, amount } => {
			lower!(TriggerAdd, objective, amount)
		}
		InstrKind::TriggerSet { objective, amount } => {
			lower!(TriggerSet, objective, amount)
		}
		InstrKind::GetAttribute {
			target,
			attribute,
			scale,
		} => lower!(GetAttribute, target, attribute, scale),
		InstrKind::GetAttributeBase {
			target,
			attribute,
			scale,
		} => lower!(GetAttributeBase, target, attribute, scale),
		InstrKind::SetAttributeBase {
			target,
			attribute,
			value,
		} => lower!(SetAttributeBase, target, attribute, value),
		InstrKind::AddAttributeModifier {
			target,
			attribute,
			uuid,
			name,
			value,
			ty,
		} => lower!(
			AddAttributeModifier,
			target,
			attribute,
			uuid,
			name,
			value,
			ty
		),
		InstrKind::RemoveAttributeModifier {
			target,
			attribute,
			uuid,
		} => lower!(RemoveAttributeModifier, target, attribute, uuid),
		InstrKind::GetAttributeModifier {
			target,
			attribute,
			uuid,
			scale,
		} => lower!(GetAttributeModifier, target, attribute, uuid, scale),
		InstrKind::DisableDatapack { pack } => lower!(DisableDatapack, pack),
		InstrKind::EnableDatapack { pack } => lower!(EnableDatapack, pack),
		InstrKind::SetDatapackPriority { pack, priority } => {
			lower!(SetDatapackPriority, pack, priority)
		}
		InstrKind::SetDatapackOrder {
			pack,
			order,
			existing,
		} => {
			lower!(SetDatapackOrder, pack, order, existing)
		}
		InstrKind::ListDatapacks { mode } => lower!(ListDatapacks, mode),
		InstrKind::ListPlayerUUIDs => lower!(ListPlayerUUIDs),
		InstrKind::SummonEntity { entity, pos, nbt } => {
			lower!(SummonEntity, entity, pos, nbt)
		}
		InstrKind::SetWorldSpawn { pos, angle } => {
			lower!(SetWorldSpawn, pos, angle)
		}
		InstrKind::ClearItems {
			targets,
			item,
			max_count,
		} => lower!(ClearItems, targets, item, max_count),
		InstrKind::SetSpawnpoint {
			targets,
			pos,
			angle,
		} => lower!(SetSpawnpoint, targets, pos, angle),
		InstrKind::SpreadPlayers {
			center,
			spread_distance,
			max_range,
			max_height,
			respect_teams,
			target,
		} => lower!(
			SpreadPlayers,
			center,
			spread_distance,
			max_range,
			max_height,
			respect_teams,
			target
		),
		InstrKind::If { condition, body } => {
			let (new_prelude, instr) = lower_kind(*body).context("Failed to lower if body")?;
			prelude.extend(new_prelude);
			MIRInstrKind::If {
				condition,
				body: Box::new(instr),
			}
		}
		InstrKind::As { target, body } => {
			let (new_prelude, instr) = lower_kind(*body).context("Failed to lower as body")?;
			prelude.extend(new_prelude);
			MIRInstrKind::As {
				target,
				body: Box::new(instr),
			}
		}
		InstrKind::At { target, body } => {
			let (new_prelude, instr) = lower_kind(*body).context("Failed to lower at body")?;
			prelude.extend(new_prelude);
			MIRInstrKind::At {
				target,
				body: Box::new(instr),
			}
		}
		InstrKind::StoreResult { location, body } => {
			let (new_prelude, instr) = lower_kind(*body).context("Failed to lower at body")?;
			prelude.extend(new_prelude);
			MIRInstrKind::StoreResult {
				location,
				body: Box::new(instr),
			}
		}
		InstrKind::StoreSuccess { location, body } => {
			let (new_prelude, instr) = lower_kind(*body).context("Failed to lower at body")?;
			prelude.extend(new_prelude);
			MIRInstrKind::StoreSuccess {
				location,
				body: Box::new(instr),
			}
		}
		InstrKind::ReturnRun { body } => {
			let (new_prelude, instr) = lower_kind(*body).context("Failed to lower retr body")?;
			prelude.extend(new_prelude);
			MIRInstrKind::ReturnRun {
				body: Box::new(instr),
			}
		}
		InstrKind::ClearEffect { target, effect } => lower!(ClearEffect, target, effect),
		InstrKind::GiveEffect {
			target,
			effect,
			duration,
			amplifier,
			hide_particles,
		} => lower!(
			GiveEffect,
			target,
			effect,
			duration,
			amplifier,
			hide_particles
		),
		InstrKind::ReturnValue { index, value } => lower!(ReturnValue, index, value),
		InstrKind::Return { value } => lower!(Return, value),
		InstrKind::Command { command } => lower!(Command, command),
		InstrKind::Comment { comment } => lower!(Comment, comment),
		InstrKind::SetGameruleBool { rule, value } => lower!(SetGameruleBool, rule, value),
		InstrKind::SetGameruleInt { rule, value } => lower!(SetGameruleInt, rule, value),
		InstrKind::GetGamerule { rule } => lower!(GetGamerule, rule),
		InstrKind::Locate {
			location_type,
			location,
		} => lower!(Locate, location_type, location),
	};

	Ok((prelude, kind))
}
