mod imp;
mod modifier;
pub mod t;
mod util;

use std::collections::HashSet;

use anyhow::{anyhow, bail, Context};

use crate::common::mc::block::{CloneMaskMode, CloneMode, FillMode, SetBlockMode};
use crate::common::mc::modifier::{Modifier, StoreModLocation};
use crate::common::mc::scoreboard_and_teams::Criterion;
use crate::common::mc::{DatapackListMode, Score};
use crate::common::ty::NBTTypeContents;
use crate::common::{val::NBTValue, val::ScoreValue, RegisterList};
use crate::linker::codegen::util::cg_data_modify_rhs;
use crate::lir::{LIRBlock, LIRInstrKind, LIRInstruction};

use self::modifier::codegen_modifier;
use self::t::macros::cgwrite;
use self::util::SpaceSepListCG;

use super::ra::{alloc_block_registers, RegAllocCx, RegAllocResult};

use t::macros::cgformat;
pub use t::Codegen;

pub struct CodegenCx {
	pub racx: RegAllocCx,
	pub score_literals: HashSet<i32>,
	pub requirements: HashSet<CodegenRequirement>,
}

impl CodegenCx {
	pub fn new() -> Self {
		Self {
			racx: RegAllocCx::new(),
			score_literals: HashSet::new(),
			requirements: HashSet::new(),
		}
	}

	pub fn add_requirement(&mut self, req: CodegenRequirement) {
		self.requirements.insert(req);
	}
}

/// Different requirements that can be imposed on the linker so that it generates functions
#[derive(Hash, PartialEq, Eq, Clone, Copy)]
pub enum CodegenRequirement {}

pub struct CodegenBlockCx<'ccx> {
	pub ccx: &'ccx mut CodegenCx,
	pub ra: RegAllocResult,
	pub regs: RegisterList,
}

pub fn codegen_block(func_id: &str, block: &LIRBlock, ccx: &mut CodegenCx) -> anyhow::Result<Vec<String>> {
	let ra = alloc_block_registers(func_id, block, &mut ccx.racx)?;

	let mut cbcx = CodegenBlockCx {
		ccx,
		ra,
		regs: block.regs.clone(),
	};

	let mut out = Vec::new();
	for (i, instr) in block.contents.iter().enumerate() {
		let command =
			codegen_instr(instr, &mut cbcx).with_context(|| format!("At instruction {i}"))?;
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
		LIRInstrKind::ResetScore(val) => Some(cgformat!(cbcx, "scoreboard players reset ", val)?),
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
		LIRInstrKind::GetData(val) => Some(cgformat!(cbcx, "data get ", val)?),
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
				)));
				cgformat!(cbcx, "data get storage ", val)?
			}
		}),
		LIRInstrKind::Say(message) => Some(format!("say {message}")),
		LIRInstrKind::Me(message) => Some(format!("me {message}")),
		LIRInstrKind::TeamMessage(message) => Some(format!("tm {message}")),
		LIRInstrKind::Tell(target, message) => Some(cgformat!(cbcx, "w ", target, " ", message)?),
		LIRInstrKind::Kill(target) => {
			if target.is_blank_this() {
				Some("kill".into())
			} else {
				Some(cgformat!(cbcx, "kill ", target)?)
			}
		}
		LIRInstrKind::Reload => Some("reload".into()),
		LIRInstrKind::SetXP(target, amount, value) => {
			// Command cannot take negative values
			let amount = if amount < &0 { 0 } else { *amount };
			Some(cgformat!(cbcx, "xp set ", target, " ", amount, " ", value)?)
		}
		LIRInstrKind::Call(fun) => Some(format!("function {fun}")),
		LIRInstrKind::BanPlayers(targets, reason) => {
			let list = SpaceSepListCG(targets);
			if let Some(reason) = reason {
				Some(cgformat!(cbcx, "ban ", list, " ", reason)?)
			} else {
				Some(cgformat!(cbcx, "ban ", list)?)
			}
		}
		LIRInstrKind::BanIP(target, reason) => {
			if let Some(reason) = reason {
				Some(cgformat!(cbcx, "ban-ip ", target, " ", reason)?)
			} else {
				Some(cgformat!(cbcx, "ban-ip ", target)?)
			}
		}
		LIRInstrKind::PardonPlayers(targets) => {
			let list = SpaceSepListCG(targets);
			Some(cgformat!(cbcx, "pardon ", list)?)
		}
		LIRInstrKind::PardonIP(target) => Some(format!("pardon-ip {target}")),
		LIRInstrKind::Op(targets) => Some(cgformat!(cbcx, "op ", SpaceSepListCG(targets))?),
		LIRInstrKind::Deop(targets) => Some(cgformat!(cbcx, "deop ", SpaceSepListCG(targets))?),
		LIRInstrKind::WhitelistAdd(targets) => {
			Some(cgformat!(cbcx, "whitelist add ", SpaceSepListCG(targets))?)
		}
		LIRInstrKind::WhitelistRemove(targets) => Some(cgformat!(
			cbcx,
			"whitelist remove ",
			SpaceSepListCG(targets)
		)?),
		LIRInstrKind::WhitelistOn => Some("whitelist on".into()),
		LIRInstrKind::WhitelistOff => Some("whitelist off".into()),
		LIRInstrKind::WhitelistReload => Some("whitelist reload".into()),
		LIRInstrKind::WhitelistList => Some("whitelist list".into()),
		LIRInstrKind::Kick(targets, reason) => {
			let list = SpaceSepListCG(targets);
			if let Some(reason) = reason {
				Some(cgformat!(cbcx, "kick ", list, " ", reason)?)
			} else {
				Some(cgformat!(cbcx, "kick ", list)?)
			}
		}
		LIRInstrKind::Banlist => Some("banlist".into()),
		LIRInstrKind::StopSound => Some("stopsound".into()),
		LIRInstrKind::StopServer => Some("stop".into()),
		LIRInstrKind::ListPlayers => Some("list".into()),
		LIRInstrKind::Publish => Some("publish".into()),
		LIRInstrKind::Seed => Some("seed".into()),
		LIRInstrKind::GetDifficulty => Some("difficulty".into()),
		LIRInstrKind::SetDifficulty(diff) => Some(cgformat!(cbcx, "difficulty ", diff)?),
		LIRInstrKind::Enchant(target, ench, lvl) => {
			if lvl == &1 {
				Some(cgformat!(cbcx, "enchant ", target, " ", ench)?)
			} else {
				Some(cgformat!(cbcx, "enchant ", target, " ", ench, " ", lvl)?)
			}
		}
		LIRInstrKind::SetBlock(data) => {
			let mut out = String::new();
			cgwrite!(&mut out, cbcx, "setblock ", data.pos, " ", data.block)?;

			// Replace mode is default and can be omitted
			if let SetBlockMode::Replace = data.mode {
			} else {
				cgwrite!(&mut out, cbcx, " ", data.mode)?;
			}
			Some(out)
		}
		LIRInstrKind::Fill(data) => {
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
		LIRInstrKind::Clone(data) => {
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
		LIRInstrKind::FillBiome(data) => {
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
		LIRInstrKind::SetWeather(weather, duration) => {
			if let Some(duration) = duration {
				Some(cgformat!(cbcx, "weather ", weather, " ", duration)?)
			} else {
				Some(cgformat!(cbcx, "weather ", weather)?)
			}
		}
		LIRInstrKind::AddTime(time) => Some(cgformat!(cbcx, "time add ", time)?),
		LIRInstrKind::SetTime(time) => {
			// Using the day preset is shorter than the time it represents
			if time.amount.0 == 1000.0 {
				Some("tmie set day".into())
			} else {
				Some(cgformat!(cbcx, "time set ", time)?)
			}
		}
		LIRInstrKind::SetTimePreset(time) => Some(cgformat!(cbcx, "time set ", time)?),
		LIRInstrKind::GetTime(query) => Some(cgformat!(cbcx, "time get ", query)?),
		LIRInstrKind::AddTag(target, tag) => Some(cgformat!(cbcx, "tag ", target, " add ", tag)?),
		LIRInstrKind::RemoveTag(target, tag) => {
			Some(cgformat!(cbcx, "tag ", target, " remove ", tag)?)
		}
		LIRInstrKind::ListTags(target) => Some(cgformat!(cbcx, "tag ", target, " list")?),
		LIRInstrKind::RideMount(target, vehicle) => {
			Some(cgformat!(cbcx, "ride ", target, " mount ", vehicle)?)
		}
		LIRInstrKind::RideDismount(target) => Some(cgformat!(cbcx, "ride ", target, " dismount")?),
		LIRInstrKind::Spectate(target, spectator) => {
			let mut out = String::new();
			cgwrite!(&mut out, cbcx, "spectate ", target)?;
			if !spectator.is_blank_this() {
				cgwrite!(&mut out, cbcx, " ", spectator)?;
			}
			Some(out)
		}
		LIRInstrKind::SpectateStop => Some("spectate".into()),
		LIRInstrKind::SetGamemode(target, gm) => {
			Some(cgformat!(cbcx, "gamemode ", gm, " ", target)?)
		}
		LIRInstrKind::DefaultGamemode(gm) => Some(cgformat!(cbcx, "defaultgamemode ", gm)?),
		LIRInstrKind::ReturnValue(val) => Some(cgformat!(cbcx, "return ", val)?),
		LIRInstrKind::ReturnFail => Some("return fail".into()),
		LIRInstrKind::ReturnRun(fun) => Some(cgformat!(cbcx, "return run ", fun)?),
		LIRInstrKind::TeleportToEntity(src, dest) => {
			let mut out = String::new();
			if src.is_empty() {
				bail!("Target list empty");
			}
			cgwrite!(&mut out, cbcx, "tp ")?;
			if !src.first().expect("Not empty").is_blank_this() {
				cgwrite!(&mut out, cbcx, SpaceSepListCG(src), " ")?;
			}
			cgwrite!(&mut out, cbcx, dest)?;

			Some(out)
		}
		LIRInstrKind::TeleportToLocation(src, dest) => {
			let mut out = String::new();
			if src.is_empty() {
				bail!("Target list empty");
			}
			cgwrite!(&mut out, cbcx, "tp ")?;
			if !src.first().expect("Not empty").is_blank_this() {
				cgwrite!(&mut out, cbcx, SpaceSepListCG(src), " ")?;
			}
			cgwrite!(&mut out, cbcx, dest)?;

			Some(out)
		}
		LIRInstrKind::TeleportWithRotation(src, dest, rot) => {
			let mut out = String::new();
			if src.is_empty() {
				bail!("Target list empty");
			}
			cgwrite!(
				&mut out,
				cbcx,
				"tp ",
				SpaceSepListCG(src),
				" ",
				dest,
				" ",
				rot
			)?;

			Some(out)
		}
		LIRInstrKind::TeleportFacingLocation(src, dest, face) => {
			let mut out = String::new();
			if src.is_empty() {
				bail!("Target list empty");
			}
			cgwrite!(
				&mut out,
				cbcx,
				"tp ",
				SpaceSepListCG(src),
				" ",
				dest,
				" facing ",
				face
			)?;

			Some(out)
		}
		LIRInstrKind::TeleportFacingEntity(src, dest, face) => {
			let mut out = String::new();
			if src.is_empty() {
				bail!("Target list empty");
			}
			cgwrite!(
				&mut out,
				cbcx,
				"tp ",
				SpaceSepListCG(src),
				" ",
				dest,
				" facing ",
				face
			)?;

			Some(out)
		}
		LIRInstrKind::GiveItem(target, item, amount) => {
			let mut out = String::new();
			cgwrite!(&mut out, cbcx, "give ", target, " ", item)?;
			if *amount == 0 || (*amount as i64) > (i32::MAX as i64) {
				bail!("Invalid item count");
			}
			if *amount != 1 {
				cgwrite!(&mut out, cbcx, amount)?;
			}
			Some(out)
		}
		LIRInstrKind::AddScoreboardObjective(obj, crit, disp) => {
			let mut out = String::new();
			cgwrite!(&mut out, cbcx, "scoreboard objectives add ", obj, " ")?;
			match crit {
				Criterion::Single(val) => val.gen_writer(&mut out, cbcx)?,
				Criterion::Compound(val) => val.gen_writer(&mut out, cbcx)?,
			}

			if let Some(disp) = disp {
				cgwrite!(&mut out, cbcx, " ", disp)?;
			}
			Some(out)
		}
		LIRInstrKind::RemoveScoreboardObjective(obj) => {
			Some(cgformat!(cbcx, "scoreboard objectives remove ", obj)?)
		}
		LIRInstrKind::ListScoreboardObjectives => Some("scoreboard objectives list".into()),
		LIRInstrKind::TriggerAdd(obj, amt) => {
			if *amt == 1 {
				Some(cgformat!(cbcx, "trigger ", obj)?)
			} else {
				Some(cgformat!(cbcx, "trigger ", obj, " add ", amt)?)
			}
		}
		LIRInstrKind::TriggerSet(obj, amt) => Some(cgformat!(cbcx, "trigger ", obj, " set ", amt)?),
		LIRInstrKind::GetAttribute(tgt, attr, scale) => {
			if *scale == 1.0 {
				Some(cgformat!(cbcx, "attribute ", tgt, " get ", attr)?)
			} else {
				Some(cgformat!(
					cbcx,
					"attribute ",
					tgt,
					" get ",
					attr,
					" ",
					scale
				)?)
			}
		}
		LIRInstrKind::GetAttributeBase(tgt, attr, scale) => {
			if *scale == 1.0 {
				Some(cgformat!(cbcx, "attribute ", tgt, " base get ", attr)?)
			} else {
				Some(cgformat!(
					cbcx,
					"attribute ",
					tgt,
					" base get ",
					attr,
					" ",
					scale
				)?)
			}
		}
		LIRInstrKind::SetAttributeBase(tgt, attr, value) => Some(cgformat!(
			cbcx,
			"attribute ",
			tgt,
			" ",
			attr,
			" base set ",
			value
		)?),
		LIRInstrKind::AddAttributeModifier(tgt, attr, uuid, name, value, ty) => Some(cgformat!(
			cbcx,
			"attribute ",
			tgt,
			" ",
			attr,
			" modifier add ",
			uuid,
			" ",
			name,
			" ",
			value,
			" ",
			ty
		)?),
		LIRInstrKind::RemoveAttributeModifier(tgt, attr, uuid) => Some(cgformat!(
			cbcx,
			"attribute ",
			tgt,
			" ",
			attr,
			" modifier remove ",
			uuid
		)?),
		LIRInstrKind::GetAttributeModifier(tgt, attr, uuid, scale) => {
			let mut out = String::new();
			cgwrite!(
				&mut out,
				cbcx,
				"attribute ",
				tgt,
				" ",
				attr,
				" modifier get ",
				uuid
			)?;
			if *scale != 1.0 {
				cgwrite!(&mut out, cbcx, " ", scale)?;
			}

			Some(out)
		}
		LIRInstrKind::DisableDatapack(pack) => Some(cgformat!(cbcx, "datapack disable ", pack)?),
		LIRInstrKind::EnableDatapack(pack) => Some(cgformat!(cbcx, "datapack enable ", pack)?),
		LIRInstrKind::SetDatapackPriority(pack, priority) => {
			Some(cgformat!(cbcx, "datapack enable ", pack, " ", priority)?)
		}
		LIRInstrKind::SetDatapackOrder(pack, order, existing) => Some(cgformat!(
			cbcx,
			"datapack enable ",
			pack,
			" ",
			order,
			" ",
			existing
		)?),
		LIRInstrKind::ListDatapacks(mode) => match mode {
			DatapackListMode::All => Some("datapack list".into()),
			other => Some(cgformat!(cbcx, "datapack list ", other)?),
		},
		LIRInstrKind::ListPlayerUUIDs => Some("list uuids".into()),
		LIRInstrKind::SummonEntity(entity, pos, nbt) => {
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
		LIRInstrKind::SetWorldSpawn(pos, angle) => {
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
		LIRInstrKind::ClearItems(targets, item, max_count) => {
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
		LIRInstrKind::SetSpawnpoint(targets, pos, angle) => {
			let mut out = String::new();
			if targets.is_empty() {
				bail!("Target list empty");
			}
			cgwrite!(&mut out, cbcx, "spawnpoint")?;
			if !(targets.first().expect("Not empty").is_blank_this()
				&& pos.are_zero()
				&& angle.is_absolute_zero())
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
		LIRInstrKind::SpreadPlayers {
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
				spread_distance,
				" ",
				max_range,
				" "
			)?;
			if let Some(max_height) = max_height {
				cgwrite!(&mut out, cbcx, "under ", max_height, " ")?;
			}
			cgwrite!(&mut out, cbcx, respect_teams, " ", target)?;
			Some(out)
		}
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
