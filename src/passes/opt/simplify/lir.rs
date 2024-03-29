use crate::common::mc::instr::MinecraftInstr;
use crate::common::mc::modifier::Modifier;
use crate::common::ty::ScoreTypeContents;
use crate::common::val::ScoreValue;
use crate::lir::{LIRBlock, LIRInstrKind};
use crate::passes::util::RunAgain;
use crate::passes::{LIRPass, LIRPassData, Pass};
use crate::project::{OptimizationLevel, ProjectSettings};
use crate::util::{remove_indices, HashSetEmptyTracker};

pub struct LIRSimplifyPass;

impl Pass for LIRSimplifyPass {
	fn get_name(&self) -> &'static str {
		"simplify_lir"
	}

	fn should_run(&self, proj: &ProjectSettings) -> bool {
		proj.op_level >= OptimizationLevel::Basic
	}
}

impl LIRPass for LIRSimplifyPass {
	fn run_pass(&mut self, data: &mut LIRPassData) -> anyhow::Result<()> {
		for func in data.lir.functions.values_mut() {
			simplify_block(&mut func.block);
		}

		Ok(())
	}
}

fn simplify_block(block: &mut LIRBlock) -> RunAgain {
	let mut out = RunAgain::new();
	let mut instrs_to_remove = HashSetEmptyTracker::new();
	loop {
		let run_again = run_iter(block, &mut instrs_to_remove);
		out.merge(run_again);
		if !run_again {
			break;
		}
	}
	remove_indices(&mut block.contents, &instrs_to_remove);
	out
}

fn run_iter(block: &mut LIRBlock, instrs_to_remove: &mut HashSetEmptyTracker<usize>) -> RunAgain {
	let mut run_again = RunAgain::new();

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
			// Adds and subtracts by 0 don't do anything
			LIRInstrKind::AddScore(_, ScoreValue::Constant(score))
			| LIRInstrKind::SubScore(_, ScoreValue::Constant(score))
				if score.get_i32() == 0 =>
			{
				true
			}
			_ => false,
		};

		if remove {
			let is_new = instrs_to_remove.insert(i);
			if is_new {
				run_again.yes();
			}
			continue;
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
			// Teleport with rotation and a zero rotation can be simplified to just a teleport
			LIRInstrKind::MC(MinecraftInstr::TeleportWithRotation {
				source,
				dest,
				rotation,
			}) if rotation.are_zero() => Some(LIRInstrKind::MC(MinecraftInstr::TeleportToLocation {
				source: source.clone(),
				dest: dest.clone(),
			})),
			// Return run get const is replaced with return value
			LIRInstrKind::ReturnRun(body) => match body.as_ref().kind {
				LIRInstrKind::GetConst(val) => Some(LIRInstrKind::ReturnValue(val)),
				_ => None,
			},
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
		run_again.yes();
	}

	run_again
}
