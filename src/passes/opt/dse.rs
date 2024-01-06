use anyhow::anyhow;
use rustc_hash::FxHashMap;

use crate::common::reg::GetUsedRegs;
use crate::common::val::MutableValue;
use crate::mir::{MIRBlock, MIRInstrKind};
use crate::passes::{MIRPass, MIRPassData, Pass};
use crate::util::{remove_indices, HashSetEmptyTracker};

pub struct DSEPass;

impl Pass for DSEPass {
	fn get_name(&self) -> &'static str {
		"dead_store_elimination"
	}
}

impl MIRPass for DSEPass {
	fn run_pass(&mut self, data: &mut MIRPassData) -> anyhow::Result<()> {
		let mut instrs_to_remove = HashSetEmptyTracker::new();
		for func in data.mir.functions.values_mut() {
			let block = data
				.mir
				.blocks
				.get_mut(&func.block)
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
fn run_dse_iter(block: &mut MIRBlock, instrs_to_remove: &mut HashSetEmptyTracker<usize>) -> bool {
	let mut run_again = false;
	let mut elim_candidates = FxHashMap::default();
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
