use anyhow::anyhow;

use crate::common::mc::{DataLocation, EntityTarget, FullDataLocation, Score};
use crate::common::val::{MutableNBTValue, MutableScoreValue, NBTValue, ScoreValue};
use crate::linker::ra::RegAllocResult;
use crate::linker::text::{
	format_lit_fake_player, LIT_OBJECTIVE, REG_OBJECTIVE, REG_STORAGE_LOCATION,
};

use super::t::macros::cgformat;
use super::{Codegen, CodegenBlockCx};

/// Returns a score and an optional score literal to add
#[allow(dead_code)]
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

#[allow(dead_code)]
pub fn get_score_val_lit(val: &ScoreValue, cbcx: &mut CodegenBlockCx) -> anyhow::Result<String> {
	let out = match val {
		ScoreValue::Constant(score) => score.get_literal_str(),
		ScoreValue::Mutable(val) => get_mut_score_val_score(val, &cbcx.ra)?.gen_str(cbcx)?,
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
			Score::new(EntityTarget::Player(reg.clone()), REG_OBJECTIVE.into())
		}
	};

	Ok(out)
}

pub fn get_mut_nbt_val_loc(
	val: &MutableNBTValue,
	ra: &RegAllocResult,
) -> anyhow::Result<FullDataLocation> {
	let out = match val {
		MutableNBTValue::Data(data) => data.clone(),
		MutableNBTValue::Reg(reg) => {
			let reg = ra
				.locals
				.get(reg)
				.ok_or(anyhow!("Local register {reg} not allocated"))?;
			FullDataLocation {
				loc: DataLocation::Storage(REG_STORAGE_LOCATION.into()),
				path: reg.clone(),
			}
		}
	};

	Ok(out)
}

pub fn create_lit_score(num: i32) -> Score {
	Score::new(
		EntityTarget::Player(format_lit_fake_player(num)),
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

/// Does codegen for the right side of a data modify
pub fn cg_data_modify_rhs(cbcx: &mut CodegenBlockCx, val: &NBTValue) -> anyhow::Result<String> {
	let string = match val {
		NBTValue::Constant(data) => cgformat!(cbcx, "value ", data.get_literal_str())?,
		NBTValue::Mutable(val) => cgformat!(cbcx, "from ", val)?,
	};

	Ok(string)
}

impl Codegen for ScoreValue {
	fn gen_writer<F>(&self, f: &mut F, cbcx: &mut CodegenBlockCx) -> anyhow::Result<()>
	where
		F: std::fmt::Write,
	{
		let (score, lit) = get_score_val_score(self, &cbcx.ra)?;
		cbcx.ccx.score_literals.extend(lit);
		score.gen_writer(f, cbcx)
	}
}

impl Codegen for MutableScoreValue {
	fn gen_writer<F>(&self, f: &mut F, cbcx: &mut CodegenBlockCx) -> anyhow::Result<()>
	where
		F: std::fmt::Write,
	{
		let score = get_mut_score_val_score(self, &cbcx.ra)?;
		score.gen_writer(f, cbcx)
	}
}

impl Codegen for NBTValue {
	fn gen_writer<F>(&self, f: &mut F, cbcx: &mut CodegenBlockCx) -> anyhow::Result<()>
	where
		F: std::fmt::Write,
	{
		match self {
			Self::Constant(val) => write!(f, "{}", val.get_literal_str())?,
			Self::Mutable(val) => val.gen_writer(f, cbcx)?,
		}
		Ok(())
	}
}

impl Codegen for MutableNBTValue {
	fn gen_writer<F>(&self, f: &mut F, cbcx: &mut CodegenBlockCx) -> anyhow::Result<()>
	where
		F: std::fmt::Write,
	{
		let loc = get_mut_nbt_val_loc(self, &cbcx.ra)?;
		loc.gen_writer(f, cbcx)?;

		Ok(())
	}
}

pub struct SpaceSepListCG<'v, CG: Codegen>(pub &'v Vec<CG>);

impl<'v, CG: Codegen> Codegen for SpaceSepListCG<'v, CG> {
	fn gen_writer<F>(&self, f: &mut F, cbcx: &mut CodegenBlockCx) -> anyhow::Result<()>
	where
		F: std::fmt::Write,
	{
		for (i, elem) in self.0.iter().enumerate() {
			elem.gen_writer(f, cbcx)?;
			if i != self.0.len() - 1 {
				write!(f, " ")?;
			}
		}
		Ok(())
	}
}
