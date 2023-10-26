use crate::common::modifier::Modifier;
use crate::common::ty::{DataTypeContents, ScoreTypeContents};
use crate::common::{DeclareBinding, ScoreValue, Value};
use crate::lir::{LIRBlock, LIRInstrKind, LIR};
use crate::mir::{MIRBlock, MIRInstrKind, MIR};
use crate::passes::{LIRPass, MIRPass, Pass};
use crate::util::remove_indices;

use anyhow::anyhow;
use dashmap::DashSet;

pub struct MIRSimplifyPass;

impl Pass for MIRSimplifyPass {
	fn get_name(&self) -> &'static str {
		"simplify_mir"
	}
}

impl MIRPass for MIRSimplifyPass {
	fn run_pass(&mut self, mir: &mut MIR) -> anyhow::Result<()> {
		for (_, block) in &mut mir.functions {
			let block = mir
				.blocks
				.get_mut(block)
				.ok_or(anyhow!("Block does not exist"))?;

			let mut instrs_to_remove = DashSet::new();
			loop {
				let run_again = run_mir_simplify_iter(block, &mut instrs_to_remove);
				if !run_again {
					break;
				}
			}
			remove_indices(&mut block.contents, &instrs_to_remove);
		}

		Ok(())
	}
}

/// Runs an iteration of the MIRSimplifyPass. Returns true if another iteration
/// should be run
fn run_mir_simplify_iter(block: &mut MIRBlock, instrs_to_remove: &mut DashSet<usize>) -> bool {
	let mut run_again = false;

	for (i, instr) in block.contents.iter_mut().enumerate() {
		let remove = match &instr.kind {
			// Reflexive property; set or swap with self
			// and also min and max with self
			MIRInstrKind::Assign {
				left,
				right: DeclareBinding::Value(Value::Mutable(right)),
			}
			| MIRInstrKind::Swap { left, right }
			| MIRInstrKind::Min {
				left,
				right: Value::Mutable(right),
			}
			| MIRInstrKind::Max {
				left,
				right: Value::Mutable(right),
			} if left.is_same_val(right) => true,
			// Multiplies and divides by 1
			MIRInstrKind::Mul {
				left: _,
				right: Value::Constant(DataTypeContents::Score(score)),
			}
			| MIRInstrKind::Div {
				left: _,
				right: Value::Constant(DataTypeContents::Score(score)),
			} if score.get_i32() == 1 => true,
			// x / 0 and x % 0 error and dont do anything
			MIRInstrKind::Div {
				left: _,
				right: Value::Constant(DataTypeContents::Score(score)),
			}
			| MIRInstrKind::Mod {
				left: _,
				right: Value::Constant(DataTypeContents::Score(score)),
			} if score.get_i32() == 0 => true,
			_ => false,
		};

		if remove {
			let is_new = instrs_to_remove.insert(i);
			if is_new {
				run_again = true;
			}
		}

		// Instructions to replace
		let kind_repl = match &instr.kind {
			// Div by -1 is same as mul by -1
			MIRInstrKind::Div {
				left,
				right: Value::Constant(DataTypeContents::Score(score)),
			} if score.get_i32() == -1 => Some(MIRInstrKind::Mul {
				left: left.clone(),
				right: Value::Constant(DataTypeContents::Score(ScoreTypeContents::Score(-1))),
			}),
			// x ^ 0 = 1
			MIRInstrKind::Pow { base, exp: 0 } => Some(MIRInstrKind::Assign {
				left: base.clone(),
				right: DeclareBinding::Value(Value::Constant(DataTypeContents::Score(
					ScoreTypeContents::Score(1),
				))),
			}),
			// A couple of canonicalizations that just help out const prop and const fold
			// x / x = 1
			MIRInstrKind::Div {
				left,
				right: Value::Mutable(right),
			} if left.is_same_val(right) => Some(MIRInstrKind::Assign {
				left: left.clone(),
				right: DeclareBinding::Value(Value::Constant(DataTypeContents::Score(
					ScoreTypeContents::Score(1),
				))),
			}),
			// x - x = 0
			MIRInstrKind::Sub {
				left,
				right: Value::Mutable(right),
			} if left.is_same_val(right) => Some(MIRInstrKind::Assign {
				left: left.clone(),
				right: DeclareBinding::Value(Value::Constant(DataTypeContents::Score(
					ScoreTypeContents::Score(0),
				))),
			}),
			// x + x = x * 2
			MIRInstrKind::Add {
				left,
				right: Value::Mutable(right),
			} if left.is_same_val(right) => Some(MIRInstrKind::Mul {
				left: left.clone(),
				right: Value::Constant(DataTypeContents::Score(ScoreTypeContents::Score(2))),
			}),
			// x * x = x ^ 2
			MIRInstrKind::Mul {
				left,
				right: Value::Mutable(right),
			} if left.is_same_val(right) => Some(MIRInstrKind::Pow {
				base: left.clone(),
				exp: 2,
			}),
			// x % x = 0
			MIRInstrKind::Mod {
				left,
				right: Value::Mutable(right),
			} if left.is_same_val(right) => Some(MIRInstrKind::Assign {
				left: left.clone(),
				right: DeclareBinding::Value(Value::Constant(DataTypeContents::Score(
					ScoreTypeContents::Score(0),
				))),
			}),
			_ => None,
		};

		if let Some(kind_repl) = kind_repl {
			instr.kind = kind_repl;
			run_again = true;
		}
	}

	run_again
}

pub struct LIRSimplifyPass;

impl Pass for LIRSimplifyPass {
	fn get_name(&self) -> &'static str {
		"simplify_lir"
	}
}

impl LIRPass for LIRSimplifyPass {
	fn run_pass(&mut self, lir: &mut LIR) -> anyhow::Result<()> {
		for (_, block) in &mut lir.functions {
			let block = lir
				.blocks
				.get_mut(block)
				.ok_or(anyhow!("Block does not exist"))?;

			// We persist the same set of removed instructions across all iterations
			// so that we only have to run the vec retain operation once, saving a lot of copies
			let mut instrs_to_remove = DashSet::new();
			loop {
				let run_again = run_lir_simplify_iter(block, &mut instrs_to_remove);
				if !run_again {
					break;
				}
			}
			remove_indices(&mut block.contents, &instrs_to_remove);
		}

		Ok(())
	}
}

/// Runs an iteration of the LIRSimplifyPass. Returns true if another iteration
/// should be run
fn run_lir_simplify_iter(block: &mut LIRBlock, instrs_to_remove: &mut DashSet<usize>) -> bool {
	let mut run_again = false;

	for (i, instr) in block.contents.iter().enumerate() {
		// Don't remove instructions that store their result or success
		if instr
			.modifiers
			.iter()
			.any(|x| matches!(x, Modifier::StoreResult(..) | Modifier::StoreSuccess(..)))
		{
			continue;
		}
		let remove = match &instr.kind {
			// Reflexive property; set to self or swap with self
			// and also mins and maxes with self
			LIRInstrKind::SetScore(left, ScoreValue::Mutable(right))
			| LIRInstrKind::SwapScore(left, right)
			| LIRInstrKind::MinScore(left, ScoreValue::Mutable(right))
			| LIRInstrKind::MaxScore(left, ScoreValue::Mutable(right))
				if left.is_value_eq(right) =>
			{
				true
			}
			// Multiplies and divides by 1
			LIRInstrKind::MulScore(_, ScoreValue::Constant(score))
			| LIRInstrKind::DivScore(_, ScoreValue::Constant(score))
				if score.get_i32() == 1 =>
			{
				true
			}
			// Divides and modulos by zero, since these produce an error and don't change the score.
			// However, if the success of this operation was stored somewhere, we need to respect that
			LIRInstrKind::DivScore(_, ScoreValue::Constant(score))
			| LIRInstrKind::ModScore(_, ScoreValue::Constant(score))
				if score.get_i32() == 0 =>
			{
				true
			}
			// Adds and subtracts by 0 or the integer limit don't do anything
			LIRInstrKind::AddScore(_, ScoreValue::Constant(score))
			| LIRInstrKind::SubScore(_, ScoreValue::Constant(score))
				if score.get_i32() == 0
					|| score.get_i32() == i32::MAX
					|| score.get_i32() == -i32::MAX =>
			{
				true
			}
			_ => false,
		};

		if remove {
			let is_new = instrs_to_remove.insert(i);
			if is_new {
				run_again = true;
			}
		}
	}

	let repl_mutated = block.contents.iter_mut().fold(false, |out, instr| {
		// Instructions to replace
		let kind_repl = match &instr.kind {
			// Add by negative is sub by positive
			LIRInstrKind::AddScore(left, ScoreValue::Constant(score))
				if score.get_i32().is_negative() =>
			{
				Some(LIRInstrKind::SubScore(
					left.clone(),
					ScoreValue::Constant(ScoreTypeContents::Score(score.get_i32().abs())),
				))
			}
			// Sub by negative is add by positive
			LIRInstrKind::SubScore(left, ScoreValue::Constant(score))
				if score.get_i32().is_negative() =>
			{
				Some(LIRInstrKind::AddScore(
					left.clone(),
					ScoreValue::Constant(ScoreTypeContents::Score(score.get_i32().abs())),
				))
			}
			// Mod by negative is same as mod by positive
			LIRInstrKind::ModScore(left, ScoreValue::Constant(score))
				if score.get_i32().is_negative() =>
			{
				Some(LIRInstrKind::ModScore(
					left.clone(),
					ScoreValue::Constant(ScoreTypeContents::Score(score.get_i32().abs())),
				))
			}
			// Div by -1 is same as mul by -1
			LIRInstrKind::DivScore(left, ScoreValue::Constant(score)) if score.get_i32() == -1 => {
				Some(LIRInstrKind::MulScore(
					left.clone(),
					ScoreValue::Constant(ScoreTypeContents::Score(-1)),
				))
			}
			// x / x == 1
			LIRInstrKind::DivScore(left, ScoreValue::Mutable(right)) if left.is_value_eq(right) => {
				Some(LIRInstrKind::SetScore(
					left.clone(),
					ScoreValue::Constant(ScoreTypeContents::Score(1)),
				))
			}
			// x * 0 == 0
			LIRInstrKind::MulScore(left, ScoreValue::Constant(score)) if score.get_i32() == 0 => {
				Some(LIRInstrKind::SetScore(
					left.clone(),
					ScoreValue::Constant(ScoreTypeContents::Score(0)),
				))
			}
			// x % 1 always equals 0
			LIRInstrKind::ModScore(left, ScoreValue::Constant(score)) if score.get_i32() == 1 => {
				Some(LIRInstrKind::SetScore(
					left.clone(),
					ScoreValue::Constant(ScoreTypeContents::Score(0)),
				))
			}
			// x - x == 0
			LIRInstrKind::SubScore(left, ScoreValue::Mutable(right)) if left.is_value_eq(right) => {
				Some(LIRInstrKind::SetScore(
					left.clone(),
					ScoreValue::Constant(ScoreTypeContents::Score(0)),
				))
			}
			// x * 2 == x + x, which is faster
			LIRInstrKind::MulScore(left, ScoreValue::Constant(score)) if score.get_i32() == 2 => {
				Some(LIRInstrKind::AddScore(
					left.clone(),
					ScoreValue::Mutable(left.clone()),
				))
			}
			// x / integer limit always equals 0
			LIRInstrKind::DivScore(left, ScoreValue::Constant(score))
				if score.get_i32() == i32::MAX || score.get_i32() == -i32::MAX =>
			{
				Some(LIRInstrKind::SetScore(
					left.clone(),
					ScoreValue::Constant(ScoreTypeContents::Score(0)),
				))
			}
			_ => None,
		};

		if let Some(kind_repl) = kind_repl {
			instr.kind = kind_repl;
			true
		} else {
			out
		}
	});

	if repl_mutated {
		run_again = true;
	}

	run_again
}
