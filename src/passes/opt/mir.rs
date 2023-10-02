use std::collections::HashMap;

use anyhow::anyhow;

use crate::common::ty::{DataTypeContents, ScoreTypeContents};
use crate::common::{DeclareBinding, Identifier, MutableValue, Value};
use crate::mir::{MIRBlock, MIRInstrKind, MIRInstruction, MIR};
use crate::passes::MIRPass;
use crate::util::remove_indices;

pub struct DSEPass;

impl MIRPass for DSEPass {
	fn run_pass(&mut self, mir: &mut MIR) -> anyhow::Result<()> {
		for (_, block) in &mut mir.functions {
			let block = mir
				.blocks
				.get_mut(block)
				.ok_or(anyhow!("Block does not exist"))?;
			loop {
				let run_again = run_dse_iter(block);
				if !run_again {
					break;
				}
			}
		}

		Ok(())
	}
}

/// Runs an iteration of DSE and returns true if another iteration should be performed
fn run_dse_iter(block: &mut MIRBlock) -> bool {
	let mut run_again = false;
	let mut elim_candidates = HashMap::new();
	let mut dead_stores = Vec::new();

	for (i, instr) in block.contents.iter().enumerate() {
		if let MIRInstrKind::Assign {
			left: MutableValue::Register(id),
			..
		} = &instr.kind
		{
			// If the candidate already exists, then that is a dead store that can be removed
			if let Some(candidate) = elim_candidates.get(id) {
				dead_stores.push(*candidate);
			}
			elim_candidates.insert(id.clone(), i);
		}

		// Check if this instruction uses any of the registers that we have marked
		// for elimination
		let used_regs = instr.kind.get_used_regs();
		for reg in used_regs {
			if let Some(candidate) = elim_candidates.get(reg) {
				// Don't remove the candidate we just set
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
		let to_remove = [dead_stores, elim_candidates].concat();
		remove_indices(&mut block.contents, &to_remove);
	}

	run_again
}

pub struct MIRSimplifyPass;

impl MIRPass for MIRSimplifyPass {
	fn run_pass(&mut self, mir: &mut MIR) -> anyhow::Result<()> {
		for (_, block) in &mut mir.functions {
			let block = mir
				.blocks
				.get_mut(block)
				.ok_or(anyhow!("Block does not exist"))?;
			loop {
				let run_again = run_mir_simplify_iter(block);
				if !run_again {
					break;
				}
			}
		}

		Ok(())
	}
}

/// Runs an iteration of the MIRSimplifyPass. Returns true if another iteration
/// should be run
fn run_mir_simplify_iter(block: &mut MIRBlock) -> bool {
	let mut run_again = false;

	let mut instrs_to_remove = Vec::new();
	for (i, instr) in block.contents.iter_mut().enumerate() {
		let remove = match &instr.kind {
			// Reflexive property; swap with self
			MIRInstrKind::Swap { left, right } if left.is_same_val(right) => true,
			// Multiplies and divides by 1
			MIRInstrKind::Mul {
				left: _,
				right: Value::Constant(DataTypeContents::Score(score)),
			}
			| MIRInstrKind::Div {
				left: _,
				right: Value::Constant(DataTypeContents::Score(score)),
			} if score.get_i32() == 1 => true,
			_ => false,
		};

		if remove {
			instrs_to_remove.push(i);
			run_again = true;
		}

		// Instructions to replace
		let kind_repl = match &instr.kind {
			_ => None,
		};

		if let Some(kind_repl) = kind_repl {
			instr.kind = kind_repl;
			run_again = true;
		}
	}

	remove_indices(&mut block.contents, &instrs_to_remove);

	run_again
}

pub struct ConstPropPass;

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

	let mut prop_candidates = HashMap::new();
	for instr in &mut block.contents {
		match &mut instr.kind {
			MIRInstrKind::Assign {
				left: MutableValue::Register(reg),
				right: DeclareBinding::Value(Value::Constant(val)),
			} => {
				prop_candidates.insert(reg.clone(), val.clone());
			}
			MIRInstrKind::Assign { right, .. } => {
				if let DeclareBinding::Value(Value::Mutable(MutableValue::Register(reg))) = &right {
					prop_candidates.remove(reg);
				}
			}
			MIRInstrKind::Declare { .. } => {}
			MIRInstrKind::Add { left, right }
			| MIRInstrKind::Sub { left, right }
			| MIRInstrKind::Mul { left, right }
			| MIRInstrKind::Div { left, right }
			| MIRInstrKind::Mod { left, right }
			| MIRInstrKind::Min { left, right }
			| MIRInstrKind::Max { left, right } => {
				let MutableValue::Register(reg) = left;
				prop_candidates.remove(reg);
				if let Value::Mutable(MutableValue::Register(reg)) = right.clone() {
					if let Some(val) = prop_candidates.get(&reg) {
						*right = Value::Constant(val.clone());
						run_again = true;
					}
				}
			}
			MIRInstrKind::Swap { left, right } => {
				let MutableValue::Register(reg) = left;
				prop_candidates.remove(reg);
				let MutableValue::Register(reg) = right;
				prop_candidates.remove(reg);
			}
			MIRInstrKind::Abs { val } | MIRInstrKind::Use { val } => {
				let MutableValue::Register(reg) = val;
				prop_candidates.remove(reg);
			}
		};
	}

	run_again
}

pub struct InstCombinePass;

impl MIRPass for InstCombinePass {
	fn run_pass(&mut self, mir: &mut MIR) -> anyhow::Result<()> {
		for (_, block) in &mut mir.functions {
			let block = mir
				.blocks
				.get_mut(block)
				.ok_or(anyhow!("Block does not exist"))?;
			loop {
				let run_again = run_instcombine_iter(block);
				if !run_again {
					break;
				}
			}
		}

		Ok(())
	}
}

/// Runs an iteration of instruction combining. Returns true if another iteration
/// should be run
fn run_instcombine_iter(block: &mut MIRBlock) -> bool {
	let mut run_again = false;
	let mut add_subs = HashMap::new();
	let mut mods = HashMap::new();

	for (i, instr) in block.contents.iter().enumerate() {
		match &instr.kind {
			MIRInstrKind::Add {
				left: MutableValue::Register(reg),
				right: Value::Constant(DataTypeContents::Score(score)),
			} if !add_subs.contains_key(reg) => {
				add_subs.insert(
					reg.clone(),
					AddSubCombiner::new(score.get_i32(), reg.clone(), i),
				);
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
				add_subs.insert(
					reg.clone(),
					AddSubCombiner::new(-score.get_i32(), reg.clone(), i),
				);
			}
			MIRInstrKind::Sub {
				left: MutableValue::Register(reg),
				right: Value::Constant(DataTypeContents::Score(score)),
			} if add_subs.contains_key(reg) => {
				if let Some(combiner) = add_subs.get_mut(reg) {
					combiner.feed(i, -score.get_i32());
				}
			}
			MIRInstrKind::Mod {
				left: MutableValue::Register(reg),
				right: Value::Constant(DataTypeContents::Score(score)),
			} if !mods.contains_key(reg) => {
				mods.insert(
					reg.clone(),
					ModCombiner::new(score.get_i32(), reg.clone(), i),
				);
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
					add_subs.remove(reg);
				}
			}
		}
	}

	if !add_subs.is_empty() {
		let mut positions_to_remove = Vec::new();
		for (_, combiner) in add_subs {
			let (pos, to_remove, instr) = combiner.finish();
			*block
				.contents
				.get_mut(pos)
				.expect("Instr at pos does not exist") = instr;
			positions_to_remove.extend(to_remove);
		}

		for (_, combiner) in mods {
			let (pos, to_remove, instr) = combiner.finish();
			*block
				.contents
				.get_mut(pos)
				.expect("Instr at pos does not exist") = instr;
			positions_to_remove.extend(to_remove);
		}

		if !positions_to_remove.is_empty() {
			run_again = true;
			remove_indices(&mut block.contents, &positions_to_remove);
		}
	}

	run_again
}

#[derive(Debug)]
struct AddSubCombiner {
	total: i32,
	reg: Identifier,
	pos: usize,
	to_remove: Vec<usize>,
}

impl AddSubCombiner {
	fn new(start_amt: i32, reg: Identifier, pos: usize) -> Self {
		Self {
			total: start_amt,
			reg,
			pos,
			to_remove: Vec::new(),
		}
	}

	fn feed(&mut self, pos: usize, amt: i32) {
		// We can in fact overflow this because it will wrap around to negative.
		// This ends up having the same behavior when it is added to the register
		self.total = self.total.wrapping_add(amt);
		self.to_remove.push(pos);
	}

	fn finish(self) -> (usize, Vec<usize>, MIRInstruction) {
		(
			self.pos,
			self.to_remove,
			MIRInstruction::new(MIRInstrKind::Add {
				left: MutableValue::Register(self.reg),
				right: Value::Constant(DataTypeContents::Score(ScoreTypeContents::Score(
					self.total,
				))),
			}),
		)
	}
}

#[derive(Debug)]
struct ModCombiner {
	max: i32,
	reg: Identifier,
	pos: usize,
	to_remove: Vec<usize>,
}

impl ModCombiner {
	fn new(start_amt: i32, reg: Identifier, pos: usize) -> Self {
		Self {
			max: start_amt,
			reg,
			pos,
			to_remove: Vec::new(),
		}
	}

	fn feed(&mut self, pos: usize, amt: i32) {
		if amt > self.max {
			self.max = amt;
		}
		self.to_remove.push(pos);
	}

	fn finish(self) -> (usize, Vec<usize>, MIRInstruction) {
		(
			self.pos,
			self.to_remove,
			MIRInstruction::new(MIRInstrKind::Mod {
				left: MutableValue::Register(self.reg),
				right: Value::Constant(DataTypeContents::Score(ScoreTypeContents::Score(self.max))),
			}),
		)
	}
}
