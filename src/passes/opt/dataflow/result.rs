use rustc_hash::FxHashMap;

use crate::common::mc::modifier::{Modifier, StoreModLocation};
use crate::common::reg::{GetUsedLocals, Local};
use crate::common::{val::MutableScoreValue, val::ScoreValue};
use crate::lir::{LIRBlock, LIRInstrKind};
use crate::passes::{LIRPass, LIRPassData, Pass};
use crate::project::{OptimizationLevel, ProjectSettings};
use crate::util::{remove_indices, HashSetEmptyTracker};

pub struct DataflowResultPass;

impl Pass for DataflowResultPass {
	fn get_name(&self) -> &'static str {
		"result_dataflow"
	}

	fn should_run(&self, proj: &ProjectSettings) -> bool {
		proj.op_level >= OptimizationLevel::Full
	}
}

impl LIRPass for DataflowResultPass {
	fn run_pass(&mut self, data: &mut LIRPassData) -> anyhow::Result<()> {
		let mut flow_points = FxHashMap::default();
		let mut finished_flow_points = Vec::new();
		let mut instrs_to_remove = HashSetEmptyTracker::new();

		for func in data.lir.functions.values_mut() {
			instrs_to_remove.clear();

			let block = &mut func.block;

			loop {
				flow_points.clear();
				finished_flow_points.clear();
				let run_again = run_iter(
					block,
					&mut instrs_to_remove,
					&mut flow_points,
					&mut finished_flow_points,
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
	flow_points: &mut FxHashMap<Local, SBDataflowPoint>,
	finished_flow_points: &mut Vec<SBDataflowPoint>,
) -> bool {
	let mut run_again = false;

	for (i, instr) in block.contents.iter().enumerate() {
		if instrs_to_remove.contains(&i) {
			continue;
		}
		match &instr.kind {
			LIRInstrKind::SetScore(MutableScoreValue::Local(left), right) => {
				if let ScoreValue::Mutable(MutableScoreValue::Local(right)) = right {
					if let Some(point) = flow_points.get_mut(right) {
						if instr.modifiers.is_empty() {
							point.store_locs.push(left.clone());
							instrs_to_remove.insert(i);
						}
					} else {
						finished_flow_points.extend(flow_points.remove(left));
						flow_points.insert(
							left.clone(),
							SBDataflowPoint {
								pos: i,
								store_locs: Vec::new(),
							},
						);
						finished_flow_points.extend(flow_points.remove(right));
						flow_points.insert(
							right.clone(),
							SBDataflowPoint {
								pos: i,
								store_locs: Vec::new(),
							},
						);
					}
				} else {
					finished_flow_points.extend(flow_points.remove(left));
					flow_points.insert(
						left.clone(),
						SBDataflowPoint {
							pos: i,
							store_locs: Vec::new(),
						},
					);
				}
			}
			LIRInstrKind::AddScore(left, right)
			| LIRInstrKind::SubScore(left, right)
			| LIRInstrKind::MulScore(left, right)
			| LIRInstrKind::DivScore(left, right)
			| LIRInstrKind::ModScore(left, right)
			| LIRInstrKind::MinScore(left, right)
			| LIRInstrKind::MaxScore(left, right) => {
				if let ScoreValue::Mutable(MutableScoreValue::Local(right)) = right {
					finished_flow_points.extend(flow_points.remove(right));
				}
				if let MutableScoreValue::Local(left) = left {
					finished_flow_points.extend(flow_points.remove(left));
					flow_points.insert(
						left.clone(),
						SBDataflowPoint {
							pos: i,
							store_locs: Vec::new(),
						},
					);
				}
			}
			_ => {
				for loc in instr.get_used_locals() {
					finished_flow_points.extend(flow_points.remove(loc));
				}
			}
		};
	}

	for point in flow_points
		.iter()
		.map(|x| x.1.clone())
		.chain(finished_flow_points.iter().cloned())
	{
		if let Some(instr) = block.contents.get_mut(point.pos) {
			if !point.store_locs.is_empty() {
				run_again = true;
			}

			instr.modifiers.extend(
				point
					.store_locs
					.into_iter()
					.map(|x| Modifier::StoreResult(StoreModLocation::Local(x, 1.0))),
			);
		}
	}

	run_again
}

#[derive(Debug, Clone)]
struct SBDataflowPoint {
	pos: usize,
	store_locs: Vec<Local>,
}
