use rustc_hash::FxHashMap;

use crate::common::reg::{GetUsedRegs, Local};
use crate::common::val::{MutableNBTValue, MutableScoreValue, MutableValue};
use crate::lir::{LIRBlock, LIRInstrKind};
use crate::mir::{MIRBlock, MIRInstrKind};
use crate::passes::{LIRPass, LIRPassData, MIRPass, MIRPassData, Pass};
use crate::project::{OptimizationLevel, ProjectSettings};
use crate::util::{remove_indices, HashSetEmptyTracker};

pub struct DSEPass;

impl Pass for DSEPass {
	fn get_name(&self) -> &'static str {
		"dead_store_elimination"
	}

	fn should_run(&self, proj: &ProjectSettings) -> bool {
		proj.op_level >= OptimizationLevel::Basic
	}
}

impl MIRPass for DSEPass {
	fn run_pass(&mut self, data: &mut MIRPassData) -> anyhow::Result<()> {
		let mut instrs_to_remove = HashSetEmptyTracker::new();
		for func in data.mir.functions.values_mut() {
			let block = &mut func.block;

			instrs_to_remove.clear();
			loop {
				let run_again = run_mir_iter(block, &mut instrs_to_remove);
				if !run_again {
					break;
				}
			}
			remove_indices(&mut block.contents, &instrs_to_remove);
		}

		Ok(())
	}
}

fn run_mir_iter(block: &mut MIRBlock, instrs_to_remove: &mut HashSetEmptyTracker<usize>) -> bool {
	let mut run_again = false;
	let mut elim_candidates = FxHashMap::default();
	let mut dead_stores = Vec::new();

	for (i, instr) in block.contents.iter().enumerate() {
		if !instrs_to_remove.contains(&i) {
			if let MIRInstrKind::Assign {
				left: MutableValue::Reg(id),
				right,
			} = &instr.kind
			{
				// We can't remove stores that have side effects
				if !right.has_side_effects() {
					// If the candidate already exists, then that is a dead store that can be removed
					if let Some(candidate) = elim_candidates.get(id) {
						dead_stores.push(*candidate);
					}
					elim_candidates.insert(id.clone(), i);
				}
			}

			if let MIRInstrKind::Remove {
				val: MutableValue::Reg(id),
				..
			} = &instr.kind
			{
				elim_candidates.insert(id.clone(), i);
			}

			if let MIRInstrKind::Call { call } = &instr.kind {
				for val in &call.ret {
					if let MutableValue::Reg(reg) = val {
						// If the candidate already exists, then that is a dead store that can be removed
						if let Some(candidate) = elim_candidates.get(reg) {
							dead_stores.push(*candidate);
						}
					}
				}
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

pub struct LIRDSEPass;

impl Pass for LIRDSEPass {
	fn get_name(&self) -> &'static str {
		"lir_dse"
	}
}

impl LIRPass for LIRDSEPass {
	fn run_pass(&mut self, data: &mut LIRPassData) -> anyhow::Result<()> {
		let mut instrs_to_remove = HashSetEmptyTracker::new();
		for func in data.lir.functions.values_mut() {
			let block = &mut func.block;

			instrs_to_remove.clear();
			loop {
				let run_again = run_lir_iter(block, &mut instrs_to_remove);
				if !run_again {
					break;
				}
			}
			remove_indices(&mut block.contents, &instrs_to_remove);
		}

		Ok(())
	}
}

fn run_lir_iter(block: &mut LIRBlock, instrs_to_remove: &mut HashSetEmptyTracker<usize>) -> bool {
	let mut run_again = false;
	let mut elim_candidates = FxHashMap::default();
	let mut dead_stores = Vec::new();

	for (i, instr) in block.contents.iter().enumerate() {
		if let LIRInstrKind::SetScore(MutableScoreValue::Local(Local::Reg(id)), ..)
		| LIRInstrKind::SetData(MutableNBTValue::Local(Local::Reg(id)), ..) = &instr.kind
		{
			if !instrs_to_remove.contains(&i) {
				if instr.modifiers.is_empty() {
					// If the candidate already exists, then that is a dead store that can be removed
					if let Some(candidate) = elim_candidates.get(id) {
						dead_stores.push(*candidate);
					}
					elim_candidates.insert(id.clone(), i);
				}
			}
		}

		// Check if this instruction uses any of the registers that we have marked
		// for elimination
		let used_regs = instr.get_used_regs();
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
