use std::collections::{HashMap, HashSet};

use anyhow::anyhow;

use crate::common::block::{BlockAllocator, BlockID};
use crate::common::function::FunctionInterface;
use crate::common::{Identifier, ResourceLocation};
use crate::mir::{MIRBlock, MIRInstrKind, MIRInstruction};
use crate::passes::{MIRPass, MIRPassData, Pass};
use crate::util::replace_and_expand_indices;

pub struct SimpleInlinePass;

impl Pass for SimpleInlinePass {
	fn get_name(&self) -> &'static str {
		"simple_inline"
	}
}

impl MIRPass for SimpleInlinePass {
	fn run_pass(&mut self, data: &mut MIRPassData) -> anyhow::Result<()> {
		let mut instrs_to_remove = Vec::new();
		let cloned_funcs = data.mir.functions.clone();
		let cloned_blocks = data.mir.blocks.clone();
		for (func, block) in &mut data.mir.functions {
			let block = data
				.mir
				.blocks
				.get_mut(block)
				.ok_or(anyhow!("Block does not exist"))?;

			loop {
				instrs_to_remove.clear();
				let run_again = run_simple_inline_iter(
					&func.id,
					block,
					&mut instrs_to_remove,
					&data.inline_candidates,
					&cloned_funcs,
					&cloned_blocks,
				)?;

				block.contents = replace_and_expand_indices(block.contents.clone(), &instrs_to_remove);
				if !run_again {
					break;
				}
			}
		}

		Ok(())
	}
}

fn run_simple_inline_iter(
	func_id: &Identifier,
	block: &mut MIRBlock,
	instrs_to_remove: &mut Vec<(usize, Vec<MIRInstruction>)>,
	inline_candidates: &HashSet<ResourceLocation>,
	cloned_funcs: &HashMap<FunctionInterface, BlockID>,
	cloned_blocks: &BlockAllocator<MIRBlock>,
) -> anyhow::Result<bool> {
	let mut run_again = false;

	for (i, instr) in block.contents.iter().enumerate() {
		if instrs_to_remove.iter().any(|x| x.0 == i) {
			continue;
		}
		if let MIRInstrKind::Call { call } = &instr.kind {
			// Don't inline this function call if it is recursive
			if &call.function == func_id {
				continue;
			}
			if !inline_candidates.contains(&call.function) {
				continue;
			}
			let func = cloned_funcs
				.iter()
				.find(|x| x.0.id == call.function)
				.ok_or(anyhow!("Called function does not exist"))?;
			let inlined_block = cloned_blocks
				.get(func.1)
				.ok_or(anyhow!("Inlined block does not exist"))?;
			// Inline the block
			let mut inlined_contents = inlined_block.contents.clone();
			cleanup_fn(&call.function, &mut inlined_contents);
			instrs_to_remove.push((i, inlined_contents));
			run_again = true;
		}
	}

	Ok(run_again)
}

/// Cleanup a function block so that it can be compatible when inlined
fn cleanup_fn(id: &ResourceLocation, block: &mut Vec<MIRInstruction>) {
	let func_id = id.to_string().replace(":", "_").replace("/", "_");
	for instr in block {
		instr.kind.replace_regs(|reg| {
			let new = fmt_inlined_reg(&func_id, reg);
			*reg = new;
		});
	}
}

fn fmt_inlined_reg(func_id: &str, reg: &Identifier) -> Identifier {
	let reg = reg.to_string();
	let reg = format!("in_{func_id}_{reg}");
	reg.into()
}
