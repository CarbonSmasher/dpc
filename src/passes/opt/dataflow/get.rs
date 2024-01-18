use rustc_hash::FxHashMap;

use crate::common::reg::GetUsedRegs;
use crate::common::{val::MutableScoreValue, Identifier};
use crate::lir::{LIRBlock, LIRInstrKind, LIR};
use crate::passes::{LIRPass, Pass};
use crate::util::{remove_indices, HashSetEmptyTracker};

pub struct DataflowGetPass;

impl Pass for DataflowGetPass {
	fn get_name(&self) -> &'static str {
		"get_dataflow"
	}
}

impl LIRPass for DataflowGetPass {
	fn run_pass(&mut self, lir: &mut LIR) -> anyhow::Result<()> {
		let mut flow_points = FxHashMap::default();
		let mut instrs_to_remove = HashSetEmptyTracker::new();

		for func in lir.functions.values_mut() {
			instrs_to_remove.clear();

			let block = &mut func.block;

			loop {
				flow_points.clear();
				let run_again = run_iter(block, &mut instrs_to_remove, &mut flow_points);
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
	flow_points: &mut FxHashMap<Identifier, DataflowPoint>,
) -> bool {
	let mut run_again = false;

	for (i, instr) in block.contents.iter_mut().enumerate() {
		if instrs_to_remove.contains(&i) {
			continue;
		}

		// Register to not remove at the end because it was just assigned
		let mut reg_to_keep = None;

		if let Some(MutableScoreValue::Reg(left)) = instr.kind.get_simple_sb_op_lhs() {
			flow_points.insert(
				left.clone(),
				DataflowPoint {
					pos: i,
					kind: instr.kind.clone(),
				},
			);
			reg_to_keep = Some(left.clone());
		}

		if let LIRInstrKind::GetScore(MutableScoreValue::Reg(right)) = &instr.kind {
			let right = right.clone();
			if let Some(point) = flow_points.get(&right) {
				instr.kind = point.kind.clone();
				instrs_to_remove.insert(point.pos);
				run_again = true;
			}
			flow_points.remove(&right);
		}

		for reg in instr.get_used_regs() {
			if let Some(keep) = &reg_to_keep {
				if keep == reg {
					continue;
				}
			}
			flow_points.remove(reg);
		}
	}

	run_again
}

#[derive(Debug, Clone)]
struct DataflowPoint {
	pos: usize,
	kind: LIRInstrKind,
}
