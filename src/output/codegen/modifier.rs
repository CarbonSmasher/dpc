use anyhow::{bail, Context};

use crate::common::mc::modifier::{
	EntityRelation, IfModCondition, IfScoreCondition, IfScoreRangeEnd, Modifier, StoreDataType,
	StoreModLocation,
};
use crate::common::ty::{DataType, ScoreTypeContents};
use crate::common::val::ScoreValue;
use crate::output::codegen::util::FloatCG;

use super::t::macros::cgformat;
use super::util::{
	create_lit_score, get_mut_score_val_score, get_nbt_local_loc, get_score_local_score,
};
use super::{Codegen, CodegenBlockCx};

pub fn codegen_modifier(
	modifier: Modifier,
	cbcx: &mut CodegenBlockCx,
) -> anyhow::Result<Option<String>> {
	let out = match modifier {
		Modifier::StoreResult(loc) => Some(format!("store result {}", loc.codegen(cbcx)?)),
		Modifier::StoreSuccess(loc) => Some(format!("store success {}", loc.codegen(cbcx)?)),
		Modifier::If { condition, negate } => {
			let keyword = if negate { "unless" } else { "if" };

			match *condition {
				IfModCondition::Score(condition) => match condition {
					IfScoreCondition::Single { left, right } => match right {
						ScoreValue::Constant(val) => {
							let lit = val.get_literal_str();
							Some(cgformat!(cbcx, keyword, " score ", left, " matches ", lit)?)
						}
						ScoreValue::Mutable(val) => {
							let val = get_mut_score_val_score(&val, cbcx)?;
							Some(cgformat!(cbcx, keyword, " score ", left, " = ", val)?)
						}
					},
					IfScoreCondition::Range { score, left, right } => {
						let out = match (left, right) {
							(IfScoreRangeEnd::Infinite, IfScoreRangeEnd::Infinite) => {
								cgformat!(cbcx, keyword, " score ", score, " matches ..", i32::MAX)?
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

								let use_general_case = if left_in && right_in {
									if let ScoreValue::Constant(left) = left.clone() {
										if let ScoreValue::Constant(right) = right.clone() {
											let left = left.get_literal_str();
											let right = right.get_literal_str();
											out = cgformat!(
												cbcx,
												keyword,
												" score ",
												score,
												" matches ",
												left,
												"..",
												right
											)?;
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
				IfModCondition::Entity(target) => {
					Some(cgformat!(cbcx, keyword, " entity ", target)?)
				}
				IfModCondition::Predicate(pred) => {
					Some(cgformat!(cbcx, keyword, " predicate ", pred)?)
				}
				IfModCondition::Function(fun, _) => {
					let mut func_id = fun;
					if let Some(mapping) = &cbcx.ccx.func_mapping {
						if let Some(new_id) = mapping.0.get(&func_id) {
							func_id = new_id.clone();
						}
					}
					Some(cgformat!(cbcx, keyword, " function ", func_id)?)
				}
				IfModCondition::Biome(pos, biome) => {
					Some(cgformat!(cbcx, keyword, " biome ", pos, " ", biome)?)
				}
				IfModCondition::Dimension(dim) => {
					Some(cgformat!(cbcx, keyword, " dimension ", dim)?)
				}
				IfModCondition::Loaded(pos) => Some(cgformat!(cbcx, keyword, " loaded ", pos)?),
				IfModCondition::DataExists(loc) => Some(cgformat!(cbcx, keyword, " data ", loc)?),
				IfModCondition::DataEquals(l, r) => Some(cgformat!(
					cbcx,
					keyword,
					" data ",
					l,
					" ",
					r.get_literal_str()
				)?),
				IfModCondition::Block(loc, block) => {
					Some(cgformat!(cbcx, keyword, " block ", loc, " ", block)?)
				}
				IfModCondition::Const(val) => {
					let left = ScoreValue::Constant(ScoreTypeContents::Score(0));
					let right_num = if val { 0 } else { 1 };
					Some(cgformat!(
						cbcx,
						keyword,
						" score ",
						left,
						" matches ",
						right_num
					)?)
				}
			}
		}
		Modifier::Anchored(location) => Some(cgformat!(cbcx, "anchored ", location)?),
		Modifier::Align(axes) => Some(cgformat!(cbcx, "align ", axes)?),
		Modifier::As(target) => Some(cgformat!(cbcx, "as ", target)?),
		Modifier::At(target) => Some(cgformat!(cbcx, "at ", target)?),
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
		Modifier::Positioned(pos) => Some(cgformat!(cbcx, "positioned ", pos)?),
		Modifier::PositionedAs(target) => Some(cgformat!(cbcx, "positioned as ", target)?),
		Modifier::PositionedOver(hm) => Some(cgformat!(cbcx, "positioned over ", hm)?),
		Modifier::Rotated(rot) => Some(cgformat!(cbcx, "rotated ", rot)?),
		Modifier::RotatedAs(target) => Some(cgformat!(cbcx, "rotated as ", target)?),
		Modifier::FacingPosition(pos) => Some(cgformat!(cbcx, "facing ", pos)?),
		Modifier::FacingEntity(target, anchor) => {
			Some(cgformat!(cbcx, "facing entity ", target, " ", anchor)?)
		}
		Modifier::Summon(entity) => Some(format!("summon {entity}")),
	};

	Ok(out)
}

fn codegen_if_score_range_side(
	if_keyword: &str,
	score: ScoreValue,
	value: ScoreValue,
	inclusive: bool,
	lt: bool,
	cbcx: &mut CodegenBlockCx,
) -> anyhow::Result<String> {
	let out = match value {
		ScoreValue::Constant(val) => {
			if inclusive {
				let lit = val.get_literal_str();
				let check = codegen_match(&lit, lt);
				cgformat!(cbcx, if_keyword, " score ", score, " matches ", check)?
			} else {
				let num = val.get_i32();
				// Constrict the range by adding or subtracting one if we can
				let constricted = if lt {
					if num == -i32::MAX {
						None
					} else {
						Some(num - 1)
					}
				} else if num == i32::MAX {
					None
				} else {
					Some(num + 1)
				};
				if let Some(constricted) = constricted {
					let check = codegen_match(&constricted.to_string(), lt);
					cgformat!(cbcx, if_keyword, " score ", score, " matches ", check)?
				} else {
					cbcx.ccx.score_literals.insert(num);
					let rhs = create_lit_score(num).gen_str(cbcx)?;
					let sign = if lt { "<" } else { ">" };
					cgformat!(cbcx, if_keyword, " score ", score, " ", sign, " ", rhs)?
				}
			}
		}
		ScoreValue::Mutable(val) => {
			let val = get_mut_score_val_score(&val, cbcx)?.gen_str(cbcx)?;
			let sign = if lt {
				codegen_score_lt(inclusive)
			} else {
				codegen_score_gt(inclusive)
			};
			cgformat!(cbcx, if_keyword, " score ", score, " ", sign, " ", val)?
		}
	};

	Ok(out)
}

fn codegen_match(lit: &str, lt: bool) -> String {
	if lt {
		format!("..{lit}")
	} else {
		format!("{lit}..")
	}
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
	fn codegen(self, cbcx: &mut CodegenBlockCx) -> anyhow::Result<String> {
		match self {
			Self::Local(loc, scale) => {
				let ty = loc.get_ty(&cbcx.regs, &cbcx.sig)?;
				match ty {
					DataType::Score(..) => {
						let loc = get_score_local_score(&loc, cbcx)?;
						cgformat!(cbcx, "score ", loc)
					}
					DataType::NBT(ty) => {
						let loc = get_nbt_local_loc(&loc, cbcx)?;
						let ty = StoreDataType::from_nbt_ty(&ty)
							.context("Type is not a valid storage type")?;
						cgformat!(cbcx, loc, " ", ty, " ", FloatCG(scale, false, true, true))
					}
					_ => bail!("Type not supported"),
				}
			}
			Self::Score(score) => Ok(cgformat!(cbcx, "score {}", score)?),
			Self::Data(data, ty, scale) => {
				cgformat!(cbcx, data, " ", ty, " ", FloatCG(scale, false, true, true))
			}
			Self::Bossbar(bar, mode) => Ok(format!("bossbar {bar} {mode:?}")),
		}
	}
}

#[cfg(test)]
mod tests {
	use rustc_hash::FxHashMap;

	use super::*;

	use crate::common::function::FunctionSignature;
	use crate::common::mc::{EntityTarget, Score};
	use crate::common::RegisterList;
	use crate::common::{ty::ScoreTypeContents, val::MutableScoreValue};
	use crate::output::ra::GlobalRegAllocResult;
	use crate::output::{codegen::CodegenCx, ra::RegAllocResult};
	use crate::project::ProjectSettings;

	#[test]
	fn test_if_score_codegen() {
		let proj = ProjectSettings::new("dpc".into());
		let ra = GlobalRegAllocResult {
			results: FxHashMap::default(),
		};
		let mut ccx = CodegenCx::new(&proj, None, ra);
		let mut cbcx = CodegenBlockCx {
			ccx: &mut ccx,
			ra: RegAllocResult::new(),
			regs: RegisterList::default(),
			func_id: "foo".into(),
			macro_line: false,
			sig: FunctionSignature::new(),
		};

		let modifier = Modifier::If {
			condition: Box::new(IfModCondition::Score(IfScoreCondition::Range {
				score: ScoreValue::Mutable(MutableScoreValue::Score(Score::new(
					EntityTarget::Player("foo".into()),
					"bar".into(),
				))),
				left: IfScoreRangeEnd::Fixed {
					value: ScoreValue::Constant(ScoreTypeContents::Score(219)),
					inclusive: false,
				},
				right: IfScoreRangeEnd::Fixed {
					value: ScoreValue::Constant(ScoreTypeContents::Score(2980)),
					inclusive: true,
				},
			})),
			negate: true,
		};

		let code = codegen_modifier(modifier, &mut cbcx)
			.expect("Failed to codegen modifier")
			.expect("Modifier missing");
		assert_eq!(
			code,
			format!("unless score foo bar matches 220.. unless score foo bar matches ..2980")
		);
	}
}
