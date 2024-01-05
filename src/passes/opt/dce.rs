use anyhow::anyhow;
use rustc_hash::FxHashSet;

use crate::passes::{MIRPass, MIRPassData, Pass};

use super::get_instr_call;

pub struct DCEPass;

impl Pass for DCEPass {
	fn get_name(&self) -> &'static str {
		"dead_code_elimination"
	}
}

impl MIRPass for DCEPass {
	fn run_pass(&mut self, data: &mut MIRPassData) -> anyhow::Result<()> {
		// Find used functions
		let mut used = FxHashSet::default();
		for func in data.mir.functions.values() {
			let block = data
				.mir
				.blocks
				.get(&func.block)
				.ok_or(anyhow!("Block does not exist"))?;

			for instr in &block.contents {
				let call = get_instr_call(&instr.kind);
				if let Some(call) = call {
					used.insert(call.function.clone());
				}
			}
		}

		// Remove unused functions
		let mut unused = FxHashSet::default();
		for (func_id, func) in &data.mir.functions {
			if func.interface.annotations.preserve {
				continue;
			}
			if !used.contains(func_id) {
				unused.insert(func_id.clone());
				data.mir.blocks.remove(&func.block);
			}
		}
		for unused in unused {
			data.mir.functions.remove(&unused);
		}

		Ok(())
	}
}
