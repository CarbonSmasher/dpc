use anyhow::anyhow;
use dashmap::{DashMap, DashSet};

use crate::common::mc::EntityTarget;
use crate::common::modifier::{
	IfModCondition, IfScoreCondition, IfScoreRangeEnd, Modifier, StoreModLocation,
};
use crate::common::target_selector::{SelectorType, TargetSelector};
use crate::common::ty::{DataTypeContents, ScoreTypeContents};
use crate::common::{Identifier, MutableValue, Value};
use crate::lir::{LIRBlock, LIRInstrKind, LIR};
use crate::passes::{LIRPass, Pass};
use crate::util::remove_indices;

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
			// Divides and modulos by zero, since these produce an error and don't change the score.
			// However, if the success of this operation was stored somewhere, we need to respect that
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

pub struct SimplifyModifiersPass;

impl Pass for SimplifyModifiersPass {
	fn get_name(&self) -> &'static str {
		"simplify_modifiers"
	}
}

impl LIRPass for SimplifyModifiersPass {
	fn run_pass(&mut self, lir: &mut LIR) -> anyhow::Result<()> {
		let mods_to_remove = DashSet::new();

		for (_, block) in &mut lir.functions {
			let block = lir
				.blocks
				.get_mut(block)
				.ok_or(anyhow!("Block does not exist"))?;

			for instr in &mut block.contents {
				if !mods_to_remove.is_empty() {
					mods_to_remove.clear();
				}
				for (i, modifier) in instr.modifiers.iter_mut().enumerate() {
					match modifier {
						Modifier::As(EntityTarget::Selector(TargetSelector {
							selector: SelectorType::This,
							params,
						})) if params.is_empty() => {
							mods_to_remove.insert(i);
						}
						Modifier::Positioned(coords) if coords.are_zero() => {
							mods_to_remove.insert(i);
						}
						Modifier::If { condition, .. } => {
							let remove = optimize_condition(condition);
							if remove {
								mods_to_remove.insert(i);
							}
						}
						_ => {}
					}
				}

				remove_indices(&mut instr.modifiers, &mods_to_remove);
			}
		}

		Ok(())
	}
}

fn optimize_condition(condition: &mut Box<IfModCondition>) -> bool {
	match **condition {
		// Unbounded check doesnt do anything
		IfModCondition::Score(IfScoreCondition::Range {
			left: IfScoreRangeEnd::Infinite,
			right: IfScoreRangeEnd::Infinite,
			..
		}) => {
			return true;
		}
		_ => {}
	}

	false
}

pub struct ScoreboardDataflowPass;

impl Pass for ScoreboardDataflowPass {
	fn get_name(&self) -> &'static str {
		"scoreboard_dataflow"
	}
}

impl LIRPass for ScoreboardDataflowPass {
	fn run_pass(&mut self, lir: &mut LIR) -> anyhow::Result<()> {
		let mut flow_points = DashMap::new();
		let mut finished_flow_points = Vec::new();
		let mut instrs_to_remove = DashSet::new();

		for (_, block) in &mut lir.functions {
			instrs_to_remove.clear();

			let block = lir
				.blocks
				.get_mut(block)
				.ok_or(anyhow!("Block does not exist"))?;

			loop {
				flow_points.clear();
				finished_flow_points.clear();
				let run_again = run_scoreboard_dataflow_iter(
					block,
					&mut instrs_to_remove,
					&mut flow_points,
					&mut finished_flow_points,
				);
				if !run_again {
					break;
				}
			}

			remove_indices(&mut block.contents, &instrs_to_remove);
		}

		Ok(())
	}
}

fn run_scoreboard_dataflow_iter(
	block: &mut LIRBlock,
	instrs_to_remove: &mut DashSet<usize>,
	flow_points: &mut DashMap<Identifier, SBDataflowPoint>,
	finished_flow_points: &mut Vec<SBDataflowPoint>,
) -> bool {
	let mut run_again = false;

	for (i, instr) in block.contents.iter_mut().enumerate() {
		if instrs_to_remove.contains(&i) {
			continue;
		}
		match &instr.kind {
			LIRInstrKind::SetScore(left, right) => {
				let MutableValue::Register(left) = left;
				if let Value::Mutable(MutableValue::Register(right)) = right {
					if let Some(mut point) = flow_points.get_mut(right) {
						point.store_regs.push(left.clone());
						instrs_to_remove.insert(i);
					}
				}
			}
			LIRInstrKind::AddScore(left, right)
			| LIRInstrKind::SubScore(left, right)
			| LIRInstrKind::MulScore(left, right)
			| LIRInstrKind::DivScore(left, right)
			| LIRInstrKind::ModScore(left, right)
			| LIRInstrKind::MinScore(left, right)
			| LIRInstrKind::MaxScore(left, right) => {
				let MutableValue::Register(left) = left;
				if let Value::Mutable(MutableValue::Register(right)) = right {
					finished_flow_points.extend(flow_points.remove(right).map(|x| x.1));
				}

				flow_points.insert(
					left.clone(),
					SBDataflowPoint {
						pos: i,
						store_regs: Vec::new(),
					},
				);
			}
			LIRInstrKind::SwapScore(left, right) => {
				let MutableValue::Register(left) = left;
				let MutableValue::Register(right) = right;
				finished_flow_points.extend(flow_points.remove(left).map(|x| x.1));
				finished_flow_points.extend(flow_points.remove(right).map(|x| x.1));
			}
			LIRInstrKind::ConstIndexToScore { score: reg, .. } | LIRInstrKind::Use(reg) => {
				let MutableValue::Register(reg) = reg;
				finished_flow_points.extend(flow_points.remove(reg).map(|x| x.1));
			}
			_ => {}
		};
	}

	for point in flow_points
		.iter()
		.map(|x| x.value().clone())
		.chain(finished_flow_points.iter().cloned())
	{
		if let Some(instr) = block.contents.get_mut(point.pos) {
			if !point.store_regs.is_empty() {
				run_again = true;
			}

			instr.modifiers.extend(
				point
					.store_regs
					.into_iter()
					.map(|x| Modifier::StoreResult(StoreModLocation::Reg(x))),
			);
		} else {
			continue;
		}
	}

	run_again
}

#[derive(Debug, Clone)]
struct SBDataflowPoint {
	pos: usize,
	store_regs: Vec<Identifier>,
}
