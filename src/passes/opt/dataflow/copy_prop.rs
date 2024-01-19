use rustc_hash::FxHashMap;

use crate::common::reg::GetUsedRegs;
use crate::common::val::{MutableNBTValue, NBTValue, ScoreValue};
use crate::common::{val::MutableScoreValue, Identifier};
use crate::lir::{LIRBlock, LIRInstrKind};
use crate::passes::{LIRPass, LIRPassData, Pass};
use crate::util::{remove_indices, HashSetEmptyTracker};

pub struct CopyPropPass;

impl Pass for CopyPropPass {
	fn get_name(&self) -> &'static str {
		"copy_propagation"
	}
}

impl LIRPass for CopyPropPass {
	fn run_pass(&mut self, data: &mut LIRPassData) -> anyhow::Result<()> {
		let mut reg_mapping = FxHashMap::default();
		let mut instrs_to_remove = HashSetEmptyTracker::new();

		for func in data.lir.functions.values_mut() {
			instrs_to_remove.clear();

			let block = &mut func.block;

			loop {
				reg_mapping.clear();
				let run_again = run_iter(block, &mut instrs_to_remove, &mut reg_mapping);
				if !run_again {
					break;
				}
			}

			remove_indices(&mut block.contents, &instrs_to_remove);
		}

		Ok(())
	}
}

fn run_iter(
	block: &mut LIRBlock,
	instrs_to_remove: &mut HashSetEmptyTracker<usize>,
	reg_mapping: &mut FxHashMap<Identifier, Identifier>,
) -> bool {
	let mut run_again = false;

	for (i, instr) in block.contents.iter_mut().enumerate() {
		if instrs_to_remove.contains(&i) {
			continue;
		}

		let mut dont_remove = Vec::new();

		if let LIRInstrKind::SetScore(
			MutableScoreValue::Reg(l),
			ScoreValue::Mutable(MutableScoreValue::Reg(r)),
		)
		| LIRInstrKind::SetData(
			MutableNBTValue::Reg(l),
			NBTValue::Mutable(MutableNBTValue::Reg(r)),
		) = &mut instr.kind
		{
			// We can copy prop these too
			if let Some(reg) = reg_mapping.get(r) {
				*r = reg.clone();
				run_again = true;
			}

			reg_mapping.insert(l.clone(), r.clone());
			dont_remove.push(l.clone());
			dont_remove.push(r.clone());
		}

		if let Some(right) = instr.kind.get_op_rhs_reg_mut() {
			if let Some(reg) = reg_mapping.get(right) {
				*right = reg.clone();
				run_again = true;
			}
		}

		for reg in instr.get_used_regs() {
			if dont_remove.contains(reg) {
				continue;
			}
			reg_mapping.remove(reg);
			reg_mapping.retain(|_, x| x != reg);
		}
	}

	run_again
}
