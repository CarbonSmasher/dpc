use anyhow::{anyhow, bail};
use rustc_hash::FxHashMap;
use std::fmt::Debug;

use crate::common::ty::{
	Byte, DataType, DataTypeContents, Double, Float, Int, Long, NBTType, NBTTypeContents,
	ScoreTypeContents, Short,
};
use crate::common::val::{MutableValue, Value};
use crate::common::{DeclareBinding, Identifier};
use crate::mir::{MIRBlock, MIRInstrKind};
use crate::passes::{MIRPass, MIRPassData, Pass};
use crate::util::{remove_indices, HashSetEmptyTracker};

use super::{ConstAnalyzer, ConstAnalyzerResult, ConstAnalyzerValue};

pub struct ConstFoldPass {
	pub(super) made_changes: bool,
}

impl ConstFoldPass {
	pub fn new() -> Self {
		Self {
			made_changes: false,
		}
	}
}

impl Default for ConstFoldPass {
	fn default() -> Self {
		Self::new()
	}
}

impl Pass for ConstFoldPass {
	fn get_name(&self) -> &'static str {
		"const_fold"
	}
}

impl MIRPass for ConstFoldPass {
	fn run_pass(&mut self, data: &mut MIRPassData) -> anyhow::Result<()> {
		let mut fold_points = FxHashMap::default();
		let mut an = ConstAnalyzer::new();
		for func in data.mir.functions.values_mut() {
			let block = data
				.mir
				.blocks
				.get_mut(&func.block)
				.ok_or(anyhow!("Block does not exist"))?;

			fold_points.clear();
			an.reset();

			let mut instrs_to_remove = HashSetEmptyTracker::new();
			loop {
				let run_again =
					run_const_fold_iter(block, &mut instrs_to_remove, &mut fold_points, &mut an)?;
				if run_again {
					self.made_changes = true;
				} else {
					break;
				}
			}
			remove_indices(&mut block.contents, &instrs_to_remove);
		}

		Ok(())
	}
}

/// Runs an iteration of const fold. Returns true if another iteration
/// should be run
fn run_const_fold_iter(
	block: &mut MIRBlock,
	instrs_to_remove: &mut HashSetEmptyTracker<usize>,
	fold_points: &mut FxHashMap<Identifier, FoldPoint>,
	an: &mut ConstAnalyzer,
) -> anyhow::Result<bool> {
	let mut run_again = false;
	// We have to store this array of instructions to replace and do it at the end
	// since we can't modify instructions in the block while we are iterating over it
	let mut instrs_to_replace = Vec::new();

	// Scope here because the analyzer holds references to the data type contents

	for (i, instr) in block.contents.iter().enumerate() {
		if instrs_to_remove.contains(&i) {
			continue;
		}

		match &instr.kind {
			MIRInstrKind::Assign {
				left: MutableValue::Reg(left),
				right: DeclareBinding::Value(Value::Constant(right)),
			} => {
				if let Some(left) = fold_points.get_mut(left) {
					if !left.finished {
						match right {
							DataTypeContents::Score(right) => {
								let FoldValue::Score(value) = &mut left.value else {
										bail!("Incorrect types");
									};
								*value = Some(right.get_i32());
							}
							DataTypeContents::NBT(NBTTypeContents::Byte(right)) => {
								let FoldValue::Byte(value) = &mut left.value else {
										bail!("Incorrect types");
									};
								*value = Some(*right);
							}
							DataTypeContents::NBT(NBTTypeContents::Short(right)) => {
								let FoldValue::Short(value) = &mut left.value else {
										bail!("Incorrect types");
									};
								*value = Some(*right);
							}
							DataTypeContents::NBT(NBTTypeContents::Int(right)) => {
								let FoldValue::Int(value) = &mut left.value else {
										bail!("Incorrect types");
									};
								*value = Some(*right);
							}
							DataTypeContents::NBT(NBTTypeContents::Long(right)) => {
								let FoldValue::Long(value) = &mut left.value else {
										bail!("Incorrect types");
									};
								*value = Some(*right);
							}
							DataTypeContents::NBT(NBTTypeContents::Float(right)) => {
								let FoldValue::Float(value) = &mut left.value else {
										bail!("Incorrect types");
									};
								*value = Some(*right);
							}
							DataTypeContents::NBT(NBTTypeContents::Double(right)) => {
								let FoldValue::Double(value) = &mut left.value else {
										bail!("Incorrect types");
									};
								*value = Some(*right);
							}
							DataTypeContents::NBT(NBTTypeContents::String(right)) => {
								let FoldValue::String(value) = &mut left.value else {
										bail!("Incorrect types");
									};
								*value = Some(right.to_string());
							}
							_ => continue,
						}
						instrs_to_remove.insert(i);
						left.has_folded = true;
						run_again = true;
					}
				}
			}
			MIRInstrKind::Add {
				left: MutableValue::Reg(left),
				right: Value::Constant(DataTypeContents::Score(right)),
			} => {
				if let Some(left) = fold_points.get_mut(left) {
					if !left.finished {
						if let FoldValue::Score(Some(value)) = &mut left.value {
							*value = value.overflowing_add(right.get_i32()).0;
							instrs_to_remove.insert(i);
							left.has_folded = true;
							run_again = true;
						}
					}
				}
			}
			MIRInstrKind::Sub {
				left: MutableValue::Reg(left),
				right: Value::Constant(DataTypeContents::Score(right)),
			} => {
				if let Some(left) = fold_points.get_mut(left) {
					if !left.finished {
						if let FoldValue::Score(Some(value)) = &mut left.value {
							*value = value.overflowing_sub(right.get_i32()).0;
							instrs_to_remove.insert(i);
							left.has_folded = true;
							run_again = true;
						}
					}
				}
			}
			MIRInstrKind::Mul {
				left: MutableValue::Reg(left),
				right: Value::Constant(DataTypeContents::Score(right)),
			} => {
				if let Some(left) = fold_points.get_mut(left) {
					if !left.finished {
						if let FoldValue::Score(Some(value)) = &mut left.value {
							*value = value.overflowing_mul(right.get_i32()).0;
							instrs_to_remove.insert(i);
							left.has_folded = true;
							run_again = true;
						}
					}
				}
			}
			MIRInstrKind::Div {
				left: MutableValue::Reg(left),
				right: Value::Constant(DataTypeContents::Score(right)),
			} => {
				if let Some(left) = fold_points.get_mut(left) {
					if !left.finished {
						if right.get_i32() != 0 {
							if let FoldValue::Score(Some(value)) = &mut left.value {
								*value /= right.get_i32();
								instrs_to_remove.insert(i);
								left.has_folded = true;
								run_again = true;
							}
						} else {
							left.finished = true;
						}
					}
				}
			}
			MIRInstrKind::Mod {
				left: MutableValue::Reg(left),
				right: Value::Constant(DataTypeContents::Score(right)),
			} => {
				if let Some(left) = fold_points.get_mut(left) {
					if !left.finished {
						if right.get_i32() != 0 {
							if let FoldValue::Score(Some(value)) = &mut left.value {
								*value %= right.get_i32();
								instrs_to_remove.insert(i);
								left.has_folded = true;
								run_again = true;
							}
						} else {
							left.finished = true;
						}
					}
				}
			}
			MIRInstrKind::Min {
				left: MutableValue::Reg(left),
				right: Value::Constant(DataTypeContents::Score(right)),
			} => {
				if let Some(left) = fold_points.get_mut(left) {
					if let FoldValue::Score(Some(value)) = &mut left.value {
						*value = std::cmp::min(*value, right.get_i32());
						instrs_to_remove.insert(i);
						left.has_folded = true;
						run_again = true;
					}
				}
			}
			MIRInstrKind::Max {
				left: MutableValue::Reg(left),
				right: Value::Constant(DataTypeContents::Score(right)),
			} => {
				if let Some(left) = fold_points.get_mut(left) {
					if !left.finished {
						if let FoldValue::Score(Some(value)) = &mut left.value {
							*value = std::cmp::max(*value, right.get_i32());
							instrs_to_remove.insert(i);
							left.has_folded = true;
							run_again = true;
						}
					}
				}
			}
			MIRInstrKind::Abs {
				val: MutableValue::Reg(left),
			} => {
				if let Some(left) = fold_points.get_mut(left) {
					if !left.finished {
						if let FoldValue::Score(Some(value)) = &mut left.value {
							*value = value.abs();
							instrs_to_remove.insert(i);
							left.has_folded = true;
							run_again = true;
						}
					}
				}
			}
			MIRInstrKind::Not {
				value: MutableValue::Reg(left),
			} => {
				if let Some(left) = fold_points.get_mut(left) {
					if !left.finished {
						if let FoldValue::Score(Some(value)) = &mut left.value {
							*value = (value != &0) as i32;
							instrs_to_remove.insert(i);
							left.has_folded = true;
							run_again = true;
						}
					}
				}
			}
			MIRInstrKind::And {
				left: MutableValue::Reg(left),
				right: Value::Constant(DataTypeContents::Score(right)),
			} => {
				if let Some(left) = fold_points.get_mut(left) {
					if !left.finished {
						if let FoldValue::Score(Some(value)) = &mut left.value {
							*value = ((value != &0) && (right.get_i32() != 0)) as i32;
							instrs_to_remove.insert(i);
							left.has_folded = true;
							run_again = true;
						}
					}
				}
			}
			MIRInstrKind::Or {
				left: MutableValue::Reg(left),
				right: Value::Constant(DataTypeContents::Score(right)),
			} => {
				if let Some(left) = fold_points.get_mut(left) {
					if !left.finished {
						if let FoldValue::Score(Some(value)) = &mut left.value {
							*value = ((value != &0) || (right.get_i32() != 0)) as i32;
							instrs_to_remove.insert(i);
							left.has_folded = true;
							run_again = true;
						}
					}
				}
			}
			MIRInstrKind::Pow {
				base: MutableValue::Reg(left),
				exp,
			} => {
				if let Some(left) = fold_points.get_mut(left) {
					if !left.finished {
						if let FoldValue::Score(Some(value)) = &mut left.value {
							*value = value.overflowing_pow((*exp).into()).0;
							instrs_to_remove.insert(i);
							left.has_folded = true;
							run_again = true;
						}
					}
				}
			}
			// Mul or div a value that is const as zero
			MIRInstrKind::Mul {
				left: MutableValue::Reg(left),
				right: Value::Mutable(..),
			}
			| MIRInstrKind::Div {
				left: MutableValue::Reg(left),
				right: Value::Mutable(..),
			} => {
				if let Some(left) = fold_points.get_mut(left) {
					if !left.finished {
						if let FoldValue::Score(Some(value)) = &mut left.value {
							if value == &0 {
								instrs_to_remove.insert(i);
								left.has_folded = true;
								run_again = true;
							}
						}
					}
				}
			}
			_ => {}
		};
		let an_result = an.feed(&instr.kind)?;
		match an_result {
			ConstAnalyzerResult::Add(reg, val) => {
				let val = match val {
					ConstAnalyzerValue::Value(val) => match val {
						DataTypeContents::Score(val) => FoldValue::Score(Some(val.get_i32())),
						DataTypeContents::NBT(NBTTypeContents::Byte(val)) => {
							FoldValue::Byte(Some(val))
						}
						DataTypeContents::NBT(NBTTypeContents::Short(val)) => {
							FoldValue::Short(Some(val))
						}
						DataTypeContents::NBT(NBTTypeContents::Int(val)) => {
							FoldValue::Int(Some(val))
						}
						DataTypeContents::NBT(NBTTypeContents::Long(val)) => {
							FoldValue::Long(Some(val))
						}
						DataTypeContents::NBT(NBTTypeContents::Float(val)) => {
							FoldValue::Float(Some(val))
						}
						DataTypeContents::NBT(NBTTypeContents::Double(val)) => {
							FoldValue::Double(Some(val))
						}
						DataTypeContents::NBT(NBTTypeContents::String(val)) => {
							FoldValue::String(Some(val.to_string()))
						}
						_ => continue,
					},
					ConstAnalyzerValue::Reset(ty) => match ty {
						DataType::Score(..) => FoldValue::Score(None),
						DataType::NBT(NBTType::Byte) => FoldValue::Byte(None),
						DataType::NBT(NBTType::Short) => FoldValue::Short(None),
						DataType::NBT(NBTType::Int) => FoldValue::Int(None),
						DataType::NBT(NBTType::Long) => FoldValue::Long(None),
						DataType::NBT(NBTType::Float) => FoldValue::Float(None),
						DataType::NBT(NBTType::Double) => FoldValue::Double(None),
						DataType::NBT(NBTType::String) => FoldValue::String(None),
						_ => continue,
					},
				};

				if let Some(existing) = fold_points.get_mut(&reg) {
					instrs_to_replace
						.push((existing.pos, create_set_instr(&reg, existing.value.clone())));
					existing.pos = i;
					existing.value = val;
					existing.finished = true;
				} else {
					fold_points.insert(
						reg,
						FoldPoint {
							pos: i,
							value: val,
							finished: false,
							has_folded: false,
						},
					);
				}
			}
			ConstAnalyzerResult::Remove(regs) => {
				for reg in regs {
					// let mut should_remove = false;
					if let Some(point) = fold_points.get_mut(reg) {
						point.finished = true;
						// if !point.finished {
						// 	instrs_to_replace.push((
						// 		point.pos,
						// 		MIRInstrKind::Assign {
						// 			left: MutableValue::Reg(reg.clone()),
						// 			right: DeclareBinding::Value(Value::Constant(
						// 				DataTypeContents::Score(ScoreTypeContents::Score(
						// 					point.value,
						// 				)),
						// 			)),
						// 		},
						// 	));
						// 	should_remove = true;
						// }
					}
					// if should_remove {
					// 	fold_points.remove(&reg);
					// }
				}
			}
			_ => (),
		}
	}

	for (reg, point) in fold_points.iter() {
		if let Some(instr) = block.contents.get_mut(point.pos) {
			instr.kind = create_set_instr(reg, point.value.clone());
		} else {
			bail!("Fold position out of range");
		}
	}
	// Replace instructions
	for (pos, instr) in instrs_to_replace {
		if let Some(existing) = block.contents.get_mut(pos) {
			existing.kind = instr;
		} else {
			bail!("Fold position out of range");
		}
	}

	Ok(run_again)
}

fn create_set_instr(reg: &Identifier, val: FoldValue) -> MIRInstrKind {
	if let Some(data) = val.to_contents() {
		MIRInstrKind::Assign {
			left: MutableValue::Reg(reg.clone()),
			right: DeclareBinding::Value(Value::Constant(data)),
		}
	} else {
		MIRInstrKind::Remove {
			val: MutableValue::Reg(reg.clone()),
		}
	}
}

struct FoldPoint {
	pos: usize,
	value: FoldValue,
	finished: bool,
	has_folded: bool,
}

impl Debug for FoldPoint {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(
			f,
			"{} = {:?}, Finished: {}, Folded: {}",
			self.pos, self.value, self.finished, self.has_folded
		)
	}
}

#[derive(Debug, Clone)]
enum FoldValue {
	Score(Option<i32>),
	Byte(Option<Byte>),
	Short(Option<Short>),
	Int(Option<Int>),
	Long(Option<Long>),
	Float(Option<Float>),
	Double(Option<Double>),
	String(Option<String>),
}

impl FoldValue {
	fn to_contents(self) -> Option<DataTypeContents> {
		match self {
			Self::Score(val) => val.map(|x| DataTypeContents::Score(ScoreTypeContents::Score(x))),
			Self::Byte(val) => val.map(|x| DataTypeContents::NBT(NBTTypeContents::Byte(x))),
			Self::Short(val) => val.map(|x| DataTypeContents::NBT(NBTTypeContents::Short(x))),
			Self::Int(val) => val.map(|x| DataTypeContents::NBT(NBTTypeContents::Int(x))),
			Self::Long(val) => val.map(|x| DataTypeContents::NBT(NBTTypeContents::Long(x))),
			Self::Float(val) => val.map(|x| DataTypeContents::NBT(NBTTypeContents::Float(x))),
			Self::Double(val) => val.map(|x| DataTypeContents::NBT(NBTTypeContents::Double(x))),
			Self::String(val) => {
				val.map(|x| DataTypeContents::NBT(NBTTypeContents::String(x.into())))
			}
		}
	}
}
