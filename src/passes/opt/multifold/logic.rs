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
			let block = &mut func.block;

			let mut removed = HashSetEmptyTracker::new();
			let mut replaced = Vec::new();
			loop {
				let run_again = run_iter(block, &mut removed, &mut replaced);
				if !run_again {
					break;
				}
			}
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
	let mut let_cond_prop = FxHashMap::<Identifier, LetCondProp>::default();
	let mut let_cond_not = FxHashMap::<Identifier, LetCondNot>::default();
	let mut manual_or = FxHashMap::<Identifier, ManualOr>::default();

	#[derive(Default)]
	struct RegsToKeep {
		let_cond_prop: bool,
		let_cond_not: bool,
		manual_or: bool,
	}

	for (i, instr) in block.contents.iter_mut().enumerate() {
		// Even though this instruction hasn't actually been removed from the vec, we treat it
		// as if it has to prevent doing the same work over and over and actually iterating indefinitely
		if removed.contains(&i) {
			continue;
		}

		let mut regs_to_keep = RegsToKeep::default();

		match &mut instr.kind {
			MIRInstrKind::Assign {
				left: MutableValue::Reg(left),
				right: DeclareBinding::Condition(cond),
			} => {
				let_cond_propagate(cond, &mut let_cond_prop, &mut run_again);
				for reg in cond.get_used_regs() {
					let_cond_prop.remove(reg);
				}
				let_cond_prop.insert(
					left.clone(),
					LetCondProp {
						finished: false,
						condition: cond.clone(),
					},
				);

				let_cond_not.insert(
					left.clone(),
					LetCondNot {
						finished: false,
						start_pos: i,
						end_pos: i,
						condition: cond.clone(),
					},
				);

				regs_to_keep.let_cond_prop = true;
				regs_to_keep.let_cond_not = true;
			}
			MIRInstrKind::If { condition, .. } | MIRInstrKind::IfElse { condition, .. } => {
				let_cond_propagate(condition, &mut let_cond_prop, &mut run_again);
			}
			MIRInstrKind::Not {
				value: MutableValue::Reg(reg),
			} => {
				if let Some(fold) = let_cond_not.get_mut(reg) {
					if !fold.finished {
						fold.end_pos = i;
						fold.finished = true;
					}
				}
			}
			MIRInstrKind::Add {
				left: MutableValue::Reg(left),
				right: Value::Mutable(MutableValue::Reg(right)),
			} => {
				manual_or.retain(|_, fold| {
					if fold.finished {
						true
					} else {
						&fold.right != left && &fold.right != right
					}
				});

				manual_or.insert(
					left.clone(),
					ManualOr {
						finished: false,
						start_pos: i,
						end_pos: i,
						right: right.clone(),
					},
				);

				regs_to_keep.manual_or = true;
			}
			MIRInstrKind::Div {
				left: MutableValue::Reg(left),
				right: Value::Mutable(MutableValue::Reg(right)),
			} if left == right => {
				manual_or.retain(|_, fold| {
					if fold.finished {
						true
					} else {
						&fold.right != left
					}
				});
				if let Some(fold) = manual_or.get_mut(left) {
					if !fold.finished {
						fold.end_pos = i;
						fold.finished = true;
					}
				}

				regs_to_keep.manual_or = true;
			}
			_ => {}
		}

		let used_regs = instr.kind.get_used_regs();
		for reg in used_regs.into_iter() {
			if !regs_to_keep.let_cond_prop {
				let_cond_prop.remove(reg);
			}
			if !regs_to_keep.let_cond_not {
				let_cond_not.retain(
					|fold_reg, fold| {
						if fold.finished {
							true
						} else {
							fold_reg != reg
						}
					},
				);
			}
			if !regs_to_keep.manual_or {
				manual_or.retain(|fold_reg, fold| {
					if fold.finished {
						true
					} else {
						fold_reg != reg && &fold.right != reg
					}
				});
			}
		}
	}

	// Finish the folds
	for (reg, fold) in let_cond_not {
		if fold.finished {
			run_again = true;
			removed.insert(fold.end_pos);
			block
				.contents
				.get_mut(fold.start_pos)
				.expect("Instr at pos does not exist")
				.kind = MIRInstrKind::Assign {
				left: MutableValue::Reg(reg),
				right: DeclareBinding::Condition(Condition::Not(Box::new(fold.condition))),
			}
		}
	}

	for (reg, fold) in manual_or {
		if fold.finished {
			run_again = true;
			removed.insert(fold.start_pos);
			block
				.contents
				.get_mut(fold.end_pos)
				.expect("Instr at pos does not exist")
				.kind = MIRInstrKind::Or {
				left: MutableValue::Reg(reg),
				right: Value::Mutable(MutableValue::Reg(fold.right)),
			};
		}
	}

	run_again
}

/// Propagates let conditions into other let conditions and ifs
struct LetCondProp {
	finished: bool,
	condition: Condition,
}

// Recursively check for boolean conditions and replace them
// We do this in place because we don't want to store where in the subconditions
// the condition to replace is
fn let_cond_propagate(
	condition: &mut Condition,
	let_cond_prop: &mut FxHashMap<Identifier, LetCondProp>,
	run_again: &mut bool,
) {
	match condition {
		Condition::Bool(Value::Mutable(MutableValue::Reg(b))) => {
			if let Some(fold) = let_cond_prop.get_mut(b) {
				if !fold.finished {
					*condition = fold.condition.clone();
					fold.finished = true;
					*run_again = true;
				}
			}
		}
		Condition::NotBool(Value::Mutable(MutableValue::Reg(b))) => {
			if let Some(fold) = let_cond_prop.get_mut(b) {
				if !fold.finished {
					*condition = Condition::Not(Box::new(fold.condition.clone()));
					fold.finished = true;
					*run_again = true;
				}
			}
		}
		Condition::Not(cond) => let_cond_propagate(cond, let_cond_prop, run_again),
		Condition::And(l, r) => {
			let_cond_propagate(l, let_cond_prop, run_again);
			let_cond_propagate(r, let_cond_prop, run_again);
		}
		_ => {}
	}
}

/// Simplifies:
/// let x = cond ..; not x
/// to:
/// let x = cond not ..
struct LetCondNot {
	finished: bool,
	start_pos: usize,
	end_pos: usize,
	condition: Condition,
}

/// Simplifies:
/// x += y; x /= x
/// to:
/// x |= y
struct ManualOr {
	finished: bool,
	start_pos: usize,
	end_pos: usize,
	right: Identifier,
}
