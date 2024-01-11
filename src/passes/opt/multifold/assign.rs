use anyhow::anyhow;
use rustc_hash::{FxHashMap, FxHashSet};

use crate::common::condition::Condition;
use crate::common::reg::GetUsedRegs;
use crate::common::ty::{DataTypeContents, ScoreTypeContents};
use crate::common::DeclareBinding;
use crate::common::{val::MutableValue, val::Value, Identifier};
use crate::mir::{MIRBlock, MIRInstrKind, MIRInstruction};
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
	let mut if_cond_assign = FxHashMap::<Identifier, IfCondAssign>::default();
	let mut assign_const_add = FxHashMap::<Identifier, AssignConstAdd>::default();

	for (i, instr) in block.contents.iter().enumerate() {
		// Even though this instruction hasn't actually been removed from the vec, we treat it
		// as if it has to prevent doing the same work over and over and actually iterating indefinitely
		if removed.contains(&i) {
			continue;
		}
		let mut regs_to_keep = FxHashSet::default();
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

				assign_const_add.insert(
					left.clone(),
					AssignConstAdd {
						finished: false,
						start_pos: i,
						end_pos: i,
						const_val: val,
						right: None,
					},
				);

				regs_to_keep.insert(left.clone());
			}
			MIRInstrKind::Add {
				left: MutableValue::Reg(left),
				right: Value::Mutable(MutableValue::Reg(right)),
			} => {
				if let Some(fold) = assign_const_add.get_mut(left) {
					if !fold.finished {
						fold.end_pos = i;
						fold.right = Some(right.clone());
						fold.finished = true;
						regs_to_keep.insert(right.clone());
					}
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
								fold.finished = true;
							}
						}
					}
				}
				_ => {}
			},
			_ => {}
		}

		let used_regs = instr.kind.get_used_regs();
		for reg in used_regs.into_iter().filter(|x| !regs_to_keep.contains(*x)) {
			if_cond_assign.get_mut(reg).map(|x| x.finished = true);
			assign_const_add.get_mut(reg).map(|x| x.finished = true);
			for fold in assign_const_add.values_mut() {
				if let Some(right) = &fold.right {
					if right == reg {
						fold.finished = true;
					}
				}
			}
		}
	}

	// Finish the folds

	for (reg, fold) in if_cond_assign {
		if let Some(condition) = fold.condition {
			run_again = true;
			removed.insert(fold.start_pos);
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

	for (reg, fold) in assign_const_add {
		if let Some(right) = fold.right {
			run_again = true;
			block
				.contents
				.get_mut(fold.start_pos)
				.expect("Instr at pos does not exist")
				.kind = MIRInstrKind::Assign {
				left: MutableValue::Reg(reg.clone()),
				right: DeclareBinding::Value(Value::Mutable(MutableValue::Reg(right))),
			};
			block
				.contents
				.get_mut(fold.end_pos)
				.expect("Instr at pos does not exist")
				.kind = MIRInstrKind::Add {
				left: MutableValue::Reg(reg),
				right: Value::Constant(DataTypeContents::Score(ScoreTypeContents::Score(
					fold.const_val,
				))),
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

/// Simplifies:
/// let x = A; x += y
/// to:
/// let x = y; x += A
struct AssignConstAdd {
	finished: bool,
	start_pos: usize,
	end_pos: usize,
	const_val: i32,
	right: Option<Identifier>,
}

/// Simplifies:
/// let temp = a; a = b; b = temp
/// to:
/// let temp = a; swap a, b;
/// The key for the fold is the temp register
#[allow(dead_code)]
struct ManualSwap {
	finished: bool,
	pos1: usize,
	pos2: usize,
	pos3: usize,
	left: Option<Identifier>,
	right: Option<Identifier>,
}
