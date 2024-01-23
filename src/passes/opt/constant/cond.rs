use intset::GrowSet;

use crate::common::ty::ScoreTypeContents;
use crate::common::DeclareBinding;
use crate::common::{condition::Condition, ty::DataTypeContents, val::Value};
use crate::mir::{MIRBlock, MIRInstrKind, MIRInstruction};
use crate::passes::{MIRPass, MIRPassData, Pass};
use crate::project::{OptimizationLevel, ProjectSettings};
use crate::util::replace_and_expand_indices;

pub struct ConstConditionPass {
	pub(super) made_changes: bool,
}

impl ConstConditionPass {
	pub fn new() -> Self {
		Self {
			made_changes: false,
		}
	}
}

impl Default for ConstConditionPass {
	fn default() -> Self {
		Self::new()
	}
}

impl Pass for ConstConditionPass {
	fn get_name(&self) -> &'static str {
		"const_condition"
	}

	fn should_run(&self, proj: &ProjectSettings) -> bool {
		proj.op_level >= OptimizationLevel::Basic
	}
}

impl MIRPass for ConstConditionPass {
	fn run_pass(&mut self, data: &mut MIRPassData) -> anyhow::Result<()> {
		for func in data.mir.functions.values_mut() {
			let block = &mut func.block;

			let mut instrs_to_remove = GrowSet::with_capacity(block.contents.len());
			let mut instrs_to_replace = Vec::new();
			loop {
				let run_again =
					run_const_condition_iter(block, &mut instrs_to_remove, &mut instrs_to_replace);
				if run_again {
					self.made_changes = true;
				} else {
					break;
				}
			}
			if self.made_changes {
				block.contents =
					replace_and_expand_indices(block.contents.clone(), &instrs_to_replace);
			}
		}

		Ok(())
	}
}

/// Runs an iteration of constant condition. Returns true if another iteration
/// should be run
fn run_const_condition_iter(
	block: &mut MIRBlock,
	instrs_to_remove: &mut GrowSet,
	instrs_to_replace: &mut Vec<(usize, Vec<MIRInstruction>)>,
) -> bool {
	let mut run_again = false;

	for (i, instr) in block.contents.iter_mut().enumerate() {
		if instrs_to_remove.contains(i) {
			continue;
		}

		match &mut instr.kind {
			MIRInstrKind::If { condition, body } => {
				let result = const_eval_result(condition);
				if let Some(result) = result {
					if result {
						instrs_to_replace.push((i, body.contents.clone()));
					} else {
						instrs_to_replace.push((i, Vec::new()));
					}
					instrs_to_remove.add(i);
					run_again = true;
				}
			}
			MIRInstrKind::IfElse {
				condition,
				first,
				second,
			} => {
				let result = const_eval_result(condition);
				if let Some(result) = result {
					if result {
						instrs_to_replace.push((i, first.contents.clone()));
					} else {
						instrs_to_replace.push((i, second.contents.clone()));
					}
					instrs_to_remove.add(i);
					run_again = true;
				}
			}
			MIRInstrKind::Assign {
				left,
				right: DeclareBinding::Condition(condition),
			} => {
				let result = const_eval_result(condition);
				if let Some(result) = result {
					instr.kind = MIRInstrKind::Assign {
						left: left.clone(),
						right: DeclareBinding::Value(Value::Constant(DataTypeContents::Score(
							ScoreTypeContents::Bool(result),
						))),
					};
				}
			}
			_ => {}
		}
	}

	run_again
}

/// Tries to constant eval parts of a condition, then returns a constant
/// result of the whole condition if possible
fn const_eval_result(condition: &mut Condition) -> Option<bool> {
	match condition {
		Condition::Not(inner) => {
			let result = const_eval_result(inner);
			result.map(|x| !x)
		}
		Condition::And(l, r) => {
			if let (Some(l), Some(r)) = (const_eval_result(l), const_eval_result(r)) {
				Some(l && r)
			} else {
				None
			}
		}
		Condition::Or(l, r) => {
			if let (Some(l), Some(r)) = (const_eval_result(l), const_eval_result(r)) {
				Some(l || r)
			} else {
				None
			}
		}
		Condition::Xor(l, r) => {
			if let (Some(l), Some(r)) = (const_eval_result(l), const_eval_result(r)) {
				Some(l ^ r)
			} else {
				None
			}
		}
		_ => {
			if let Some(result) = const_eval_condition(condition) {
				*condition = Condition::Bool(Value::Constant(DataTypeContents::Score(
					ScoreTypeContents::Bool(result),
				)));
				Some(result)
			} else {
				None
			}
		}
	}
}

fn const_eval_condition(condition: &Condition) -> Option<bool> {
	match condition {
		Condition::Equal(
			Value::Constant(DataTypeContents::Score(l)),
			Value::Constant(DataTypeContents::Score(r)),
		) => Some(l.get_i32() == r.get_i32()),
		Condition::GreaterThan(
			Value::Constant(DataTypeContents::Score(l)),
			Value::Constant(DataTypeContents::Score(r)),
		) => Some(l.get_i32() > r.get_i32()),
		Condition::GreaterThanOrEqual(
			Value::Constant(DataTypeContents::Score(l)),
			Value::Constant(DataTypeContents::Score(r)),
		) => Some(l.get_i32() >= r.get_i32()),
		Condition::LessThan(
			Value::Constant(DataTypeContents::Score(l)),
			Value::Constant(DataTypeContents::Score(r)),
		) => Some(l.get_i32() < r.get_i32()),
		Condition::LessThanOrEqual(
			Value::Constant(DataTypeContents::Score(l)),
			Value::Constant(DataTypeContents::Score(r)),
		) => Some(l.get_i32() <= r.get_i32()),
		Condition::Bool(Value::Constant(DataTypeContents::Score(val))) => Some(val.get_i32() == 1),
		Condition::NotBool(Value::Constant(DataTypeContents::Score(val))) => {
			Some(val.get_i32() == 0)
		}
		Condition::Exists(Value::Constant(..)) => Some(true),
		Condition::Not(condition) => const_eval_condition(condition).map(|x| !x),
		Condition::And(l, r) => {
			if let (Some(l), Some(r)) = (const_eval_condition(l), const_eval_condition(r)) {
				Some(l && r)
			} else {
				None
			}
		}
		Condition::Or(l, r) => {
			if let (Some(l), Some(r)) = (const_eval_condition(l), const_eval_condition(r)) {
				Some(l || r)
			} else {
				None
			}
		}
		Condition::Xor(l, r) => {
			if let (Some(l), Some(r)) = (const_eval_condition(l), const_eval_condition(r)) {
				Some(l ^ r)
			} else {
				None
			}
		}
		_ => None,
	}
}
