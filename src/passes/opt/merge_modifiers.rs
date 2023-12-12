use anyhow::anyhow;

use crate::common::mc::modifier::{IfModCondition, IfScoreCondition, IfScoreRangeEnd, Modifier};
use crate::lir::{LIRInstruction, LIR};
use crate::passes::{LIRPass, Pass};

pub struct MergeModifiersPass;

impl Pass for MergeModifiersPass {
	fn get_name(&self) -> &'static str {
		"merge_modifiers"
	}
}

impl LIRPass for MergeModifiersPass {
	fn run_pass(&mut self, lir: &mut LIR) -> anyhow::Result<()> {
		for (_, block) in &mut lir.functions {
			let block = lir
				.blocks
				.get_mut(block)
				.ok_or(anyhow!("Block does not exist"))?;

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
			(l, r) => (l.clone(), Some(r.clone())),
		};
		mods.push(new_mods.0);
		if i == instr.modifiers.len() - 1 {
			mods.extend(new_mods.1);
		}
	}

	instr.modifiers = mods;
}
