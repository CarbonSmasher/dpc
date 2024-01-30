use rustc_hash::FxHashMap;

use crate::common::reg::{GetUsedLocals, Local};
use crate::common::val::MutableScoreValue;
use crate::lir::{LIRBlock, LIRInstrKind};
use crate::passes::{LIRPass, LIRPassData, Pass};
use crate::project::{OptimizationLevel, ProjectSettings};
use crate::util::{remove_indices, HashSetEmptyTracker};

pub struct DataflowGetPass;

impl Pass for DataflowGetPass {
	fn get_name(&self) -> &'static str {
		"get_dataflow"
	}

	fn should_run(&self, proj: &ProjectSettings) -> bool {
		proj.op_level >= OptimizationLevel::Full
	}
}

impl LIRPass for DataflowGetPass {
	fn run_pass(&mut self, data: &mut LIRPassData) -> anyhow::Result<()> {
		let mut flow_points = FxHashMap::default();
		let mut instrs_to_remove = HashSetEmptyTracker::new();

		for func in data.lir.functions.values_mut() {
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
	flow_points: &mut FxHashMap<Local, DataflowPoint>,
) -> bool {
	let mut run_again = false;

	for (i, instr) in block.contents.iter_mut().enumerate() {
		if instrs_to_remove.contains(&i) {
			continue;
		}

		// Local to not remove at the end because it was just assigned
		let mut loc_to_keep = None;

		if let Some(MutableScoreValue::Local(left)) = instr.kind.get_simple_sb_op_lhs() {
			flow_points.insert(
				left.clone(),
				DataflowPoint {
					pos: i,
					kind: instr.kind.clone(),
				},
			);
			loc_to_keep = Some(left.clone());
		}

		if let LIRInstrKind::GetScore(MutableScoreValue::Local(right)) = &instr.kind {
			let right = right.clone();
			if let Some(point) = flow_points.get(&right) {
				instr.kind = point.kind.clone();
				instrs_to_remove.insert(point.pos);
				run_again = true;
			}
			flow_points.remove(&right);
		}

		for loc in instr.get_used_locals() {
			if let Some(keep) = &loc_to_keep {
				if keep == loc {
					continue;
				}
			}
			flow_points.remove(loc);
		}
	}

	run_again
}

#[derive(Debug, Clone)]
struct DataflowPoint {
	pos: usize,
	kind: LIRInstrKind,
}
