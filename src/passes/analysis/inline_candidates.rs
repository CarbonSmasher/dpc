use std::collections::{HashSet, VecDeque};

use crate::common::ResourceLocation;
use crate::mir::{MIRInstrKind, MIR};
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
		for (fun, _) in &data.mir.functions {
			data.inline_candidates.insert(fun.id.clone());
			let mut call_stack = VecDeque::new();
			check_recursion(
				&fun.id,
				&data.mir,
				&mut data.inline_candidates,
				&mut call_stack,
			)?;
		}

		Ok(())
	}
}

fn check_recursion(
	func: &ResourceLocation,
	mir: &MIR,
	candidates: &mut HashSet<ResourceLocation>,
	call_stack: &mut VecDeque<ResourceLocation>,
) -> anyhow::Result<()> {
	call_stack.push_back(func.clone());

	let func_item = mir
		.functions
		.iter()
		.find(|x| &x.0.id == func)
		.ok_or(anyhow!("Called function does not exist"))?;
	let block = mir
		.blocks
		.get(func_item.1)
		.ok_or(anyhow!("Block does not exist"))?;

	for instr in &block.contents {
		if let MIRInstrKind::Call { call } = &instr.kind {
			// Recursion!
			if call_stack.contains(&call.function) {
				candidates.remove(&call.function);
				continue;
			}

			check_recursion(&call.function, mir, candidates, call_stack)?;
		}
	}

	call_stack.pop_back();

	Ok(())
}
