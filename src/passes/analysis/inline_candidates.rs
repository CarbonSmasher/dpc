use std::collections::HashSet;

use crate::common::ResourceLocation;
use crate::mir::MIR;
use crate::passes::opt::get_instr_call;
use crate::passes::{MIRPass, MIRPassData, Pass};

use anyhow::anyhow;

pub struct InlineCandidatesPass;

impl Pass for InlineCandidatesPass {
	fn get_name(&self) -> &'static str {
		"inline_candidates"
	}
}

impl MIRPass for InlineCandidatesPass {
	fn run_pass(&mut self, data: &mut MIRPassData) -> anyhow::Result<()> {
		let mut call_stack = CallStack {
			set: HashSet::new(),
		};
		let mut checked = HashSet::new();
		for fun in data.mir.functions.keys() {
			checked.clear();
			data.inline_candidates.insert(fun.id.clone());
			check_recursion(
				&fun.id,
				data.mir,
				&mut data.inline_candidates,
				&mut call_stack,
				&mut checked,
			)?;
		}

		Ok(())
	}
}

fn check_recursion<'fun>(
	func: &'fun ResourceLocation,
	mir: &'fun MIR,
	candidates: &mut HashSet<ResourceLocation>,
	call_stack: &mut CallStack,
	checked: &mut HashSet<&'fun ResourceLocation>,
) -> anyhow::Result<()> {
	checked.insert(func);
	call_stack.set.insert(func.clone());

	let func_item = mir
		.get_fn(func)
		.ok_or(anyhow!("Called function does not exist"))?;
	if func_item.0.annotations.no_inline {
		candidates.remove(func);
	}
	let block = mir
		.blocks
		.get(func_item.1)
		.ok_or(anyhow!("Block does not exist"))?;

	for instr in &block.contents {
		let call = get_instr_call(&instr.kind);
		if let Some(call) = call {
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

	call_stack.set.remove(func);

	Ok(())
}

struct CallStack {
	set: HashSet<ResourceLocation>,
}
