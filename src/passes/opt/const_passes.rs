use std::fmt::Debug;

use anyhow::{anyhow, Context};
use dashmap::DashMap;

use crate::common::condition::Condition;
use crate::common::ty::{DataTypeContents, ScoreTypeContents};
use crate::common::{val::MutableValue, val::Value, DeclareBinding, Identifier};
use crate::mir::{MIRBlock, MIRInstrKind};
use crate::passes::{MIRPass, MIRPassData, Pass};
use crate::util::{remove_indices, DashSetEmptyTracker};

/// Combines the ConstProp and ConstFold passes and runs them both
/// until no changes are made
pub struct ConstComboPass;

impl Pass for ConstComboPass {
	fn get_name(&self) -> &'static str {
		"const_combo"
	}
}

impl MIRPass for ConstComboPass {
	fn run_pass(&mut self, data: &mut MIRPassData) -> anyhow::Result<()> {
		loop {
			let mut prop = ConstPropPass::new();
			prop.run_pass(data).context("ConstProp failed")?;
			let mut fold = ConstFoldPass::new();
			fold.run_pass(data).context("ConstFold failed")?;
			let mut cond = ConstConditionPass::new();
			cond.run_pass(data).context("ConstCondition failed")?;
			if !prop.made_changes() && !fold.made_changes && !cond.made_changes {
				break;
			}
		}

		Ok(())
	}
}

pub struct ConstPropPass {
	made_changes: bool,
}

impl ConstPropPass {
	pub fn new() -> Self {
		Self {
			made_changes: false,
		}
	}
}

impl Pass for ConstPropPass {
	fn get_name(&self) -> &'static str {
		"const_prop"
	}

	fn made_changes(&self) -> bool {
		self.made_changes
	}
}

impl MIRPass for ConstPropPass {
	fn run_pass(&mut self, data: &mut MIRPassData) -> anyhow::Result<()> {
		for (_, block) in &mut data.mir.functions {
			let block = data
				.mir
				.blocks
				.get_mut(block)
				.ok_or(anyhow!("Block does not exist"))?;
			loop {
				let run_again = run_const_prop_iter(block);
				if run_again {
					self.made_changes = true;
				} else {
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
		const_prop_instr(&mut instr.kind, &mut an, &mut run_again);
		an.feed(&instr.kind);
	}

	run_again
}

fn const_prop_instr(instr: &mut MIRInstrKind, an: &mut ConstAnalyzer, run_again: &mut bool) {
	match instr {
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
					*run_again = true;
				}
			}
		}
		MIRInstrKind::If { condition, body } => match condition {
			Condition::Equal(l, r)
			| Condition::GreaterThan(l, r)
			| Condition::GreaterThanOrEqual(l, r)
			| Condition::LessThan(l, r)
			| Condition::LessThanOrEqual(l, r) => {
				if let Value::Mutable(MutableValue::Register(reg)) = l.clone() {
					if let Some(val) = an.vals.get(&reg) {
						*l = Value::Constant(val.clone());
						*run_again = true;
					}
				}
				if let Value::Mutable(MutableValue::Register(reg)) = r.clone() {
					if let Some(val) = an.vals.get(&reg) {
						*r = Value::Constant(val.clone());
						*run_again = true;
					}
				}
				const_prop_instr(body, an, run_again);
			}
			_ => {}
		},
		_ => {}
	};
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
			vals: DashMap::with_capacity(0),
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
			MIRInstrKind::Assign { left, right, .. } => {
				let mut out = Vec::new();
				if let MutableValue::Register(reg) = left {
					out.push(reg.clone());
				}
				if let DeclareBinding::Value(Value::Mutable(MutableValue::Register(reg))) = &right {
					if self.store_self {
						self.vals.remove(reg);
						out.push(reg.clone());
					}
				} else {
					let used = right.get_used_regs();
					out.extend(used.into_iter().cloned())
				}
				ConstAnalyzerResult::Remove(out)
			}
			MIRInstrKind::Add { left, .. }
			| MIRInstrKind::Sub { left, .. }
			| MIRInstrKind::Mul { left, .. }
			| MIRInstrKind::Div { left, .. }
			| MIRInstrKind::Mod { left, .. }
			| MIRInstrKind::Min { left, .. }
			| MIRInstrKind::Max { left, .. }
			| MIRInstrKind::Pow { base: left, .. }
			| MIRInstrKind::Merge { left, .. }
			| MIRInstrKind::Push { left, .. }
			| MIRInstrKind::PushFront { left, .. }
			| MIRInstrKind::Insert { left, .. } => {
				if let MutableValue::Register(reg) = left {
					if self.store_self {
						self.vals.remove(reg);
					}
					ConstAnalyzerResult::Remove(vec![reg.clone()])
				} else {
					ConstAnalyzerResult::Other
				}
			}
			MIRInstrKind::Swap {
				left: MutableValue::Register(left_reg),
				right: MutableValue::Register(right_reg),
			} => {
				if self.store_self {
					self.vals.remove(left_reg);
					self.vals.remove(right_reg);
				}
				ConstAnalyzerResult::Remove(vec![left_reg.clone(), right_reg.clone()])
			}
			MIRInstrKind::Abs {
				val: MutableValue::Register(reg),
			}
			| MIRInstrKind::Use {
				val: MutableValue::Register(reg),
			}
			| MIRInstrKind::Remove {
				val: MutableValue::Register(reg),
			} => {
				if self.store_self {
					self.vals.remove(reg);
				}
				ConstAnalyzerResult::Remove(vec![reg.clone()])
			}
			other => {
				let used = other.get_used_regs();
				if self.store_self {
					for used in &used {
						self.vals.remove(*used);
					}
				}
				ConstAnalyzerResult::Remove(used.into_iter().cloned().collect())
			}
		}
	}
}

enum ConstAnalyzerResult<'cont> {
	Other,
	Add(Identifier, &'cont DataTypeContents),
	Remove(Vec<Identifier>),
}

pub struct ConstFoldPass {
	made_changes: bool,
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

pub struct ConstConditionPass {
	made_changes: bool,
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
