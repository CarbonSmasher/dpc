use anyhow::anyhow;

use crate::common::{condition::Condition, ty::DataTypeContents, val::Value};
use crate::mir::{MIRBlock, MIRInstrKind};
use crate::passes::{MIRPass, MIRPassData, Pass};
use crate::util::{remove_indices, HashSetEmptyTracker};

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
}

impl MIRPass for ConstConditionPass {
	fn run_pass(&mut self, data: &mut MIRPassData) -> anyhow::Result<()> {
		for func in data.mir.functions.values_mut() {
			let block = data
				.mir
				.blocks
				.get_mut(&func.block)
				.ok_or(anyhow!("Block does not exist"))?;

			let mut instrs_to_remove = HashSetEmptyTracker::new();
			loop {
				let run_again = run_const_condition_iter(block, &mut instrs_to_remove);
				if run_again {
					self.made_changes = true;
				} else {
					break;
				}
			}
			remove_indices(&mut block.contents, &instrs_to_remove);
		}

		Ok(())
	}
}

/// Runs an iteration of constant condition. Returns true if another iteration
/// should be run
fn run_const_condition_iter(
	block: &mut MIRBlock,
	instrs_to_remove: &mut HashSetEmptyTracker<usize>,
) -> bool {
	let mut run_again = false;

	for (i, instr) in block.contents.iter_mut().enumerate() {
		if instrs_to_remove.contains(&i) {
			continue;
		}

		if let MIRInstrKind::If { condition, body } = &mut instr.kind {
			let result = const_eval_condition(condition);
			if let Some(result) = result {
				if result {
					instr.kind = *body.clone();
				} else {
					instrs_to_remove.insert(i);
				}
				run_again = true;
			}
		}
	}

	run_again
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
		Condition::Exists(Value::Constant(..)) => Some(true),
		Condition::Not(condition) => const_eval_condition(condition).map(|x| !x),
		_ => None,
	}
}
