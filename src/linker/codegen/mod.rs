mod entity_target;
mod modifier;
mod t;
mod util;

use std::collections::HashSet;

use anyhow::{anyhow, bail};

use crate::common::mc::Score;
use crate::common::modifier::Modifier;
use crate::common::ty::{DataType, NBTTypeContents};
use crate::common::RegisterList;
use crate::common::{ty::DataTypeContents, Value};
use crate::linker::text::{format_local_storage_path, REG_STORAGE_LOCATION};
use crate::lir::{LIRBlock, LIRInstrKind, LIRInstruction};

use self::modifier::codegen_modifier;
use self::util::{get_mut_val_reg, get_val_score};

use super::text::REG_OBJECTIVE;

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
	for instr in &block.contents {
		let command = codegen_instr(instr, &mut cbcx)?;
		out.extend(command);
	}

	Ok(out)
}

pub fn codegen_instr(
	instr: &LIRInstruction,
	cbcx: &mut CodegenBlockCx,
) -> anyhow::Result<Option<String>> {
	let mut out = CommandBuilder::new();

	let cmd = match &instr.kind {
		LIRInstrKind::SetScore(left, right) => {
			let left_scoreholder = get_mut_val_reg(&left, &cbcx.ra, &cbcx.regs)?;
			Some(match right {
				Value::Constant(data) => {
					let lit = match data {
						DataTypeContents::Score(score) => score.get_literal_str(),
						_ => bail!("LIR instruction given wrong type"),
					};
					format!("scoreboard players set {left_scoreholder} {REG_OBJECTIVE} {lit}")
				}
				Value::Mutable(val) => match val.get_ty(&cbcx.regs)? {
					DataType::Score(..) => {
						let right_scoreholder = get_mut_val_reg(&val, &cbcx.ra, &cbcx.regs)?;
						format!("scoreboard players operation {left_scoreholder} {REG_OBJECTIVE} = {right_scoreholder} {REG_OBJECTIVE}")
					}
					DataType::NBT(..) => {
						let right_loc = get_mut_val_reg(&val, &cbcx.ra, &cbcx.regs)?;
						format!("execute store result score {left_scoreholder} {REG_OBJECTIVE} run data get storage {REG_STORAGE_LOCATION} {} 1.0", format_local_storage_path(right_loc))
					}
				},
			})
		}
		LIRInstrKind::AddScore(left, right) => {
			let left_scoreholder = get_mut_val_reg(&left, &cbcx.ra, &cbcx.regs)?;
			Some(match right {
				Value::Constant(data) => {
					let lit = match data {
						DataTypeContents::Score(score) => score.get_i32(),
						_ => bail!("LIR instruction given wrong type"),
					};
					// Negative signs in add/remove commands are illegal
					if lit.is_negative() {
						format!(
							"scoreboard players remove {left_scoreholder} {REG_OBJECTIVE} {}",
							lit.abs()
						)
					} else {
						format!("scoreboard players add {left_scoreholder} {REG_OBJECTIVE} {lit}")
					}
				}
				Value::Mutable(val) => {
					let right_scoreholder = get_mut_val_reg(&val, &cbcx.ra, &cbcx.regs)?;
					format!("scoreboard players operation {left_scoreholder} {REG_OBJECTIVE} += {right_scoreholder} {REG_OBJECTIVE}")
				}
			})
		}
		LIRInstrKind::SubScore(left, right) => {
			let left_scoreholder = get_mut_val_reg(&left, &cbcx.ra, &cbcx.regs)?;
			Some(match right {
				Value::Constant(data) => {
					let lit = match data {
						DataTypeContents::Score(score) => score.get_i32(),
						_ => bail!("LIR instruction given wrong type"),
					};
					// Negative signs in add/remove commands are illegal
					if lit.is_negative() {
						format!(
							"scoreboard players add {left_scoreholder} {REG_OBJECTIVE} {}",
							lit.abs()
						)
					} else {
						format!(
							"scoreboard players remove {left_scoreholder} {REG_OBJECTIVE} {lit}"
						)
					}
				}
				Value::Mutable(val) => {
					let right_scoreholder = get_mut_val_reg(&val, &cbcx.ra, &cbcx.regs)?;
					format!("scoreboard players operation {left_scoreholder} {REG_OBJECTIVE} -= {right_scoreholder} {REG_OBJECTIVE}")
				}
			})
		}
		LIRInstrKind::MulScore(left, right) => {
			let left_scoreholder = get_mut_val_reg(&left, &cbcx.ra, &cbcx.regs)?.clone();
			let (right_score, to_add) = get_val_score(&right, &cbcx.ra, &cbcx.regs)?;
			cbcx.ccx.score_literals.extend(to_add);

			let mut out =
				format!("scoreboard players operation {left_scoreholder} {REG_OBJECTIVE} *= ");
			right_score.gen_writer(&mut out, cbcx)?;
			Some(out)
		}
		LIRInstrKind::DivScore(left, right) => {
			let left_scoreholder = get_mut_val_reg(&left, &cbcx.ra, &cbcx.regs)?.clone();
			let (right_score, to_add) = get_val_score(&right, &cbcx.ra, &cbcx.regs)?;
			cbcx.ccx.score_literals.extend(to_add);

			let mut out =
				format!("scoreboard players operation {left_scoreholder} {REG_OBJECTIVE} /= ");
			right_score.gen_writer(&mut out, cbcx)?;
			Some(out)
		}
		LIRInstrKind::ModScore(left, right) => {
			let left_scoreholder = get_mut_val_reg(&left, &cbcx.ra, &cbcx.regs)?.clone();
			let (right_score, to_add) = get_val_score(&right, &cbcx.ra, &cbcx.regs)?;
			cbcx.ccx.score_literals.extend(to_add);

			let mut out =
				format!("scoreboard players operation {left_scoreholder} {REG_OBJECTIVE} %= ");
			right_score.gen_writer(&mut out, cbcx)?;
			Some(out)
		}
		LIRInstrKind::MinScore(left, right) => {
			let left_scoreholder = get_mut_val_reg(&left, &cbcx.ra, &cbcx.regs)?.clone();
			let (right_score, to_add) = get_val_score(&right, &cbcx.ra, &cbcx.regs)?;
			cbcx.ccx.score_literals.extend(to_add);

			let mut out =
				format!("scoreboard players operation {left_scoreholder} {REG_OBJECTIVE} < ");
			right_score.gen_writer(&mut out, cbcx)?;
			Some(out)
		}
		LIRInstrKind::MaxScore(left, right) => {
			let left_scoreholder = get_mut_val_reg(&left, &cbcx.ra, &cbcx.regs)?.clone();
			let (right_score, to_add) = get_val_score(&right, &cbcx.ra, &cbcx.regs)?;
			cbcx.ccx.score_literals.extend(to_add);

			let mut out =
				format!("scoreboard players operation {left_scoreholder} {REG_OBJECTIVE} > ");
			right_score.gen_writer(&mut out, cbcx)?;
			Some(out)
		}
		LIRInstrKind::SwapScore(left, right) => {
			let left_scoreholder = get_mut_val_reg(&left, &cbcx.ra, &cbcx.regs)?;
			let right_scoreholder = get_mut_val_reg(&right, &cbcx.ra, &cbcx.regs)?;

			Some(format!(
				"scoreboard players operation {left_scoreholder} {REG_OBJECTIVE} >< {right_scoreholder} {REG_OBJECTIVE}"
			))
		}
		LIRInstrKind::SetData(left, right) => {
			let left_loc = get_mut_val_reg(left, &cbcx.ra, &cbcx.regs)?;
			Some(match right {
				Value::Constant(data) => {
					let lit = match data {
						DataTypeContents::NBT(nbt) => nbt.get_literal_str(),
						_ => bail!("LIR instruction given wrong type"),
					};
					format!("data merge storage {REG_STORAGE_LOCATION} {{{left_loc}:{lit}}}")
				}
				Value::Mutable(val) => match val.get_ty(&cbcx.regs)? {
					DataType::Score(..) => {
						let right_scoreholder = get_mut_val_reg(&val, &cbcx.ra, &cbcx.regs)?;
						format!("execute store result storage {REG_STORAGE_LOCATION} {left_loc} run scoreboard players get {right_scoreholder} {REG_OBJECTIVE}")
					}
					DataType::NBT(..) => {
						let right_loc = get_mut_val_reg(val, &cbcx.ra, &cbcx.regs)?;
						format!("data modify storage {REG_STORAGE_LOCATION} {left_loc} set from storage {REG_STORAGE_LOCATION} {right_loc}")
					}
				},
			})
		}
		LIRInstrKind::ConstIndexToScore {
			score,
			value,
			index,
		} => {
			let scoreholder = get_mut_val_reg(&score, &cbcx.ra, &cbcx.regs)?;
			Some(match value {
				Value::Constant(val) => match val {
					DataTypeContents::NBT(NBTTypeContents::Arr(arr)) => {
						let lit = arr
							.const_index(*index)
							.ok_or(anyhow!("Const index out of range"))?;
						format!("scoreboard players set {scoreholder} {REG_OBJECTIVE} {lit}")
					}
					_ => bail!("Cannot index non-array type"),
				},
				Value::Mutable(val) => {
					let loc = get_mut_val_reg(val, &cbcx.ra, &cbcx.regs)?;
					format!("execute store result score {scoreholder} {REG_OBJECTIVE} run data get storage {REG_STORAGE_LOCATION} {loc}")
				}
			})
		}
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

	for modifier in instr.modifiers.clone() {
		out.modifiers.push(modifier);
	}

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
