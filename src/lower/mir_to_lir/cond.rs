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
use crate::util::Only;

/// Returns a list of instructions to add before where the
/// condition is used, the conditions to use for one or more if
/// modifiers, and whether to negate each of those modifiers
pub(super) fn lower_condition(
	condition: Condition,
	lbcx: &mut LowerBlockCx,
) -> anyhow::Result<(Vec<LIRInstruction>, Vec<LoweringCondition>)> {
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
			out.push(LoweringCondition::new(cond));
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
			out.push(LoweringCondition::new(cond));
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
						x.negate = !x.negate;
						vec![x]
					})
					.collect();
				let cond = lower_or(terms, &mut prelude, lbcx)?;
				out.push(cond);
			} else {
				let condition = condition.first().expect("Length is 1");
				out.push(condition.clone().negate());
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
			out.push(cond);
		}
		Condition::Xor(l, r) => {
			let (lp, lc) = lower_condition(*l, lbcx).context("Failed to lower and lhs")?;
			let (rp, rc) = lower_condition(*r, lbcx).context("Failed to lower and rhs")?;
			prelude.extend(lp);
			prelude.extend(rp);
			let cond = lower_xor(lc, rc, &mut prelude, lbcx)?;
			out.push(cond);
		}
		Condition::GreaterThan(l, r) => {
			out.push(LoweringCondition::new(IfModCondition::Score(
				IfScoreCondition::Range {
					score: l.to_score_value()?,
					left: IfScoreRangeEnd::Fixed {
						value: r.to_score_value()?,
						inclusive: false,
					},
					right: IfScoreRangeEnd::Infinite,
				},
			)));
		}
		Condition::GreaterThanOrEqual(l, r) => {
			out.push(LoweringCondition::new(IfModCondition::Score(
				IfScoreCondition::Range {
					score: l.to_score_value()?,
					left: IfScoreRangeEnd::Fixed {
						value: r.to_score_value()?,
						inclusive: true,
					},
					right: IfScoreRangeEnd::Infinite,
				},
			)));
		}
		Condition::LessThan(l, r) => {
			out.push(LoweringCondition::new(IfModCondition::Score(
				IfScoreCondition::Range {
					score: l.to_score_value()?,
					left: IfScoreRangeEnd::Infinite,
					right: IfScoreRangeEnd::Fixed {
						value: r.to_score_value()?,
						inclusive: false,
					},
				},
			)));
		}
		Condition::LessThanOrEqual(l, r) => {
			out.push(LoweringCondition::new(IfModCondition::Score(
				IfScoreCondition::Range {
					score: l.to_score_value()?,
					left: IfScoreRangeEnd::Infinite,
					right: IfScoreRangeEnd::Fixed {
						value: r.to_score_value()?,
						inclusive: true,
					},
				},
			)));
		}
		Condition::Bool(val) => {
			out.push(lower_bool_cond(val, true, lbcx)?);
		}
		Condition::NotBool(val) => {
			out.push(lower_bool_cond(val, false, lbcx)?);
		}
		Condition::Entity(ent) => {
			out.push(LoweringCondition::new(IfModCondition::Entity(ent)));
		}
		Condition::Predicate(pred) => {
			out.push(LoweringCondition::new(IfModCondition::Predicate(pred)));
		}
		Condition::Biome(loc, biome) => {
			out.push(LoweringCondition::new(IfModCondition::Biome(loc, biome)));
		}
		Condition::Loaded(loc) => {
			out.push(LoweringCondition::new(IfModCondition::Loaded(loc)));
		}
		Condition::Dimension(dim) => {
			out.push(LoweringCondition::new(IfModCondition::Dimension(dim)));
		}
		Condition::Function(func) => {
			out.push(LoweringCondition::new(IfModCondition::Function(
				func,
				Vec::new(),
			)));
		}
	};

	Ok((prelude, out))
}

pub(super) fn lower_bool_cond(
	val: Value,
	check: bool,
	lbcx: &LowerBlockCx,
) -> anyhow::Result<LoweringCondition> {
	let ty = val.get_ty(&lbcx.registers, &lbcx.sig)?;
	match ty {
		DataType::Score(..) => Ok(LoweringCondition {
			condition: IfModCondition::Score(IfScoreCondition::Single {
				left: val.to_score_value()?,
				right: ScoreValue::Constant(ScoreTypeContents::Bool(check)),
			}),
			negate: false,
			is_bool_cond: true,
		}),
		_ => bail!("Condition does not allow this type"),
	}
}

pub(super) fn lower_or(
	terms: Vec<Vec<LoweringCondition>>,
	prelude: &mut Vec<LIRInstruction>,
	lbcx: &mut LowerBlockCx,
) -> anyhow::Result<LoweringCondition> {
	if terms.len() < 2 {
		bail!("Missing terms for or condition");
	}

	// Choose the best method based on some heuristics
	const MAX_COST_FOR_INLINE: f32 = 40.0;
	let use_if_func = terms
		.iter()
		.flatten()
		.fold(0.0, |acc, x| acc + x.condition.get_cost())
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
	mut terms: Vec<Vec<LoweringCondition>>,
	prelude: &mut Vec<LIRInstruction>,
	lbcx: &mut LowerBlockCx,
) -> anyhow::Result<LoweringCondition> {
	let or_reg = lbcx.new_additional_reg();
	lbcx.registers.insert(
		or_reg.clone(),
		Register {
			id: or_reg.clone(),
			ty: DataType::Score(ScoreType::Bool),
		},
	);
	let first_cond = terms.pop().expect("Len >= 2");
	lower_let_cond_impl(
		MutableValue::Reg(or_reg.clone()),
		first_cond,
		prelude,
		true,
		lbcx,
	)?;
	for term in terms {
		let instr = lower_single_saturating_or(&MutableValue::Reg(or_reg.clone()), term, lbcx)?;
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

	// This is a bool cond even though it is saturating
	Ok(LoweringCondition {
		condition: out,
		negate: false,
		is_bool_cond: true,
	})
}

/// Lower an or using an if function call. This method will
/// short-circuit if any condition is false, which makes it better
/// for more expensive conditions
fn lower_or_if_function(
	terms: Vec<Vec<LoweringCondition>>,
	lbcx: &mut LowerBlockCx,
) -> anyhow::Result<LoweringCondition> {
	let mut func_instrs = Vec::new();
	let return_instr = LIRInstrKind::ReturnValue(1);
	for term in terms {
		let mut instr = LIRInstruction::new(return_instr.clone());
		for cond in term {
			instr.modifiers.push(cond.to_if_mod());
		}
		func_instrs.push(instr);
	}
	let (if_function, regs) = lower_subblock_impl(func_instrs, lbcx)?;
	Ok(LoweringCondition::new(IfModCondition::Function(
		if_function,
		regs,
	)))
}

pub(super) fn lower_xor(
	left: Vec<LoweringCondition>,
	right: Vec<LoweringCondition>,
	prelude: &mut Vec<LIRInstruction>,
	lbcx: &mut LowerBlockCx,
) -> anyhow::Result<LoweringCondition> {
	let xor_reg = lbcx.new_additional_reg();
	lbcx.registers.insert(
		xor_reg.clone(),
		Register {
			id: xor_reg.clone(),
			ty: DataType::Score(ScoreType::Bool),
		},
	);
	lower_let_cond_impl(
		MutableValue::Reg(xor_reg.clone()),
		left,
		prelude,
		true,
		lbcx,
	)?;

	if let Some(LoweringCondition {
		condition:
			IfModCondition::Score(
				IfScoreCondition::Single {
					left: left2,
					right: ScoreValue::Constant(val),
				}
				| IfScoreCondition::Range {
					score: left2,
					left:
						IfScoreRangeEnd::Fixed {
							value: ScoreValue::Constant(val),
							inclusive: true,
						},
					right: _,
				},
			),
		negate: false,
		is_bool_cond: true,
	}) = right.only()
	{
		if val.get_i32() == 1 {
			prelude.push(LIRInstruction::new(LIRInstrKind::SubScore(
				MutableScoreValue::Reg(xor_reg.clone()),
				left2.clone(),
			)));
		} else {
			prelude.push(lower_if_single(
				right,
				LIRInstrKind::SubScore(
					MutableScoreValue::Reg(xor_reg.clone()),
					ScoreValue::Constant(ScoreTypeContents::Score(1)),
				),
			));
		}
	} else {
		prelude.push(lower_if_single(
			right,
			LIRInstrKind::SubScore(
				MutableScoreValue::Reg(xor_reg.clone()),
				ScoreValue::Constant(ScoreTypeContents::Score(1)),
			),
		));
	}

	// Since this operation will produce -1 to 1, we can just check for not zero
	let out = IfModCondition::Score(IfScoreCondition::Single {
		left: ScoreValue::Mutable(MutableScoreValue::Reg(xor_reg.clone())),
		right: ScoreValue::Constant(ScoreTypeContents::Bool(false)),
	});

	// This is a bool cond even though it is saturating
	Ok(LoweringCondition {
		condition: out,
		negate: true,
		is_bool_cond: true,
	})
}

fn lower_if_single(conditions: Vec<LoweringCondition>, body: LIRInstrKind) -> LIRInstruction {
	let mut instr = LIRInstruction::new(body);
	for condition in conditions {
		instr.modifiers.insert(0, condition.to_if_mod());
	}
	instr
}

pub(super) fn lower_let_cond(
	left: MutableValue,
	condition: &Condition,
	lir_instrs: &mut Vec<LIRInstruction>,
	lbcx: &mut LowerBlockCx,
) -> anyhow::Result<()> {
	let (prelude, conditions) = lower_condition(condition.clone(), lbcx)?;
	lir_instrs.extend(prelude);
	lower_let_cond_impl(left, conditions, lir_instrs, false, lbcx)?;

	Ok(())
}

/// allow_saturation can be set to allow for some optimizations
pub(super) fn lower_let_cond_impl(
	left: MutableValue,
	conditions: Vec<LoweringCondition>,
	lir_instrs: &mut Vec<LIRInstruction>,
	allow_saturation: bool,
	lbcx: &mut LowerBlockCx,
) -> anyhow::Result<()> {
	// If saturation is allowed, we can just set the target to the source
	// for some conditions
	if allow_saturation {
		if let Some(LoweringCondition {
			condition:
				IfModCondition::Score(
					IfScoreCondition::Single {
						left: left2,
						right: ScoreValue::Constant(val),
					}
					| IfScoreCondition::Range {
						score: left2,
						left:
							IfScoreRangeEnd::Fixed {
								value: ScoreValue::Constant(val),
								inclusive: true,
							},
						right: _,
					},
				),
			negate: false,
			is_bool_cond: true,
		}) = conditions.only()
		{
			if val.get_i32() == 1 {
				lir_instrs.push(LIRInstruction::new(LIRInstrKind::SetScore(
					left.to_mutable_score_value()?,
					left2.clone(),
				)));

				return Ok(());
			}
		}
	}

	let store_loc = match left.get_ty(&lbcx.registers, &lbcx.sig)? {
		DataType::Score(..) => {
			StoreModLocation::from_mut_score_val(&left.to_mutable_score_value()?)?
		}
		_ => bail!("Invalid type"),
	};
	let mut instr = LIRInstruction::new(LIRInstrKind::NoOp);
	instr.modifiers.push(Modifier::StoreSuccess(store_loc));
	for condition in conditions {
		instr.modifiers.push(condition.to_if_mod());
	}
	lir_instrs.push(instr);

	Ok(())
}

/// Lower a single saturating or of multiple conditions
fn lower_single_saturating_or(
	left: &MutableValue,
	right: Vec<LoweringCondition>,
	lbcx: &mut LowerBlockCx,
) -> anyhow::Result<LIRInstruction> {
	// We can just add these
	if let Some(LoweringCondition {
		condition:
			IfModCondition::Score(
				IfScoreCondition::Single {
					left: left2,
					right: ScoreValue::Constant(val),
				}
				| IfScoreCondition::Range {
					score: left2,
					left:
						IfScoreRangeEnd::Fixed {
							value: ScoreValue::Constant(val),
							inclusive: true,
						},
					right: _,
				},
			),
		negate: false,
		is_bool_cond: true,
	}) = right.only()
	{
		if val.get_i32() == 1 {
			return Ok(LIRInstruction::new(LIRInstrKind::AddScore(
				left.clone().to_mutable_score_value()?,
				left2.clone(),
			)));
		}
	}

	let instr = lower_add(
		left.clone(),
		Value::Constant(DataTypeContents::Score(ScoreTypeContents::Score(1))),
		lbcx,
	)?;
	let mut instr = LIRInstruction::new(instr);
	for cond in right {
		instr.modifiers.push(cond.to_if_mod());
	}

	Ok(instr)
}

/// A condition that is in the process of being lowered.
/// Contains extra information about the condition
/// for lowering to optimize on
#[derive(Clone, Debug)]
pub(super) struct LoweringCondition {
	pub condition: IfModCondition,
	pub negate: bool,
	pub is_bool_cond: bool,
}

impl LoweringCondition {
	pub fn new(condition: IfModCondition) -> Self {
		Self {
			condition,
			negate: false,
			is_bool_cond: false,
		}
	}

	pub fn to_if_mod(self) -> Modifier {
		Modifier::If {
			condition: Box::new(self.condition),
			negate: self.negate,
		}
	}

	pub fn negate(self) -> Self {
		Self {
			condition: self.condition,
			negate: !self.negate,
			is_bool_cond: self.is_bool_cond,
		}
	}
}
