use anyhow::anyhow;

use crate::common::condition::Condition;
use crate::common::ty::{DataTypeContents, ScoreTypeContents};
use crate::common::val::{MutableValue, Value};
use crate::common::DeclareBinding;
use crate::mir::{MIRBlock, MIRInstrKind};
use crate::passes::{MIRPass, MIRPassData, Pass};

use super::{ConstAnalyzer, ConstAnalyzerValue};

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

impl Default for ConstPropPass {
	fn default() -> Self {
		Self::new()
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
		for func in data.mir.functions.values_mut() {
			let block = data
				.mir
				.blocks
				.get_mut(&func.block)
				.ok_or(anyhow!("Block does not exist"))?;
			loop {
				let run_again = run_const_prop_iter(block)?;
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
fn run_const_prop_iter(block: &mut MIRBlock) -> anyhow::Result<bool> {
	let mut run_again = false;

	let mut an = ConstAnalyzer::new();
	for instr in &mut block.contents {
		const_prop_instr(&mut instr.kind, &mut an, &mut run_again);
		an.feed(&instr.kind)?;
	}

	Ok(run_again)
}

// TODO: Remove assignments and operations with an uninitialized value on the rhs
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
		| MIRInstrKind::Insert { right, .. }
		| MIRInstrKind::And { right, .. }
		| MIRInstrKind::Or { right, .. } => {
			if let Value::Mutable(MutableValue::Register(reg)) = right.clone() {
				if let Some(val) = an.vals.get(&reg) {
					if let ConstAnalyzerValue::Value(val) = val.value() {
						*right = Value::Constant(val.clone());
						*run_again = true;
					}
				}
			}
		}
		// Prop get to get const
		MIRInstrKind::Get { value, scale } => {
			if let MutableValue::Register(reg) = value.clone() {
				if let Some(val) = an.vals.get(&reg) {
					if let ConstAnalyzerValue::Value(val) = val.value() {
						if let Some(val) = val.try_get_i32() {
							let scaled = ((val as f64) * *scale) as i32;
							*instr = MIRInstrKind::GetConst { value: scaled };
						}
					}
				}
			}
		}
		MIRInstrKind::If { condition, body } => {
			match condition {
				Condition::Equal(l, r)
				| Condition::GreaterThan(l, r)
				| Condition::GreaterThanOrEqual(l, r)
				| Condition::LessThan(l, r)
				| Condition::LessThanOrEqual(l, r) => {
					if let Value::Mutable(MutableValue::Register(reg)) = l.clone() {
						if let Some(val) = an.vals.get(&reg) {
							if let ConstAnalyzerValue::Value(val) = val.value() {
								*l = Value::Constant(val.clone());
								*run_again = true;
							}
						}
					}
					if let Value::Mutable(MutableValue::Register(reg)) = r.clone() {
						if let Some(val) = an.vals.get(&reg) {
							if let ConstAnalyzerValue::Value(val) = val.value() {
								*r = Value::Constant(val.clone());
								*run_again = true;
							}
						}
					}
				}
				Condition::Exists(val) => {
					if let Value::Mutable(MutableValue::Register(reg)) = val.clone() {
						if let Some(val) = an.vals.get(&reg) {
							match val.value() {
								ConstAnalyzerValue::Reset(..) => {
									*condition = Condition::Bool(Value::Constant(
										DataTypeContents::Score(ScoreTypeContents::Bool(false)),
									));
								}
								ConstAnalyzerValue::Value(..) => {
									*condition = Condition::Bool(Value::Constant(
										DataTypeContents::Score(ScoreTypeContents::Bool(true)),
									));
								}
							}
							*run_again = true;
						}
					}
				}
				_ => {}
			}
			// Since eq and bool both check if a value is equal to something,
			// that value is then guaranteed to be the value it is equal to
			// in the body of the if
			let previous = match condition {
				Condition::Bool(Value::Mutable(MutableValue::Register(reg))) => (
					Some(reg.clone()),
					an.vals.insert(
						reg.clone(),
						ConstAnalyzerValue::Value(DataTypeContents::Score(
							ScoreTypeContents::Bool(true),
						)),
					),
				),
				Condition::Equal(
					Value::Mutable(MutableValue::Register(reg)),
					Value::Constant(right),
				) => (
					Some(reg.clone()),
					an.vals
						.insert(reg.clone(), ConstAnalyzerValue::Value(right.clone())),
				),
				_ => (None, None),
			};
			const_prop_instr(body, an, run_again);
			// Now we have to restore the previous value
			if let Some(previous_reg) = previous.0 {
				if let Some(previous_val) = previous.1 {
					an.vals.insert(previous_reg, previous_val);
				} else {
					an.vals.remove(&previous_reg);
				}
			}
		}
		MIRInstrKind::As { body, .. }
		| MIRInstrKind::At { body, .. }
		| MIRInstrKind::StoreResult { body, .. }
		| MIRInstrKind::StoreSuccess { body, .. }
		| MIRInstrKind::Positioned { body, .. }
		| MIRInstrKind::ReturnRun { body } => {
			const_prop_instr(body, an, run_again);
		}
		_ => {}
	};
}
