use rustc_hash::FxHashMap;

use crate::common::mc::modifier::{Modifier, StoreModLocation};
use crate::common::reg::GetUsedRegs;
use crate::common::{val::MutableScoreValue, val::ScoreValue, Identifier};
use crate::lir::{LIRBlock, LIRInstrKind, LIR};
use crate::passes::{LIRPass, Pass};
use crate::util::{remove_indices, HashSetEmptyTracker};

use super::OptimizableValue;

pub struct ScoreboardDataflowPass;

impl Pass for ScoreboardDataflowPass {
	fn get_name(&self) -> &'static str {
		"scoreboard_dataflow"
	}
}

impl LIRPass for ScoreboardDataflowPass {
	fn run_pass(&mut self, lir: &mut LIR) -> anyhow::Result<()> {
		let mut flow_points = FxHashMap::default();
		let mut finished_flow_points = Vec::new();
		let mut instrs_to_remove = HashSetEmptyTracker::new();

		for func in lir.functions.values_mut() {
			instrs_to_remove.clear();

			let block = &mut func.block;

			loop {
				flow_points.clear();
				finished_flow_points.clear();
				let run_again = run_scoreboard_dataflow_iter(
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

fn run_scoreboard_dataflow_iter(
	block: &mut LIRBlock,
	instrs_to_remove: &mut HashSetEmptyTracker<usize>,
	flow_points: &mut FxHashMap<OptimizableValue, SBDataflowPoint>,
	finished_flow_points: &mut Vec<SBDataflowPoint>,
) -> bool {
	let mut run_again = false;

	for (i, instr) in block.contents.iter_mut().enumerate() {
		if instrs_to_remove.contains(&i) {
			continue;
		}
		match &instr.kind {
			LIRInstrKind::SetScore(left, right) => {
				if let MutableScoreValue::Reg(left) = left {
					if let ScoreValue::Mutable(right) = right {
						if let Some(right) = right.to_optimizable_value() {
							if let Some(point) = flow_points.get_mut(&right) {
								point.store_regs.push(left.clone());
								instrs_to_remove.insert(i);
							} else {
								finished_flow_points.extend(
									flow_points.remove(&OptimizableValue::Reg(left.clone())),
								);
								flow_points.insert(
									OptimizableValue::Reg(left.clone()),
									SBDataflowPoint {
										pos: i,
										store_regs: Vec::new(),
									},
								);
								finished_flow_points.extend(flow_points.remove(&right));
								flow_points.insert(
									right,
									SBDataflowPoint {
										pos: i,
										store_regs: Vec::new(),
									},
								);
							}
						}
					} else {
						finished_flow_points
							.extend(flow_points.remove(&OptimizableValue::Reg(left.clone())));
						flow_points.insert(
							OptimizableValue::Reg(left.clone()),
							SBDataflowPoint {
								pos: i,
								store_regs: Vec::new(),
							},
						);
					}
				}
			}
			LIRInstrKind::AddScore(left, right)
			| LIRInstrKind::SubScore(left, right)
			| LIRInstrKind::MulScore(left, right)
			| LIRInstrKind::DivScore(left, right)
			| LIRInstrKind::ModScore(left, right)
			| LIRInstrKind::MinScore(left, right)
			| LIRInstrKind::MaxScore(left, right) => {
				if let Some(left) = left.to_optimizable_value() {
					if let ScoreValue::Mutable(right) = right {
						if let Some(right) = right.to_optimizable_value() {
							finished_flow_points.extend(flow_points.remove(&right));
						}
					}

					flow_points.insert(
						left,
						SBDataflowPoint {
							pos: i,
							store_regs: Vec::new(),
						},
					);
				}
			}
			LIRInstrKind::SwapScore(left, right) => {
				if let Some(left) = left.to_optimizable_value() {
					if let Some(right) = right.to_optimizable_value() {
						finished_flow_points.extend(flow_points.remove(&left));
						finished_flow_points.extend(flow_points.remove(&right));
					}
				}
			}
			LIRInstrKind::ConstIndexToScore { score: val, .. } => {
				if let Some(val) = val.to_optimizable_value() {
					finished_flow_points.extend(flow_points.remove(&val));
				}
			}
			LIRInstrKind::Use(val) => {
				if let Some(val) = val.to_optimizable_value() {
					finished_flow_points.extend(flow_points.remove(&val));
				}
			}
			_ => {
				let regs = instr.get_used_regs();
				for reg in regs {
					finished_flow_points
						.extend(flow_points.remove(&OptimizableValue::Reg(reg.clone())));
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
			if !point.store_regs.is_empty() {
				run_again = true;
			}

			instr.modifiers.extend(
				point
					.store_regs
					.into_iter()
					.map(|x| Modifier::StoreResult(StoreModLocation::Reg(x, 1.0))),
			);
		} else {
			continue;
		}
	}

	run_again
}

#[derive(Debug, Clone)]
struct SBDataflowPoint {
	pos: usize,
	store_regs: Vec<Identifier>,
}
