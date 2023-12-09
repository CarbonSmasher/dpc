use crate::common::{val::MutableValue, DeclareBinding};
use crate::ir::{InstrKind, IR};
use crate::mir::{MIRBlock, MIRInstrKind, MIRInstruction, MIR};

use anyhow::anyhow;

macro_rules! lower {
	($mir_block:expr, $kind:ident) => {
		lower!($mir_block, MIRInstrKind::$kind)
	};

	($mir_block:expr, $kind:ident, $($arg:ident),+) => {
		lower!($mir_block, MIRInstrKind::$kind {$($arg),+})
	};

	($mir_block:expr, $val:expr) => {
		$mir_block
			.contents
			.push(MIRInstruction::new($val))
	};
}

/// Lower IR to MIR
pub fn lower_ir(mut ir: IR) -> anyhow::Result<MIR> {
	let mut mir = MIR::with_capacity(ir.functions.len(), ir.blocks.count());

	for (interface, block) in ir.functions {
		let block = ir
			.blocks
			.remove(&block)
			.ok_or(anyhow!("Block does not exist"))?;
		let mut mir_block = MIRBlock::with_capacity(block.contents.len());

		for ir_instr in block.contents {
			match ir_instr.kind {
				InstrKind::Declare { left, ty, right } => {
					let left_clone = left.clone();
					lower!(mir_block, Declare, left, ty);
					lower!(
						mir_block,
						MIRInstrKind::Assign {
							left: MutableValue::Register(left_clone),
							right,
						}
					);
				}
				InstrKind::Assign { left, right } => {
					lower!(
						mir_block,
						MIRInstrKind::Assign {
							left,
							right: DeclareBinding::Value(right),
						}
					);
				}
				InstrKind::Add { left, right } => lower!(mir_block, Add, left, right),
				InstrKind::Sub { left, right } => lower!(mir_block, Sub, left, right),
				InstrKind::Mul { left, right } => lower!(mir_block, Mul, left, right),
				InstrKind::Div { left, right } => lower!(mir_block, Div, left, right),
				InstrKind::Mod { left, right } => lower!(mir_block, Mod, left, right),
				InstrKind::Min { left, right } => lower!(mir_block, Min, left, right),
				InstrKind::Max { left, right } => lower!(mir_block, Max, left, right),
				InstrKind::Swap { left, right } => lower!(mir_block, Swap, left, right),
				InstrKind::Abs { val } => lower!(mir_block, Abs, val),
				InstrKind::Pow { base, exp } => lower!(mir_block, Pow, base, exp),
				InstrKind::Get { value } => lower!(mir_block, Get, value),
				InstrKind::Merge { left, right } => lower!(mir_block, Merge, left, right),
				InstrKind::Push { left, right } => lower!(mir_block, Push, left, right),
				InstrKind::PushFront { left, right } => lower!(mir_block, PushFront, left, right),
				InstrKind::Insert { left, right, index } => {
					lower!(mir_block, Insert, left, right, index)
				}
				InstrKind::Use { val } => lower!(mir_block, Use, val),
				InstrKind::Call { call } => lower!(mir_block, Call, call),
				InstrKind::Say { message } => lower!(mir_block, Say, message),
				InstrKind::Tell { target, message } => {
					lower!(mir_block, Tell, target, message)
				}
				InstrKind::Me { message } => {
					lower!(mir_block, Me, message)
				}
				InstrKind::TeamMessage { message } => {
					lower!(mir_block, TeamMessage, message)
				}
				InstrKind::BanPlayers { targets, reason } => {
					lower!(mir_block, BanPlayers, targets, reason)
				}
				InstrKind::BanIP { target, reason } => {
					lower!(mir_block, BanIP, target, reason)
				}
				InstrKind::PardonPlayers { targets } => {
					lower!(mir_block, PardonPlayers, targets)
				}
				InstrKind::PardonIP { target } => {
					lower!(mir_block, PardonIP, target)
				}
				InstrKind::Op { targets } => {
					lower!(mir_block, Op, targets)
				}
				InstrKind::Deop { targets } => {
					lower!(mir_block, Deop, targets)
				}
				InstrKind::WhitelistAdd { targets } => {
					lower!(mir_block, WhitelistAdd, targets)
				}
				InstrKind::WhitelistRemove { targets } => {
					lower!(mir_block, WhitelistRemove, targets)
				}
				InstrKind::Kick { targets, reason } => {
					lower!(mir_block, Kick, targets, reason)
				}
				InstrKind::SetDifficulty { difficulty } => {
					lower!(mir_block, SetDifficulty, difficulty)
				}
				InstrKind::ListPlayers => lower!(mir_block, ListPlayers),
				InstrKind::Seed => lower!(mir_block, Seed),
				InstrKind::Banlist => lower!(mir_block, Banlist),
				InstrKind::WhitelistList => lower!(mir_block, WhitelistList),
				InstrKind::WhitelistOn => lower!(mir_block, WhitelistOn),
				InstrKind::WhitelistOff => lower!(mir_block, WhitelistOff),
				InstrKind::WhitelistReload => lower!(mir_block, WhitelistReload),
				InstrKind::StopServer => lower!(mir_block, StopServer),
				InstrKind::StopSound => lower!(mir_block, StopSound),
				InstrKind::GetDifficulty => lower!(mir_block, GetDifficulty),
				InstrKind::Publish => lower!(mir_block, Publish),
				InstrKind::Enchant {
					target,
					enchantment,
					level,
				} => {
					lower!(mir_block, Enchant, target, enchantment, level)
				}
				InstrKind::Kill { target } => lower!(mir_block, Kill, target),
				InstrKind::Reload => lower!(mir_block, Reload),
				InstrKind::SetXP {
					target,
					amount,
					value,
				} => lower!(mir_block, SetXP, target, amount, value),
				InstrKind::SetBlock { data } => lower!(mir_block, SetBlock, data),
				InstrKind::Fill { data } => lower!(mir_block, Fill, data),
				InstrKind::Clone { data } => lower!(mir_block, Clone, data),
				InstrKind::SetWeather { weather, duration } => {
					lower!(mir_block, SetWeather, weather, duration)
				}
				InstrKind::AddTime { time } => lower!(mir_block, AddTime, time),
				InstrKind::SetTime { time } => lower!(mir_block, SetTime, time),
				InstrKind::SetTimePreset { time } => lower!(mir_block, SetTimePreset, time),
				InstrKind::GetTime { query } => lower!(mir_block, GetTime, query),
				InstrKind::AddTag { target, tag } => lower!(mir_block, AddTag, target, tag),
				InstrKind::RemoveTag { target, tag } => lower!(mir_block, RemoveTag, target, tag),
				InstrKind::ListTags { target } => lower!(mir_block, ListTags, target),
				InstrKind::RideMount { target, vehicle } => {
					lower!(mir_block, RideMount, target, vehicle)
				}
				InstrKind::RideDismount { target } => lower!(mir_block, RideDismount, target),
				InstrKind::FillBiome { data } => lower!(mir_block, FillBiome, data),
				InstrKind::Spectate { target, spectator } => {
					lower!(mir_block, Spectate, target, spectator)
				}
				InstrKind::SpectateStop => lower!(mir_block, SpectateStop),
				InstrKind::SetGamemode { target, gamemode } => {
					lower!(mir_block, SetGamemode, target, gamemode)
				}
				InstrKind::DefaultGamemode { gamemode } => {
					lower!(mir_block, DefaultGamemode, gamemode)
				}
				InstrKind::TeleportToEntity { source, dest } => {
					lower!(mir_block, TeleportToEntity, source, dest)
				}
				InstrKind::TeleportToLocation { source, dest } => {
					lower!(mir_block, TeleportToLocation, source, dest)
				}
				InstrKind::TeleportWithRotation {
					source,
					dest,
					rotation,
				} => {
					lower!(mir_block, TeleportWithRotation, source, dest, rotation)
				}
				InstrKind::TeleportFacingLocation {
					source,
					dest,
					facing,
				} => {
					lower!(mir_block, TeleportFacingLocation, source, dest, facing)
				}
				InstrKind::TeleportFacingEntity {
					source,
					dest,
					facing,
				} => {
					lower!(mir_block, TeleportFacingEntity, source, dest, facing)
				}
				InstrKind::GiveItem {
					target,
					item,
					amount,
				} => {
					lower!(mir_block, GiveItem, target, item, amount)
				}
				InstrKind::AddScoreboardObjective {
					objective,
					criterion,
					display_name,
				} => {
					lower!(
						mir_block,
						AddScoreboardObjective,
						objective,
						criterion,
						display_name
					)
				}
				InstrKind::RemoveScoreboardObjective { objective } => {
					lower!(mir_block, RemoveScoreboardObjective, objective)
				}
				InstrKind::ListScoreboardObjectives => {
					lower!(mir_block, ListScoreboardObjectives)
				}
				InstrKind::TriggerAdd { objective, amount } => {
					lower!(mir_block, TriggerAdd, objective, amount)
				}
				InstrKind::TriggerSet { objective, amount } => {
					lower!(mir_block, TriggerSet, objective, amount)
				}
			}
		}

		let id = mir.blocks.add(mir_block);
		mir.functions.insert(interface, id);
	}

	Ok(mir)
}
