use anyhow::anyhow;
use dashmap::{DashMap, DashSet};

use crate::common::modifier::{Modifier, StoreModLocation};
use crate::common::{Identifier, MutableValue, Value};
use crate::lir::{LIRBlock, LIRInstrKind, LIR};
use crate::passes::{LIRPass, Pass};
use crate::util::remove_indices;

pub struct ScoreboardDataflowPass;

impl Pass for ScoreboardDataflowPass {
	fn get_name(&self) -> &'static str {
		"scoreboard_dataflow"
	}
}

impl LIRPass for ScoreboardDataflowPass {
	fn run_pass(&mut self, lir: &mut LIR) -> anyhow::Result<()> {
		let mut flow_points = DashMap::new();
		let mut finished_flow_points = Vec::new();
		let mut instrs_to_remove = DashSet::new();

		for (_, block) in &mut lir.functions {
			instrs_to_remove.clear();

			let block = lir
				.blocks
				.get_mut(block)
				.ok_or(anyhow!("Block does not exist"))?;

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
	instrs_to_remove: &mut DashSet<usize>,
	flow_points: &mut DashMap<Identifier, SBDataflowPoint>,
	finished_flow_points: &mut Vec<SBDataflowPoint>,
) -> bool {
	let mut run_again = false;

	for (i, instr) in block.contents.iter_mut().enumerate() {
		if instrs_to_remove.contains(&i) {
			continue;
		}
		match &instr.kind {
			LIRInstrKind::SetScore(left, right) => {
				let MutableValue::Register(left) = left;
				if let Value::Mutable(MutableValue::Register(right)) = right {
					if let Some(mut point) = flow_points.get_mut(right) {
						point.store_regs.push(left.clone());
						instrs_to_remove.insert(i);
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
				let MutableValue::Register(left) = left;
				if let Value::Mutable(MutableValue::Register(right)) = right {
					finished_flow_points.extend(flow_points.remove(right).map(|x| x.1));
				}

				flow_points.insert(
					left.clone(),
					SBDataflowPoint {
						pos: i,
						store_regs: Vec::new(),
					},
				);
			}
			LIRInstrKind::SwapScore(left, right) => {
				let MutableValue::Register(left) = left;
				let MutableValue::Register(right) = right;
				finished_flow_points.extend(flow_points.remove(left).map(|x| x.1));
				finished_flow_points.extend(flow_points.remove(right).map(|x| x.1));
			}
			LIRInstrKind::ConstIndexToScore { score: reg, .. } | LIRInstrKind::Use(reg) => {
				let MutableValue::Register(reg) = reg;
				finished_flow_points.extend(flow_points.remove(reg).map(|x| x.1));
			}
			_ => {}
		};
	}

	for point in flow_points
		.iter()
		.map(|x| x.value().clone())
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
					.map(|x| Modifier::StoreResult(StoreModLocation::Reg(x))),
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
