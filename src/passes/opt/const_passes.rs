use anyhow::anyhow;
use dashmap::DashMap;

use crate::common::ty::{DataTypeContents, ScoreTypeContents};
use crate::common::{DeclareBinding, Identifier, MutableValue, Value};
use crate::mir::{MIRBlock, MIRInstrKind, MIR};
use crate::passes::{MIRPass, Pass};
use crate::util::{remove_indices, DashSetEmptyTracker};

pub struct ConstPropPass;

impl Pass for ConstPropPass {
	fn get_name(&self) -> &'static str {
		"const_prop"
	}
}

impl MIRPass for ConstPropPass {
	fn run_pass(&mut self, mir: &mut MIR) -> anyhow::Result<()> {
		for (_, block) in &mut mir.functions {
			let block = mir
				.blocks
				.get_mut(block)
				.ok_or(anyhow!("Block does not exist"))?;
			loop {
				let run_again = run_const_prop_iter(block);
				if !run_again {
					break;
				}
			}
		}

		Ok(())
	}
}

/// Runs an iteration of const prop. Returns true if another iteration
/// should be run
fn run_const_prop_iter(block: &mut MIRBlock) -> bool {
	let mut run_again = false;

	let mut an = ConstAnalyzer::new();
	for instr in &mut block.contents {
		match &mut instr.kind {
			MIRInstrKind::Assign {
				right: DeclareBinding::Value(right),
				..
			}
			| MIRInstrKind::Add { right, .. }
			| MIRInstrKind::Sub { right, .. }
			| MIRInstrKind::Mul { right, .. }
			| MIRInstrKind::Div { right, .. }
			| MIRInstrKind::Mod { right, .. }
			| MIRInstrKind::Min { right, .. }
			| MIRInstrKind::Max { right, .. }
			| MIRInstrKind::Merge { right, .. }
			| MIRInstrKind::Push { right, .. }
			| MIRInstrKind::PushFront { right, .. }
			| MIRInstrKind::Insert { right, .. } => {
				if let Value::Mutable(MutableValue::Register(reg)) = right.clone() {
					if let Some(val) = an.vals.get(&reg) {
						*right = Value::Constant(val.clone());
						run_again = true;
					}
				}
			}
			_ => {}
		};
		an.feed(&instr.kind);
	}

	run_again
}

struct ConstAnalyzer<'cont> {
	vals: DashMap<Identifier, &'cont DataTypeContents>,
	store_self: bool,
}

impl<'cont> ConstAnalyzer<'cont> {
	fn new() -> Self {
		Self {
			vals: DashMap::new(),
			store_self: true,
		}
	}

	fn new_dont_store() -> Self {
		Self {
			vals: DashMap::new(),
			store_self: false,
		}
	}

	fn feed(&mut self, kind: &'cont MIRInstrKind) -> ConstAnalyzerResult {
		match kind {
			MIRInstrKind::Assign {
				left: MutableValue::Register(reg),
				right: DeclareBinding::Value(Value::Constant(val)),
			} => {
				if self.store_self {
					self.vals.insert(reg.clone(), val);
				}
				ConstAnalyzerResult::Add(reg.clone(), val)
			}
			MIRInstrKind::Assign { right, .. } => {
				if let DeclareBinding::Value(Value::Mutable(MutableValue::Register(reg))) = &right {
					if self.store_self {
						self.vals.remove(reg);
					}
					ConstAnalyzerResult::Remove(vec![reg.clone()])
				} else {
					ConstAnalyzerResult::Other
				}
			}
			MIRInstrKind::Add { left, .. }
			| MIRInstrKind::Sub { left, .. }
			| MIRInstrKind::Mul { left, .. }
			| MIRInstrKind::Div { left, .. }
			| MIRInstrKind::Mod { left, .. }
			| MIRInstrKind::Min { left, .. }
			| MIRInstrKind::Max { left, .. }
			| MIRInstrKind::Merge { left, .. }
			| MIRInstrKind::Push { left, .. }
			| MIRInstrKind::PushFront { left, .. }
			| MIRInstrKind::Insert { left, .. } => {
				let MutableValue::Register(reg) = left;
				if self.store_self {
					self.vals.remove(reg);
				}
				ConstAnalyzerResult::Remove(vec![reg.clone()])
			}
			MIRInstrKind::Swap { left, right } => {
				let MutableValue::Register(left_reg) = left;
				let MutableValue::Register(right_reg) = right;
				if self.store_self {
					self.vals.remove(left_reg);
					self.vals.remove(right_reg);
				}
				ConstAnalyzerResult::Remove(vec![left_reg.clone(), right_reg.clone()])
			}
			MIRInstrKind::Abs { val } | MIRInstrKind::Use { val } => {
				let MutableValue::Register(reg) = val;
				if self.store_self {
					self.vals.remove(reg);
				}
				ConstAnalyzerResult::Remove(vec![reg.clone()])
			}
			_ => ConstAnalyzerResult::Other,
		}
	}
}

enum ConstAnalyzerResult<'cont> {
	Other,
	Add(Identifier, &'cont DataTypeContents),
	Remove(Vec<Identifier>),
}

pub struct ConstFoldPass;

impl Pass for ConstFoldPass {
	fn get_name(&self) -> &'static str {
		"const_fold"
	}
}

impl MIRPass for ConstFoldPass {
	fn run_pass(&mut self, mir: &mut MIR) -> anyhow::Result<()> {
		for (_, block) in &mut mir.functions {
			let block = mir
				.blocks
				.get_mut(block)
				.ok_or(anyhow!("Block does not exist"))?;

			let mut instrs_to_remove = DashSetEmptyTracker::new();
			loop {
				let run_again = run_const_fold_iter(block, &mut instrs_to_remove);
				if !run_again {
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
fn run_const_fold_iter(
	block: &mut MIRBlock,
	instrs_to_remove: &mut DashSetEmptyTracker<usize>,
) -> bool {
	let mut run_again = false;

	let fold_points: DashMap<Identifier, FoldPoint> = DashMap::new();
	// Scope here because the analyzer holds references to the data type contents
	{
		let mut an = ConstAnalyzer::new_dont_store();
		for (i, instr) in block.contents.iter().enumerate() {
			if instrs_to_remove.contains(&i) {
				continue;
			}

			match &instr.kind {
				MIRInstrKind::Add {
					left: MutableValue::Register(left),
					right: Value::Constant(DataTypeContents::Score(right)),
				} => {
					if let Some(mut left) = fold_points.get_mut(left) {
						left.value = left.value.overflowing_add(right.get_i32()).0;
						instrs_to_remove.insert(i);
						run_again = true;
					}
				}
				MIRInstrKind::Add {
					left: MutableValue::Register(left),
					right: Value::Mutable(MutableValue::Register(right)),
				} => {
					if let Some(mut left) = fold_points.get_mut(left) {
						if let Some(right) = an.vals.get(right) {
							if let DataTypeContents::Score(right) = right.value() {
								left.value = left.value.overflowing_add(right.get_i32()).0;
								instrs_to_remove.insert(i);
								run_again = true;
							}
						}
					}
				}
				MIRInstrKind::Sub {
					left: MutableValue::Register(left),
					right: Value::Constant(DataTypeContents::Score(right)),
				} => {
					if let Some(mut left) = fold_points.get_mut(left) {
						left.value = left.value.overflowing_sub(right.get_i32()).0;
						instrs_to_remove.insert(i);
						run_again = true;
					}
				}
				MIRInstrKind::Sub {
					left: MutableValue::Register(left),
					right: Value::Mutable(MutableValue::Register(right)),
				} => {
					if let Some(mut left) = fold_points.get_mut(left) {
						if let Some(right) = an.vals.get(right) {
							if let DataTypeContents::Score(right) = right.value() {
								left.value = left.value.overflowing_sub(right.get_i32()).0;
								instrs_to_remove.insert(i);
								run_again = true;
							}
						}
					}
				}
				MIRInstrKind::Mul {
					left: MutableValue::Register(left),
					right: Value::Constant(DataTypeContents::Score(right)),
				} => {
					if let Some(mut left) = fold_points.get_mut(left) {
						left.value = left.value.overflowing_mul(right.get_i32()).0;
						instrs_to_remove.insert(i);
						run_again = true;
					}
				}
				MIRInstrKind::Mul {
					left: MutableValue::Register(left),
					right: Value::Mutable(MutableValue::Register(right)),
				} => {
					if let Some(mut left) = fold_points.get_mut(left) {
						if let Some(right) = an.vals.get(right) {
							if let DataTypeContents::Score(right) = right.value() {
								left.value = left.value.overflowing_mul(right.get_i32()).0;
								instrs_to_remove.insert(i);
								run_again = true;
							}
						}
					}
				}
				MIRInstrKind::Div {
					left: MutableValue::Register(left),
					right: Value::Constant(DataTypeContents::Score(right)),
				} => {
					if let Some(mut left) = fold_points.get_mut(left) {
						if right.get_i32() != 0 {
							left.value = left.value / right.get_i32();
							instrs_to_remove.insert(i);
							run_again = true;
						}
					}
				}
				MIRInstrKind::Div {
					left: MutableValue::Register(left),
					right: Value::Mutable(MutableValue::Register(right)),
				} => {
					if let Some(mut left) = fold_points.get_mut(left) {
						if let Some(right) = an.vals.get(right) {
							if let DataTypeContents::Score(right) = right.value() {
								if right.get_i32() != 0 {
									left.value = left.value / right.get_i32();
									instrs_to_remove.insert(i);
									run_again = true;
								}
							}
						}
					}
				}
				MIRInstrKind::Mod {
					left: MutableValue::Register(left),
					right: Value::Constant(DataTypeContents::Score(right)),
				} => {
					if let Some(mut left) = fold_points.get_mut(left) {
						if right.get_i32() != 0 {
							left.value = left.value % right.get_i32();
							instrs_to_remove.insert(i);
							run_again = true;
						}
					}
				}
				MIRInstrKind::Mod {
					left: MutableValue::Register(left),
					right: Value::Mutable(MutableValue::Register(right)),
				} => {
					if let Some(mut left) = fold_points.get_mut(left) {
						if let Some(right) = an.vals.get(right) {
							if let DataTypeContents::Score(right) = right.value() {
								if right.get_i32() != 0 {
									left.value = left.value % right.get_i32();
									instrs_to_remove.insert(i);
									run_again = true;
								}
							}
						}
					}
				}
				MIRInstrKind::Min {
					left: MutableValue::Register(left),
					right: Value::Constant(DataTypeContents::Score(right)),
				} => {
					if let Some(mut left) = fold_points.get_mut(left) {
						left.value = std::cmp::min(left.value, right.get_i32());
						instrs_to_remove.insert(i);
						run_again = true;
					}
				}
				MIRInstrKind::Min {
					left: MutableValue::Register(left),
					right: Value::Mutable(MutableValue::Register(right)),
				} => {
					if let Some(mut left) = fold_points.get_mut(left) {
						if let Some(right) = an.vals.get(right) {
							if let DataTypeContents::Score(right) = right.value() {
								left.value = std::cmp::min(left.value, right.get_i32());
								instrs_to_remove.insert(i);
								run_again = true;
							}
						}
					}
				}
				MIRInstrKind::Max {
					left: MutableValue::Register(left),
					right: Value::Constant(DataTypeContents::Score(right)),
				} => {
					if let Some(mut left) = fold_points.get_mut(left) {
						left.value = std::cmp::max(left.value, right.get_i32());
						instrs_to_remove.insert(i);
						run_again = true;
					}
				}
				MIRInstrKind::Max {
					left: MutableValue::Register(left),
					right: Value::Mutable(MutableValue::Register(right)),
				} => {
					if let Some(mut left) = fold_points.get_mut(left) {
						if let Some(right) = an.vals.get(right) {
							if let DataTypeContents::Score(right) = right.value() {
								left.value = std::cmp::max(left.value, right.get_i32());
								instrs_to_remove.insert(i);
								run_again = true;
							}
						}
					}
				}
				MIRInstrKind::Abs {
					val: MutableValue::Register(left),
				} => {
					if let Some(mut left) = fold_points.get_mut(left) {
						left.value = left.value.abs();
						instrs_to_remove.insert(i);
						run_again = true;
					}
				}
				MIRInstrKind::Pow {
					base: MutableValue::Register(left),
					exp,
				} => {
					if let Some(mut left) = fold_points.get_mut(left) {
						left.value = left.value.pow((*exp).into());
						instrs_to_remove.insert(i);
						run_again = true;
					}
				}
				_ => {}
			};
			let an_result = an.feed(&instr.kind);
			match an_result {
				ConstAnalyzerResult::Add(reg, val) => {
					if let DataTypeContents::Score(val) = val {
						fold_points.insert(
							reg,
							FoldPoint {
								pos: i,
								value: val.get_i32(),
							},
						);
					}
				}
				ConstAnalyzerResult::Remove(regs) => {
					for reg in regs {
						fold_points.remove(&reg);
					}
				}
				_ => (),
			}
		}
	}

	for (reg, point) in fold_points.into_iter() {
		if let Some(instr) = block.contents.get_mut(point.pos) {
			instr.kind = MIRInstrKind::Assign {
				left: MutableValue::Register(reg),
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
}