use crate::common::reg::GetUsedRegs;
use crate::mir::{MIRBlock, MIRInstrKind};
use crate::passes::{MIRPass, MIRPassData, Pass};
use crate::util::{remove_indices, HashSetEmptyTracker};

use anyhow::anyhow;
use rustc_hash::FxHashSet;

/// Cleans up extra instructions that aren't needed.
/// This pass won't actually change behavior at all, as it only removes pointless
/// instructions that are only effective at the language level, such as declarations
/// that are never used. This makes IR easier to read and can speed up some other
/// passes
pub struct CleanupPass;

impl Pass for CleanupPass {
	fn get_name(&self) -> &'static str {
		"cleanup"
	}
}

impl MIRPass for CleanupPass {
	fn run_pass(&mut self, data: &mut MIRPassData) -> anyhow::Result<()> {
		for func in data.mir.functions.values_mut() {
			let block = data
				.mir
				.blocks
				.get_mut(&func.block)
				.ok_or(anyhow!("Block does not exist"))?;

			let mut instrs_to_remove = HashSetEmptyTracker::new();
			loop {
				let run_again = run_iter(block, &mut instrs_to_remove);
				if !run_again {
					break;
				}
			}
			remove_indices(&mut block.contents, &instrs_to_remove);
		}

		Ok(())
	}
}

fn run_iter(block: &mut MIRBlock, instrs_to_remove: &mut HashSetEmptyTracker<usize>) -> bool {
	let mut run_again = false;

	let mut used_regs = FxHashSet::default();
	for instr in &block.contents {
		used_regs.extend(instr.kind.get_used_regs().into_iter().cloned());
	}

	for (i, instr) in block.contents.iter_mut().enumerate() {
		if instrs_to_remove.contains(&i) {
			continue;
		}
		let remove = match &instr.kind {
			MIRInstrKind::Declare { left, .. } => !used_regs.contains(left),
			MIRInstrKind::NoOp => true,
			_ => false,
		};

		if remove {
			instrs_to_remove.insert(i);
			run_again = true;
		}
	}

	run_again
}
