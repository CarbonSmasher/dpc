use std::collections::HashSet;

use anyhow::{anyhow, bail};

use crate::common::ty::{DataType, NBTTypeContents};
use crate::common::RegisterList;
use crate::common::{ty::DataTypeContents, MutableValue, Value};
use crate::linker::text::{format_local_storage_path, REG_STORAGE_LOCATION};
use crate::lir::{LIRBlock, LIRInstrKind, LIRInstruction};
use crate::mc::{Score, TargetSelector};

use super::text::{format_lit_fake_player, LIT_OBJECTIVE, REG_OBJECTIVE};

use super::ra::{alloc_block_registers, RegAllocCx, RegAllocResult};

pub struct CodegenCx {
	pub racx: RegAllocCx,
	pub score_literals: HashSet<i32>,
}

impl CodegenCx {
	pub fn new() -> Self {
		Self {
			racx: RegAllocCx::new(),
			score_literals: HashSet::new(),
		}
	}
}

pub fn codegen_block(block: &LIRBlock, ccx: &mut CodegenCx) -> anyhow::Result<Vec<String>> {
	let ra = alloc_block_registers(block, &mut ccx.racx)?;

	let mut out = Vec::new();
	for instr in &block.contents {
		let commands = codegen_instr(instr, &ra, &block.regs, ccx)?;
		out.extend(commands);
	}

	Ok(out)
}

pub fn codegen_instr(
	instr: &LIRInstruction,
	ra: &RegAllocResult,
	regs: &RegisterList,
	ccx: &mut CodegenCx,
) -> anyhow::Result<Vec<String>> {
	let mut out = Vec::new();

	match &instr.kind {
		LIRInstrKind::SetScore(left, right) => {
			let left_scoreholder = get_mut_val_reg(&left, ra, regs)?;
			let cmd = match right {
				Value::Constant(data) => {
					let lit = match data {
						DataTypeContents::Score(score) => score.get_literal_str(),
						_ => bail!("LIR instruction given wrong type"),
					};
					format!("scoreboard players set {left_scoreholder} {REG_OBJECTIVE} {lit}")
				}
				Value::Mutable(val) => match val.get_ty(regs)? {
					DataType::Score(..) => {
						let right_scoreholder = get_mut_val_reg(&val, ra, regs)?;
						format!("scoreboard players operation {left_scoreholder} {REG_OBJECTIVE} = {right_scoreholder} {REG_OBJECTIVE}")
					}
					DataType::NBT(..) => {
						let right_loc = get_mut_val_reg(&val, ra, regs)?;
						format!("execute store result score {left_scoreholder} {REG_OBJECTIVE} run data get storage {REG_STORAGE_LOCATION} {} 1.0", format_local_storage_path(right_loc))
					}
				},
			};
			out.push(cmd);
		}
		LIRInstrKind::AddScore(left, right) => {
			let left_scoreholder = get_mut_val_reg(&left, ra, regs)?;
			let cmd = match right {
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
					let right_scoreholder = get_mut_val_reg(&val, ra, regs)?;
					format!("scoreboard players operation {left_scoreholder} {REG_OBJECTIVE} += {right_scoreholder} {REG_OBJECTIVE}")
				}
			};
			out.push(cmd);
		}
		LIRInstrKind::SubScore(left, right) => {
			let left_scoreholder = get_mut_val_reg(&left, ra, regs)?;
			let cmd = match right {
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
					let right_scoreholder = get_mut_val_reg(&val, ra, regs)?;
					format!("scoreboard players operation {left_scoreholder} {REG_OBJECTIVE} -= {right_scoreholder} {REG_OBJECTIVE}")
				}
			};
			out.push(cmd);
		}
		LIRInstrKind::MulScore(left, right) => {
			let left_scoreholder = get_mut_val_reg(&left, ra, regs)?;
			let (right_score, to_add) = get_val_score(&right, ra, regs)?;
			ccx.score_literals.extend(to_add);

			let cmd = format!(
				"scoreboard players operation {left_scoreholder} {REG_OBJECTIVE} *= {} {}",
				right_score.holder.codegen_str(),
				right_score.objective
			);

			out.push(cmd);
		}
		LIRInstrKind::DivScore(left, right) => {
			let left_scoreholder = get_mut_val_reg(&left, ra, regs)?;
			let (right_score, to_add) = get_val_score(&right, ra, regs)?;
			ccx.score_literals.extend(to_add);

			let cmd = format!(
				"scoreboard players operation {left_scoreholder} {REG_OBJECTIVE} /= {} {}",
				right_score.holder.codegen_str(),
				right_score.objective
			);

			out.push(cmd);
		}
		LIRInstrKind::ModScore(left, right) => {
			let left_scoreholder = get_mut_val_reg(&left, ra, regs)?;
			let (right_score, to_add) = get_val_score(&right, ra, regs)?;
			ccx.score_literals.extend(to_add);

			let cmd = format!(
				"scoreboard players operation {left_scoreholder} {REG_OBJECTIVE} %= {} {}",
				right_score.holder.codegen_str(),
				right_score.objective
			);

			out.push(cmd);
		}
		LIRInstrKind::MinScore(left, right) => {
			let left_scoreholder = get_mut_val_reg(&left, ra, regs)?;
			let (right_score, to_add) = get_val_score(&right, ra, regs)?;
			ccx.score_literals.extend(to_add);

			let cmd = format!(
				"scoreboard players operation {left_scoreholder} {REG_OBJECTIVE} < {} {}",
				right_score.holder.codegen_str(),
				right_score.objective
			);

			out.push(cmd);
		}
		LIRInstrKind::MaxScore(left, right) => {
			let left_scoreholder = get_mut_val_reg(&left, ra, regs)?;
			let (right_score, to_add) = get_val_score(&right, ra, regs)?;
			ccx.score_literals.extend(to_add);

			let cmd = format!(
				"scoreboard players operation {left_scoreholder} {REG_OBJECTIVE} > {} {}",
				right_score.holder.codegen_str(),
				right_score.objective
			);

			out.push(cmd);
		}
		LIRInstrKind::SwapScore(left, right) => {
			let left_scoreholder = get_mut_val_reg(&left, ra, regs)?;
			let right_scoreholder = get_mut_val_reg(&right, ra, regs)?;

			let cmd = format!(
				"scoreboard players operation {left_scoreholder} {REG_OBJECTIVE} >< {right_scoreholder} {REG_OBJECTIVE}"
			);

			out.push(cmd);
		}
		LIRInstrKind::SetData(left, right) => {
			let left_loc = get_mut_val_reg(left, ra, regs)?;
			let cmd = match right {
				Value::Constant(data) => {
					let lit = match data {
						DataTypeContents::NBT(nbt) => nbt.get_literal_str(),
						_ => bail!("LIR instruction given wrong type"),
					};
					format!("data merge storage {REG_STORAGE_LOCATION} {{{left_loc}:{lit}}}")
				}
				Value::Mutable(val) => match val.get_ty(regs)? {
					DataType::Score(..) => {
						let right_scoreholder = get_mut_val_reg(&val, ra, regs)?;
						format!("execute store result storage {REG_STORAGE_LOCATION} {left_loc} run scoreboard players get {right_scoreholder} {REG_OBJECTIVE}")
					}
					DataType::NBT(..) => {
						let right_loc = get_mut_val_reg(val, ra, regs)?;
						format!("data modify storage {REG_STORAGE_LOCATION} {left_loc} set from storage {REG_STORAGE_LOCATION} {right_loc}")
					}
				},
			};

			out.push(cmd);
		}
		LIRInstrKind::ConstIndexToScore {
			score,
			value,
			index,
		} => {
			let scoreholder = get_mut_val_reg(&score, ra, regs)?;
			let cmd = match value {
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
					let loc = get_mut_val_reg(val, ra, regs)?;
					format!("execute store result score {scoreholder} {REG_OBJECTIVE} run data get storage {REG_STORAGE_LOCATION} {loc}")
				}
			};

			out.push(cmd);
		}
	}

	Ok(out)
}

/// Returns a score and an optional score literal to add
fn get_val_score(
	val: &Value,
	ra: &RegAllocResult,
	regs: &RegisterList,
) -> anyhow::Result<(Score, Option<i32>)> {
	let out = match val {
		Value::Constant(val) => match val {
			DataTypeContents::Score(score) => {
				let num = score.get_i32();
				(
					Score::new(
						TargetSelector::Player(format_lit_fake_player(num)),
						LIT_OBJECTIVE.into(),
					),
					Some(num),
				)
			}
			_ => bail!("LIR instruction given wrong type"),
		},
		Value::Mutable(val) => {
			let score = Score::new(
				TargetSelector::Player(get_mut_val_reg(val, ra, regs)?.clone()),
				REG_OBJECTIVE.into(),
			);
			(score, None)
		}
	};

	Ok(out)
}

fn get_mut_val_reg<'ra>(
	val: &MutableValue,
	ra: &'ra RegAllocResult,
	regs: &RegisterList,
) -> anyhow::Result<&'ra String> {
	match val {
		MutableValue::Register(reg) => match val.get_ty(regs)? {
			DataType::Score(..) => ra
				.regs
				.get(reg)
				.ok_or(anyhow!("Register {reg} not allocated")),
			DataType::NBT(..) => ra
				.locals
				.get(reg)
				.ok_or(anyhow!("Register {reg} not allocated")),
		},
	}
}
