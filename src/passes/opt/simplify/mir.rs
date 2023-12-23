use crate::common::condition::Condition;
use crate::common::ty::{DataTypeContents, NBTTypeContents, ScoreTypeContents};
use crate::common::val::MutableValue;
use crate::common::{val::Value, DeclareBinding};
use crate::mir::{MIRBlock, MIRInstrKind};
use crate::passes::{MIRPass, MIRPassData, Pass};
use crate::util::remove_indices;

use anyhow::anyhow;
use dashmap::DashSet;
use num_traits::Zero;

pub struct MIRSimplifyPass;

impl Pass for MIRSimplifyPass {
	fn get_name(&self) -> &'static str {
		"simplify_mir"
	}
}

impl MIRPass for MIRSimplifyPass {
	fn run_pass(&mut self, data: &mut MIRPassData) -> anyhow::Result<()> {
		for (_, block) in &mut data.mir.functions {
			let block = data
				.mir
				.blocks
				.get_mut(block)
				.ok_or(anyhow!("Block does not exist"))?;

			let mut instrs_to_remove = DashSet::new();
			loop {
				let run_again = run_mir_simplify_iter(block, &mut instrs_to_remove);
				if !run_again {
					break;
				}
			}
			remove_indices(&mut block.contents, &instrs_to_remove);
		}

		Ok(())
	}
}

/// Runs an iteration of the MIRSimplifyPass. Returns true if another iteration
/// should be run
fn run_mir_simplify_iter(block: &mut MIRBlock, instrs_to_remove: &mut DashSet<usize>) -> bool {
	let mut run_again = false;

	for (i, instr) in block.contents.iter_mut().enumerate() {
		let remove = match &instr.kind {
			// Reflexive property; set or swap with self
			// and also min and max with self
			MIRInstrKind::Assign {
				left,
				right: DeclareBinding::Value(Value::Mutable(right)),
			}
			| MIRInstrKind::Swap { left, right }
			| MIRInstrKind::Min {
				left,
				right: Value::Mutable(right),
			}
			| MIRInstrKind::Max {
				left,
				right: Value::Mutable(right),
			} if left.is_same_val(right) => true,
			// Multiplies and divides by 1
			MIRInstrKind::Mul {
				left: _,
				right: Value::Constant(DataTypeContents::Score(score)),
			}
			| MIRInstrKind::Div {
				left: _,
				right: Value::Constant(DataTypeContents::Score(score)),
			} if score.get_i32() == 1 => true,
			// x / 0 and x % 0 error and dont do anything
			MIRInstrKind::Div {
				left: _,
				right: Value::Constant(DataTypeContents::Score(score)),
			}
			| MIRInstrKind::Mod {
				left: _,
				right: Value::Constant(DataTypeContents::Score(score)),
			} if score.get_i32() == 0 => true,
			MIRInstrKind::AddTime { time } if time.amount.is_zero() => true,
			MIRInstrKind::AddXP { amount, .. } | MIRInstrKind::TriggerAdd { amount, .. }
				if amount.is_zero() =>
			{
				true
			}
			// Merge with empty compound doesn't do anything
			MIRInstrKind::Merge {
				right: Value::Constant(DataTypeContents::NBT(NBTTypeContents::Compound(_, comp))),
				..
			} if comp.is_empty() => true,
			_ => false,
		};

		if remove {
			let is_new = instrs_to_remove.insert(i);
			if is_new {
				run_again = true;
			}

			continue;
		}

		// Instructions to replace
		let kind_repl = match &instr.kind {
			// Div by -1 is same as mul by -1
			MIRInstrKind::Div {
				left,
				right: Value::Constant(DataTypeContents::Score(score)),
			} if score.get_i32() == -1 => Some(MIRInstrKind::Mul {
				left: left.clone(),
				right: Value::Constant(DataTypeContents::Score(ScoreTypeContents::Score(-1))),
			}),
			// x ^ 0 = 1
			MIRInstrKind::Pow { base, exp: 0 } => Some(MIRInstrKind::Assign {
				left: base.clone(),
				right: DeclareBinding::Value(Value::Constant(DataTypeContents::Score(
					ScoreTypeContents::Score(1),
				))),
			}),
			MIRInstrKind::Sub {
				left,
				right: Value::Mutable(right),
			} if left.is_same_val(right) => Some(MIRInstrKind::Assign {
				left: left.clone(),
				right: DeclareBinding::Value(Value::Constant(DataTypeContents::Score(
					ScoreTypeContents::Score(0),
				))),
			}),
			// x + x = x * 2
			MIRInstrKind::Add {
				left,
				right: Value::Mutable(right),
			} if left.is_same_val(right) => Some(MIRInstrKind::Mul {
				left: left.clone(),
				right: Value::Constant(DataTypeContents::Score(ScoreTypeContents::Score(2))),
			}),
			// x * x = x ^ 2
			MIRInstrKind::Mul {
				left,
				right: Value::Mutable(right),
			} if left.is_same_val(right) => Some(MIRInstrKind::Pow {
				base: left.clone(),
				exp: 2,
			}),
			// Merge with compound with single item = set that item
			MIRInstrKind::Merge {
				left,
				right: Value::Constant(DataTypeContents::NBT(NBTTypeContents::Compound(_, right))),
			} if right.0.len() == 1 => {
				let right = right.0.iter().nth(0).expect("Len should be 1");
				Some(MIRInstrKind::Assign {
					left: MutableValue::Property(Box::new(left.clone()), right.0.clone()),
					right: DeclareBinding::Value(Value::Constant(DataTypeContents::NBT(
						right.1.clone(),
					))),
				})
			}
			MIRInstrKind::If {
				condition: Condition::Equal(Value::Mutable(left1), right1),
				body,
			} => {
				if let MIRInstrKind::Assign {
					left: left2,
					right: DeclareBinding::Value(right2),
				} = body.as_ref()
				{
					if left1.is_same_val(left2) && right1.is_value_eq(right2) {
						Some(MIRInstrKind::Assign {
							left: left1.clone(),
							right: DeclareBinding::Value(right1.clone()),
						})
					} else {
						None
					}
				} else {
					None
				}
			}
			MIRInstrKind::If {
				condition: Condition::Not(condition),
				body,
			} => {
				if let Condition::Equal(Value::Mutable(left1), right1) = condition.as_ref() {
					if let MIRInstrKind::Assign {
						left: left2,
						right: DeclareBinding::Value(right2),
					} = body.as_ref()
					{
						if left1.is_same_val(left2) && right1.is_value_eq(right2) {
							Some(MIRInstrKind::Assign {
								left: left1.clone(),
								right: DeclareBinding::Value(right1.clone()),
							})
						} else {
							None
						}
					} else {
						None
					}
				} else {
					None
				}
			}
			MIRInstrKind::If {
				condition:
					Condition::GreaterThan(Value::Mutable(left1), right1)
					| Condition::GreaterThanOrEqual(Value::Mutable(left1), right1),
				body,
			} => {
				if let MIRInstrKind::Assign {
					left: left2,
					right: DeclareBinding::Value(right2),
				} = body.as_ref()
				{
					if left1.is_same_val(left2) && right1.is_value_eq(right2) {
						Some(MIRInstrKind::Min {
							left: left1.clone(),
							right: right1.clone(),
						})
					} else {
						None
					}
				} else {
					None
				}
			}
			MIRInstrKind::If {
				condition:
					Condition::LessThan(Value::Mutable(left1), right1)
					| Condition::LessThanOrEqual(Value::Mutable(left1), right1),
				body,
			} => {
				if let MIRInstrKind::Assign {
					left: left2,
					right: DeclareBinding::Value(right2),
				} = body.as_ref()
				{
					if left1.is_same_val(left2) && right1.is_value_eq(right2) {
						Some(MIRInstrKind::Max {
							left: left1.clone(),
							right: right1.clone(),
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

		if let Some(kind_repl) = kind_repl {
			instr.kind = kind_repl;
			run_again = true;
		}
	}

	run_again
}
