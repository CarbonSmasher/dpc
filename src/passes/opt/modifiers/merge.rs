use crate::common::mc::modifier::{IfModCondition, IfScoreCondition, IfScoreRangeEnd, Modifier};
use crate::common::mc::pos::{AbsOrRelCoord, Coordinates};
use crate::common::ty::Double;
use crate::lir::LIRInstruction;
use crate::passes::{LIRPass, LIRPassData, Pass};

pub struct MergeModifiersPass;

impl Pass for MergeModifiersPass {
	fn get_name(&self) -> &'static str {
		"merge_modifiers"
	}
}

impl LIRPass for MergeModifiersPass {
	fn run_pass(&mut self, data: &mut LIRPassData) -> anyhow::Result<()> {
		for func in data.lir.functions.values_mut() {
			let block = &mut func.block;

			for instr in &mut block.contents {
				if instr.modifiers.len() >= 2 {
					merge_modifiers(instr);
				}
			}
		}

		Ok(())
	}
}

fn merge_modifiers(instr: &mut LIRInstruction) {
	let mut mods = Vec::with_capacity(instr.modifiers.len());
	for (i, window) in instr.modifiers.windows(2).enumerate() {
		let l = &window[0];
		let r = &window[1];
		let new_mods = match (l, r) {
			(
				Modifier::If {
					condition: condition1,
					negate: negate1,
				},
				Modifier::If {
					condition: condition2,
					negate: negate2,
				},
			) => {
				if !negate1 && !negate2 {
					match (condition1.as_ref(), condition2.as_ref()) {
						(
							IfModCondition::Score(IfScoreCondition::Range {
								score: score1,
								left: left @ IfScoreRangeEnd::Fixed { .. },
								right: IfScoreRangeEnd::Infinite,
							}),
							IfModCondition::Score(IfScoreCondition::Range {
								score: score2,
								left: IfScoreRangeEnd::Infinite,
								right: right @ IfScoreRangeEnd::Fixed { .. },
							}),
						)
						| (
							IfModCondition::Score(IfScoreCondition::Range {
								score: score1,
								left: IfScoreRangeEnd::Infinite,
								right: right @ IfScoreRangeEnd::Fixed { .. },
							}),
							IfModCondition::Score(IfScoreCondition::Range {
								score: score2,
								left: left @ IfScoreRangeEnd::Fixed { .. },
								right: IfScoreRangeEnd::Infinite,
							}),
						) if score1.is_value_eq(score2) => {
							let new = IfModCondition::Score(IfScoreCondition::Range {
								score: score1.clone(),
								left: left.clone(),
								right: right.clone(),
							});
							(
								Modifier::If {
									condition: Box::new(new),
									negate: false,
								},
								None,
							)
						}
						_ => (l.clone(), Some(r.clone())),
					}
				} else {
					(l.clone(), Some(r.clone()))
				}
			}
			(Modifier::Positioned(pos1), Modifier::Positioned(pos2)) => match (pos1, pos2) {
				(Coordinates::XYZ(x1, y1, z1), Coordinates::XYZ(x2, y2, z2)) => (
					Modifier::Positioned(Coordinates::XYZ(
						merge_abs_rel_coords(x1.clone(), x2.clone()),
						merge_abs_rel_coords(y1.clone(), y2.clone()),
						merge_abs_rel_coords(z1.clone(), z2.clone()),
					)),
					None,
				),
				(Coordinates::Local(x1, y1, z1), Coordinates::Local(x2, y2, z2)) => (
					Modifier::Positioned(Coordinates::Local(x1 + x2, y1 + y2, z1 + z2)),
					None,
				),
				_ => (l.clone(), Some(r.clone())),
			},
			(l, r) => (l.clone(), Some(r.clone())),
		};
		mods.push(new_mods.0);
		if i == instr.modifiers.len() - 2 {
			mods.extend(new_mods.1);
		}
	}

	instr.modifiers = mods;
}

fn merge_abs_rel_coords(
	l: AbsOrRelCoord<Double>,
	r: AbsOrRelCoord<Double>,
) -> AbsOrRelCoord<Double> {
	match (l, r.clone()) {
		// Absolute on right overrides the left
		(AbsOrRelCoord::Rel(..) | AbsOrRelCoord::Abs(..), AbsOrRelCoord::Abs(..)) => r,
		// Absolute with right added
		(AbsOrRelCoord::Abs(l), AbsOrRelCoord::Rel(r)) => AbsOrRelCoord::Abs(l + r),
		// Relative with right added
		(AbsOrRelCoord::Rel(l), AbsOrRelCoord::Rel(r)) => AbsOrRelCoord::Rel(l + r),
	}
}
