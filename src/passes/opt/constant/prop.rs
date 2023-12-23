use anyhow::anyhow;

use crate::common::condition::Condition;
use crate::common::val::{MutableValue, Value};
use crate::common::DeclareBinding;
use crate::mir::{MIRBlock, MIRInstrKind};
use crate::passes::{MIRPass, MIRPassData, Pass};

use super::ConstAnalyzer;

pub struct ConstPropPass {
	pub(super) made_changes: bool,
}

impl ConstPropPass {
	pub fn new() -> Self {
		Self {
			made_changes: false,
		}
	}
}

impl Pass for ConstPropPass {
	fn get_name(&self) -> &'static str {
		"const_prop"
	}

	fn made_changes(&self) -> bool {
		self.made_changes
	}
}

impl MIRPass for ConstPropPass {
	fn run_pass(&mut self, data: &mut MIRPassData) -> anyhow::Result<()> {
		for (_, block) in &mut data.mir.functions {
			let block = data
				.mir
				.blocks
				.get_mut(block)
				.ok_or(anyhow!("Block does not exist"))?;
			loop {
				let run_again = run_const_prop_iter(block);
				if run_again {
					self.made_changes = true;
				} else {
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

	let mut an = ConstAnalyzer::new();
	for instr in &mut block.contents {
		const_prop_instr(&mut instr.kind, &mut an, &mut run_again);
		an.feed(&instr.kind);
	}

	run_again
}

fn const_prop_instr(instr: &mut MIRInstrKind, an: &mut ConstAnalyzer, run_again: &mut bool) {
	match instr {
		MIRInstrKind::Assign {
			right: DeclareBinding::Value(right),
			..
		}
		| MIRInstrKind::Add { right, .. }
		| MIRInstrKind::Sub { right, .. }
		| MIRInstrKind::Mul { right, .. }
		| MIRInstrKind::Div { right, .. }
		| MIRInstrKind::Mod { right, .. }
		| MIRInstrKind::Min { right, .. }
		| MIRInstrKind::Max { right, .. }
		| MIRInstrKind::Merge { right, .. }
		| MIRInstrKind::Push { right, .. }
		| MIRInstrKind::PushFront { right, .. }
		| MIRInstrKind::Insert { right, .. } => {
			if let Value::Mutable(MutableValue::Register(reg)) = right.clone() {
				if let Some(val) = an.vals.get(&reg) {
					*right = Value::Constant(val.clone());
					*run_again = true;
				}
			}
		}
		MIRInstrKind::If { condition, body } => match condition {
			Condition::Equal(l, r)
			| Condition::GreaterThan(l, r)
			| Condition::GreaterThanOrEqual(l, r)
			| Condition::LessThan(l, r)
			| Condition::LessThanOrEqual(l, r) => {
				if let Value::Mutable(MutableValue::Register(reg)) = l.clone() {
					if let Some(val) = an.vals.get(&reg) {
						*l = Value::Constant(val.clone());
						*run_again = true;
					}
				}
				if let Value::Mutable(MutableValue::Register(reg)) = r.clone() {
					if let Some(val) = an.vals.get(&reg) {
						*r = Value::Constant(val.clone());
						*run_again = true;
					}
				}
				const_prop_instr(body, an, run_again);
			}
			_ => {}
		},
		_ => {}
	};
}
