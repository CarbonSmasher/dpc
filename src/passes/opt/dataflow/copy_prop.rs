use rustc_hash::FxHashMap;

use crate::common::reg::{GetUsedLocals, Local};
use crate::common::val::MutableScoreValue;
use crate::common::val::{MutableNBTValue, NBTValue, ScoreValue};
use crate::lir::{LIRBlock, LIRInstrKind};
use crate::passes::{LIRPass, LIRPassData, Pass};
use crate::project::{OptimizationLevel, ProjectSettings};

pub struct CopyPropPass;

impl Pass for CopyPropPass {
	fn get_name(&self) -> &'static str {
		"copy_propagation"
	}

	fn should_run(&self, proj: &ProjectSettings) -> bool {
		proj.op_level >= OptimizationLevel::Basic
	}
}

impl LIRPass for CopyPropPass {
	fn run_pass(&mut self, data: &mut LIRPassData) -> anyhow::Result<()> {
		let mut loc_mapping = FxHashMap::default();

		for func in data.lir.functions.values_mut() {
			let block = &mut func.block;

			loop {
				loc_mapping.clear();
				let run_again = run_iter(block, &mut loc_mapping);
				if !run_again {
					break;
				}
			}
		}

		Ok(())
	}
}

fn run_iter(block: &mut LIRBlock, loc_mapping: &mut FxHashMap<Local, Local>) -> bool {
	let mut run_again = false;

	for instr in &mut block.contents {
		let mut dont_remove = Vec::new();

		if let LIRInstrKind::SetScore(
			MutableScoreValue::Local(l),
			ScoreValue::Mutable(MutableScoreValue::Local(r)),
		)
		| LIRInstrKind::SetData(
			MutableNBTValue::Local(l),
			NBTValue::Mutable(MutableNBTValue::Local(r)),
		) = &mut instr.kind
		{
			// We can copy prop these too
			if let Some(loc) = loc_mapping.get(r) {
				*r = loc.clone();
				run_again = true;
			}

			// We can't prop into call args or return values
			if !matches!(l, Local::CallArg(..) | Local::ReturnValue(..)) {
				loc_mapping.insert(l.clone(), r.clone());
				dont_remove.push(l.clone());
				dont_remove.push(r.clone());
			}
		}

		if let Some(right) = instr.kind.get_op_rhs_mut() {
			if let Some(loc) = loc_mapping.get(right) {
				*right = loc.clone();
				run_again = true;
			}
		}

		for loc in instr.get_used_locals() {
			if dont_remove.contains(loc) {
				continue;
			}
			loc_mapping.remove(loc);
			loc_mapping.retain(|_, x| x != loc);
		}
	}

	run_again
}
