use crate::common::condition::Condition;
use crate::common::ty::{DataTypeContents, ScoreTypeContents};
use crate::common::val::{MutableValue, Value};
use crate::common::DeclareBinding;
use crate::mir::{MIRBlock, MIRInstrKind};
use crate::passes::{MIRPass, MIRPassData, Pass};
use crate::project::{OptimizationLevel, ProjectSettings};

use super::{ConstAnalyzerValue, StoringConstAnalyzer};

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

	fn should_run(&self, proj: &ProjectSettings) -> bool {
		proj.op_level >= OptimizationLevel::Basic
	}
}

impl MIRPass for ConstPropPass {
	fn run_pass(&mut self, data: &mut MIRPassData) -> anyhow::Result<()> {
		let mut an = StoringConstAnalyzer::new();
		for func in data.mir.functions.values_mut() {
			let block = &mut func.block;
			an.reset();
			loop {
				let run_again = run_const_prop_iter(block, &mut an)?;
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
fn run_const_prop_iter(
	block: &mut MIRBlock,
	an: &mut StoringConstAnalyzer,
) -> anyhow::Result<bool> {
	let mut run_again = false;

	for instr in &mut block.contents {
		const_prop_instr(&mut instr.kind, an, &mut run_again);
		an.feed(&instr.kind)?;
	}

	Ok(run_again)
}

// TODO: Remove assignments and operations with an uninitialized value on the rhs
fn const_prop_instr(instr: &mut MIRInstrKind, an: &mut StoringConstAnalyzer, run_again: &mut bool) {
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
			if let Value::Mutable(MutableValue::Reg(reg)) = right {
				if let Some(val) = an.vals.get(reg) {
					if let ConstAnalyzerValue::Value(val) = val {
						*right = Value::Constant(val.clone());
						*run_again = true;
					}
				}
			}
		}
		// Prop get to get const
		MIRInstrKind::Get { value, scale } => {
			if let MutableValue::Reg(reg) = value {
				if let Some(val) = an.vals.get(reg) {
					if let ConstAnalyzerValue::Value(val) = val {
						if let Some(val) = val.try_get_i32() {
							let scaled = ((val as f64) * *scale) as i32;
							*instr = MIRInstrKind::GetConst { value: scaled };
						}
					}
				}
			}
		}
		MIRInstrKind::If { condition, body }
		| MIRInstrKind::IfElse {
			condition,
			first: body,
			..
		} => {
			const_prop_condition(condition, an, run_again);
			// Since eq and bool both check if a value is equal to something,
			// that value is then guaranteed to be the value it is equal to
			// in the body of the if
			let previous = match condition {
				Condition::Bool(Value::Mutable(MutableValue::Reg(reg))) => (
					Some(reg.clone()),
					an.vals.insert(
						reg.clone(),
						ConstAnalyzerValue::Value(DataTypeContents::Score(
							ScoreTypeContents::Bool(true),
						)),
					),
				),
				Condition::Equal(
					Value::Mutable(MutableValue::Reg(reg)),
					Value::Constant(right),
				) => (
					Some(reg.clone()),
					an.vals
						.insert(reg.clone(), ConstAnalyzerValue::Value(right.clone())),
				),
				_ => (None, None),
			};
			for instr in &mut body.contents {
				const_prop_instr(&mut instr.kind, an, run_again);
			}
			// Now we have to restore the previous value
			if let Some(previous_reg) = previous.0 {
				if let Some(previous_val) = previous.1 {
					an.vals.insert(previous_reg, previous_val);
				} else {
					an.vals.remove(&previous_reg);
				}
			}
		}
		MIRInstrKind::Assign {
			right: DeclareBinding::Condition(cond),
			..
		} => const_prop_condition(cond, an, run_again),
		MIRInstrKind::Modify { body, .. } | MIRInstrKind::ReturnRun { body } => {
			for instr in &mut body.contents {
				const_prop_instr(&mut instr.kind, an, run_again);
			}
		}
		_ => {}
	};
}

fn const_prop_condition(
	condition: &mut Condition,
	an: &mut StoringConstAnalyzer,
	run_again: &mut bool,
) {
	match condition {
		Condition::Equal(l, r)
		| Condition::GreaterThan(l, r)
		| Condition::GreaterThanOrEqual(l, r)
		| Condition::LessThan(l, r)
		| Condition::LessThanOrEqual(l, r) => {
			if let Value::Mutable(MutableValue::Reg(reg)) = l {
				if let Some(val) = an.vals.get(reg) {
					if let ConstAnalyzerValue::Value(val) = val {
						*l = Value::Constant(val.clone());
						*run_again = true;
					}
				}
			}
			if let Value::Mutable(MutableValue::Reg(reg)) = r {
				if let Some(val) = an.vals.get(reg) {
					if let ConstAnalyzerValue::Value(val) = val {
						*r = Value::Constant(val.clone());
						*run_again = true;
					}
				}
			}
		}
		Condition::Bool(b) | Condition::NotBool(b) => {
			if let Value::Mutable(MutableValue::Reg(reg)) = b {
				if let Some(val) = an.vals.get(reg) {
					if let ConstAnalyzerValue::Value(val) = val {
						*b = Value::Constant(val.clone());
						*run_again = true;
					}
				}
			}
		}
		Condition::Exists(val) => {
			if let Value::Mutable(MutableValue::Reg(reg)) = val {
				if let Some(val) = an.vals.get(reg) {
					match val {
						ConstAnalyzerValue::Reset(..) => {
							*condition = Condition::Bool(Value::Constant(DataTypeContents::Score(
								ScoreTypeContents::Bool(false),
							)));
						}
						ConstAnalyzerValue::Value(..) => {
							*condition = Condition::Bool(Value::Constant(DataTypeContents::Score(
								ScoreTypeContents::Bool(true),
							)));
						}
					}
					*run_again = true;
				}
			}
		}
		Condition::Not(cond) => const_prop_condition(cond, an, run_again),
		Condition::And(l, r) => {
			const_prop_condition(l, an, run_again);
			const_prop_condition(r, an, run_again);
		}
		_ => {}
	}
}
