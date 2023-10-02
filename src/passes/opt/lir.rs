use std::collections::HashMap;

use anyhow::anyhow;
use rayon::prelude::*;

use crate::common::ty::{DataTypeContents, ScoreTypeContents};
use crate::common::Value;
use crate::lir::{LIRBlock, LIRInstrKind, LIRInstruction, LIR};
use crate::passes::LIRPass;
use crate::util::{insert_indices, remove_indices};

pub struct LIRSimplifyPass;

impl LIRPass for LIRSimplifyPass {
	fn run_pass(&mut self, lir: &mut LIR) -> anyhow::Result<()> {
		for (_, block) in &mut lir.functions {
			let block = lir
				.blocks
				.get_mut(block)
				.ok_or(anyhow!("Block does not exist"))?;
			loop {
				let run_again = run_lir_simplify_iter(block);
				if !run_again {
					break;
				}
			}
		}

		Ok(())
	}
}

/// Runs an iteration of the LIRSimplifyPass. Returns true if another iteration
/// should be run
fn run_lir_simplify_iter(block: &mut LIRBlock) -> bool {
	let mut run_again = false;

	let mut instrs_to_remove = Vec::new();
	for (i, instr) in block.contents.iter_mut().enumerate() {
		let remove = match &instr.kind {
			// Reflexive property; set to self or swap with self
			LIRInstrKind::SetScore(left, Value::Mutable(right))
			| LIRInstrKind::SwapScore(left, right)
				if left.is_same_val(right) =>
			{
				true
			}
			// Multiplies and divides by 1
			LIRInstrKind::MulScore(_, Value::Constant(DataTypeContents::Score(score)))
			| LIRInstrKind::DivScore(_, Value::Constant(DataTypeContents::Score(score)))
				if score.get_i32() == 1 =>
			{
				true
			}
			// Divides and modulos by zero, since these produce an error and don't change the score
			LIRInstrKind::DivScore(_, Value::Constant(DataTypeContents::Score(score)))
			| LIRInstrKind::ModScore(_, Value::Constant(DataTypeContents::Score(score)))
				if score.get_i32() == 0 =>
			{
				true
			}
			// Adds and subtracts by 0 or the integer limit don't do anything
			LIRInstrKind::AddScore(_, Value::Constant(DataTypeContents::Score(score)))
			| LIRInstrKind::SubScore(_, Value::Constant(DataTypeContents::Score(score)))
				if score.get_i32() == 0
					|| score.get_i32() == i32::MAX
					|| score.get_i32() == -i32::MAX =>
			{
				true
			}
			_ => false,
		};

		if remove {
			instrs_to_remove.push(i);
			run_again = true;
		}

		// Instructions to replace
		let kind_repl = match &instr.kind {
			// Add by negative is sub by positive
			LIRInstrKind::AddScore(left, Value::Constant(DataTypeContents::Score(score)))
				if score.get_i32().is_negative() =>
			{
				Some(LIRInstrKind::SubScore(
					left.clone(),
					Value::Constant(DataTypeContents::Score(ScoreTypeContents::Score(
						score.get_i32().abs(),
					))),
				))
			}
			// Sub by negative is add by positive
			LIRInstrKind::SubScore(left, Value::Constant(DataTypeContents::Score(score)))
				if score.get_i32().is_negative() =>
			{
				Some(LIRInstrKind::AddScore(
					left.clone(),
					Value::Constant(DataTypeContents::Score(ScoreTypeContents::Score(
						score.get_i32().abs(),
					))),
				))
			}
			// Mod by negative is same as mod by positive
			LIRInstrKind::ModScore(left, Value::Constant(DataTypeContents::Score(score)))
				if score.get_i32().is_negative() =>
			{
				Some(LIRInstrKind::ModScore(
					left.clone(),
					Value::Constant(DataTypeContents::Score(ScoreTypeContents::Score(
						score.get_i32().abs(),
					))),
				))
			}
			// Div by -1 is same as mul by -1
			LIRInstrKind::DivScore(left, Value::Constant(DataTypeContents::Score(score)))
				if score.get_i32() == -1 =>
			{
				Some(LIRInstrKind::MulScore(
					left.clone(),
					Value::Constant(DataTypeContents::Score(ScoreTypeContents::Score(-1))),
				))
			}
			// x / x == 1
			LIRInstrKind::DivScore(left, Value::Mutable(right)) if left.is_same_val(right) => {
				Some(LIRInstrKind::SetScore(
					left.clone(),
					Value::Constant(DataTypeContents::Score(ScoreTypeContents::Score(1))),
				))
			}
			// x * 0 == 0
			LIRInstrKind::MulScore(left, Value::Constant(DataTypeContents::Score(score)))
				if score.get_i32() == 0 =>
			{
				Some(LIRInstrKind::SetScore(
					left.clone(),
					Value::Constant(DataTypeContents::Score(ScoreTypeContents::Score(0))),
				))
			}
			// x % 1 always equals 0
			LIRInstrKind::ModScore(left, Value::Constant(DataTypeContents::Score(score)))
				if score.get_i32() == 1 =>
			{
				Some(LIRInstrKind::SetScore(
					left.clone(),
					Value::Constant(DataTypeContents::Score(ScoreTypeContents::Score(0))),
				))
			}
			// x - x == 0
			LIRInstrKind::SubScore(left, Value::Mutable(right)) if left.is_same_val(right) => {
				Some(LIRInstrKind::SetScore(
					left.clone(),
					Value::Constant(DataTypeContents::Score(ScoreTypeContents::Score(0))),
				))
			}
			// x * 2 == x + x, which is faster
			LIRInstrKind::MulScore(left, Value::Constant(DataTypeContents::Score(score)))
				if score.get_i32() == 2 =>
			{
				Some(LIRInstrKind::AddScore(
					left.clone(),
					Value::Mutable(left.clone()),
				))
			}
			// x / integer limit always equals 0
			LIRInstrKind::DivScore(left, Value::Constant(DataTypeContents::Score(score)))
				if score.get_i32() == i32::MAX || score.get_i32() == -i32::MAX =>
			{
				Some(LIRInstrKind::SetScore(
					left.clone(),
					Value::Constant(DataTypeContents::Score(ScoreTypeContents::Score(0))),
				))
			}
			_ => None,
		};

		if let Some(kind_repl) = kind_repl {
			instr.kind = kind_repl;
			run_again = true;
		}
	}

	remove_indices(&mut block.contents, &instrs_to_remove);

	run_again
}

pub struct InsertRegFinishesPass;

impl LIRPass for InsertRegFinishesPass {
	fn run_pass(&mut self, lir: &mut LIR) -> anyhow::Result<()> {
		for (_, block) in &mut lir.functions {
			let block = lir
				.blocks
				.get_mut(block)
				.ok_or(anyhow!("Block does not exist"))?;
			let mut last_used_positions = HashMap::new();
			for (i, instr) in block.contents.iter().enumerate() {
				for reg in instr.kind.get_used_regs() {
					last_used_positions.insert(reg.clone(), i);
				}
			}

			let finish_instrs: Vec<_> = last_used_positions
				.par_iter()
				.map(|(reg, pos)| {
					(
						pos + 1,
						LIRInstruction::new(LIRInstrKind::FinishUsing(reg.clone())),
					)
				})
				.collect();
			block.contents = insert_indices(block.contents.clone(), &finish_instrs);
		}

		Ok(())
	}
}
