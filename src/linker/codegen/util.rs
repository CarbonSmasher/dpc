use anyhow::anyhow;

use crate::common::mc::{DataLocation, EntityTarget, FullDataLocation, Score};
use crate::common::val::{MutableNBTValue, MutableScoreValue, NBTValue, ScoreValue};
use crate::linker::ra::RegAllocResult;
use crate::linker::text::{
	format_arg_fake_player, format_arg_local_storage_entry, format_lit_fake_player, LIT_OBJECTIVE,
	REG_OBJECTIVE, REG_STORAGE_LOCATION,
};
use crate::lower::cleanup_fn_id;

use super::t::macros::cgformat;
use super::{Codegen, CodegenBlockCx};

/// Returns a score and an optional score literal to add
#[allow(dead_code)]
pub fn get_score_val_score(
	val: &ScoreValue,
	ra: &RegAllocResult,
	func_id: &str,
) -> anyhow::Result<(Score, ScoreLiteral)> {
	let out = match val {
		ScoreValue::Constant(score) => {
			let num = score.get_i32();
			(create_lit_score(num), ScoreLiteral(Some(num)))
		}
		ScoreValue::Mutable(val) => (
			get_mut_score_val_score(val, ra, func_id)?,
			ScoreLiteral(None),
		),
	};

	Ok(out)
}

#[allow(dead_code)]
pub fn get_score_val_lit(
	val: &ScoreValue,
	cbcx: &mut CodegenBlockCx,
	func_id: &str,
) -> anyhow::Result<String> {
	let out = match val {
		ScoreValue::Constant(score) => score.get_literal_str(),
		ScoreValue::Mutable(val) => {
			get_mut_score_val_score(val, &cbcx.ra, func_id)?.gen_str(cbcx)?
		}
	};

	Ok(out)
}

pub fn get_mut_score_val_score(
	val: &MutableScoreValue,
	ra: &RegAllocResult,
	func_id: &str,
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
		MutableScoreValue::Arg(arg) => {
			let arg = format_arg_fake_player(*arg, func_id);
			Score::new(EntityTarget::Player(arg), REG_OBJECTIVE.into())
		}
		MutableScoreValue::CallArg(arg, func, ..) => {
			let func_id = cleanup_fn_id(func);
			let arg = format_arg_local_storage_entry(*arg, &func_id);
			Score::new(EntityTarget::Player(arg), REG_OBJECTIVE.into())
		}
	};

	Ok(out)
}

pub fn get_mut_nbt_val_loc(
	val: &MutableNBTValue,
	ra: &RegAllocResult,
	func_id: &str,
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
		MutableNBTValue::Arg(arg) => {
			let arg = format_arg_local_storage_entry(*arg, func_id);
			FullDataLocation {
				loc: DataLocation::Storage(REG_STORAGE_LOCATION.into()),
				path: arg,
			}
		}
		MutableNBTValue::CallArg(arg, func, ..) => {
			let func_id = cleanup_fn_id(func);
			let arg = format_arg_local_storage_entry(*arg, &func_id);
			FullDataLocation {
				loc: DataLocation::Storage(REG_STORAGE_LOCATION.into()),
				path: arg,
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
