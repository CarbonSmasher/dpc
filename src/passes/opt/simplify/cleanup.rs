use crate::common::reg::GetUsedRegs;
use crate::mir::{MIRBlock, MIRInstrKind};
use crate::passes::util::RunAgain;
use crate::passes::{MIRPass, MIRPassData, Pass};
use crate::util::{remove_indices, HashSetEmptyTracker};

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
			cleanup_block(&mut func.block);
		}

		Ok(())
	}
}

fn cleanup_block(block: &mut MIRBlock) -> RunAgain {
	let mut out = RunAgain::new();
	let mut instrs_to_remove = HashSetEmptyTracker::new();
	loop {
		let run_again = run_iter(block, &mut instrs_to_remove);
		out.merge(run_again);
		if !run_again {
			break;
		}
	}
	remove_indices(&mut block.contents, &instrs_to_remove);
	out
}

fn run_iter(block: &mut MIRBlock, instrs_to_remove: &mut HashSetEmptyTracker<usize>) -> RunAgain {
	let mut run_again = RunAgain::new();

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
			run_again.yes();
		}

		for body in instr.kind.get_bodies_mut() {
			run_again.merge(cleanup_block(body));
		}
	}

	run_again
}
