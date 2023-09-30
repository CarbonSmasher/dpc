use std::collections::HashMap;

use crate::common::MutableValue;
use crate::mir::{MIRBlock, MIRInstrKind};
use crate::passes::MIRPass;
use crate::util::remove_indices;

pub struct DSEPass;

impl MIRPass for DSEPass {
	fn run_pass(&mut self, mir: &mut crate::mir::MIR) -> anyhow::Result<()> {
		for (_, block) in &mut mir.functions {
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

	if !dead_stores.is_empty() {
		run_again = true;
		// Any remaining elimination candidates are also unused stores
		let elim_candidates: Vec<_> = elim_candidates.values().cloned().collect();
		let to_remove = [dead_stores, elim_candidates].concat();
		remove_indices(&mut block.contents, &to_remove);
	}

	run_again
}
