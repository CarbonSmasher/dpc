use anyhow::anyhow;

use crate::common::mc::{DataLocation, DataPath, EntityTarget, FullDataLocation, Score};
use crate::common::val::{MutableNBTValue, MutableScoreValue, NBTValue, ScoreValue};
use crate::lower::cleanup_fn_id;
use crate::output::text::{
	format_arg_fake_player, format_arg_local_storage_entry, format_lit_fake_player,
	format_ret_fake_player, format_ret_local_storage_entry, LIT_OBJECTIVE, REG_OBJECTIVE,
	REG_STORAGE_LOCATION,
};

use super::t::macros::cgformat;
use super::{Codegen, CodegenBlockCx, CodegenRequirement};

/// Returns a score and an optional score literal to add
pub fn get_score_val_score(
	val: &ScoreValue,
	cbcx: &mut CodegenBlockCx,
) -> anyhow::Result<(Score, ScoreLiteral)> {
	let out = match val {
		ScoreValue::Constant(score) => {
			let num = score.get_i32();
			(create_lit_score(num), ScoreLiteral(Some(num)))
		}
		ScoreValue::Mutable(val) => (get_mut_score_val_score(val, cbcx)?, ScoreLiteral(None)),
	};

	Ok(out)
}

pub fn get_mut_score_val_score(
	val: &MutableScoreValue,
	cbcx: &mut CodegenBlockCx,
) -> anyhow::Result<Score> {
	let out = match val {
		MutableScoreValue::Score(score) => score.clone(),
		MutableScoreValue::Reg(reg) => {
			let reg = cbcx
				.ra
				.regs
				.get(reg)
				.ok_or(anyhow!("Register {reg} not allocated"))?;
			cbcx.ccx
				.add_requirement(CodegenRequirement::UseRegObjective);
			Score::new(EntityTarget::Player(reg.clone()), REG_OBJECTIVE.into())
		}
		MutableScoreValue::Arg(arg) => {
			let arg = format_arg_fake_player(*arg, &cbcx.func_id);

			cbcx.ccx
				.add_requirement(CodegenRequirement::UseRegObjective);
			Score::new(EntityTarget::Player(arg), REG_OBJECTIVE.into())
		}
		MutableScoreValue::CallArg(arg, func, ..) => {
			let func_id = cleanup_fn_id(func);
			let arg = format_arg_local_storage_entry(*arg, &func_id);
			cbcx.ccx
				.add_requirement(CodegenRequirement::UseRegObjective);
			Score::new(EntityTarget::Player(arg), REG_OBJECTIVE.into())
		}
		MutableScoreValue::ReturnValue(ret) => {
			let ret = format_ret_fake_player(*ret, &cbcx.func_id);
			cbcx.ccx
				.add_requirement(CodegenRequirement::UseRegObjective);
			Score::new(EntityTarget::Player(ret), REG_OBJECTIVE.into())
		}
		MutableScoreValue::CallReturnValue(ret, func, ..) => {
			let func_id = cleanup_fn_id(func);
			let ret = format_ret_local_storage_entry(*ret, &func_id);
			cbcx.ccx
				.add_requirement(CodegenRequirement::UseRegObjective);
			Score::new(EntityTarget::Player(ret), REG_OBJECTIVE.into())
		}
	};

	Ok(out)
}

pub fn get_mut_nbt_val_loc(
	val: &MutableNBTValue,
	cbcx: &mut CodegenBlockCx,
) -> anyhow::Result<FullDataLocation> {
	let out = match val {
		MutableNBTValue::Data(data) => data.clone(),
		MutableNBTValue::Reg(reg) => {
			let reg = cbcx
				.ra
				.locals
				.get(reg)
				.ok_or(anyhow!("Local register {reg} not allocated"))?;
			FullDataLocation {
				loc: DataLocation::Storage(REG_STORAGE_LOCATION.into()),
				path: DataPath::String(reg.clone()),
			}
		}
		MutableNBTValue::Property(val, prop) => {
			let mut loc = get_mut_nbt_val_loc(val, cbcx)?;
			loc.path = DataPath::Access(Box::new(std::mem::take(&mut loc.path)), prop.clone());
			loc
		}
		MutableNBTValue::Index(val, idx) => {
			let mut loc = get_mut_nbt_val_loc(val, cbcx)?;
			loc.path = DataPath::Index(Box::new(std::mem::take(&mut loc.path)), *idx);
			loc
		}
		MutableNBTValue::Arg(arg) => {
			let arg = format_arg_local_storage_entry(*arg, &cbcx.func_id);
			cbcx.ccx.add_requirement(CodegenRequirement::UseRegStorage);
			FullDataLocation {
				loc: DataLocation::Storage(REG_STORAGE_LOCATION.into()),
				path: DataPath::String(arg),
			}
		}
		MutableNBTValue::CallArg(arg, func, ..) => {
			let func_id = cleanup_fn_id(func);
			let arg = format_arg_local_storage_entry(*arg, &func_id);
			cbcx.ccx.add_requirement(CodegenRequirement::UseRegStorage);
			FullDataLocation {
				loc: DataLocation::Storage(REG_STORAGE_LOCATION.into()),
				path: DataPath::String(arg),
			}
		}
		MutableNBTValue::ReturnValue(ret) => {
			let ret = format_ret_local_storage_entry(*ret, &cbcx.func_id);
			cbcx.ccx.add_requirement(CodegenRequirement::UseRegStorage);
			FullDataLocation {
				loc: DataLocation::Storage(REG_STORAGE_LOCATION.into()),
				path: DataPath::String(ret),
			}
		}
		MutableNBTValue::CallReturnValue(ret, func, ..) => {
			let func_id = cleanup_fn_id(func);
			let ret = format_arg_local_storage_entry(*ret, &func_id);
			cbcx.ccx.add_requirement(CodegenRequirement::UseRegStorage);
			FullDataLocation {
				loc: DataLocation::Storage(REG_STORAGE_LOCATION.into()),
				path: DataPath::String(ret),
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

pub struct FloatCG(pub f64, pub bool, pub bool, pub bool);

impl Codegen for FloatCG {
	fn gen_writer<F>(&self, f: &mut F, cbcx: &mut CodegenBlockCx) -> anyhow::Result<()>
	where
		F: std::fmt::Write,
	{
		let _ = cbcx;
		cg_float(f, self.0, self.1, self.2, self.3);
		Ok(())
	}
}

/// Utility function to format floating-point numbers as small as possible
/// with different formatting options
pub fn cg_float<F: std::fmt::Write>(
	f: &mut F,
	num: f64,
	omit_zero: bool,
	trim_trailing_zero: bool,
	trim_leading_zero: bool,
) {
	if omit_zero && num == 0.0 {
		return;
	}

	let mut out = format!("{}", num);
	if trim_leading_zero {
		if out.starts_with("0.") {
			out = out[1..].into();
		} else if out.starts_with("-0.") {
			out = "-".to_string() + out[2..].into();
		}
	}
	if trim_trailing_zero && out.ends_with(".0") {
		out = out[..out.len() - 2].into();
	}

	let _ = write!(f, "{out}");
}
