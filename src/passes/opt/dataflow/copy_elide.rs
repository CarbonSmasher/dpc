use std::cell::RefCell;

use rustc_hash::FxHashMap;

use crate::common::val::ScoreValue;
use crate::common::{val::MutableScoreValue, Identifier};
use crate::lir::{LIRBlock, LIRInstrKind};
use crate::passes::{LIRPass, LIRPassData, Pass};
use crate::project::{OptimizationLevel, ProjectSettings};
use crate::util::{remove_indices, HashSetEmptyTracker};

pub struct CopyElisionPass;

impl Pass for CopyElisionPass {
	fn get_name(&self) -> &'static str {
		"copy_elision"
	}

	fn should_run(&self, proj: &ProjectSettings) -> bool {
		proj.op_level >= OptimizationLevel::Basic
	}
}

impl LIRPass for CopyElisionPass {
	fn run_pass(&mut self, data: &mut LIRPassData) -> anyhow::Result<()> {
		let mut arg_mapping = FxHashMap::default();
		let mut ret_mapping = FxHashMap::default();
		let mut instrs_to_remove = HashSetEmptyTracker::new();

		for func in data.lir.functions.values_mut() {
			instrs_to_remove.clear();

			let block = &mut func.block;

			loop {
				arg_mapping.clear();
				ret_mapping.clear();
				let run_again = run_iter(
					block,
					&mut instrs_to_remove,
					&mut arg_mapping,
					&mut ret_mapping,
				);
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
	arg_mapping: &mut FxHashMap<Identifier, MutableScoreValue>,
	ret_mapping: &mut FxHashMap<Identifier, MutableScoreValue>,
) -> bool {
	let mut run_again = false;

	for (i, instr) in block.contents.iter_mut().enumerate() {
		if instrs_to_remove.contains(&i) {
			continue;
		}

		match &instr.kind {
			LIRInstrKind::SetScore(
				MutableScoreValue::Reg(l),
				ScoreValue::Mutable(MutableScoreValue::Arg(r)),
			) => {
				arg_mapping.insert(l.clone(), MutableScoreValue::Arg(*r));
				// We don't want to create weird assign arg to self instructions, continue
				continue;
			}
			LIRInstrKind::SetScore(
				MutableScoreValue::Reg(l),
				ScoreValue::Mutable(r @ MutableScoreValue::CallReturnValue(..)),
			) => {
				ret_mapping.insert(l.clone(), r.clone());
				// We don't want to create weird assign ret to self instructions, continue
				continue;
			}
			// Remove any other assignments to regs since now the reg
			// isn't the arg or ret anymore
			LIRInstrKind::SetScore(MutableScoreValue::Reg(l), ..) => {
				arg_mapping.remove(l);
				ret_mapping.remove(l);
			}
			_ => {}
		}

		// Replace the uses of registers that are args or call rets
		// with the values themselves
		let run_again_2 = RefCell::new(false); // Closure thing
		instr.replace_mut_score_vals(&|val| {
			if let MutableScoreValue::Reg(reg) = val {
				if let Some(arg) = arg_mapping.get(reg) {
					*val = arg.clone();
					*run_again_2.borrow_mut() = true;
					return;
				}
				if let Some(ret) = ret_mapping.get(reg) {
					*val = ret.clone();
					*run_again_2.borrow_mut() = true;
				}
			}
		});
		run_again = run_again_2.take();
	}

	run_again
}
