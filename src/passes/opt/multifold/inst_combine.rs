use anyhow::anyhow;
use rustc_hash::FxHashMap;
use tinyvec::TinyVec;

use crate::common::ty::{DataTypeContents, ScoreTypeContents};
use crate::common::{val::MutableValue, val::Value, Identifier};
use crate::mir::{MIRBlock, MIRInstrKind};
use crate::passes::{MIRPass, MIRPassData, Pass};
use crate::util::{remove_indices, HashSetEmptyTracker};

pub struct InstCombinePass;

impl Pass for InstCombinePass {
	fn get_name(&self) -> &'static str {
		"instruction_combine"
	}
}

impl MIRPass for InstCombinePass {
	fn run_pass(&mut self, data: &mut MIRPassData) -> anyhow::Result<()> {
		for func in data.mir.functions.values_mut() {
			let block = data
				.mir
				.blocks
				.get_mut(&func.block)
				.ok_or(anyhow!("Block does not exist"))?;
			let mut removed_indices = HashSetEmptyTracker::new();
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
	removed_indices: &mut HashSetEmptyTracker<usize>,
) -> bool {
	let mut run_again = false;
	let mut add_subs = FxHashMap::default();
	let mut muls = FxHashMap::default();
	let mut mods = FxHashMap::default();

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
				if let Some(combiner) = add_subs.get_mut(reg) {
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
				if let Some(combiner) = add_subs.get_mut(reg) {
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
				if let Some(combiner) = muls.get_mut(reg) {
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
				if let Some(combiner) = mods.get_mut(reg) {
					combiner.feed(i, score.get_i32());
				}
			}
			other => {
				let used_regs = other.get_used_regs();
				for reg in used_regs {
					// Mark any combiners that are combining this register as finished
					add_subs.get_mut(reg).map(|x| x.finished = true);
					muls.get_mut(reg).map(|x| x.finished = true);
					mods.get_mut(reg).map(|x| x.finished = true);
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
