use crate::common::ResourceLocation;
use crate::mir::{MIRInstrKind, MIR};
use crate::passes::opt::get_instr_calls;
use crate::passes::{MIRPass, MIRPassData, Pass};

use anyhow::anyhow;
use rustc_hash::FxHashSet;

pub struct InlineCandidatesPass;

impl Pass for InlineCandidatesPass {
	fn get_name(&self) -> &'static str {
		"inline_candidates"
	}
}

impl MIRPass for InlineCandidatesPass {
	fn run_pass(&mut self, data: &mut MIRPassData) -> anyhow::Result<()> {
		let mut call_stack = CallStack {
			set: FxHashSet::default(),
		};
		let mut checked = FxHashSet::default();
		for (func_id, func) in &data.mir.functions {
			checked.clear();
			data.inline_candidates.insert(func_id.clone());
			check_recursion(
				func_id,
				data.mir,
				&mut data.inline_candidates,
				&mut call_stack,
				&mut checked,
			)?;
			if func.block.contents.iter().any(|x| {
				matches!(
					x.kind,
					MIRInstrKind::Return { .. } | MIRInstrKind::ReturnRun { .. }
				)
			}) {
				data.inline_candidates.remove(func_id);
			}
		}

		Ok(())
	}
}

fn check_recursion<'fun>(
	func_id: &'fun ResourceLocation,
	mir: &'fun MIR,
	candidates: &mut FxHashSet<ResourceLocation>,
	call_stack: &mut CallStack,
	checked: &mut FxHashSet<&'fun ResourceLocation>,
) -> anyhow::Result<()> {
	checked.insert(func_id);
	call_stack.set.insert(func_id.clone());

	let func_item = mir
		.functions
		.get(func_id)
		.ok_or(anyhow!("Called function does not exist"))?;
	if func_item.interface.annotations.no_inline {
		candidates.remove(func_id);
	}
	let block = &func_item.block;

	for instr in &block.contents {
		let calls = get_instr_calls(&instr.kind);
		for call in calls {
			// Recursion!
			if call_stack.set.contains(&call.function) {
				candidates.remove(&call.function);
				continue;
			}
			// Don't check functions that we have already determined
			if checked.contains(&call.function) {
				continue;
			}

			check_recursion(&call.function, mir, candidates, call_stack, checked)?;
		}
	}

	call_stack.set.remove(func_id);

	Ok(())
}

struct CallStack {
	set: FxHashSet<ResourceLocation>,
}
