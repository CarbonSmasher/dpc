mod imp;
mod modifier;
pub mod t;
pub mod util;

use std::collections::HashSet;

use anyhow::{anyhow, bail, Context};

use crate::common::mc::block::{CloneMaskMode, CloneMode, FillMode, SetBlockMode};
use crate::common::mc::instr::MinecraftInstr;
use crate::common::mc::modifier::{Modifier, StoreModLocation};
use crate::common::mc::scoreboard_and_teams::Criterion;
use crate::common::mc::{DatapackListMode, Score};
use crate::common::ty::NBTTypeContents;
use crate::common::val::MutableScoreValue;
use crate::common::{val::NBTValue, val::ScoreValue, RegisterList};
use crate::lir::{LIRBlock, LIRInstrKind, LIRInstruction};
use crate::output::codegen::util::cg_data_modify_rhs;
use crate::project::ProjectSettings;

use self::modifier::codegen_modifier;
use self::t::macros::cgwrite;
use self::util::{create_lit_score, get_mut_score_val_score, FloatCG, SpaceSepListCG};

use super::ra::{alloc_block_registers, RegAllocCx, RegAllocResult};
use super::strip::FunctionMapping;

use t::macros::cgformat;
pub use t::Codegen;

pub struct CodegenCx<'proj> {
	pub project: &'proj ProjectSettings,
	pub func_mapping: Option<FunctionMapping>,
	pub racx: RegAllocCx,
	pub score_literals: HashSet<i32>,
	pub requirements: HashSet<CodegenRequirement>,
}

impl<'proj> CodegenCx<'proj> {
	pub fn new(project: &'proj ProjectSettings, func_mapping: Option<FunctionMapping>) -> Self {
		Self {
			project,
			func_mapping,
			racx: RegAllocCx::new(),
			score_literals: HashSet::new(),
			requirements: HashSet::new(),
		}
	}

	pub fn add_requirement(&mut self, req: CodegenRequirement) {
		self.requirements.insert(req);
	}
}

/// Different requirements that can be imposed on the output so that it generates
/// certain functions only when necessary
#[derive(Hash, PartialEq, Eq, Clone, Copy)]
pub enum CodegenRequirement {}

pub struct CodegenBlockCx<'ccx, 'proj> {
	pub ccx: &'ccx mut CodegenCx<'proj>,
	pub ra: RegAllocResult,
	pub regs: RegisterList,
	pub func_id: String,
	pub macro_line: bool,
}

pub fn codegen_block(
	func_id: &str,
	block: &LIRBlock,
	ccx: &mut CodegenCx,
) -> anyhow::Result<Vec<String>> {
	let ra = alloc_block_registers(func_id, block, &mut ccx.racx)?;

	let mut cbcx = CodegenBlockCx {
		ccx,
		ra,
		regs: block.regs.clone(),
		func_id: func_id.into(),
		macro_line: false,
	};

	let mut out = Vec::new();
	for (i, instr) in block.contents.iter().enumerate() {
		let mut command =
			codegen_instr(instr, &mut cbcx).with_context(|| format!("At instruction {i}"))?;
		command = command.map(|x| if cbcx.macro_line { format!("${x}") } else { x });
		cbcx.macro_line = false;
		out.extend(command);
	}

	Ok(out)
}

pub fn codegen_instr(
	instr: &LIRInstruction,
	cbcx: &mut CodegenBlockCx,
) -> anyhow::Result<Option<String>> {
	let mut out = CommandBuilder::new();

	let mut modifiers = Vec::new();

	let cmd = match &instr.kind {
		LIRInstrKind::SetScore(left, right) => Some(match right {
			ScoreValue::Constant(data) => {
				let lit = data.get_literal_str();
				cgformat!(cbcx, "scoreboard players set ", left, " ", lit)?
			}
			ScoreValue::Mutable(right) => {
				cgformat!(cbcx, "scoreboard players operation ", left, " = ", right)?
			}
		}),
		LIRInstrKind::AddScore(left, right) => {
			Some(match right {
				ScoreValue::Constant(data) => {
					let lit = data.get_i32();
					// Negative signs in add/remove commands are illegal
					if lit.is_negative() {
						cgformat!(cbcx, "scoreboard players remove ", left, " ", lit.abs())?
					} else {
						cgformat!(cbcx, "scoreboard players add ", left, " ", lit)?
					}
				}
				ScoreValue::Mutable(val) => {
					cgformat!(cbcx, "scoreboard players operation ", left, " += ", val)?
				}
			})
		}
		LIRInstrKind::SubScore(left, right) => {
			Some(match right {
				ScoreValue::Constant(data) => {
					let lit = data.get_i32();
					// Negative signs in add/remove commands are illegal
					if lit.is_negative() {
						cgformat!(cbcx, "scoreboard players add ", left, " ", lit.abs())?
					} else {
						cgformat!(cbcx, "scoreboard players remove ", left, " ", lit)?
					}
				}
				ScoreValue::Mutable(val) => {
					cgformat!(cbcx, "scoreboard players operation ", left, " -= ", val)?
				}
			})
		}
		LIRInstrKind::MulScore(left, right) => Some(cgformat!(
			cbcx,
			"scoreboard players operation ",
			left,
			" *= ",
			right
		)?),
		LIRInstrKind::DivScore(left, right) => Some(cgformat!(
			cbcx,
			"scoreboard players operation ",
			left,
			" /= ",
			right
		)?),
		LIRInstrKind::ModScore(left, right) => Some(cgformat!(
			cbcx,
			"scoreboard players operation ",
			left,
			" %= ",
			right
		)?),
		LIRInstrKind::MinScore(left, right) => Some(cgformat!(
			cbcx,
			"scoreboard players operation ",
			left,
			" < ",
			right
		)?),
		LIRInstrKind::MaxScore(left, right) => Some(cgformat!(
			cbcx,
			"scoreboard players operation ",
			left,
			" > ",
			right
		)?),
		LIRInstrKind::SwapScore(left, right) => Some(cgformat!(
			cbcx,
			"scoreboard players operation ",
			left,
			" >< ",
			right
		)?),
		LIRInstrKind::ResetScore(val) => {
			// It is faster to reset the player than the score.
			// Since we use fake players for registers that only have the
			// one register objective, we can reset the whole player for
			// a performance and code size gain
			if let MutableScoreValue::Reg(..) = val {
				let score = get_mut_score_val_score(val, &cbcx.ra, &cbcx.func_id)?;
				Some(cgformat!(cbcx, "scoreboard players reset ", score.holder)?)
			} else {
				Some(cgformat!(cbcx, "scoreboard players reset ", val)?)
			}
		}
		LIRInstrKind::SetData(left, right) => {
			let rhs = cg_data_modify_rhs(cbcx, right)?;
			Some(cgformat!(cbcx, "data modify ", left, " set ", rhs)?)
		}
		LIRInstrKind::RemoveData(val) => Some(cgformat!(cbcx, "data remove ", val)?),
		LIRInstrKind::MergeData(left, right) => {
			if let NBTValue::Constant(rhs) = right {
				Some(cgformat!(
					cbcx,
					"data merge ",
					left,
					" ",
					rhs.get_literal_str()
				)?)
			} else {
				let rhs = cg_data_modify_rhs(cbcx, right)?;
				Some(cgformat!(cbcx, "data modify ", left, " merge ", rhs)?)
			}
		}
		LIRInstrKind::GetScore(val) => Some(cgformat!(cbcx, "scoreboard players get ", val)?),
		LIRInstrKind::GetData(val, scale) => {
			let mut out = String::new();
			cgwrite!(&mut out, cbcx, "data get ", val)?;
			if *scale != 1.0 {
				cgwrite!(&mut out, cbcx, " ", scale)?;
			}
			Some(out)
		}
		LIRInstrKind::GetConst(val) => {
			let lit = create_lit_score(*val);
			Some(cgformat!(cbcx, "scoreboard players get ", lit)?)
		}
		LIRInstrKind::PushData(left, right) => {
			let rhs = cg_data_modify_rhs(cbcx, right)?;
			Some(cgformat!(cbcx, "data modify ", left, " append ", rhs)?)
		}
		LIRInstrKind::PushFrontData(left, right) => {
			let rhs = cg_data_modify_rhs(cbcx, right)?;
			Some(cgformat!(cbcx, "data modify ", left, " prepend ", rhs)?)
		}
		LIRInstrKind::InsertData(left, right, i) => {
			let rhs = cg_data_modify_rhs(cbcx, right)?;
			Some(cgformat!(
				cbcx,
				"data modify ",
				left,
				" insert ",
				i,
				" ",
				rhs
			)?)
		}
		LIRInstrKind::ConstIndexToScore {
			score,
			value,
			index,
		} => Some(match value {
			NBTValue::Constant(val) => match val {
				NBTTypeContents::Arr(arr) => {
					let lit = arr
						.const_index(*index)
						.ok_or(anyhow!("Const index out of range"))?;
					cgformat!(cbcx, "scoreboard players set ", score, lit)?
				}
				_ => bail!("Cannot index non-array type"),
			},
			NBTValue::Mutable(val) => {
				modifiers.push(Modifier::StoreResult(StoreModLocation::from_mut_score_val(
					score,
				)?));
				cgformat!(cbcx, "data get storage ", val)?
			}
		}),
		LIRInstrKind::Call(fun) => {
			let mut func_id = fun;
			if let Some(mapping) = &cbcx.ccx.func_mapping {
				if let Some(new_id) = mapping.0.get(func_id) {
					func_id = new_id;
				}
			}
			Some(format!("function {func_id}"))
		}
		LIRInstrKind::ReturnValue(val) => Some(cgformat!(cbcx, "return ", val)?),
		LIRInstrKind::ReturnFail => Some("return fail".into()),
		LIRInstrKind::ReturnRun(instr) => {
			let cmd = codegen_instr(instr, cbcx)
				.context("Failed to codegen return run subinstruction")?
				.context("Return run command is missing after codegen")?;
			Some(cgformat!(cbcx, "return run ", cmd)?)
		}
		LIRInstrKind::MC(instr) => match instr {
			MinecraftInstr::Say { message } => Some(format!("say {message}")),
			MinecraftInstr::Me { message } => Some(format!("me {message}")),
			MinecraftInstr::TeamMessage { message } => Some(format!("tm {message}")),
			MinecraftInstr::Tell { target, message } => {
				Some(cgformat!(cbcx, "w ", target, " ", message)?)
			}
			MinecraftInstr::Kill { target } => {
				if target.is_blank_this() {
					Some("kill".into())
				} else {
					Some(cgformat!(cbcx, "kill ", target)?)
				}
			}
			MinecraftInstr::Reload => Some("reload".into()),
			MinecraftInstr::SetXP {
				target,
				amount,
				value,
			} => {
				// Command cannot take negative values
				let amount = if amount < &0 { 0 } else { *amount };
				Some(cgformat!(cbcx, "xp set ", target, " ", amount, " ", value)?)
			}
			MinecraftInstr::AddXP {
				target,
				amount,
				value,
			} => Some(cgformat!(cbcx, "xp add ", target, " ", amount, " ", value)?),
			MinecraftInstr::GetXP { target, value } => {
				Some(cgformat!(cbcx, "xp query ", target, " ", value)?)
			}
			MinecraftInstr::BanPlayers { targets, reason } => {
				let list = SpaceSepListCG(targets);
				if let Some(reason) = reason {
					Some(cgformat!(cbcx, "ban ", list, " ", reason)?)
				} else {
					Some(cgformat!(cbcx, "ban ", list)?)
				}
			}
			MinecraftInstr::BanIP { target, reason } => {
				if let Some(reason) = reason {
					Some(cgformat!(cbcx, "ban-ip ", target, " ", reason)?)
				} else {
					Some(cgformat!(cbcx, "ban-ip ", target)?)
				}
			}
			MinecraftInstr::PardonPlayers { targets } => {
				let list = SpaceSepListCG(targets);
				Some(cgformat!(cbcx, "pardon ", list)?)
			}
			MinecraftInstr::PardonIP { target } => Some(format!("pardon-ip {target}")),
			MinecraftInstr::Op { targets } => {
				Some(cgformat!(cbcx, "op ", SpaceSepListCG(targets))?)
			}
			MinecraftInstr::Deop { targets } => {
				Some(cgformat!(cbcx, "deop ", SpaceSepListCG(targets))?)
			}
			MinecraftInstr::WhitelistAdd { targets } => {
				Some(cgformat!(cbcx, "whitelist add ", SpaceSepListCG(targets))?)
			}
			MinecraftInstr::WhitelistRemove { targets } => Some(cgformat!(
				cbcx,
				"whitelist remove ",
				SpaceSepListCG(targets)
			)?),
			MinecraftInstr::WhitelistOn => Some("whitelist on".into()),
			MinecraftInstr::WhitelistOff => Some("whitelist off".into()),
			MinecraftInstr::WhitelistReload => Some("whitelist reload".into()),
			MinecraftInstr::WhitelistList => Some("whitelist list".into()),
			MinecraftInstr::Kick { targets, reason } => {
				let list = SpaceSepListCG(targets);
				if let Some(reason) = reason {
					Some(cgformat!(cbcx, "kick ", list, " ", reason)?)
				} else {
					Some(cgformat!(cbcx, "kick ", list)?)
				}
			}
			MinecraftInstr::Banlist => Some("banlist".into()),
			MinecraftInstr::StopSound => Some("stopsound".into()),
			MinecraftInstr::StopServer => Some("stop".into()),
			MinecraftInstr::ListPlayers => Some("list".into()),
			MinecraftInstr::Publish => Some("publish".into()),
			MinecraftInstr::Seed => Some("seed".into()),
			MinecraftInstr::GetDifficulty => Some("difficulty".into()),
			MinecraftInstr::SetDifficulty { difficulty } => {
				Some(cgformat!(cbcx, "difficulty ", difficulty)?)
			}
			MinecraftInstr::Enchant {
				target,
				enchantment,
				level,
			} => {
				if level == &1 {
					Some(cgformat!(cbcx, "enchant ", target, " ", enchantment)?)
				} else {
					Some(cgformat!(
						cbcx,
						"enchant ",
						target,
						" ",
						enchantment,
						" ",
						level
					)?)
				}
			}
			MinecraftInstr::SetBlock { data } => {
				let mut out = String::new();
				cgwrite!(&mut out, cbcx, "setblock ", data.pos, " ", data.block)?;

				// Replace mode is default and can be omitted
				if let SetBlockMode::Replace = data.mode {
				} else {
					cgwrite!(&mut out, cbcx, " ", data.mode)?;
				}
				Some(out)
			}
			MinecraftInstr::Fill { data } => {
				let mut out = String::new();
				cgwrite!(&mut out, cbcx, "fill ", data.start, " ", data.end, " ", data.block)?;

				// Replace mode is default and can be omitted if there is no filter
				if let FillMode::Replace(filter) = &data.mode {
					cgwrite!(&mut out, cbcx, "replace")?;
					if let Some(filter) = filter {
						cgwrite!(&mut out, cbcx, " ", filter)?;
					}
				} else {
					cgwrite!(&mut out, cbcx, " ", data.mode)?;
				}
				Some(out)
			}
			MinecraftInstr::Clone { data } => {
				let mut out = String::new();
				cgwrite!(&mut out, cbcx, "clone ")?;

				if let Some(source) = &data.source_dimension {
					cgwrite!(&mut out, cbcx, "from ", source)?;
				}

				cgwrite!(&mut out, cbcx, data.start, " ", data.end)?;

				if let Some(target) = &data.target_dimension {
					cgwrite!(&mut out, cbcx, "to ", target)?;
				}

				cgwrite!(&mut out, cbcx, " ", data.destination, " ")?;

				// Replace mode is default and can be omitted if there is no filter
				match &data.mask_mode {
					CloneMaskMode::Filtered(filter) => {
						cgwrite!(&mut out, cbcx, "filter ", filter)?;
					}
					CloneMaskMode::Replace => {}
					other => cgwrite!(&mut out, cbcx, " ", other)?,
				}

				// Normal mode is default and can be omitted
				if let CloneMode::Normal = data.mode {
				} else {
					cgwrite!(&mut out, cbcx, " ", data.mode)?;
				}

				Some(out)
			}
			MinecraftInstr::FillBiome { data } => {
				let mut out = String::new();
				cgwrite!(
					&mut out,
					cbcx,
					"fillbiome ",
					data.start,
					" ",
					data.end,
					" ",
					data.biome
				)?;

				if let Some(filter) = &data.replace {
					cgwrite!(&mut out, cbcx, " replace ", filter)?;
				}

				Some(out)
			}
			MinecraftInstr::SetWeather { weather, duration } => {
				if let Some(duration) = duration {
					Some(cgformat!(cbcx, "weather ", weather, " ", duration)?)
				} else {
					Some(cgformat!(cbcx, "weather ", weather)?)
				}
			}
			MinecraftInstr::AddTime { time } => Some(cgformat!(cbcx, "time add ", time)?),
			MinecraftInstr::SetTime { time } => {
				// Using the day preset is shorter than the time it represents
				if time.amount == 1000.0 {
					Some("tmie set day".into())
				} else {
					Some(cgformat!(cbcx, "time set ", time)?)
				}
			}
			MinecraftInstr::SetTimePreset { time } => Some(cgformat!(cbcx, "time set ", time)?),
			MinecraftInstr::GetTime { query } => Some(cgformat!(cbcx, "time get ", query)?),
			MinecraftInstr::AddTag { target, tag } => {
				Some(cgformat!(cbcx, "tag ", target, " add ", tag)?)
			}
			MinecraftInstr::RemoveTag { target, tag } => {
				Some(cgformat!(cbcx, "tag ", target, " remove ", tag)?)
			}
			MinecraftInstr::ListTags { target } => Some(cgformat!(cbcx, "tag ", target, " list")?),
			MinecraftInstr::RideMount { target, vehicle } => {
				Some(cgformat!(cbcx, "ride ", target, " mount ", vehicle)?)
			}
			MinecraftInstr::RideDismount { target } => {
				Some(cgformat!(cbcx, "ride ", target, " dismount")?)
			}
			MinecraftInstr::Spectate { target, spectator } => {
				let mut out = String::new();
				cgwrite!(&mut out, cbcx, "spectate ", target)?;
				if !spectator.is_blank_this() {
					cgwrite!(&mut out, cbcx, " ", spectator)?;
				}
				Some(out)
			}
			MinecraftInstr::SpectateStop => Some("spectate".into()),
			MinecraftInstr::SetGamemode { target, gamemode } => {
				Some(cgformat!(cbcx, "gamemode ", gamemode, " ", target)?)
			}
			MinecraftInstr::DefaultGamemode { gamemode } => {
				Some(cgformat!(cbcx, "defaultgamemode ", gamemode)?)
			}
			MinecraftInstr::TeleportToEntity { source, dest } => {
				let mut out = String::new();
				cgwrite!(&mut out, cbcx, "tp ")?;
				if !source.is_blank_this() {
					cgwrite!(&mut out, cbcx, source, " ")?;
				}
				cgwrite!(&mut out, cbcx, dest)?;

				Some(out)
			}
			MinecraftInstr::TeleportToLocation { source, dest } => {
				let mut out = String::new();
				cgwrite!(&mut out, cbcx, "tp ")?;
				if !source.is_blank_this() {
					cgwrite!(&mut out, cbcx, source, " ")?;
				}
				cgwrite!(&mut out, cbcx, dest)?;

				Some(out)
			}
			MinecraftInstr::TeleportWithRotation {
				source,
				dest,
				rotation,
			} => {
				let mut out = String::new();
				cgwrite!(&mut out, cbcx, "tp ", source, " ", dest, " ", rotation)?;

				Some(out)
			}
			MinecraftInstr::TeleportFacingLocation {
				source,
				dest,
				facing,
			} => {
				let mut out = String::new();
				cgwrite!(&mut out, cbcx, "tp ", source, " ", dest, " facing ", facing)?;

				Some(out)
			}
			MinecraftInstr::TeleportFacingEntity {
				source,
				dest,
				facing,
			} => {
				let mut out = String::new();
				cgwrite!(&mut out, cbcx, "tp ", source, " ", dest, " facing ", facing)?;

				Some(out)
			}
			MinecraftInstr::GiveItem {
				target,
				item,
				amount,
			} => {
				let mut out = String::new();
				cgwrite!(&mut out, cbcx, "give ", target, " ", item)?;
				if *amount == 0 || (*amount as i64) > (i32::MAX as i64) {
					bail!("Invalid item count");
				}
				if *amount != 1 {
					cgwrite!(&mut out, cbcx, " ", amount)?;
				}
				Some(out)
			}
			MinecraftInstr::AddScoreboardObjective {
				objective,
				criterion,
				display_name,
			} => {
				let mut out = String::new();
				cgwrite!(&mut out, cbcx, "scoreboard objectives add ", objective, " ")?;
				match criterion {
					Criterion::Single(val) => val.gen_writer(&mut out, cbcx)?,
					Criterion::Compound(val) => val.gen_writer(&mut out, cbcx)?,
				}

				if let Some(disp) = display_name {
					cgwrite!(&mut out, cbcx, " ", disp)?;
				}
				Some(out)
			}
			MinecraftInstr::RemoveScoreboardObjective { objective } => {
				Some(cgformat!(cbcx, "scoreboard objectives remove ", objective)?)
			}
			MinecraftInstr::ListScoreboardObjectives => Some("scoreboard objectives list".into()),
			MinecraftInstr::TriggerAdd { objective, amount } => {
				if *amount == 1 {
					Some(cgformat!(cbcx, "trigger ", objective)?)
				} else {
					Some(cgformat!(cbcx, "trigger ", objective, " add ", amount)?)
				}
			}
			MinecraftInstr::TriggerSet { objective, amount } => {
				Some(cgformat!(cbcx, "trigger ", objective, " set ", amount)?)
			}
			MinecraftInstr::GetAttribute {
				target,
				attribute,
				scale,
			} => {
				if *scale == 1.0 {
					Some(cgformat!(cbcx, "attribute ", target, " get ", attribute)?)
				} else {
					Some(cgformat!(
						cbcx,
						"attribute ",
						target,
						" get ",
						attribute,
						" ",
						FloatCG(*scale, false, true, true)
					)?)
				}
			}
			MinecraftInstr::GetAttributeBase {
				target,
				attribute,
				scale,
			} => {
				if *scale == 1.0 {
					Some(cgformat!(
						cbcx,
						"attribute ",
						target,
						" base get ",
						attribute
					)?)
				} else {
					Some(cgformat!(
						cbcx,
						"attribute ",
						target,
						" base get ",
						attribute,
						" ",
						FloatCG(*scale, false, true, true)
					)?)
				}
			}
			MinecraftInstr::SetAttributeBase {
				target,
				attribute,
				value,
			} => Some(cgformat!(
				cbcx,
				"attribute ",
				target,
				" ",
				attribute,
				" base set ",
				FloatCG(*value, false, true, true)
			)?),
			MinecraftInstr::AddAttributeModifier {
				target,
				attribute,
				uuid,
				name,
				value,
				ty,
			} => Some(cgformat!(
				cbcx,
				"attribute ",
				target,
				" ",
				attribute,
				" modifier add ",
				uuid,
				" ",
				name,
				" ",
				FloatCG(*value, false, true, true),
				" ",
				ty
			)?),
			MinecraftInstr::RemoveAttributeModifier {
				target,
				attribute,
				uuid,
			} => Some(cgformat!(
				cbcx,
				"attribute ",
				target,
				" ",
				attribute,
				" modifier remove ",
				uuid
			)?),
			MinecraftInstr::GetAttributeModifier {
				target,
				attribute,
				uuid,
				scale,
			} => {
				let mut out = String::new();
				cgwrite!(
					&mut out,
					cbcx,
					"attribute ",
					target,
					" ",
					attribute,
					" modifier get ",
					uuid
				)?;
				if *scale != 1.0 {
					cgwrite!(&mut out, cbcx, " ", FloatCG(*scale, false, true, true))?;
				}

				Some(out)
			}
			MinecraftInstr::DisableDatapack { pack } => {
				Some(cgformat!(cbcx, "datapack disable ", pack)?)
			}
			MinecraftInstr::EnableDatapack { pack } => {
				Some(cgformat!(cbcx, "datapack enable ", pack)?)
			}
			MinecraftInstr::SetDatapackPriority { pack, priority } => {
				Some(cgformat!(cbcx, "datapack enable ", pack, " ", priority)?)
			}
			MinecraftInstr::SetDatapackOrder {
				pack,
				order,
				existing,
			} => Some(cgformat!(
				cbcx,
				"datapack enable ",
				pack,
				" ",
				order,
				" ",
				existing
			)?),
			MinecraftInstr::ListDatapacks { mode } => match mode {
				DatapackListMode::All => Some("datapack list".into()),
				other => Some(cgformat!(cbcx, "datapack list ", other)?),
			},
			MinecraftInstr::ListPlayerUUIDs => Some("list uuids".into()),
			MinecraftInstr::SummonEntity { entity, pos, nbt } => {
				let mut out = String::new();
				cgwrite!(&mut out, cbcx, "summon ", entity)?;
				if !pos.are_zero() || !nbt.is_empty() {
					cgwrite!(&mut out, cbcx, " ", pos)?;
				}
				if !nbt.is_empty() {
					cgwrite!(&mut out, cbcx, " ", nbt.get_literal_str())?;
				}
				Some(out)
			}
			MinecraftInstr::SetWorldSpawn { pos, angle } => {
				let mut out = String::new();
				cgwrite!(&mut out, cbcx, "setworldspawn")?;
				if !pos.are_zero() || !angle.is_absolute_zero() {
					cgwrite!(&mut out, cbcx, " ", pos)?;
				}
				if !angle.is_absolute_zero() {
					cgwrite!(&mut out, cbcx, " ", angle)?;
				}
				Some(out)
			}
			MinecraftInstr::ClearItems {
				targets,
				item,
				max_count,
			} => {
				let mut out = String::new();
				if targets.is_empty() {
					bail!("Target list empty");
				}
				cgwrite!(&mut out, cbcx, "clear")?;
				if !targets.first().expect("Not empty").is_blank_this() {
					cgwrite!(&mut out, cbcx, SpaceSepListCG(targets))?;
				}
				if let Some(item) = item {
					cgwrite!(&mut out, cbcx, " ", item)?;
				}
				if let Some(max_count) = max_count {
					cgwrite!(&mut out, cbcx, " ", max_count)?;
				}
				Some(out)
			}
			MinecraftInstr::SetSpawnpoint {
				targets,
				pos,
				angle,
			} => {
				let mut out = String::new();
				if targets.is_empty() {
					bail!("Target list empty");
				}
				cgwrite!(&mut out, cbcx, "spawnpoint")?;
				if !(targets.first().expect("Not empty").is_blank_this()
					&& pos.are_zero() && angle.is_absolute_zero())
				{
					cgwrite!(&mut out, cbcx, " ", SpaceSepListCG(targets))?;
				}
				if !pos.are_zero() || !angle.is_absolute_zero() {
					cgwrite!(&mut out, cbcx, " ", pos)?;
				}
				if !angle.is_absolute_zero() {
					cgwrite!(&mut out, cbcx, " ", angle)?;
				}
				Some(out)
			}
			MinecraftInstr::SpreadPlayers {
				center,
				spread_distance,
				max_range,
				max_height,
				respect_teams,
				target,
			} => {
				let mut out = String::new();
				cgwrite!(
					&mut out,
					cbcx,
					"spreadplayers ",
					center,
					" ",
					FloatCG((*spread_distance).into(), false, true, true),
					" ",
					FloatCG((*max_range).into(), false, true, true),
					" "
				)?;
				if let Some(max_height) = max_height {
					cgwrite!(
						&mut out,
						cbcx,
						"under ",
						FloatCG((*max_height).into(), false, true, true),
						" "
					)?;
				}
				cgwrite!(&mut out, cbcx, respect_teams, " ", target)?;
				Some(out)
			}
			MinecraftInstr::ClearEffect { target, effect } => {
				let mut out = String::new();
				cgwrite!(&mut out, cbcx, "effect clear")?;
				if !target.is_blank_this() || effect.is_some() {
					cgwrite!(&mut out, cbcx, " ", target)?;
				}
				if let Some(effect) = effect {
					cgwrite!(&mut out, cbcx, " ", effect)?;
				}

				Some(out)
			}
			MinecraftInstr::GiveEffect {
				target,
				effect,
				duration,
				amplifier,
				hide_particles,
			} => {
				let mut out = String::new();
				cgwrite!(&mut out, cbcx, "effect give ", target, " ", effect)?;
				if !duration.is_default() || *amplifier != 1 || *hide_particles {
					cgwrite!(&mut out, cbcx, " ", duration)?;
				}
				if *amplifier != 1 || *hide_particles {
					cgwrite!(&mut out, cbcx, " ", amplifier)?;
				}
				if *hide_particles {
					cgwrite!(&mut out, cbcx, " true")?;
				}

				Some(out)
			}
			MinecraftInstr::SetGameruleBool { rule, value } => {
				Some(format!("gamerule {rule} {value}"))
			}
			MinecraftInstr::SetGameruleInt { rule, value } => {
				Some(format!("gamerule {rule} {value}"))
			}
			MinecraftInstr::GetGamerule { rule } => Some(format!("gamerule {rule}")),
			MinecraftInstr::Locate {
				location_type,
				location,
			} => Some(format!("locate {location_type:?} {location}")),
			MinecraftInstr::LootGive { player, source } => {
				Some(cgformat!(cbcx, "loot give ", player, " ", source)?)
			}
			MinecraftInstr::LootInsert { pos, source } => {
				Some(cgformat!(cbcx, "loot insert ", pos, " ", source)?)
			}
			MinecraftInstr::LootSpawn { pos, source } => {
				Some(cgformat!(cbcx, "loot spawn ", pos, " ", source)?)
			}
			MinecraftInstr::LootReplaceBlock {
				pos,
				slot,
				count,
				source,
			} => {
				if *count == 1 {
					Some(cgformat!(
						cbcx,
						"loot replace block ",
						pos,
						" ",
						slot,
						" ",
						source
					)?)
				} else {
					Some(cgformat!(
						cbcx,
						"loot replace block ",
						pos,
						" ",
						slot,
						" ",
						count,
						" ",
						source
					)?)
				}
			}
			MinecraftInstr::LootReplaceEntity {
				target,
				slot,
				count,
				source,
			} => {
				if *count == 1 {
					Some(cgformat!(
						cbcx,
						"loot replace entity ",
						target,
						" ",
						slot,
						" ",
						source
					)?)
				} else {
					Some(cgformat!(
						cbcx,
						"loot replace entity ",
						target,
						" ",
						slot,
						" ",
						count,
						" ",
						source
					)?)
				}
			}
			MinecraftInstr::ItemModify {
				location,
				slot,
				modifier,
			} => Some(cgformat!(
				cbcx,
				"item modify ",
				location,
				" ",
				slot,
				" ",
				modifier
			)?),
			MinecraftInstr::ItemReplaceWith {
				location,
				slot,
				item,
				count,
			} => {
				if *count == 1 {
					Some(cgformat!(
						cbcx,
						"item replace ",
						location,
						" ",
						slot,
						" with ",
						item
					)?)
				} else {
					Some(cgformat!(
						cbcx,
						"item replace ",
						location,
						" ",
						slot,
						" with ",
						item,
						" ",
						count
					)?)
				}
			}
			MinecraftInstr::ItemReplaceFrom {
				dest,
				slot,
				source,
				modifier,
			} => {
				if let Some(modifier) = modifier {
					Some(cgformat!(
						cbcx,
						"item replace ",
						dest,
						" ",
						slot,
						" with ",
						source,
						" ",
						modifier
					)?)
				} else {
					Some(cgformat!(
						cbcx,
						"item replace ",
						dest,
						" ",
						slot,
						" with ",
						source
					)?)
				}
			}
			MinecraftInstr::PlaceFeature { feature, pos } => {
				let mut out = cgformat!(cbcx, "place feature ", feature)?;
				if !pos.are_zero() {
					cgwrite!(&mut out, cbcx, " ", pos)?;
				}
				Some(out)
			}
			MinecraftInstr::PlaceJigsaw {
				pool,
				target,
				max_depth,
				pos,
			} => {
				let mut out = cgformat!(cbcx, "place jigsaw ", pool, " ", target, " ", max_depth)?;
				if !pos.are_zero() {
					cgwrite!(&mut out, cbcx, " ", pos)?;
				}
				Some(out)
			}
			MinecraftInstr::PlaceStructure { structure, pos } => {
				let mut out = cgformat!(cbcx, "place structure ", structure)?;
				if !pos.are_zero() {
					cgwrite!(&mut out, cbcx, " ", pos)?;
				}
				Some(out)
			}
			MinecraftInstr::WorldBorderAdd { dist, time } => {
				let mut out =
					cgformat!(cbcx, "worldborder add ", FloatCG(*dist, false, true, true))?;
				if *time != 0 {
					cgwrite!(&mut out, cbcx, " ", time)?;
				}
				Some(out)
			}
			MinecraftInstr::WorldBorderSet { dist, time } => {
				let mut out =
					cgformat!(cbcx, "worldborder set ", FloatCG(*dist, false, true, true))?;
				if *time != 0 {
					cgwrite!(&mut out, cbcx, " ", time)?;
				}
				Some(out)
			}
			MinecraftInstr::WorldBorderGet => Some("worldborder get".into()),
			MinecraftInstr::WorldBorderCenter { pos } => {
				Some(cgformat!(cbcx, "worldborder center ", pos)?)
			}
			MinecraftInstr::WorldBorderDamage { damage } => Some(cgformat!(
				cbcx,
				"worldborder damage amount ",
				FloatCG(*damage, false, true, true)
			)?),
			MinecraftInstr::WorldBorderBuffer { buffer } => Some(cgformat!(
				cbcx,
				"worldborder damage buffer ",
				FloatCG(*buffer, false, true, true)
			)?),
			MinecraftInstr::WorldBorderWarningDistance { dist } => Some(cgformat!(
				cbcx,
				"worldborder warning distance ",
				FloatCG(*dist, false, true, true)
			)?),
			MinecraftInstr::WorldBorderWarningTime { time } => {
				Some(cgformat!(cbcx, "worldborder warning time ", time)?)
			}
		},
		LIRInstrKind::Command(cmd) => Some(cmd.clone()),
		LIRInstrKind::Comment(cmt) => Some(format!("#{cmt}")),
		LIRInstrKind::Use(..) | LIRInstrKind::NoOp => None,
	};

	out.modifiers.extend(modifiers);
	out.modifiers.extend(instr.modifiers.clone());

	out.generate(cmd, cbcx)
}

struct CommandBuilder {
	modifiers: Vec<Modifier>,
}

impl CommandBuilder {
	fn new() -> Self {
		Self {
			modifiers: Vec::new(),
		}
	}

	fn generate(
		self,
		command: Option<String>,
		cbcx: &mut CodegenBlockCx,
	) -> anyhow::Result<Option<String>> {
		let mut out = String::new();

		let command = if let Some(command) = command {
			command
		} else {
			// If the command is a no-op and none of the modifiers have any side effects
			// then it can be omitted
			if !self.modifiers.iter().any(|x| x.has_extra_side_efects()) {
				return Ok(None);
			} else {
				"say foo".into()
			}
		};

		if !self.modifiers.is_empty() {
			out.push_str("execute ");
			for modifier in self.modifiers {
				if let Some(modifier) = codegen_modifier(modifier, cbcx)? {
					out.push_str(&modifier);
					out.push(' ');
				}
			}
			out.push_str("run ");
		}

		out.push_str(&command);

		Ok(Some(out))
	}
}

impl Codegen for Score {
	fn gen_writer<F>(&self, f: &mut F, cbcx: &mut CodegenBlockCx) -> anyhow::Result<()>
	where
		F: std::fmt::Write,
	{
		self.holder.gen_writer(f, cbcx)?;
		write!(f, " {}", self.objective)?;
		Ok(())
	}
}
