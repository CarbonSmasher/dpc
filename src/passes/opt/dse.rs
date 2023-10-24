use std::collections::HashMap;

use anyhow::anyhow;
use dashmap::DashSet;

use crate::common::MutableValue;
use crate::mir::{MIRBlock, MIRInstrKind, MIR};
use crate::passes::{MIRPass, Pass};
use crate::util::remove_indices;

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
