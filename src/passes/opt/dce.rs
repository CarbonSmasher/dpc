use anyhow::anyhow;
use dashmap::DashSet;

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
		let used = DashSet::new();
		for block in data.mir.functions.values() {
			let block = data
				.mir
				.blocks
				.get(block)
				.ok_or(anyhow!("Block does not exist"))?;

			for instr in &block.contents {
				let call = get_instr_call(&instr.kind);
				if let Some(call) = call {
					used.insert(call.function.clone());
				}
			}
		}

		// Remove unused functions
		let unused = DashSet::new();
		for (func, block) in &data.mir.functions {
			if func.annotations.preserve {
				continue;
			}
			if !used.contains(&func.id) {
				unused.insert(func.clone());
				data.mir.blocks.remove(block);
			}
		}
		for unused in unused {
			data.mir.functions.remove(&unused);
		}

		Ok(())
	}
}
