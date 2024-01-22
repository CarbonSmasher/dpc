use std::cell::RefCell;

use rustc_hash::{FxHashMap, FxHashSet};

use crate::common::val::ScoreValue;
use crate::common::{val::MutableScoreValue, Identifier};
use crate::lir::{LIRBlock, LIRInstrKind};
use crate::passes::{LIRPass, LIRPassData, Pass};
use crate::project::{OptimizationLevel, ProjectSettings};

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
		let mut used_args = FxHashSet::default();
		let mut call_ret_mapping = FxHashMap::default();

		for func in data.lir.functions.values_mut() {
			let block = &mut func.block;

			loop {
				arg_mapping.clear();
				used_args.clear();
				call_ret_mapping.clear();
				let run_again = run_iter(
					block,
					&mut arg_mapping,
					&mut used_args,
					&mut call_ret_mapping,
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
	arg_mapping: &mut FxHashMap<Identifier, MutableScoreValue>,
	used_args: &mut FxHashSet<usize>,
	call_ret_mapping: &mut FxHashMap<Identifier, MutableScoreValue>,
) -> bool {
	let mut run_again = false;

	for instr in &mut block.contents.iter_mut() {
		match &instr.kind {
			LIRInstrKind::SetScore(
				MutableScoreValue::Reg(l),
				ScoreValue::Mutable(MutableScoreValue::Arg(r)),
			) => {
				if !used_args.contains(r) {
					arg_mapping.insert(l.clone(), MutableScoreValue::Arg(*r));
					// We don't want to create weird assign arg to self instructions, continue
					continue;
				}
			}
			LIRInstrKind::SetScore(
				MutableScoreValue::Reg(l),
				ScoreValue::Mutable(r @ MutableScoreValue::CallReturnValue(..)),
			) => {
				call_ret_mapping.insert(l.clone(), r.clone());
				// We don't want to create weird assign ret to self instructions, continue
				continue;
			}
			// Remove any other assignments to regs since now the reg
			// isn't the arg or ret anymore
			LIRInstrKind::SetScore(MutableScoreValue::Reg(l), ..) => {
				arg_mapping.remove(l);
				call_ret_mapping.remove(l);
			}
			_ => {}
		}

		// Arguments that have been read before replacement are
		// marked as unusable in the future since the value has now
		// changed
		if let Some(arg) = instr.kind.get_read_score_arg() {
			used_args.insert(*arg);
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
				if let Some(call_ret) = call_ret_mapping.get(reg) {
					*val = call_ret.clone();
					*run_again_2.borrow_mut() = true;
				}
			}
		});
		run_again = run_again_2.take();
	}

	run_again
}