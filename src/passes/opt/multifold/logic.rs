use anyhow::anyhow;
use rustc_hash::FxHashMap;

use crate::common::condition::Condition;
use crate::common::reg::GetUsedRegs;
use crate::common::DeclareBinding;
use crate::common::{val::MutableValue, val::Value, Identifier};
use crate::mir::{MIRBlock, MIRInstrKind, MIRInstruction};
use crate::passes::{MIRPass, MIRPassData, Pass};
use crate::util::{remove_indices, HashSetEmptyTracker};

pub struct MultifoldLogicPass;

impl Pass for MultifoldLogicPass {
	fn get_name(&self) -> &'static str {
		"multifold_logic"
	}
}

impl MIRPass for MultifoldLogicPass {
	fn run_pass(&mut self, data: &mut MIRPassData) -> anyhow::Result<()> {
		for func in data.mir.functions.values_mut() {
			let block = data
				.mir
				.blocks
				.get_mut(&func.block)
				.ok_or(anyhow!("Block does not exist"))?;
			let mut removed = HashSetEmptyTracker::new();
			let mut replaced = Vec::new();
			loop {
				let run_again = run_iter(block, &mut removed, &mut replaced);
				if !run_again {
					break;
				}
			}
			// block.contents = replace_and_expand_indices(block.contents, &replaced);
			remove_indices(&mut block.contents, &removed);
		}

		Ok(())
	}
}

fn run_iter(
	block: &mut MIRBlock,
	removed: &mut HashSetEmptyTracker<usize>,
	replaced: &mut Vec<(usize, Vec<MIRInstruction>)>,
) -> bool {
	let _ = replaced;
	let mut run_again = false;
	let mut assign_if_bool = FxHashMap::<Identifier, AssignIfBool>::default();

	#[derive(Default)]
	struct RegsToKeep {
		assign_if_bool: bool,
	}

	for (i, instr) in block.contents.iter().enumerate() {
		// Even though this instruction hasn't actually been removed from the vec, we treat it
		// as if it has to prevent doing the same work over and over and actually iterating indefinitely
		if removed.contains(&i) {
			continue;
		}

		let mut regs_to_keep = RegsToKeep::default();

		match &instr.kind {
			MIRInstrKind::Assign {
				left: MutableValue::Reg(left),
				right: DeclareBinding::Condition(cond),
			} => {
				assign_if_bool.insert(
					left.clone(),
					AssignIfBool {
						finished: false,
						start_pos: i,
						end_pos: i,
						condition: cond.clone(),
						body: None,
					},
				);

				regs_to_keep.assign_if_bool = true;
			}
			MIRInstrKind::If {
				condition:
					Condition::Bool(Value::Mutable(MutableValue::Reg(b)))
					| Condition::NotBool(Value::Mutable(MutableValue::Reg(b))),
				body,
			} => {
				if let Some(fold) = assign_if_bool.get_mut(b) {
					if !fold.finished {
						fold.body = Some(*body.clone());
						fold.end_pos = i;
						fold.finished = true;
					}
				}
			}
			_ => {}
		}

		let used_regs = instr.kind.get_used_regs();
		for reg in used_regs.into_iter() {
			if !regs_to_keep.assign_if_bool {
				assign_if_bool.retain(
					|fold_reg, fold| {
						if fold.finished {
							true
						} else {
							fold_reg != reg
						}
					},
				);
			}
		}
	}

	// Finish the folds
	for (_, fold) in assign_if_bool {
		if let Some(body) = fold.body {
			run_again = true;
			removed.insert(fold.start_pos);
			block
				.contents
				.get_mut(fold.end_pos)
				.expect("Instr at pos does not exist")
				.kind = MIRInstrKind::If {
				condition: fold.condition,
				body: Box::new(body),
			};
		}
	}

	run_again
}

/// Simplifies:
/// let x = cond ..; if bool x
/// to:
/// if ..
struct AssignIfBool {
	finished: bool,
	start_pos: usize,
	end_pos: usize,
	condition: Condition,
	body: Option<MIRInstrKind>,
}
