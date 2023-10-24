use std::collections::HashMap;

use anyhow::anyhow;
use dashmap::{DashMap, DashSet};
use tinyvec::TinyVec;

use crate::common::ty::{DataTypeContents, ScoreTypeContents};
use crate::common::{DeclareBinding, Identifier, MutableValue, Value};
use crate::mir::{MIRBlock, MIRInstrKind, MIR};
use crate::passes::{MIRPass, Pass};
use crate::util::{remove_indices, DashSetEmptyTracker};

pub struct DSEPass;

impl Pass for DSEPass {
	fn get_name(&self) -> &'static str {
		"dead_store_elimination"
	}
}

impl MIRPass for DSEPass {
	fn run_pass(&mut self, mir: &mut MIR) -> anyhow::Result<()> {
		let mut instrs_to_remove = DashSet::new();
		for (_, block) in &mut mir.functions {
			let block = mir
				.blocks
				.get_mut(block)
				.ok_or(anyhow!("Block does not exist"))?;

			instrs_to_remove.clear();
			loop {
				let run_again = run_dse_iter(block, &mut instrs_to_remove);
				if !run_again {
					break;
				}
			}
			remove_indices(&mut block.contents, &instrs_to_remove);
		}

		Ok(())
	}
}

/// Runs an iteration of DSE and returns true if another iteration should be performed
fn run_dse_iter(block: &mut MIRBlock, instrs_to_remove: &mut DashSet<usize>) -> bool {
	let mut run_again = false;
	let mut elim_candidates = HashMap::new();
	let mut dead_stores = Vec::new();

	for (i, instr) in block.contents.iter().enumerate() {
		if let MIRInstrKind::Assign {
			left: MutableValue::Register(id),
			..
		} = &instr.kind
		{
			if !instrs_to_remove.contains(&i) {
				// If the candidate already exists, then that is a dead store that can be removed
				if let Some(candidate) = elim_candidates.get(id) {
					dead_stores.push(*candidate);
				}
				elim_candidates.insert(id.clone(), i);
			}
		}

		// Check if this instruction uses any of the registers that we have marked
		// for elimination
		let used_regs = instr.kind.get_used_regs();
		for reg in used_regs {
			if let Some(candidate) = elim_candidates.get(reg) {
				// Don't remove the candidate we just added
				if *candidate == i {
					continue;
				}
				elim_candidates.remove(reg);
			}
		}
	}

	if !dead_stores.is_empty() || !elim_candidates.is_empty() {
		run_again = true;
		// Any remaining elimination candidates are also unused stores
		let elim_candidates: Vec<_> = elim_candidates.values().cloned().collect();
		instrs_to_remove.extend(dead_stores);
		instrs_to_remove.extend(elim_candidates);
	}

	run_again
}

pub struct MIRSimplifyPass;

impl Pass for MIRSimplifyPass {
	fn get_name(&self) -> &'static str {
		"simplify_mir"
	}
}

impl MIRPass for MIRSimplifyPass {
	fn run_pass(&mut self, mir: &mut MIR) -> anyhow::Result<()> {
		for (_, block) in &mut mir.functions {
			let block = mir
				.blocks
				.get_mut(block)
				.ok_or(anyhow!("Block does not exist"))?;

			let mut instrs_to_remove = DashSet::new();
			loop {
				let run_again = run_mir_simplify_iter(block, &mut instrs_to_remove);
				if !run_again {
					break;
				}
			}
			remove_indices(&mut block.contents, &instrs_to_remove);
		}

		Ok(())
	}
}

/// Runs an iteration of the MIRSimplifyPass. Returns true if another iteration
/// should be run
fn run_mir_simplify_iter(block: &mut MIRBlock, instrs_to_remove: &mut DashSet<usize>) -> bool {
	let mut run_again = false;

	for (i, instr) in block.contents.iter_mut().enumerate() {
		let remove = match &instr.kind {
			// Reflexive property; set or swap with self
			// and also min and max with self
			MIRInstrKind::Assign {
				left,
				right: DeclareBinding::Value(Value::Mutable(right)),
			}
			| MIRInstrKind::Swap { left, right }
			| MIRInstrKind::Min {
				left,
				right: Value::Mutable(right),
			}
			| MIRInstrKind::Max {
				left,
				right: Value::Mutable(right),
			} if left.is_same_val(right) => true,
			// Multiplies and divides by 1
			MIRInstrKind::Mul {
				left: _,
				right: Value::Constant(DataTypeContents::Score(score)),
			}
			| MIRInstrKind::Div {
				left: _,
				right: Value::Constant(DataTypeContents::Score(score)),
			} if score.get_i32() == 1 => true,
			// x / 0 and x % 0 error and dont do anything
			MIRInstrKind::Div {
				left: _,
				right: Value::Constant(DataTypeContents::Score(score)),
			}
			| MIRInstrKind::Mod {
				left: _,
				right: Value::Constant(DataTypeContents::Score(score)),
			} if score.get_i32() == 0 => true,
			_ => false,
		};

		if remove {
			let is_new = instrs_to_remove.insert(i);
			if is_new {
				run_again = true;
			}
		}

		// Instructions to replace
		let kind_repl = match &instr.kind {
			// Div by -1 is same as mul by -1
			MIRInstrKind::Div {
				left,
				right: Value::Constant(DataTypeContents::Score(score)),
			} if score.get_i32() == -1 => Some(MIRInstrKind::Mul {
				left: left.clone(),
				right: Value::Constant(DataTypeContents::Score(ScoreTypeContents::Score(-1))),
			}),
			// A couple of canonicalizations that just help out const prop and const fold
			// x / x = 1
			MIRInstrKind::Div {
				left,
				right: Value::Mutable(right),
			} if left.is_same_val(right) => Some(MIRInstrKind::Assign {
				left: left.clone(),
				right: DeclareBinding::Value(Value::Constant(DataTypeContents::Score(
					ScoreTypeContents::Score(1),
				))),
			}),
			// x - x = 0
			MIRInstrKind::Sub {
				left,
				right: Value::Mutable(right),
			} if left.is_same_val(right) => Some(MIRInstrKind::Assign {
				left: left.clone(),
				right: DeclareBinding::Value(Value::Constant(DataTypeContents::Score(
					ScoreTypeContents::Score(0),
				))),
			}),
			// x + x = x * 2
			MIRInstrKind::Add {
				left,
				right: Value::Mutable(right),
			} if left.is_same_val(right) => Some(MIRInstrKind::Mul {
				left: left.clone(),
				right: Value::Constant(DataTypeContents::Score(ScoreTypeContents::Score(2))),
			}),
			// x * x = x ^ 2
			MIRInstrKind::Mul {
				left,
				right: Value::Mutable(right),
			} if left.is_same_val(right) => Some(MIRInstrKind::Pow {
				base: left.clone(),
				exp: 2,
			}),
			// x % x = 0
			MIRInstrKind::Mod {
				left,
				right: Value::Mutable(right),
			} if left.is_same_val(right) => Some(MIRInstrKind::Assign {
				left: left.clone(),
				right: DeclareBinding::Value(Value::Constant(DataTypeContents::Score(
					ScoreTypeContents::Score(0),
				))),
			}),
			_ => None,
		};

		if let Some(kind_repl) = kind_repl {
			instr.kind = kind_repl;
			run_again = true;
		}
	}

	run_again
}

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
			| MIRInstrKind::Max { right, .. } => {
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
			| MIRInstrKind::Max { left, .. } => {
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

pub struct InstCombinePass;

impl Pass for InstCombinePass {
	fn get_name(&self) -> &'static str {
		"instruction_combine"
	}
}

impl MIRPass for InstCombinePass {
	fn run_pass(&mut self, mir: &mut MIR) -> anyhow::Result<()> {
		for (_, block) in &mut mir.functions {
			let block = mir
				.blocks
				.get_mut(block)
				.ok_or(anyhow!("Block does not exist"))?;
			let mut removed_indices = DashSetEmptyTracker::new();
			loop {
				let run_again = run_instcombine_iter(block, &mut removed_indices);
				if !run_again {
					break;
				}
			}
			remove_indices(&mut block.contents, &removed_indices);
		}

		Ok(())
	}
}

/// Runs an iteration of instruction combining. Returns true if another iteration
/// should be run
fn run_instcombine_iter(
	block: &mut MIRBlock,
	removed_indices: &mut DashSetEmptyTracker<usize>,
) -> bool {
	let mut run_again = false;
	let add_subs = DashMap::new();
	let muls = DashMap::new();
	let mods = DashMap::new();

	for (i, instr) in block.contents.iter().enumerate() {
		// Even though this instruction hasn't actually been removed from the vec, we treat it
		// as if it has to prevent doing the same work over and over and actually iterating indefinitely
		if removed_indices.contains(&i) {
			continue;
		}
		match &instr.kind {
			MIRInstrKind::Add {
				left: MutableValue::Register(reg),
				right: Value::Constant(DataTypeContents::Score(score)),
			} if !add_subs.contains_key(reg) => {
				add_subs.insert(reg.clone(), AddSubCombiner::new(score.get_i32(), i));
			}
			MIRInstrKind::Add {
				left: MutableValue::Register(reg),
				right: Value::Constant(DataTypeContents::Score(score)),
			} if add_subs.contains_key(reg) => {
				if let Some(mut combiner) = add_subs.get_mut(reg) {
					combiner.feed(i, score.get_i32());
				}
			}
			MIRInstrKind::Sub {
				left: MutableValue::Register(reg),
				right: Value::Constant(DataTypeContents::Score(score)),
			} if !add_subs.contains_key(reg) => {
				add_subs.insert(reg.clone(), AddSubCombiner::new(-score.get_i32(), i));
			}
			MIRInstrKind::Sub {
				left: MutableValue::Register(reg),
				right: Value::Constant(DataTypeContents::Score(score)),
			} if add_subs.contains_key(reg) => {
				if let Some(mut combiner) = add_subs.get_mut(reg) {
					combiner.feed(i, -score.get_i32());
				}
			}
			MIRInstrKind::Mul {
				left: MutableValue::Register(reg),
				right: Value::Constant(DataTypeContents::Score(score)),
			} if !muls.contains_key(reg) => {
				muls.insert(reg.clone(), MulCombiner::new(score.get_i32(), i));
			}
			MIRInstrKind::Mul {
				left: MutableValue::Register(reg),
				right: Value::Constant(DataTypeContents::Score(score)),
			} if muls.contains_key(reg) => {
				if let Some(mut combiner) = muls.get_mut(reg) {
					combiner.feed(i, score.get_i32());
				}
			}
			MIRInstrKind::Mod {
				left: MutableValue::Register(reg),
				right: Value::Constant(DataTypeContents::Score(score)),
			} if !mods.contains_key(reg) => {
				mods.insert(reg.clone(), ModCombiner::new(score.get_i32(), i));
			}
			MIRInstrKind::Mod {
				left: MutableValue::Register(reg),
				right: Value::Constant(DataTypeContents::Score(score)),
			} if mods.contains_key(reg) => {
				if let Some(mut combiner) = mods.get_mut(reg) {
					combiner.feed(i, score.get_i32());
				}
			}
			other => {
				let used_regs = other.get_used_regs();
				for reg in used_regs {
					// Mark any combiners that are combining this register as finished
					add_subs.get_mut(reg).map(|mut x| x.finished = true);
					muls.get_mut(reg).map(|mut x| x.finished = true);
					mods.get_mut(reg).map(|mut x| x.finished = true);
				}
			}
		}
	}

	if !add_subs.is_empty() || !muls.is_empty() || !mods.is_empty() {
		let mut positions_to_remove = Vec::new();
		for (reg, combiner) in add_subs {
			if let Some((pos, to_remove, instr)) = combiner.finish(reg) {
				block
					.contents
					.get_mut(pos)
					.expect("Instr at pos does not exist")
					.kind = instr;
				positions_to_remove.extend(to_remove);
			}
		}

		for (reg, combiner) in muls {
			if let Some((pos, to_remove, instr)) = combiner.finish(reg) {
				block
					.contents
					.get_mut(pos)
					.expect("Instr at pos does not exist")
					.kind = instr;
				positions_to_remove.extend(to_remove);
			}
		}

		for (reg, combiner) in mods {
			if let Some((pos, to_remove, instr)) = combiner.finish(reg) {
				block
					.contents
					.get_mut(pos)
					.expect("Instr at pos does not exist")
					.kind = instr;
				positions_to_remove.extend(to_remove);
			}
		}

		if !positions_to_remove.is_empty() {
			run_again = true;
		}

		removed_indices.extend(positions_to_remove);
	}

	run_again
}

#[derive(Debug)]
struct AddSubCombiner {
	total: i32,
	pos: usize,
	to_remove: TinyVec<[usize; 5]>,
	finished: bool,
}

impl AddSubCombiner {
	fn new(start_amt: i32, pos: usize) -> Self {
		Self {
			total: start_amt,
			pos,
			to_remove: TinyVec::new(),
			finished: false,
		}
	}

	fn feed(&mut self, pos: usize, amt: i32) {
		if self.finished {
			return;
		}
		// We can in fact overflow this because it will wrap around to negative.
		// This ends up having the same behavior when it is added to the register
		self.total = self.total.wrapping_add(amt);
		self.to_remove.push(pos);
	}

	fn finish(self, reg: Identifier) -> Option<(usize, TinyVec<[usize; 5]>, MIRInstrKind)> {
		if self.to_remove.is_empty() {
			None
		} else {
			Some((
				self.pos,
				self.to_remove,
				MIRInstrKind::Add {
					left: MutableValue::Register(reg),
					right: Value::Constant(DataTypeContents::Score(ScoreTypeContents::Score(
						self.total,
					))),
				},
			))
		}
	}
}

#[derive(Debug)]
struct MulCombiner {
	total: i32,
	pos: usize,
	to_remove: TinyVec<[usize; 5]>,
	finished: bool,
}

impl MulCombiner {
	fn new(start_amt: i32, pos: usize) -> Self {
		Self {
			total: start_amt,
			pos,
			to_remove: TinyVec::new(),
			finished: false,
		}
	}

	fn feed(&mut self, pos: usize, amt: i32) {
		if self.finished {
			return;
		}
		if let Some(total) = self.total.checked_mul(amt) {
			self.total = total;
			self.to_remove.push(pos);
		}
	}

	fn finish(self, reg: Identifier) -> Option<(usize, TinyVec<[usize; 5]>, MIRInstrKind)> {
		if self.to_remove.is_empty() {
			None
		} else {
			Some((
				self.pos,
				self.to_remove,
				MIRInstrKind::Mul {
					left: MutableValue::Register(reg),
					right: Value::Constant(DataTypeContents::Score(ScoreTypeContents::Score(
						self.total,
					))),
				},
			))
		}
	}
}

#[derive(Debug)]
struct ModCombiner {
	max: i32,
	pos: usize,
	to_remove: TinyVec<[usize; 5]>,
	finished: bool,
}

impl ModCombiner {
	fn new(start_amt: i32, pos: usize) -> Self {
		Self {
			max: start_amt,
			pos,
			to_remove: TinyVec::new(),
			finished: false,
		}
	}

	fn feed(&mut self, pos: usize, amt: i32) {
		if self.finished {
			return;
		}
		if amt > self.max {
			self.max = amt;
		}
		self.to_remove.push(pos);
	}

	fn finish(self, reg: Identifier) -> Option<(usize, TinyVec<[usize; 5]>, MIRInstrKind)> {
		if self.to_remove.is_empty() {
			None
		} else {
			Some((
				self.pos,
				self.to_remove,
				MIRInstrKind::Mod {
					left: MutableValue::Register(reg),
					right: Value::Constant(DataTypeContents::Score(ScoreTypeContents::Score(
						self.max,
					))),
				},
			))
		}
	}
}
