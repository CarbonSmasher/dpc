use std::collections::HashMap;

use crate::common::ty::DataTypeContents;
use crate::common::{DeclareBinding, MutableValue, Value};
use crate::mir::{MIRBlock, MIRInstrKind, MIR};
use crate::passes::MIRPass;
use crate::util::remove_indices;

pub struct DSEPass;

impl MIRPass for DSEPass {
	fn run_pass(&mut self, mir: &mut MIR) -> anyhow::Result<()> {
		for (_, block) in &mut mir.functions {
			loop {
				let run_again = run_dse_iter(block);
				if !run_again {
					break;
				}
			}
		}

		Ok(())
	}
}

/// Runs an iteration of DSE and returns true if another iteration should be performed
fn run_dse_iter(block: &mut MIRBlock) -> bool {
	let mut run_again = false;
	let mut elim_candidates = HashMap::new();
	let mut dead_stores = Vec::new();

	for (i, instr) in block.contents.iter().enumerate() {
		if let MIRInstrKind::Assign {
			left: MutableValue::Register(id),
			..
		} = &instr.kind
		{
			// If the candidate already exists, then that is a dead store that can be removed
			if let Some(candidate) = elim_candidates.get(id) {
				dead_stores.push(*candidate);
			}
			elim_candidates.insert(id.clone(), i);
		}

		// Check if this instruction uses any of the registers that we have marked
		// for elimination
		let used_regs = instr.kind.get_used_regs();
		for reg in used_regs {
			if let Some(candidate) = elim_candidates.get(reg) {
				// Don't remove the candidate we just set
				if *candidate == i {
					continue;
				}
				elim_candidates.remove(reg);
			}
		}
	}

	if !dead_stores.is_empty() || !elim_candidates.is_empty() {
		run_again = true;
		// Any remaining elimination candidates are also unused stores
		let elim_candidates: Vec<_> = elim_candidates.values().cloned().collect();
		let to_remove = [dead_stores, elim_candidates].concat();
		remove_indices(&mut block.contents, &to_remove);
	}

	run_again
}

pub struct MIRSimplifyPass;

impl MIRPass for MIRSimplifyPass {
	fn run_pass(&mut self, mir: &mut MIR) -> anyhow::Result<()> {
		for (_, block) in &mut mir.functions {
			loop {
				let run_again = run_mir_simplify_iter(block);
				if !run_again {
					break;
				}
			}
		}

		Ok(())
	}
}

/// Runs an iteration of the MIRSimplifyPass. Returns true if another iteration
/// should be run
fn run_mir_simplify_iter(block: &mut MIRBlock) -> bool {
	let mut run_again = false;

	let mut instrs_to_remove = Vec::new();
	for (i, instr) in block.contents.iter_mut().enumerate() {
		let remove = match &instr.kind {
			// Reflexive property; swap with self
			MIRInstrKind::Swap { left, right } if left.is_same_val(right) => true,
			// Multiplies and divides by 1
			MIRInstrKind::Mul {
				left: _,
				right: Value::Constant(DataTypeContents::Score(score)),
			}
			| MIRInstrKind::Div {
				left: _,
				right: Value::Constant(DataTypeContents::Score(score)),
			} if score.get_i32() == 1 => true,
			_ => false,
		};

		if remove {
			instrs_to_remove.push(i);
			run_again = true;
		}

		// Instructions to replace
		let kind_repl = match &instr.kind {
			_ => None,
		};

		if let Some(kind_repl) = kind_repl {
			instr.kind = kind_repl;
			run_again = true;
		}
	}

	remove_indices(&mut block.contents, &instrs_to_remove);

	run_again
}

pub struct ConstPropPass;

impl MIRPass for ConstPropPass {
	fn run_pass(&mut self, mir: &mut MIR) -> anyhow::Result<()> {
		for (_, block) in &mut mir.functions {
			loop {
				let run_again = run_const_prop_iter(block);
				if !run_again {
					break;
				}
			}
		}

		Ok(())
	}
}

/// Runs an iteration of const prop. Returns true if another iteration
/// should be run
fn run_const_prop_iter(block: &mut MIRBlock) -> bool {
	let mut run_again = false;

	let mut prop_candidates = HashMap::new();
	for instr in &mut block.contents {
		match &mut instr.kind {
			MIRInstrKind::Assign {
				left: MutableValue::Register(reg),
				right: DeclareBinding::Value(Value::Constant(val)),
			} => {
				prop_candidates.insert(reg.clone(), val.clone());
			}
			MIRInstrKind::Assign { right, .. } => {
				if let DeclareBinding::Value(Value::Mutable(MutableValue::Register(reg))) = &right {
					prop_candidates.remove(reg);
				}
			}
			MIRInstrKind::Declare { .. } => {}
			MIRInstrKind::Add { left, right }
			| MIRInstrKind::Sub { left, right }
			| MIRInstrKind::Mul { left, right }
			| MIRInstrKind::Div { left, right }
			| MIRInstrKind::Mod { left, right }
			| MIRInstrKind::Min { left, right }
			| MIRInstrKind::Max { left, right } => {
				let MutableValue::Register(reg) = left;
				prop_candidates.remove(reg);
				if let Value::Mutable(MutableValue::Register(reg)) = right.clone() {
					if let Some(val) = prop_candidates.get(&reg) {
						*right = Value::Constant(val.clone());
						run_again = true;
					}
				}
			}
			MIRInstrKind::Swap { left, right } => {
				let MutableValue::Register(reg) = left;
				prop_candidates.remove(reg);
				let MutableValue::Register(reg) = right;
				prop_candidates.remove(reg);
			}
			MIRInstrKind::Abs { val } | MIRInstrKind::Use { val } => {
				let MutableValue::Register(reg) = val;
				prop_candidates.remove(reg);
			}
		};
	}

	run_again
}
