use std::collections::HashSet;

use anyhow::anyhow;

use crate::common::{ty::DataTypeContents, MutableValue, Value};
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
	let regs = alloc_block_registers(block, &mut ccx.racx);

	let mut out = Vec::new();
	for instr in &block.contents {
		let commands = codegen_instr(instr, &regs, ccx)?;
		out.extend(commands);
	}

	Ok(out)
}

pub fn codegen_instr(
	instr: &LIRInstruction,
	regs: &RegAllocResult,
	ccx: &mut CodegenCx,
) -> anyhow::Result<Vec<String>> {
	let mut out = Vec::new();

	match &instr.kind {
		LIRInstrKind::SetScore(left, right) => {
			let left_scoreholder = get_mut_val_reg(&left, regs)?;
			let cmd = match right {
				Value::Constant(data) => {
					let lit = match data {
						DataTypeContents::Score(score) => score.get_literal_str(),
					};
					format!("scoreboard players set {left_scoreholder} {REG_OBJECTIVE} {lit}")
				}
				Value::Mutable(val) => {
					let right_scoreholder = get_mut_val_reg(&val, regs)?;
					format!("scoreboard players operation {left_scoreholder} {REG_OBJECTIVE} = {right_scoreholder} {REG_OBJECTIVE}")
				}
			};
			out.push(cmd);
		}
		LIRInstrKind::AddScore(left, right) => {
			let left_scoreholder = get_mut_val_reg(&left, regs)?;
			let cmd = match right {
				Value::Constant(data) => {
					let lit = match data {
						DataTypeContents::Score(score) => score.get_i32(),
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
					let right_scoreholder = get_mut_val_reg(&val, regs)?;
					format!("scoreboard players operation {left_scoreholder} {REG_OBJECTIVE} += {right_scoreholder} {REG_OBJECTIVE}")
				}
			};
			out.push(cmd);
		}
		LIRInstrKind::SubScore(left, right) => {
			let left_scoreholder = get_mut_val_reg(&left, regs)?;
			let cmd = match right {
				Value::Constant(data) => {
					let lit = match data {
						DataTypeContents::Score(score) => score.get_i32(),
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
					let right_scoreholder = get_mut_val_reg(&val, regs)?;
					format!("scoreboard players operation {left_scoreholder} {REG_OBJECTIVE} -= {right_scoreholder} {REG_OBJECTIVE}")
				}
			};
			out.push(cmd);
		}
		LIRInstrKind::MulScore(left, right) => {
			let left_scoreholder = get_mut_val_reg(&left, regs)?;
			let (right_score, to_add) = get_val_score(&right, regs)?;
			ccx.score_literals.extend(to_add);

			let cmd = format!(
				"scoreboard players operation {left_scoreholder} {REG_OBJECTIVE} *= {} {}",
				right_score.holder.codegen_str(),
				right_score.objective
			);

			out.push(cmd);
		}
		LIRInstrKind::DivScore(left, right) => {
			let left_scoreholder = get_mut_val_reg(&left, regs)?;
			let (right_score, to_add) = get_val_score(&right, regs)?;
			ccx.score_literals.extend(to_add);

			let cmd = format!(
				"scoreboard players operation {left_scoreholder} {REG_OBJECTIVE} /= {} {}",
				right_score.holder.codegen_str(),
				right_score.objective
			);

			out.push(cmd);
		}
		LIRInstrKind::ModScore(left, right) => {
			let left_scoreholder = get_mut_val_reg(&left, regs)?;
			let (right_score, to_add) = get_val_score(&right, regs)?;
			ccx.score_literals.extend(to_add);

			let cmd = format!(
				"scoreboard players operation {left_scoreholder} {REG_OBJECTIVE} %= {} {}",
				right_score.holder.codegen_str(),
				right_score.objective
			);

			out.push(cmd);
		}
		LIRInstrKind::MinScore(left, right) => {
			let left_scoreholder = get_mut_val_reg(&left, regs)?;
			let (right_score, to_add) = get_val_score(&right, regs)?;
			ccx.score_literals.extend(to_add);

			let cmd = format!(
				"scoreboard players operation {left_scoreholder} {REG_OBJECTIVE} < {} {}",
				right_score.holder.codegen_str(),
				right_score.objective
			);

			out.push(cmd);
		}
		LIRInstrKind::MaxScore(left, right) => {
			let left_scoreholder = get_mut_val_reg(&left, regs)?;
			let (right_score, to_add) = get_val_score(&right, regs)?;
			ccx.score_literals.extend(to_add);

			let cmd = format!(
				"scoreboard players operation {left_scoreholder} {REG_OBJECTIVE} > {} {}",
				right_score.holder.codegen_str(),
				right_score.objective
			);

			out.push(cmd);
		}
		LIRInstrKind::SwapScore(left, right) => {
			let left_scoreholder = get_mut_val_reg(&left, regs)?;
			let right_scoreholder = get_mut_val_reg(&right, regs)?;

			let cmd = format!(
				"scoreboard players operation {left_scoreholder} {REG_OBJECTIVE} >< {right_scoreholder} {REG_OBJECTIVE}"
			);

			out.push(cmd);
		}
	}

	Ok(out)
}

/// Returns a score and an optional score literal to add
fn get_val_score(val: &Value, regs: &RegAllocResult) -> anyhow::Result<(Score, Option<i32>)> {
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
		},
		Value::Mutable(val) => {
			let score = Score::new(
				TargetSelector::Player(get_mut_val_reg(val, regs)?.clone()),
				REG_OBJECTIVE.into(),
			);
			(score, None)
		}
	};

	Ok(out)
}

fn get_mut_val_reg<'regs>(
	val: &MutableValue,
	regs: &'regs RegAllocResult,
) -> anyhow::Result<&'regs String> {
	match val {
		MutableValue::Register(reg) => regs
			.regs
			.get(reg)
			.ok_or(anyhow!("Register {reg} not allocated")),
	}
}
