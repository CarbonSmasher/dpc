mod entity_target;
mod modifier;
pub mod t;
mod util;

use std::collections::HashSet;

use anyhow::{anyhow, bail, Context};

use crate::common::mc::Score;
use crate::common::modifier::{Modifier, StoreModLocation};
use crate::common::ty::NBTTypeContents;
use crate::common::{ty::DataTypeContents, Value};
use crate::common::{RegisterList, ScoreValue};
use crate::linker::codegen::util::cg_data_modify_rhs;
use crate::linker::text::REG_STORAGE_LOCATION;
use crate::lir::{LIRBlock, LIRInstrKind, LIRInstruction};

use self::modifier::codegen_modifier;
use self::util::get_mut_val_reg;

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

pub fn codegen_block(block: &LIRBlock, ccx: &mut CodegenCx) -> anyhow::Result<Vec<String>> {
	let ra = alloc_block_registers(block, &mut ccx.racx)?;

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

	dbg!(&cbcx.ra);

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
		LIRInstrKind::SetData(left, right) => {
			let rhs = cg_data_modify_rhs(cbcx, right)?;
			Some(cgformat!(cbcx, "data modify ", left, " set ", rhs)?)
		}
		LIRInstrKind::MergeData(left, right) => {
			let rhs = cg_data_modify_rhs(cbcx, right)?;
			Some(cgformat!(cbcx, "data modify ", left, " merge ", rhs)?)
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
			Value::Constant(val) => match val {
				DataTypeContents::NBT(NBTTypeContents::Arr(arr)) => {
					let lit = arr
						.const_index(*index)
						.ok_or(anyhow!("Const index out of range"))?;
					cgformat!(cbcx, "scoreboard players set ", score, lit)?
				}
				_ => bail!("Cannot index non-array type"),
			},
			Value::Mutable(val) => {
				let loc = get_mut_val_reg(val, &cbcx.ra, &cbcx.regs)?.clone();
				modifiers.push(Modifier::StoreResult(StoreModLocation::from_mut_score_val(
					score,
				)));
				cgformat!(cbcx, "data get storage ", REG_STORAGE_LOCATION, loc)?
			}
		}),
		LIRInstrKind::Say(message) => Some(format!("say {message}")),
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
