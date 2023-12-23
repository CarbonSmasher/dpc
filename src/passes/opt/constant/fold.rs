use anyhow::anyhow;
use dashmap::DashMap;
use std::fmt::Debug;

use crate::common::ty::{DataTypeContents, ScoreTypeContents};
use crate::common::val::{MutableValue, Value};
use crate::common::{DeclareBinding, Identifier};
use crate::mir::{MIRBlock, MIRInstrKind};
use crate::passes::{MIRPass, MIRPassData, Pass};
use crate::util::{remove_indices, DashSetEmptyTracker};

use super::{ConstAnalyzer, ConstAnalyzerResult};

pub struct ConstFoldPass {
	pub(super) made_changes: bool,
}

impl ConstFoldPass {
	pub fn new() -> Self {
		Self {
			made_changes: false,
		}
	}
}

impl Pass for ConstFoldPass {
	fn get_name(&self) -> &'static str {
		"const_fold"
	}
}

impl MIRPass for ConstFoldPass {
	fn run_pass(&mut self, data: &mut MIRPassData) -> anyhow::Result<()> {
		let mut fold_points = DashMap::new();
		for (_, block) in &mut data.mir.functions {
			let block = data
				.mir
				.blocks
				.get_mut(block)
				.ok_or(anyhow!("Block does not exist"))?;

			fold_points.clear();

			let mut instrs_to_remove = DashSetEmptyTracker::new();
			loop {
				let run_again = run_const_fold_iter(block, &mut instrs_to_remove, &mut fold_points);
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

/// Runs an iteration of const fold. Returns true if another iteration
/// should be run
fn run_const_fold_iter(
	block: &mut MIRBlock,
	instrs_to_remove: &mut DashSetEmptyTracker<usize>,
	fold_points: &mut DashMap<Identifier, FoldPoint>,
) -> bool {
	let mut run_again = false;

	// Scope here because the analyzer holds references to the data type contents
	{
		let mut an = ConstAnalyzer::new_dont_store();
		for (i, instr) in block.contents.iter().enumerate() {
			if instrs_to_remove.contains(&i) {
				continue;
			}

			match &instr.kind {
				MIRInstrKind::Assign {
					left: MutableValue::Register(left),
					right: DeclareBinding::Value(Value::Constant(DataTypeContents::Score(right))),
				} => {
					if let Some(mut left) = fold_points.get_mut(left) {
						if !left.finished {
							left.value = right.get_i32();
							instrs_to_remove.insert(i);
							left.has_folded = true;
							run_again = true;
						}
					}
				}
				MIRInstrKind::Add {
					left: MutableValue::Register(left),
					right: Value::Constant(DataTypeContents::Score(right)),
				} => {
					if let Some(mut left) = fold_points.get_mut(left) {
						if !left.finished {
							left.value = left.value.overflowing_add(right.get_i32()).0;
							instrs_to_remove.insert(i);
							left.has_folded = true;
							run_again = true;
						}
					}
				}
				MIRInstrKind::Sub {
					left: MutableValue::Register(left),
					right: Value::Constant(DataTypeContents::Score(right)),
				} => {
					if let Some(mut left) = fold_points.get_mut(left) {
						if !left.finished {
							left.value = left.value.overflowing_sub(right.get_i32()).0;
							instrs_to_remove.insert(i);
							left.has_folded = true;
							run_again = true;
						}
					}
				}
				MIRInstrKind::Mul {
					left: MutableValue::Register(left),
					right: Value::Constant(DataTypeContents::Score(right)),
				} => {
					if let Some(mut left) = fold_points.get_mut(left) {
						if !left.finished {
							left.value = left.value.overflowing_mul(right.get_i32()).0;
							instrs_to_remove.insert(i);
							left.has_folded = true;
							run_again = true;
						}
					}
				}
				MIRInstrKind::Div {
					left: MutableValue::Register(left),
					right: Value::Constant(DataTypeContents::Score(right)),
				} => {
					if let Some(mut left) = fold_points.get_mut(left) {
						if !left.finished {
							if right.get_i32() != 0 {
								left.value = left.value / right.get_i32();
								instrs_to_remove.insert(i);
								left.has_folded = true;
								run_again = true;
							} else {
								left.finished = true;
							}
						}
					}
				}
				MIRInstrKind::Mod {
					left: MutableValue::Register(left),
					right: Value::Constant(DataTypeContents::Score(right)),
				} => {
					if let Some(mut left) = fold_points.get_mut(left) {
						if !left.finished {
							if right.get_i32() != 0 {
								left.value = left.value % right.get_i32();
								instrs_to_remove.insert(i);
								left.has_folded = true;
								run_again = true;
							} else {
								left.finished = true;
							}
						}
					}
				}
				MIRInstrKind::Min {
					left: MutableValue::Register(left),
					right: Value::Constant(DataTypeContents::Score(right)),
				} => {
					if let Some(mut left) = fold_points.get_mut(left) {
						if !left.finished {
							left.value = std::cmp::min(left.value, right.get_i32());
							instrs_to_remove.insert(i);
							left.has_folded = true;
							run_again = true;
						}
					}
				}
				MIRInstrKind::Max {
					left: MutableValue::Register(left),
					right: Value::Constant(DataTypeContents::Score(right)),
				} => {
					if let Some(mut left) = fold_points.get_mut(left) {
						if !left.finished {
							left.value = std::cmp::max(left.value, right.get_i32());
							instrs_to_remove.insert(i);
							left.has_folded = true;
							run_again = true;
						}
					}
				}
				MIRInstrKind::Abs {
					val: MutableValue::Register(left),
				} => {
					if let Some(mut left) = fold_points.get_mut(left) {
						if !left.finished {
							left.value = left.value.abs();
							instrs_to_remove.insert(i);
							left.has_folded = true;
							run_again = true;
						}
					}
				}
				MIRInstrKind::Pow {
					base: MutableValue::Register(left),
					exp,
				} => {
					if let Some(mut left) = fold_points.get_mut(left) {
						if !left.finished {
							left.value = left.value.pow((*exp).into());
							instrs_to_remove.insert(i);
							left.has_folded = true;
							run_again = true;
						}
					}
				}
				_ => {}
			};
			let an_result = an.feed(&instr.kind);
			match an_result {
				ConstAnalyzerResult::Add(reg, val) => {
					if let DataTypeContents::Score(val) = val {
						if let Some(mut existing) = fold_points.get_mut(&reg) {
							existing.finished = true;
						} else {
							fold_points.insert(
								reg,
								FoldPoint {
									pos: i,
									value: val.get_i32(),
									finished: false,
									has_folded: false,
								},
							);
						}
					}
				}
				ConstAnalyzerResult::Remove(regs) => {
					for reg in regs {
						if let Some(mut point) = fold_points.get_mut(&reg) {
							// We only have to finish points if they have folded already
							point.finished = true;
							if point.has_folded {}
						}
					}
				}
				_ => (),
			}
		}
	}

	for val in fold_points.iter() {
		let reg = val.key();
		let point = val.value();
		if let Some(instr) = block.contents.get_mut(point.pos) {
			instr.kind = MIRInstrKind::Assign {
				left: MutableValue::Register(reg.clone()),
				right: DeclareBinding::Value(Value::Constant(DataTypeContents::Score(
					ScoreTypeContents::Score(point.value),
				))),
			}
		}
	}

	run_again
}

struct FoldPoint {
	pos: usize,
	value: i32,
	finished: bool,
	has_folded: bool,
}

impl Debug for FoldPoint {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(
			f,
			"{} = {}, Finished: {}, Folded: {}",
			self.pos, self.value, self.finished, self.has_folded
		)
	}
}
