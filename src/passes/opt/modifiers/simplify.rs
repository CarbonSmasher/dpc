use anyhow::anyhow;

use crate::common::mc::modifier::{IfModCondition, IfScoreCondition, IfScoreRangeEnd, Modifier};
use crate::common::val::ScoreValue;
use crate::lir::{LIRInstrKind, LIR};
use crate::passes::{LIRPass, Pass};
use crate::util::{remove_indices, DashSetEmptyTracker};

pub struct SimplifyModifiersPass;

impl Pass for SimplifyModifiersPass {
	fn get_name(&self) -> &'static str {
		"simplify_modifiers"
	}
}

impl LIRPass for SimplifyModifiersPass {
	fn run_pass(&mut self, lir: &mut LIR) -> anyhow::Result<()> {
		let mut mods_to_remove = DashSetEmptyTracker::new();

		for func in lir.functions.values_mut() {
			let block = lir
				.blocks
				.get_mut(&func.block)
				.ok_or(anyhow!("Block does not exist"))?;

			for instr in &mut block.contents {
				if !mods_to_remove.is_empty() {
					mods_to_remove.clear();
				}
				let modifier_count = instr.modifiers.len();
				for (i, modifier) in instr.modifiers.iter_mut().enumerate() {
					if mods_to_remove.contains(&i) {
						continue;
					}

					match modifier {
						Modifier::Positioned(coords) if coords.are_zero() => {
							mods_to_remove.insert(i);
						}
						Modifier::As(target) if target.is_blank_this() => {
							mods_to_remove.insert(i);
						}
						Modifier::If { condition, negate } => {
							let result = optimize_condition(condition);
							match result {
								OptimizeConditionResult::Invert => {
									*negate = !*negate;
								}
								OptimizeConditionResult::Guaranteed => {
									if *negate {
										// Remove this if and all modifiers after since they cannot be reached
										mods_to_remove.extend(i..modifier_count);
										instr.kind = LIRInstrKind::NoOp;
									} else {
										mods_to_remove.insert(i);
									}
								}
								OptimizeConditionResult::Impossible => {
									if *negate {
										mods_to_remove.insert(i);
									} else {
										// Remove this if and all modifiers after since they cannot be reached
										mods_to_remove.extend(i..modifier_count);
										instr.kind = LIRInstrKind::NoOp;
									}
								}
								OptimizeConditionResult::Nothing => {}
							}
						}
						_ => {}
					}
				}

				remove_indices(&mut instr.modifiers, &mods_to_remove);
			}
		}

		Ok(())
	}
}

fn optimize_condition(condition: &mut Box<IfModCondition>) -> OptimizeConditionResult {
	match condition.as_mut() {
		// Reflexive property and other cases where the same value
		// appears on both sides of the condition
		IfModCondition::Score(
			IfScoreCondition::Single { left, right }
			| IfScoreCondition::Range {
				score: left,
				left: IfScoreRangeEnd::Fixed {
					value: right,
					inclusive: true,
				},
				right: _,
			}
			| IfScoreCondition::Range {
				score: left,
				left: _,
				right: IfScoreRangeEnd::Fixed {
					value: right,
					inclusive: true,
				},
			},
		) if left.is_value_eq(right) => {
			return OptimizeConditionResult::Guaranteed;
		}
		// These are always false since they aren't inclusive
		IfModCondition::Score(
			IfScoreCondition::Range {
				score: left,
				left: IfScoreRangeEnd::Fixed {
					value: right,
					inclusive: false,
				},
				right: _,
			}
			| IfScoreCondition::Range {
				score: left,
				left: _,
				right: IfScoreRangeEnd::Fixed {
					value: right,
					inclusive: false,
				},
			},
		) if left.is_value_eq(right) => {
			return OptimizeConditionResult::Impossible;
		}
		// Replace ranges that are only one number long with singles
		// or mark them as impossible if they don't match anything
		IfModCondition::Score(IfScoreCondition::Range {
			score,
			left:
				IfScoreRangeEnd::Fixed {
					value: ScoreValue::Constant(left_val),
					inclusive: left_inc,
				},
			right:
				IfScoreRangeEnd::Fixed {
					value: ScoreValue::Constant(right_val),
					inclusive: right_inc,
				},
		}) if left_val.is_value_eq(right_val) => {
			// If both sides are exclusive then this matches nothing and the condition is impossible
			if !*left_inc && !*right_inc {
				return OptimizeConditionResult::Impossible;
			} else {
				*condition = Box::new(IfModCondition::Score(IfScoreCondition::Single {
					left: score.clone(),
					right: ScoreValue::Constant(left_val.clone()),
				}));
			}
		}
		// Replace constant ranges that would be smaller if inverted
		IfModCondition::Score(IfScoreCondition::Range {
			score,
			left:
				IfScoreRangeEnd::Fixed {
					value: ScoreValue::Constant(value),
					inclusive,
				},
			right: IfScoreRangeEnd::Infinite,
		}) if check_constant_range_size(value.get_i32(), i32::MAX) => {
			*condition = Box::new(IfModCondition::Score(IfScoreCondition::Range {
				score: score.clone(),
				left: IfScoreRangeEnd::Infinite,
				right: IfScoreRangeEnd::Fixed {
					value: ScoreValue::Constant(value.clone()),
					inclusive: !*inclusive,
				},
			}));
			return OptimizeConditionResult::Invert;
		}
		IfModCondition::Score(IfScoreCondition::Range {
			score,
			left: IfScoreRangeEnd::Infinite,
			right:
				IfScoreRangeEnd::Fixed {
					value: ScoreValue::Constant(value),
					inclusive,
				},
		}) if check_constant_range_size(value.get_i32(), i32::MAX) => {
			*condition = Box::new(IfModCondition::Score(IfScoreCondition::Range {
				score: score.clone(),
				left: IfScoreRangeEnd::Fixed {
					value: ScoreValue::Constant(value.clone()),
					inclusive: !*inclusive,
				},
				right: IfScoreRangeEnd::Infinite,
			}));
			return OptimizeConditionResult::Invert;
		}
		// Reorder ranges and singles with constants on the left
		// and mutables on the right since they generate less optimal code
		IfModCondition::Score(IfScoreCondition::Single {
			left: left @ ScoreValue::Constant(..),
			right: right @ ScoreValue::Mutable(..),
		}) => {
			std::mem::swap(left, right);
		}
		// A const fold
		IfModCondition::Score(IfScoreCondition::Single {
			left: ScoreValue::Constant(left),
			right: ScoreValue::Constant(right),
		}) => {
			if left.get_i32() == right.get_i32() {
				return OptimizeConditionResult::Guaranteed;
			} else {
				return OptimizeConditionResult::Impossible;
			}
		}
		_ => {}
	}

	OptimizeConditionResult::Nothing
}

enum OptimizeConditionResult {
	/// Do nothing
	Nothing,
	/// Invert the negation of the conditon
	Invert,
	/// This condition is always true, remove the if modifier
	Guaranteed,
	/// This condition is always false. Remove the if modifier and all modifiers after it
	Impossible,
}

/// Checks if a constant range's size is greater than the integer limit
fn check_constant_range_size(left: i32, right: i32) -> bool {
	let range = (left as i64 - right as i64).abs();
	range > i32::MAX as i64
}
