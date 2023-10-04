use anyhow::{anyhow, bail};

use crate::common::modifier::{IfModCondition, IfScoreCondition, IfScoreRangeEnd, Modifier};
use crate::common::ty::{get_op_tys, ArraySize, DataType, DataTypeContents, ScoreTypeContents};
use crate::common::{DeclareBinding, Identifier, MutableValue, Register, ScoreValue, Value, RegisterList};
use crate::lir::{LIRBlock, LIRInstrKind, LIRInstruction, LIR};
use crate::mir::{MIRInstrKind, MIR};

/// Lower IR to LIR
pub fn lower_mir(mut mir: MIR) -> anyhow::Result<LIR> {
	let mut lir = LIR::new();

	for (interface, block) in mir.functions {
		let block = mir
			.blocks
			.remove(&block)
			.ok_or(anyhow!("Block does not exist"))?;
		let mut lir_instrs = Vec::new();

		let mut lbcx = LowerBlockCx::new();

		for ir_instr in block.contents {
			match ir_instr.kind {
				MIRInstrKind::Declare { left, ty } => {
					// Add as a register
					let reg = Register {
						id: left.clone(),
						ty,
					};
					lbcx.registers.insert(left.clone(), reg);
				}
				MIRInstrKind::Assign { left, right } => {
					lir_instrs.extend(lower_assign(left, right, &mut lbcx)?);
				}
				MIRInstrKind::Add { left, right } => {
					lir_instrs.push(LIRInstruction::new(lower_add(left, right, &lbcx)?));
				}
				MIRInstrKind::Sub { left, right } => {
					lir_instrs.push(LIRInstruction::new(lower_sub(left, right, &lbcx)?));
				}
				MIRInstrKind::Mul { left, right } => {
					lir_instrs.push(LIRInstruction::new(lower_mul(left, right, &lbcx)?));
				}
				MIRInstrKind::Div { left, right } => {
					lir_instrs.push(LIRInstruction::new(lower_div(left, right, &lbcx)?));
				}
				MIRInstrKind::Mod { left, right } => {
					lir_instrs.push(LIRInstruction::new(lower_mod(left, right, &lbcx)?));
				}
				MIRInstrKind::Min { left, right } => {
					lir_instrs.push(LIRInstruction::new(lower_min(left, right, &lbcx)?));
				}
				MIRInstrKind::Max { left, right } => {
					lir_instrs.push(LIRInstruction::new(lower_max(left, right, &lbcx)?));
				}
				MIRInstrKind::Swap { left, right } => {
					lir_instrs.extend(lower_swap(left, right, &mut lbcx)?);
				}
				MIRInstrKind::Abs { val } => {
					lir_instrs.push(lower_abs(val, &lbcx)?);
				}
				MIRInstrKind::Use { val } => {
					lir_instrs.push(LIRInstruction::new(LIRInstrKind::Use(val)));
				}
			}
		}

		let mut lir_block = LIRBlock::new(lbcx.registers);
		lir_block.contents = lir_instrs;

		let id = lir.blocks.add(lir_block);
		lir.functions.insert(interface, id);
	}

	Ok(lir)
}

struct LowerBlockCx {
	registers: RegisterList,
	additional_reg_count: u32,
}

impl LowerBlockCx {
	fn new() -> Self {
		Self {
			registers: RegisterList::new(),
			additional_reg_count: 0,
		}
	}

	fn new_additional_reg(&mut self) -> Identifier {
		let old_val = self.additional_reg_count;
		self.additional_reg_count += 1;
		Identifier::from(format!("__lir_lower_{old_val}"))
	}
}

fn lower_assign(
	left: MutableValue,
	right: DeclareBinding,
	lbcx: &mut LowerBlockCx,
) -> anyhow::Result<Vec<LIRInstruction>> {
	let mut out = Vec::new();

	let left_ty = left.get_ty(&lbcx.registers)?;

	let right_val = match &right {
		DeclareBinding::Value(val) => val.clone(),
		DeclareBinding::Cast(ty, val) => {
			let val_ty = val.get_ty(&lbcx.registers)?;
			// If the cast is not trivial, we have to declare a new register,
			// initialize it with the cast, and then assign the result to our declaration
			let assign_val = if val_ty.is_trivially_castable(&ty) {
				val.clone()
			} else {
				let new_reg = lbcx.new_additional_reg();
				// Declare the new register
				let reg = Register {
					id: new_reg.clone(),
					ty: ty.clone(),
				};
				lbcx.registers.insert(new_reg.clone(), reg);

				// Run the cast
				let cast_instr = match (ty, val_ty) {
					(DataType::Score(score), DataType::NBT(nbt)) => {
						if nbt.is_castable_to_score(&score) {
							LIRInstrKind::SetScore(
								MutableValue::Register(new_reg.clone()),
								Value::Mutable(val.clone()),
							)
						} else {
							bail!("Impossible non-trivial cast");
						}
					}
					_ => bail!("Impossible non-trivial cast"),
				};
				out.push(LIRInstruction::new(cast_instr));
				MutableValue::Register(new_reg)
			};
			Value::Mutable(assign_val)
		}
		DeclareBinding::Index { ty, val, index } => {
			let new_reg = lbcx.new_additional_reg();
			// Declare the new register
			let reg = Register {
				id: new_reg.clone(),
				ty: ty.clone(),
			};
			lbcx.registers.insert(new_reg.clone(), reg);

			// Add the index instruction
			match (val.get_ty(&lbcx.registers)?, index) {
				(DataType::NBT(..), Value::Constant(DataTypeContents::Score(score))) => {
					let index = match score {
						ScoreTypeContents::Bool(val) => *val as ArraySize,
						ScoreTypeContents::UScore(val) => *val as ArraySize,
						_ => bail!("Non-score type cannot be used as index"),
					};
					out.push(LIRInstruction::new(LIRInstrKind::ConstIndexToScore {
						score: MutableValue::Register(new_reg.clone()),
						value: val.clone(),
						index,
					}));
				}
				_ => bail!("Cannot use index declaration with these types"),
			}
			Value::Mutable(MutableValue::Register(new_reg))
		}
	};

	let kind = match left_ty {
		DataType::Score(..) => LIRInstrKind::SetScore(left, right_val),
		DataType::NBT(..) => LIRInstrKind::SetData(left, right_val),
	};
	out.push(LIRInstruction::new(kind));

	Ok(out)
}

fn lower_add(
	left: MutableValue,
	right: Value,
	lbcx: &LowerBlockCx,
) -> anyhow::Result<LIRInstrKind> {
	let tys = get_op_tys(&left, &right, &lbcx.registers)?;
	let kind = match tys {
		(DataType::Score(..), DataType::Score(..)) => LIRInstrKind::AddScore(left, right),
		_ => bail!("Instruction does not allow this type"),
	};

	Ok(kind)
}

fn lower_sub(
	left: MutableValue,
	right: Value,
	lbcx: &LowerBlockCx,
) -> anyhow::Result<LIRInstrKind> {
	let tys = get_op_tys(&left, &right, &lbcx.registers)?;
	let kind = match tys {
		(DataType::Score(..), DataType::Score(..)) => LIRInstrKind::SubScore(left, right),
		_ => bail!("Instruction does not allow this type"),
	};

	Ok(kind)
}

fn lower_mul(
	left: MutableValue,
	right: Value,
	lbcx: &LowerBlockCx,
) -> anyhow::Result<LIRInstrKind> {
	let tys = get_op_tys(&left, &right, &lbcx.registers)?;
	let kind = match tys {
		(DataType::Score(..), DataType::Score(..)) => LIRInstrKind::MulScore(left, right),
		_ => bail!("Instruction does not allow this type"),
	};

	Ok(kind)
}

fn lower_div(
	left: MutableValue,
	right: Value,
	lbcx: &LowerBlockCx,
) -> anyhow::Result<LIRInstrKind> {
	let tys = get_op_tys(&left, &right, &lbcx.registers)?;
	let kind = match tys {
		(DataType::Score(..), DataType::Score(..)) => LIRInstrKind::DivScore(left, right),
		_ => bail!("Instruction does not allow this type"),
	};

	Ok(kind)
}

fn lower_mod(
	left: MutableValue,
	right: Value,
	lbcx: &LowerBlockCx,
) -> anyhow::Result<LIRInstrKind> {
	let tys = get_op_tys(&left, &right, &lbcx.registers)?;
	let kind = match tys {
		(DataType::Score(..), DataType::Score(..)) => LIRInstrKind::ModScore(left, right),
		_ => bail!("Instruction does not allow this type"),
	};

	Ok(kind)
}

fn lower_min(
	left: MutableValue,
	right: Value,
	lbcx: &LowerBlockCx,
) -> anyhow::Result<LIRInstrKind> {
	let tys = get_op_tys(&left, &right, &lbcx.registers)?;
	let kind = match tys {
		(DataType::Score(..), DataType::Score(..)) => LIRInstrKind::MinScore(left, right),
		_ => bail!("Instruction does not allow this type"),
	};

	Ok(kind)
}

fn lower_max(
	left: MutableValue,
	right: Value,
	lbcx: &LowerBlockCx,
) -> anyhow::Result<LIRInstrKind> {
	let tys = get_op_tys(&left, &right, &lbcx.registers)?;
	let kind = match tys {
		(DataType::Score(..), DataType::Score(..)) => LIRInstrKind::MaxScore(left, right),
		_ => bail!("Instruction does not allow this type"),
	};

	Ok(kind)
}

fn lower_swap(
	left: MutableValue,
	right: MutableValue,
	lbcx: &mut LowerBlockCx,
) -> anyhow::Result<Vec<LIRInstruction>> {
	let mut out = Vec::new();

	match (
		left.get_ty(&lbcx.registers)?,
		right.get_ty(&lbcx.registers)?,
	) {
		(DataType::Score(..), DataType::Score(..)) => {
			out.push(LIRInstruction::new(LIRInstrKind::SwapScore(left, right)))
		}
		(DataType::NBT(left_ty), DataType::NBT(..)) => {
			// Create a temporary register to store into
			let temp_reg = lbcx.new_additional_reg();
			let reg = Register {
				id: temp_reg.clone(),
				ty: DataType::NBT(left_ty),
			};
			lbcx.registers.insert(temp_reg.clone(), reg);
			// Create the three assignments that represent the swap.
			// This is equal to: temp = a; a = b; b = temp;
			out.push(LIRInstruction::new(LIRInstrKind::SetData(
				MutableValue::Register(temp_reg.clone()),
				Value::Mutable(left.clone()),
			)));
			out.push(LIRInstruction::new(LIRInstrKind::SetData(
				left.clone(),
				Value::Mutable(right.clone()),
			)));
			out.push(LIRInstruction::new(LIRInstrKind::SetData(
				right.clone(),
				Value::Mutable(MutableValue::Register(temp_reg.clone())),
			)));
		}
		_ => bail!("Instruction does not allow this type"),
	};

	Ok(out)
}

fn lower_abs(val: MutableValue, lbcx: &LowerBlockCx) -> anyhow::Result<LIRInstruction> {
	let kind = match val.get_ty(&lbcx.registers)? {
		DataType::Score(..) => LIRInstrKind::MulScore(
			val.clone(),
			Value::Constant(DataTypeContents::Score(ScoreTypeContents::Score(-1))),
		),
		_ => bail!("Instruction does not allow this type"),
	};

	let modifier = Modifier::If {
		condition: Box::new(IfModCondition::Score(IfScoreCondition::Range {
			score: val.to_mutable_score_value(),
			left: IfScoreRangeEnd::Infinite,
			right: IfScoreRangeEnd::Fixed {
				value: ScoreValue::Constant(ScoreTypeContents::Score(-1)),
				inclusive: true,
			},
		})),
		negate: false,
	};

	let instr = LIRInstruction::with_modifiers(kind, vec![modifier]);

	Ok(instr)
}
