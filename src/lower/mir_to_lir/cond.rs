use anyhow::{bail, Context};

use super::{lower_add, lower_subblock_impl, LowerBlockCx};
use crate::common::condition::Condition;
use crate::common::cost::GetCost;
use crate::common::mc::modifier::{
	IfModCondition, IfScoreCondition, IfScoreRangeEnd, Modifier, StoreModLocation,
};
use crate::common::ty::{DataType, DataTypeContents, ScoreType, ScoreTypeContents};
use crate::common::val::{MutableScoreValue, MutableValue, NBTValue, ScoreValue, Value};
use crate::common::Register;
use crate::lir::{LIRInstrKind, LIRInstruction};

/// Returns a list of instructions to add before where the
/// condition is used, the conditions to use for one or more if
/// modifiers, and whether to negate each of those modifiers
pub(super) fn lower_condition(
	condition: Condition,
	lbcx: &mut LowerBlockCx,
) -> anyhow::Result<(Vec<LIRInstruction>, Vec<(IfModCondition, bool)>)> {
	let mut prelude = Vec::new();
	let mut out = Vec::new();
	// Make this account for ScoreValue changes
	match condition {
		Condition::Equal(l, r) => {
			let lty = l.get_ty(&lbcx.registers, &lbcx.sig)?;
			let rty = r.get_ty(&lbcx.registers, &lbcx.sig)?;
			let cond = match (lty, rty) {
				(DataType::Score(..), DataType::Score(..)) => {
					IfModCondition::Score(IfScoreCondition::Single {
						left: l.to_score_value()?,
						right: r.to_score_value()?,
					})
				}
				_ => bail!("Condition does not allow these types"),
			};
			out.push((cond, false));
		}
		Condition::Exists(val) => {
			let cond = match val.get_ty(&lbcx.registers, &lbcx.sig)? {
				DataType::Score(..) => match val.to_score_value()? {
					ScoreValue::Constant(..) => IfModCondition::Const(true),
					ScoreValue::Mutable(val) => IfModCondition::Score(IfScoreCondition::Range {
						score: ScoreValue::Mutable(val),
						left: IfScoreRangeEnd::Infinite,
						right: IfScoreRangeEnd::Infinite,
					}),
				},
				DataType::NBT(..) => match val.to_nbt_value()? {
					NBTValue::Constant(..) => IfModCondition::Const(true),
					NBTValue::Mutable(val) => IfModCondition::DataExists(val),
				},
				_ => bail!("Type not supported"),
			};
			out.push((cond, false));
		}
		Condition::Not(condition) => {
			let (other_prelude, condition) =
				lower_condition(*condition, lbcx).context("Failed to lower not condition")?;
			// For len != 1, we have to lower to the not of all the terms,
			// based on DeMorgan's theorems
			prelude.extend(other_prelude);
			if condition.len() != 1 {
				// Invert each
				let terms = condition
					.into_iter()
					.map(|mut x| {
						x.1 = !x.1;
						vec![x]
					})
					.collect();
				let cond = lower_or(terms, &mut prelude, lbcx)?;
				out.push((cond, false));
			} else {
				let (condition, negate) = condition.first().expect("Length is 1");
				out.push((condition.clone(), !negate));
			}
		}
		Condition::And(l, r) => {
			let (lp, lc) = lower_condition(*l, lbcx).context("Failed to lower and lhs")?;
			let (rp, rc) = lower_condition(*r, lbcx).context("Failed to lower and rhs")?;
			prelude.extend(lp);
			prelude.extend(rp);
			out.extend(lc);
			out.extend(rc);
		}
		Condition::Or(l, r) => {
			let (lp, lc) = lower_condition(*l, lbcx).context("Failed to lower and lhs")?;
			let (rp, rc) = lower_condition(*r, lbcx).context("Failed to lower and rhs")?;
			prelude.extend(lp);
			prelude.extend(rp);
			let cond = lower_or(vec![lc, rc], &mut prelude, lbcx)?;
			out.push((cond, false));
		}
		Condition::GreaterThan(l, r) => {
			out.push((
				IfModCondition::Score(IfScoreCondition::Range {
					score: l.to_score_value()?,
					left: IfScoreRangeEnd::Fixed {
						value: r.to_score_value()?,
						inclusive: false,
					},
					right: IfScoreRangeEnd::Infinite,
				}),
				false,
			));
		}
		Condition::GreaterThanOrEqual(l, r) => {
			out.push((
				IfModCondition::Score(IfScoreCondition::Range {
					score: l.to_score_value()?,
					left: IfScoreRangeEnd::Fixed {
						value: r.to_score_value()?,
						inclusive: true,
					},
					right: IfScoreRangeEnd::Infinite,
				}),
				false,
			));
		}
		Condition::LessThan(l, r) => {
			out.push((
				IfModCondition::Score(IfScoreCondition::Range {
					score: l.to_score_value()?,
					left: IfScoreRangeEnd::Infinite,
					right: IfScoreRangeEnd::Fixed {
						value: r.to_score_value()?,
						inclusive: false,
					},
				}),
				false,
			));
		}
		Condition::LessThanOrEqual(l, r) => {
			out.push((
				IfModCondition::Score(IfScoreCondition::Range {
					score: l.to_score_value()?,
					left: IfScoreRangeEnd::Infinite,
					right: IfScoreRangeEnd::Fixed {
						value: r.to_score_value()?,
						inclusive: true,
					},
				}),
				false,
			));
		}
		Condition::Bool(val) => {
			out.push((lower_bool_cond(val, true, lbcx)?, false));
		}
		Condition::NotBool(val) => {
			out.push((lower_bool_cond(val, false, lbcx)?, false));
		}
		Condition::Entity(ent) => {
			out.push((IfModCondition::Entity(ent), false));
		}
		Condition::Predicate(pred) => {
			out.push((IfModCondition::Predicate(pred), false));
		}
		Condition::Biome(loc, biome) => {
			out.push((IfModCondition::Biome(loc, biome), false));
		}
		Condition::Loaded(loc) => {
			out.push((IfModCondition::Loaded(loc), false));
		}
		Condition::Dimension(dim) => {
			out.push((IfModCondition::Dimension(dim), false));
		}
	};

	Ok((prelude, out))
}

pub(super) fn lower_bool_cond(
	val: Value,
	check: bool,
	lbcx: &LowerBlockCx,
) -> anyhow::Result<IfModCondition> {
	let ty = val.get_ty(&lbcx.registers, &lbcx.sig)?;
	match ty {
		DataType::Score(..) => Ok(IfModCondition::Score(IfScoreCondition::Single {
			left: val.to_score_value()?,
			right: ScoreValue::Constant(ScoreTypeContents::Bool(check)),
		})),
		_ => bail!("Condition does not allow this type"),
	}
}

pub(super) fn lower_or(
	terms: Vec<Vec<(IfModCondition, bool)>>,
	prelude: &mut Vec<LIRInstruction>,
	lbcx: &mut LowerBlockCx,
) -> anyhow::Result<IfModCondition> {
	if terms.len() < 2 {
		bail!("Missing terms for or condition");
	}

	// Choose the best method based on some heuristics
	const MAX_COST_FOR_INLINE: f32 = 40.0;
	let use_if_func = terms
		.iter()
		.flatten()
		.fold(0.0, |acc, x| acc + x.0.get_cost())
		>= MAX_COST_FOR_INLINE;

	if use_if_func {
		lower_or_if_function(terms, lbcx)
	} else {
		lower_or_inline_scores(terms, prelude, lbcx)
	}
}

/// Lower an or using the inline operation method.
/// This will do a saturating or of all the conditions using the scoreboard
fn lower_or_inline_scores(
	mut terms: Vec<Vec<(IfModCondition, bool)>>,
	prelude: &mut Vec<LIRInstruction>,
	lbcx: &mut LowerBlockCx,
) -> anyhow::Result<IfModCondition> {
	let or_reg = lbcx.new_additional_reg();
	lbcx.registers.insert(
		or_reg.clone(),
		Register {
			id: or_reg.clone(),
			ty: DataType::Score(ScoreType::Bool),
		},
	);
	let first_cond = terms.pop().expect("Len >= 2");
	lower_let_cond_impl(MutableValue::Reg(or_reg.clone()), first_cond, prelude, lbcx)?;
	let add_instr = lower_add(
		MutableValue::Reg(or_reg.clone()),
		Value::Constant(DataTypeContents::Score(ScoreTypeContents::Score(1))),
		lbcx,
	)?;
	for term in terms {
		let mut instr = LIRInstruction::new(add_instr.clone());
		for cond in term {
			instr.modifiers.push(Modifier::If {
				condition: Box::new(cond.0),
				negate: cond.1,
			});
		}
		prelude.push(instr);
	}

	let out = IfModCondition::Score(IfScoreCondition::Range {
		score: ScoreValue::Mutable(MutableScoreValue::Reg(or_reg)),
		left: IfScoreRangeEnd::Fixed {
			value: ScoreValue::Constant(ScoreTypeContents::Score(1)),
			inclusive: true,
		},
		right: IfScoreRangeEnd::Infinite,
	});

	Ok(out)
}

/// Lower an or using an if function call. This method will
/// short-circuit if any condition is false, which makes it better
/// for more expensive conditions
fn lower_or_if_function(
	terms: Vec<Vec<(IfModCondition, bool)>>,
	lbcx: &mut LowerBlockCx,
) -> anyhow::Result<IfModCondition> {
	let mut func_instrs = Vec::new();
	let return_instr = LIRInstrKind::ReturnValue(1);
	for term in terms {
		let mut instr = LIRInstruction::new(return_instr.clone());
		for cond in term {
			instr.modifiers.push(Modifier::If {
				condition: Box::new(cond.0),
				negate: cond.1,
			});
		}
		func_instrs.push(instr);
	}
	let if_function = lower_subblock_impl(func_instrs, lbcx)?;
	Ok(IfModCondition::Function(if_function))
}

pub(super) fn lower_let_cond(
	left: MutableValue,
	condition: &Condition,
	lir_instrs: &mut Vec<LIRInstruction>,
	lbcx: &mut LowerBlockCx,
) -> anyhow::Result<()> {
	let (prelude, conditions) = lower_condition(condition.clone(), lbcx)?;
	lir_instrs.extend(prelude);
	lower_let_cond_impl(left, conditions, lir_instrs, lbcx)?;

	Ok(())
}

pub(super) fn lower_let_cond_impl(
	left: MutableValue,
	conditions: Vec<(IfModCondition, bool)>,
	lir_instrs: &mut Vec<LIRInstruction>,
	lbcx: &mut LowerBlockCx,
) -> anyhow::Result<()> {
	let store_loc = match left.get_ty(&lbcx.registers, &lbcx.sig)? {
		DataType::Score(..) => {
			StoreModLocation::from_mut_score_val(&left.to_mutable_score_value()?)?
		}
		_ => bail!("Invalid type"),
	};
	let mut instr = LIRInstruction::new(LIRInstrKind::NoOp);
	instr.modifiers.push(Modifier::StoreSuccess(store_loc));
	for (condition, negate) in conditions {
		instr.modifiers.push(Modifier::If {
			condition: Box::new(condition),
			negate,
		});
	}
	lir_instrs.push(instr);

	Ok(())
}
