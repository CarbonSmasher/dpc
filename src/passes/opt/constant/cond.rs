use anyhow::anyhow;

use crate::{
	common::{condition::Condition, ty::DataTypeContents, val::Value},
	mir::{MIRBlock, MIRInstrKind},
	passes::{MIRPass, MIRPassData, Pass},
	util::{remove_indices, DashSetEmptyTracker},
};

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

impl Pass for ConstConditionPass {
	fn get_name(&self) -> &'static str {
		"const_condition"
	}
}

impl MIRPass for ConstConditionPass {
	fn run_pass(&mut self, data: &mut MIRPassData) -> anyhow::Result<()> {
		for (_, block) in &mut data.mir.functions {
			let block = data
				.mir
				.blocks
				.get_mut(block)
				.ok_or(anyhow!("Block does not exist"))?;

			let mut instrs_to_remove = DashSetEmptyTracker::new();
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

/// Runs an iteration of const prop. Returns true if another iteration
/// should be run
fn run_const_condition_iter(
	block: &mut MIRBlock,
	instrs_to_remove: &mut DashSetEmptyTracker<usize>,
) -> bool {
	let mut run_again = false;

	for (i, instr) in block.contents.iter_mut().enumerate() {
		if instrs_to_remove.contains(&i) {
			continue;
		}

		match &mut instr.kind {
			MIRInstrKind::If { condition, body } => {
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
			_ => {}
		};
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
