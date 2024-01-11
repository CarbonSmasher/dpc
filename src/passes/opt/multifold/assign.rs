use anyhow::anyhow;
use rustc_hash::FxHashMap;

use crate::common::condition::Condition;
use crate::common::reg::GetUsedRegs;
use crate::common::ty::DataTypeContents;
use crate::common::DeclareBinding;
use crate::common::{val::MutableValue, val::Value, Identifier};
use crate::mir::{MIRBlock, MIRInstrKind};
use crate::passes::{MIRPass, MIRPassData, Pass};
use crate::util::{remove_indices, HashSetEmptyTracker};

pub struct MultifoldAssignPass;

impl Pass for MultifoldAssignPass {
	fn get_name(&self) -> &'static str {
		"multifold_assign"
	}
}

impl MIRPass for MultifoldAssignPass {
	fn run_pass(&mut self, data: &mut MIRPassData) -> anyhow::Result<()> {
		for func in data.mir.functions.values_mut() {
			let block = data
				.mir
				.blocks
				.get_mut(&func.block)
				.ok_or(anyhow!("Block does not exist"))?;
			let mut removed_indices = HashSetEmptyTracker::new();
			loop {
				let run_again = run_iter(block, &mut removed_indices);
				if !run_again {
					break;
				}
			}
			remove_indices(&mut block.contents, &removed_indices);
		}

		Ok(())
	}
}

fn run_iter(block: &mut MIRBlock, removed_indices: &mut HashSetEmptyTracker<usize>) -> bool {
	let mut run_again = false;
	let mut if_cond_assign = FxHashMap::<Identifier, IfCondAssign>::default();

	for (i, instr) in block.contents.iter().enumerate() {
		// Even though this instruction hasn't actually been removed from the vec, we treat it
		// as if it has to prevent doing the same work over and over and actually iterating indefinitely
		if removed_indices.contains(&i) {
			continue;
		}
		match &instr.kind {
			MIRInstrKind::Assign {
				left: MutableValue::Reg(left),
				right: DeclareBinding::Value(Value::Constant(DataTypeContents::Score(val))),
			} => {
				let val = val.get_i32();
				let invert = if val == 0 {
					Some(false)
				} else if val == 1 {
					Some(true)
				} else {
					None
				};
				if let Some(invert) = invert {
					if_cond_assign.insert(
						left.clone(),
						IfCondAssign {
							finished: false,
							start_pos: i,
							end_pos: i,
							invert,
							condition: None,
						},
					);
				}
			}
			MIRInstrKind::If { condition, body } => match body.as_ref() {
				MIRInstrKind::Assign {
					left: MutableValue::Reg(left),
					right: DeclareBinding::Value(Value::Constant(DataTypeContents::Score(right))),
				} => {
					let right = right.get_i32();
					if right == 0 || right == 1 {
						if let Some(fold) = if_cond_assign.get_mut(left) {
							if !fold.finished {
								if (right == 1 && !fold.invert) || (right == 0 && fold.invert) {
									fold.end_pos = i;
									let mut condition = condition.clone();
									if fold.invert {
										condition = Condition::Not(Box::new(condition));
									}
									fold.condition = Some(condition);
								}
							}
						}
					}
				}
				_ => {}
			},
			other => {
				let used_regs = other.get_used_regs();
				for reg in used_regs {
					if_cond_assign.get_mut(reg).map(|x| x.finished = true);
				}
			}
		}
	}

	for (reg, fold) in if_cond_assign {
		if let Some(condition) = fold.condition {
			run_again = true;
			removed_indices.insert(fold.start_pos);
			block
				.contents
				.get_mut(fold.end_pos)
				.expect("Instr at pos does not exist")
				.kind = MIRInstrKind::Assign {
				left: MutableValue::Reg(reg),
				right: DeclareBinding::Condition(condition),
			};
		}
	}

	run_again
}

/// Simplifies:
/// let x = 0; if {condition}: x = 1
/// to:
/// let x = cond {condition}
struct IfCondAssign {
	finished: bool,
	start_pos: usize,
	end_pos: usize,
	/// Whether the pattern starts with x = 0 or x = 1
	invert: bool,
	condition: Option<Condition>,
}
