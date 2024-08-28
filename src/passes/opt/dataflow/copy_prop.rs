use intset::GrowSet;
use rustc_hash::{FxHashMap, FxHashSet};

use crate::common::reg::{GetUsedLocals, Local};
use crate::common::val::MutableScoreValue;
use crate::common::val::{MutableNBTValue, NBTValue, ScoreValue};
use crate::lir::{LIRBlock, LIRInstrKind};
use crate::passes::util::usage_analysis::analyze_write_after_copy;
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
		let mut blacklist = FxHashSet::default();

		for func in data.lir.functions.values_mut() {
			let block = &mut func.block;

			let writes_after_copies = analyze_write_after_copy(block);

			loop {
				loc_mapping.clear();
				let run_again = run_iter(
					block,
					&mut loc_mapping,
					&mut blacklist,
					&writes_after_copies,
				);
				if !run_again {
					break;
				}
			}
		}

		Ok(())
	}
}

fn run_iter(
	block: &mut LIRBlock,
	loc_mapping: &mut FxHashMap<Local, Local>,
	blacklist: &mut FxHashSet<Local>,
	writes_after_copies: &GrowSet,
) -> bool {
	let mut run_again = false;

	for (i, instr) in block.contents.iter_mut().enumerate() {
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
				// Don't add blacklisted props
				if !blacklist.contains(l) {
					// Add this to the blacklist if it modifies the value
					if writes_after_copies.contains(i) {
						blacklist.insert(l.clone());
					}
					loc_mapping.insert(l.clone(), r.clone());
					dont_remove.push(l.clone());
					dont_remove.push(r.clone());
				}
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
