use anyhow::anyhow;

use crate::common::modifier::{
	AnchorModLocation, EntityRelation, IfModCondition, IfScoreCondition, IfScoreRangeEnd, Modifier,
	StoreModLocation,
};
use crate::common::ScoreValue;
use crate::linker::text::REG_OBJECTIVE;

use super::util::{create_lit_score, get_mut_score_val_score, get_score_val_lit};
use super::CodegenBlockCx;

pub fn codegen_modifier(
	modifier: Modifier,
	cbcx: &mut CodegenBlockCx,
) -> anyhow::Result<Option<String>> {
	let out = match modifier {
		Modifier::StoreResult(loc) => Some(format!("store result {}", loc.codegen(cbcx)?)),
		Modifier::StoreSuccess(loc) => Some(format!("store success {}", loc.codegen(cbcx)?)),
		Modifier::If { condition, negate } => {
			let keyword = if negate { "unless" } else { "if" };

			let out = match *condition {
				IfModCondition::Score(condition) => match condition {
					IfScoreCondition::Single { left, right } => {
						let left = get_mut_score_val_score(&left, &cbcx.ra)?.codegen_str();
						let right = get_score_val_lit(&right, &cbcx.ra)?;

						Some(format!("{keyword} score {left} = {right}"))
					}
					IfScoreCondition::Range { score, left, right } => {
						let score = get_mut_score_val_score(&score, &cbcx.ra)?.codegen_str();
						let out = match (left, right) {
							(IfScoreRangeEnd::Infinite, IfScoreRangeEnd::Infinite) => {
								format!(
									"{keyword} score {score} matches {}..{}",
									-i32::MAX,
									i32::MAX
								)
							}
							(
								IfScoreRangeEnd::Fixed {
									value: left,
									inclusive: left_in,
								},
								IfScoreRangeEnd::Infinite,
							) => codegen_if_score_range_side(
								keyword, score, left, left_in, false, cbcx,
							)?,
							(
								IfScoreRangeEnd::Infinite,
								IfScoreRangeEnd::Fixed {
									value: right,
									inclusive: right_in,
								},
							) => codegen_if_score_range_side(
								keyword, score, right, right_in, true, cbcx,
							)?,
							(
								IfScoreRangeEnd::Fixed {
									value: left,
									inclusive: left_in,
								},
								IfScoreRangeEnd::Fixed {
									value: right,
									inclusive: right_in,
								},
							) => {
								let mut out = String::new();

								let use_general_case =
									if left_in && right_in {
										if let ScoreValue::Constant(left) = left.clone() {
											if let ScoreValue::Constant(right) = right.clone() {
												let left = left.get_literal_str();
												let right = right.get_literal_str();
												out =  format!("{keyword} score {score} matches {left}..{right}");
												false
											} else {
												true
											}
										} else {
											true
										}
									} else {
										true
									};

								if use_general_case {
									let left = codegen_if_score_range_side(
										keyword,
										score.clone(),
										left,
										left_in,
										false,
										cbcx,
									)?;
									let right = codegen_if_score_range_side(
										keyword, score, right, right_in, true, cbcx,
									)?;
									out = format!("{left} {right}");
								}

								out
							}
						};

						Some(out)
					}
				},
			};

			out
		}
		Modifier::Anchored(location) => match location {
			AnchorModLocation::Eyes => Some("anchored eyes".into()),
			AnchorModLocation::Feet => Some("anchored feet".into()),
		},
		Modifier::As(target) => Some(format!("as {}", target.codegen_str())),
		Modifier::At(target) => Some(format!("at {}", target.codegen_str())),
		Modifier::In(dimension) => Some(format!("in {dimension}")),
		Modifier::On(relation) => {
			let string = match relation {
				EntityRelation::Attacker => "attacker",
				EntityRelation::Controller => "controller",
				EntityRelation::Leasher => "leasher",
				EntityRelation::Origin => "origin",
				EntityRelation::Owner => "owner",
				EntityRelation::Passengers => "passengers",
				EntityRelation::Target => "target",
				EntityRelation::Vehicle => "vehicle",
			};

			Some(format!("on {string}"))
		}
	};

	Ok(out)
}

fn codegen_if_score_range_side(
	if_keyword: &str,
	score: String,
	value: ScoreValue,
	inclusive: bool,
	lt: bool,
	cbcx: &mut CodegenBlockCx,
) -> anyhow::Result<String> {
	let out = match value {
		ScoreValue::Constant(val) => {
			if inclusive {
				let lit = val.get_literal_str();
				let check = if lt {
					format!("..{lit}")
				} else {
					format!("{lit}..")
				};
				format!("{if_keyword} score {score} matches {check}")
			} else {
				let num = val.get_i32();
				cbcx.ccx.score_literals.insert(num);
				let rhs = create_lit_score(num).codegen_str();
				let sign = if lt { "<" } else { ">" };
				format!("{if_keyword} score {score} {sign} {rhs}")
			}
		}
		ScoreValue::Mutable(val) => {
			let val = get_mut_score_val_score(&val, &cbcx.ra)?.codegen_str();
			let sign = if lt {
				codegen_score_lt(inclusive)
			} else {
				codegen_score_gt(inclusive)
			};
			format!("{if_keyword} score {score} {sign} {val}")
		}
	};

	Ok(out)
}

fn codegen_score_gt(inclusive: bool) -> &'static str {
	if inclusive {
		">="
	} else {
		">"
	}
}

fn codegen_score_lt(inclusive: bool) -> &'static str {
	if inclusive {
		"<="
	} else {
		"<"
	}
}

impl StoreModLocation {
	fn codegen(self, cbcx: &CodegenBlockCx) -> anyhow::Result<String> {
		match self {
			Self::Score(score) => Ok(format!("score {}", score.codegen_str())),
			Self::Reg(reg) => {
				let reg = cbcx
					.ra
					.regs
					.get(&reg)
					.ok_or(anyhow!("Register {reg} not allocated"))?;
				Ok(format!("score {reg} {REG_OBJECTIVE}"))
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	use crate::common::RegisterList;
	use crate::common::{ty::ScoreTypeContents, MutableScoreValue};
	use crate::linker::{codegen::CodegenCx, ra::RegAllocResult};
	use crate::mc::{Score, TargetSelector};

	#[test]
	fn test_if_score_codegen() {
		let mut ccx = CodegenCx::new();
		let mut cbcx = CodegenBlockCx {
			ccx: &mut ccx,
			ra: RegAllocResult::new(),
			regs: RegisterList::new(),
		};

		let modifier = Modifier::If {
			condition: Box::new(IfModCondition::Score(IfScoreCondition::Range {
				score: MutableScoreValue::Score(Score::new(
					TargetSelector::Player("foo".into()),
					"bar".into(),
				)),
				left: IfScoreRangeEnd::Fixed {
					value: ScoreValue::Constant(ScoreTypeContents::Score(219)),
					inclusive: false,
				},
				right: IfScoreRangeEnd::Fixed {
					value: ScoreValue::Constant(ScoreTypeContents::UScore(2980)),
					inclusive: true,
				},
			})),
			negate: true,
		};

		let code = codegen_modifier(modifier, &mut cbcx)
			.expect("Failed to codegen modifier")
			.expect("Modifier missing");
		let lit_fmt = create_lit_score(219).codegen_str();
		assert_eq!(
			code,
			format!("unless score foo bar > {lit_fmt} unless score foo bar matches ..2980")
		);
	}
}
