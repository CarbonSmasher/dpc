use anyhow::{anyhow, bail};

use crate::common::ty::DataTypeContents;
use crate::common::{ty::DataType, MutableValue, RegisterList};
use crate::common::{MutableScoreValue, ScoreValue, Value};
use crate::linker::ra::RegAllocResult;
use crate::linker::text::{format_lit_fake_player, LIT_OBJECTIVE, REG_OBJECTIVE};
use crate::mc::{Score, TargetSelector};

/// Returns a score and an optional score literal to add
pub fn get_val_score(
	val: &Value,
	ra: &RegAllocResult,
	regs: &RegisterList,
) -> anyhow::Result<(Score, ScoreLiteral)> {
	let out = match val {
		Value::Constant(val) => match val {
			DataTypeContents::Score(score) => {
				let num = score.get_i32();
				(create_lit_score(num), ScoreLiteral(Some(num)))
			}
			_ => bail!("LIR instruction given wrong type"),
		},
		Value::Mutable(val) => {
			let score = Score::new(
				TargetSelector::Player(get_mut_val_reg(val, ra, regs)?.clone()),
				REG_OBJECTIVE.into(),
			);
			(score, ScoreLiteral(None))
		}
	};

	Ok(out)
}

/// Returns a score and an optional score literal to add
pub fn get_score_val_score(
	val: &ScoreValue,
	ra: &RegAllocResult,
) -> anyhow::Result<(Score, ScoreLiteral)> {
	let out = match val {
		ScoreValue::Constant(score) => {
			let num = score.get_i32();
			(create_lit_score(num), ScoreLiteral(Some(num)))
		}
		ScoreValue::Mutable(val) => (get_mut_score_val_score(val, ra)?, ScoreLiteral(None)),
	};

	Ok(out)
}

pub fn get_score_val_lit(val: &ScoreValue, ra: &RegAllocResult) -> anyhow::Result<String> {
	let out = match val {
		ScoreValue::Constant(score) => score.get_literal_str(),
		ScoreValue::Mutable(val) => get_mut_score_val_score(val, ra)?.codegen_str(),
	};

	Ok(out)
}

pub fn get_mut_score_val_score(
	val: &MutableScoreValue,
	ra: &RegAllocResult,
) -> anyhow::Result<Score> {
	let out = match val {
		MutableScoreValue::Score(score) => score.clone(),
		MutableScoreValue::Reg(reg) => {
			let reg = ra
				.regs
				.get(reg)
				.ok_or(anyhow!("Register {reg} not allocated"))?;
			Score::new(TargetSelector::Player(reg.clone()), REG_OBJECTIVE.into())
		}
	};

	Ok(out)
}

pub fn get_mut_val_reg<'ra>(
	val: &MutableValue,
	ra: &'ra RegAllocResult,
	regs: &RegisterList,
) -> anyhow::Result<&'ra String> {
	match val {
		MutableValue::Register(reg) => match val.get_ty(regs)? {
			DataType::Score(..) => ra
				.regs
				.get(reg)
				.ok_or(anyhow!("Register {reg} not allocated")),
			DataType::NBT(..) => ra
				.locals
				.get(reg)
				.ok_or(anyhow!("Register {reg} not allocated")),
		},
	}
}

pub fn create_lit_score(num: i32) -> Score {
	Score::new(
		TargetSelector::Player(format_lit_fake_player(num)),
		LIT_OBJECTIVE.into(),
	)
}

/// A score literal that must be added to the literal list
#[must_use]
pub struct ScoreLiteral(pub Option<i32>);

impl IntoIterator for ScoreLiteral {
	type IntoIter = std::option::IntoIter<i32>;
	type Item = i32;

	fn into_iter(self) -> Self::IntoIter {
		self.0.into_iter()
	}
}
