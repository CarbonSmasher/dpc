use crate::common::condition::Condition;
use crate::common::cost::GetCost;
use crate::mir::MIRBlock;
use crate::passes::util::RunAgain;
use crate::passes::{MIRPass, MIRPassData, Pass};

/// Reorders and + or conditions based on the cost of their terms
pub struct ReorderConditionsPass;

impl Pass for ReorderConditionsPass {
	fn get_name(&self) -> &'static str {
		"reorder_conditions"
	}
}

impl MIRPass for ReorderConditionsPass {
	fn run_pass(&mut self, data: &mut MIRPassData) -> anyhow::Result<()> {
		for func in data.mir.functions.values_mut() {
			run_block(&mut func.block);
		}

		Ok(())
	}
}

fn run_block(block: &mut MIRBlock) -> RunAgain {
	let mut out = RunAgain::new();
	loop {
		let run_again = run_iter(block);
		out.merge(run_again);
		if !run_again {
			break;
		}
	}
	out
}

fn run_iter(block: &mut MIRBlock) -> RunAgain {
	let mut run_again = RunAgain::new();

	for instr in &mut block.contents {
		if let Some(condition) = instr.kind.get_condition_mut() {
			reorder(condition);
		}

		for body in instr.kind.get_bodies_mut() {
			run_again.merge(run_block(body));
		}
	}

	run_again
}

fn reorder(condition: &mut Condition) {
	match condition {
		Condition::And(l, r) | Condition::Or(l, r) => {
			reorder(l);
			reorder(r);
			if l.get_cost() > r.get_cost() {
				std::mem::swap(l, r);
			}
		}
		_ => {}
	}
}
