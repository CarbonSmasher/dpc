use crate::common::condition::Condition;
use crate::common::mc::instr::MinecraftInstr;
use crate::common::mc::modifier::StoreModLocation;
use crate::common::ty::{DataTypeContents, NBTTypeContents, ScoreTypeContents};
use crate::common::val::MutableValue;
use crate::common::{val::Value, DeclareBinding};
use crate::mir::{MIRBlock, MIRInstrKind};
use crate::passes::{MIRPass, MIRPassData, Pass};
use crate::util::{remove_indices, HashSetEmptyTracker, Only};

use num_traits::Zero;

pub struct MIRSimplifyPass;

impl Pass for MIRSimplifyPass {
	fn get_name(&self) -> &'static str {
		"simplify_mir"
	}
}

impl MIRPass for MIRSimplifyPass {
	fn run_pass(&mut self, data: &mut MIRPassData) -> anyhow::Result<()> {
		for func in data.mir.functions.values_mut() {
			let block = &mut func.block;

			let mut instrs_to_remove = HashSetEmptyTracker::new();
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
fn run_mir_simplify_iter(
	block: &mut MIRBlock,
	instrs_to_remove: &mut HashSetEmptyTracker<usize>,
) -> bool {
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
			// Adds and subs by 0
			MIRInstrKind::Add {
				right: Value::Constant(DataTypeContents::Score(val)),
				..
			}
			| MIRInstrKind::Sub {
				right: Value::Constant(DataTypeContents::Score(val)),
				..
			} if val.get_i32() == 0 => true,
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
			// x || 0 can be removed
			MIRInstrKind::Or {
				left: _,
				right: Value::Constant(DataTypeContents::Score(score)),
			} if score.get_i32() == 0 => true,
			// x && 1 can be removed
			MIRInstrKind::And {
				left: _,
				right: Value::Constant(DataTypeContents::Score(score)),
			} if score.get_i32() == 1 => true,
			// And / or with self is identity
			MIRInstrKind::And {
				left,
				right: Value::Mutable(right),
			}
			| MIRInstrKind::Or {
				left,
				right: Value::Mutable(right),
			} if left.is_same_val(right) => true,
			// Different Minecraft add instructions with zero as the amount can be removed
			MIRInstrKind::MC(MinecraftInstr::AddTime { time }) if time.amount.is_zero() => true,
			MIRInstrKind::MC(
				MinecraftInstr::AddXP { amount, .. } | MinecraftInstr::TriggerAdd { amount, .. },
			) if amount.is_zero() => true,
			MIRInstrKind::MC(MinecraftInstr::WorldBorderAdd { dist, .. }) if *dist == 0.0 => true,
			// Merge with empty compound doesn't do anything
			MIRInstrKind::Merge {
				right: Value::Constant(DataTypeContents::NBT(NBTTypeContents::Compound(_, comp))),
				..
			} if comp.is_empty() => true,
			// Get instructions without their results stored don't do anything
			MIRInstrKind::GetConst { .. }
			| MIRInstrKind::Get { .. }
			| MIRInstrKind::MC(
				MinecraftInstr::GetAttribute { .. }
				| MinecraftInstr::GetAttributeBase { .. }
				| MinecraftInstr::GetAttributeModifier { .. }
				| MinecraftInstr::GetDifficulty
				| MinecraftInstr::GetGamerule { .. }
				| MinecraftInstr::GetTime { .. }
				| MinecraftInstr::GetXP { .. },
			) => true,
			// Empty block inside of an if can be removed
			MIRInstrKind::If { body, .. } => body.contents.is_empty(),
			MIRInstrKind::NoOp => true,
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
			// x = cond bool y is same as x = y
			MIRInstrKind::Assign {
				left,
				right: DeclareBinding::Condition(Condition::Bool(right)),
			} => Some(MIRInstrKind::Assign {
				left: left.clone(),
				right: DeclareBinding::Value(right.clone()),
			}),
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
				let right = right.0.iter().next().expect("Len should be 1");
				Some(MIRInstrKind::Assign {
					left: MutableValue::Property(Box::new(left.clone()), right.0.clone()),
					right: DeclareBinding::Value(Value::Constant(DataTypeContents::NBT(
						right.1.clone(),
					))),
				})
			}
			// x && 0 is always false
			MIRInstrKind::And {
				left,
				right: Value::Constant(DataTypeContents::Score(score)),
			} if score.get_i32() == 0 => Some(MIRInstrKind::Assign {
				left: left.clone(),
				right: DeclareBinding::Value(Value::Constant(DataTypeContents::Score(
					ScoreTypeContents::Bool(false),
				))),
			}),
			// x || 1 is always true
			MIRInstrKind::Or {
				left,
				right: Value::Constant(DataTypeContents::Score(score)),
			} if score.get_i32() == 1 => Some(MIRInstrKind::Assign {
				left: left.clone(),
				right: DeclareBinding::Value(Value::Constant(DataTypeContents::Score(
					ScoreTypeContents::Bool(true),
				))),
			}),
			MIRInstrKind::If {
				condition: Condition::Equal(Value::Mutable(left1), right1),
				body,
			} => match body.contents.only().map(|x| &x.kind) {
				Some(MIRInstrKind::Assign {
					left: left2,
					right: DeclareBinding::Value(right2),
				}) if left1.is_same_val(left2) && right1.is_value_eq(right2) => Some(MIRInstrKind::Assign {
					left: left1.clone(),
					right: DeclareBinding::Value(right1.clone()),
				}),
				_ => None,
			},
			// Not conditions
			MIRInstrKind::If {
				condition: Condition::Not(condition),
				body,
			} => match condition.as_ref() {
				// if x != y: x = y -> x = y
				Condition::Equal(Value::Mutable(left1), right1) => {
					match body.contents.only().map(|x| &x.kind) {
						Some(MIRInstrKind::Assign {
							left: left2,
							right: DeclareBinding::Value(right2),
						}) if left1.is_same_val(left2) && right1.is_value_eq(right2) => Some(MIRInstrKind::Assign {
							left: left1.clone(),
							right: DeclareBinding::Value(right1.clone()),
						}),
						_ => None,
					}
				}
				// if x != y: x = y -> x = y
				Condition::Bool(b) => match body.contents.only().map(|x| &x.kind) {
					Some(
						MIRInstrKind::Mul {
							left,
							right: Value::Constant(DataTypeContents::Score(val)),
						}
						| MIRInstrKind::Assign {
							left,
							right:
								DeclareBinding::Value(Value::Constant(DataTypeContents::Score(val))),
						},
					) if val.get_i32() == 0 => Some(MIRInstrKind::Mul {
						left: left.clone(),
						right: b.clone(),
					}),
					_ => None,
				},
				_ => None,
			},
			MIRInstrKind::If {
				condition:
					Condition::GreaterThan(Value::Mutable(left1), right1)
					| Condition::GreaterThanOrEqual(Value::Mutable(left1), right1),
				body,
			} => {
				match body.contents.only().map(|x| &x.kind) {
					// if x > y: x = y -> min x, y;
					Some(MIRInstrKind::Assign {
						left: left2,
						right: DeclareBinding::Value(right2),
					}) if left1.is_same_val(left2) && right1.is_value_eq(right2) => Some(MIRInstrKind::Min {
						left: left1.clone(),
						right: right1.clone(),
					}),
					_ => None,
				}
			}
			MIRInstrKind::If {
				condition:
					Condition::LessThan(Value::Mutable(left1), right1)
					| Condition::LessThanOrEqual(Value::Mutable(left1), right1),
				body,
			} => {
				match body.contents.only().map(|x| &x.kind) {
					// if x < y: x = y -> max x, y;
					Some(MIRInstrKind::Assign {
						left: left2,
						right: DeclareBinding::Value(right2),
					}) if left1.is_same_val(left2) && right1.is_value_eq(right2) => Some(MIRInstrKind::Max {
						left: left1.clone(),
						right: right1.clone(),
					}),
					_ => None,
				}
			}
			MIRInstrKind::If {
				condition: Condition::Bool(b),
				body,
			} => match body.contents.only().map(|x| &x.kind) {
				// if x: y += 1 -> y += x
				Some(MIRInstrKind::Add {
					left,
					right: Value::Constant(DataTypeContents::Score(val)),
				}) if val.get_i32() == 1 => Some(MIRInstrKind::Add {
					left: left.clone(),
					right: b.clone(),
				}),
				// if x: y -= 1 -> y -= x
				Some(MIRInstrKind::Sub {
					left,
					right: Value::Constant(DataTypeContents::Score(val)),
				}) if val.get_i32() == 1 => Some(MIRInstrKind::Sub {
					left: left.clone(),
					right: b.clone(),
				}),
				_ => None,
			},
			MIRInstrKind::StoreResult {
				location: StoreModLocation::Reg(left, left_scale),
				body,
			} => {
				match body.contents.only().map(|x| &x.kind) {
					// str x: get y -> x = y (essentially)
					Some(MIRInstrKind::Get {
						value: right,
						scale: right_scale,
					}) => {
						if left_scale * right_scale == 1.0 {
							Some(MIRInstrKind::Assign {
								left: MutableValue::Reg(left.clone()),
								right: DeclareBinding::Value(Value::Mutable(right.clone())),
							})
						} else {
							None
						}
					}
					_ => None,
				}
			}
			MIRInstrKind::StoreSuccess {
				location: StoreModLocation::Reg(left, left_scale),
				body,
			} if left_scale == &1.0 => {
				match body.contents.only().map(|x| &x.kind) {
					// Canonicalize to let cond
					Some(MIRInstrKind::If { condition, body }) => {
						match body.contents.only().map(|x| &x.kind) {
							Some(MIRInstrKind::NoOp) | None => Some(MIRInstrKind::Assign {
								left: MutableValue::Reg(left.clone()),
								right: DeclareBinding::Condition(condition.clone()),
							}),
							_ => None,
						}
					}
					_ => None,
				}
			}
			_ => None,
		};

		if let Some(kind_repl) = kind_repl {
			instr.kind = kind_repl;
			run_again = true;
		}

		if let MIRInstrKind::If { condition, .. }
		| MIRInstrKind::Assign {
			right: DeclareBinding::Condition(condition),
			..
		} = &mut instr.kind
		{
			simplify_condition(condition, &mut run_again);
		}
	}

	run_again
}

fn simplify_condition(condition: &mut Condition, run_again: &mut bool) {
	match condition {
		Condition::Not(inner) => match inner.as_ref() {
			// not bool -> nbool for better lowering
			Condition::Bool(b) => *condition = Condition::NotBool(b.clone()),
			_ => simplify_condition(inner, run_again),
		},
		_ => {}
	}
}
