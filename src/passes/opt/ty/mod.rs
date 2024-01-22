use crate::common::block::Block;
use crate::common::condition::Condition;
use crate::common::ty::{DataType, DataTypeContents, ScoreType};
use crate::common::val::{MutableValue, Value};
use crate::common::{Register, RegisterList};
use crate::mir::{MIRBlock, MIRInstrKind};
use crate::passes::util::RunAgain;
use crate::passes::{MIRPass, MIRPassData, Pass};
use crate::project::{OptimizationLevel, ProjectSettings};
use crate::util::remove_indices;

use intset::GrowSet;

/// Does optimizations on values based on type information
pub struct TypeBasedOptimizationPass;

impl Pass for TypeBasedOptimizationPass {
	fn get_name(&self) -> &'static str {
		"type_based_optimization"
	}

	fn should_run(&self, proj: &ProjectSettings) -> bool {
		proj.op_level >= OptimizationLevel::More
	}
}

impl MIRPass for TypeBasedOptimizationPass {
	fn run_pass(&mut self, data: &mut MIRPassData) -> anyhow::Result<()> {
		for func in data.mir.functions.values_mut() {
			run_block(&mut func.block);
		}

		Ok(())
	}
}

fn run_block(block: &mut MIRBlock) -> RunAgain {
	let mut out = RunAgain::new();
	let mut instrs_to_remove = block.get_index_set();
	loop {
		let run_again = run_iter(block, &mut instrs_to_remove);
		out.merge(run_again);
		if !run_again {
			break;
		}
	}
	remove_indices(&mut block.contents, &instrs_to_remove);
	out
}

fn run_iter(block: &mut MIRBlock, instrs_to_remove: &mut GrowSet) -> RunAgain {
	let mut run_again = RunAgain::new();

	let mut regs = RegisterList::default();

	for (i, instr) in block.contents.iter_mut().enumerate() {
		if instrs_to_remove.contains(i) {
			continue;
		}
		if let MIRInstrKind::Declare { left, ty } = &instr.kind {
			regs.insert(
				left.clone(),
				Register {
					id: left.clone(),
					ty: ty.clone(),
				},
			);
		};
		let remove = match &instr.kind {
			// Bool is already 0 or 1, don't need to abs it
			MIRInstrKind::Abs {
				val: MutableValue::Reg(reg),
			} => {
				if let Some(Register {
					ty: DataType::Score(ScoreType::Bool),
					..
				}) = regs.get(reg)
				{
					true
				} else {
					false
				}
			}
			_ => false,
		};

		if remove {
			instrs_to_remove.add(i);
			run_again.yes();
			continue;
		}

		let repl = match &instr.kind {
			// Mul of two bools -> and
			MIRInstrKind::Mul {
				left: MutableValue::Reg(l),
				right: Value::Mutable(MutableValue::Reg(r)),
			} => {
				if let Some(Register {
					ty: DataType::Score(ScoreType::Bool),
					..
				}) = regs.get(l)
				{
					if let Some(Register {
						ty: DataType::Score(ScoreType::Bool),
						..
					}) = regs.get(r)
					{
						Some(MIRInstrKind::And {
							left: MutableValue::Reg(l.clone()),
							right: Value::Mutable(MutableValue::Reg(r.clone())),
						})
					} else {
						None
					}
				} else {
					None
				}
			}
			_ => None,
		};

		if let Some(repl) = repl {
			instr.kind = repl;
			run_again.yes();
		}

		if let Some(condition) = instr.kind.get_condition_mut() {
			visit_condition(condition, &regs);
		}

		for body in instr.kind.get_bodies_mut() {
			run_again.merge(run_block(body));
		}
	}

	run_again
}

fn visit_condition(condition: &mut Condition, regs: &RegisterList) {
	match condition {
		Condition::And(l, r) | Condition::Or(l, r) => {
			visit_condition(l, regs);
			visit_condition(r, regs);
		}
		Condition::Equal(
			Value::Mutable(MutableValue::Reg(l)),
			Value::Constant(DataTypeContents::Score(r)),
		) => {
			if let Some(Register {
				ty: DataType::Score(ScoreType::Bool),
				..
			}) = regs.get(l)
			{
				if r.get_i32() == 1 {
					*condition = Condition::Bool(Value::Mutable(MutableValue::Reg(l.clone())));
				} else if r.get_i32() == 0 {
					*condition = Condition::NotBool(Value::Mutable(MutableValue::Reg(l.clone())));
				}
			}
		}
		_ => {}
	}
}
