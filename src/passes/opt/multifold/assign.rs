use rustc_hash::FxHashMap;

use crate::common::condition::Condition;
use crate::common::op::Operation;
use crate::common::reg::GetUsedRegs;
use crate::common::ty::{DataTypeContents, ScoreTypeContents};
use crate::common::DeclareBinding;
use crate::common::{val::MutableValue, val::Value, Identifier};
use crate::mir::{MIRBlock, MIRInstrKind, MIRInstruction};
use crate::passes::{MIRPass, MIRPassData, Pass};
use crate::project::{OptimizationLevel, ProjectSettings};
use crate::util::{remove_indices, HashSetEmptyTracker, Only};

pub struct MultifoldAssignPass;

impl Pass for MultifoldAssignPass {
	fn get_name(&self) -> &'static str {
		"multifold_assign"
	}

	fn should_run(&self, proj: &ProjectSettings) -> bool {
		proj.op_level >= OptimizationLevel::More
	}
}

impl MIRPass for MultifoldAssignPass {
	fn run_pass(&mut self, data: &mut MIRPassData) -> anyhow::Result<()> {
		data.mir.for_all_blocks_mut(&mut |block| {
			let mut removed = HashSetEmptyTracker::new();
			let mut replaced = Vec::new();
			loop {
				let run_again = run_iter(block, &mut removed, &mut replaced);
				if !run_again {
					break;
				}
			}
			remove_indices(&mut block.contents, &removed);

			Ok(())
		})?;

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
	let mut overwrite_op = FxHashMap::<Identifier, OverwriteOp>::default();
	let mut stack_peak = FxHashMap::<Identifier, StackPeak>::default();

	#[derive(Default)]
	struct RegsToKeep {
		if_cond_assign: bool,
		assign_const_add: bool,
		overwrite_op: bool,
		stack_peak: bool,
	}

	for (i, instr) in block.contents.iter().enumerate() {
		// Even though this instruction hasn't actually been removed from the vec, we treat it
		// as if it has to prevent doing the same work over and over and actually iterating indefinitely
		if removed.contains(&i) {
			continue;
		}

		let mut regs_to_keep = RegsToKeep::default();
		let mut dont_create_new_overwrite_op = false;

		match &instr.kind {
			MIRInstrKind::Assign {
				left: MutableValue::Reg(left),
				right,
			} => {
				if let Some(fold) = overwrite_op.get_mut(left) {
					if !fold.finished {
						fold.right = Some(right.clone());
						fold.end_pos = i;
						fold.finished = true;
						dont_create_new_overwrite_op = true;
					}
				}

				if let DeclareBinding::Value(Value::Mutable(MutableValue::Reg(right))) = right {
					if let Some(fold) = stack_peak.get_mut(right) {
						if &fold.original_reg == left {
							if !fold.finished {
								fold.end_pos = i;
								fold.finished = true;
								regs_to_keep.stack_peak = true;
							}
						} else {
							fold.finished = true;
						}
					} else {
						stack_peak.insert(
							left.clone(),
							StackPeak {
								finished: false,
								start_pos: i,
								end_pos: i,
								original_reg: right.clone(),
								op_poses: Vec::new(),
								ops: Vec::new(),
							},
						);
						regs_to_keep.stack_peak = true;
					}
				}
			}
			_ => {}
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
					regs_to_keep.if_cond_assign = true;
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

				regs_to_keep.assign_const_add = true;
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
						regs_to_keep.assign_const_add = true;
					}
				}
			}
			MIRInstrKind::If { condition, body } => match body.contents.only().map(|x| &x.kind) {
				Some(MIRInstrKind::Assign {
					left: MutableValue::Reg(left),
					right: DeclareBinding::Value(Value::Constant(DataTypeContents::Score(right))),
				}) => {
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

		if let Some(MutableValue::Reg(left)) = instr.kind.get_op_lhs() {
			if !dont_create_new_overwrite_op {
				overwrite_op.insert(
					left.clone(),
					OverwriteOp {
						finished: false,
						start_pos: i,
						end_pos: i,
						right: None,
					},
				);
				regs_to_keep.overwrite_op = true;
			}

			let mut remove_stack_peak = false;
			// Remove stack peaks with the original reg as the lhs
			for fold in stack_peak.values_mut() {
				if &fold.original_reg == left {
					remove_stack_peak = true;
				}
			}
			if let Some(fold) = stack_peak.get_mut(left) {
				if !fold.finished {
					let op = Operation::from_instr(instr.kind.clone());
					if let Some(op) = op {
						// If the rhs is the same reg as the original reg, we have to invalidate
						if let Some(Value::Mutable(MutableValue::Reg(rhs))) = op.get_rhs() {
							if rhs == &fold.original_reg {
								fold.finished = true;
								// We have to totally remove it
								remove_stack_peak = true;
							}
						}
						if !fold.finished {
							fold.op_poses.push(i);
							fold.ops.push(op);
							regs_to_keep.stack_peak = true;
						}
					}
				}
			}
			if remove_stack_peak {
				stack_peak.remove(left);
			}
		}

		let used_regs = instr.kind.get_used_regs();
		for reg in used_regs.into_iter() {
			if !regs_to_keep.if_cond_assign {
				if_cond_assign.get_mut(reg).map(|x| x.finished = true);
			}
			if !regs_to_keep.assign_const_add {
				assign_const_add.get_mut(reg).map(|x| x.finished = true);
				for fold in assign_const_add.values_mut() {
					if let Some(right) = &fold.right {
						if right == reg {
							fold.finished = true;
						}
					}
				}
			}
			if !regs_to_keep.overwrite_op {
				overwrite_op.get_mut(reg).map(|x| x.finished = true);
			}
			if !regs_to_keep.stack_peak {
				stack_peak.retain(|fold_reg, fold| {
					if fold.finished {
						true
					} else {
						fold_reg != reg && &fold.original_reg != reg
					}
				});
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

	for (reg, fold) in overwrite_op {
		if let Some(right) = fold.right {
			run_again = true;
			removed.insert(fold.start_pos);
			block
				.contents
				.get_mut(fold.end_pos)
				.expect("Instr at pos does not exist")
				.kind = MIRInstrKind::Assign {
				left: MutableValue::Reg(reg),
				right,
			};
		}
	}

	for (_, fold) in stack_peak {
		if fold.finished && !fold.ops.is_empty() {
			run_again = true;
			removed.insert(fold.start_pos);
			removed.insert(fold.end_pos);
			for (mut op, pos) in fold.ops.into_iter().zip(fold.op_poses) {
				op.set_lhs(MutableValue::Reg(fold.original_reg.clone()));
				block
					.contents
					.get_mut(pos)
					.expect("Instr at pos does not exist")
					.kind = op.to_instr();
			}
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

/// Simplifies:
/// x o= ..; x = y
/// to:
/// x = y
struct OverwriteOp {
	finished: bool,
	start_pos: usize,
	end_pos: usize,
	right: Option<DeclareBinding>,
}

/// Simplifies:
/// x = y; x o= ..; ...; y = x
/// to:
/// y o= ..; ...
#[derive(Debug)]
struct StackPeak {
	finished: bool,
	start_pos: usize,
	end_pos: usize,
	original_reg: Identifier,
	op_poses: Vec<usize>,
	ops: Vec<Operation>,
}
