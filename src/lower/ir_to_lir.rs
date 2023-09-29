use std::collections::HashMap;

use crate::common::ty::{get_op_tys, DataType};
use crate::common::{DeclareBinding, Identifier, MutableValue, Register, Value};
use crate::ir::{InstrKind, IR};
use crate::lir::{LIRBlock, LIRInstrKind, LIRInstruction, LIR};

/// Lower IR to LIR
pub fn lower_ir(ir: IR) -> anyhow::Result<LIR> {
	let mut lir = LIR::new();

	for (interface, block) in ir.functions {
		let mut lir_block = LIRBlock::new();

		let mut lbcx = LowerBlockCx::new();

		for ir_instr in block.contents {
			match ir_instr.kind {
				InstrKind::Declare { left, ty, right } => {
					// Add as a register
					let reg = Register {
						id: left.clone(),
						ty,
					};
					lbcx.registers.insert(left.clone(), reg);

					// Assign the value
					let instrs = lower_declare(MutableValue::Register(left), right, &mut lbcx)?;
					lir_block.contents.extend(instrs);
				}
				InstrKind::Assign { left, right } => {
					lir_block
						.contents
						.push(LIRInstruction::new(lower_assign(left, right, &lbcx)?));
				}
				InstrKind::Add { left, right } => {
					lir_block
						.contents
						.push(LIRInstruction::new(lower_add(left, right, &lbcx)?));
				}
				InstrKind::Sub { left, right } => {
					lir_block
						.contents
						.push(LIRInstruction::new(lower_sub(left, right, &lbcx)?));
				}
				InstrKind::Mul { left, right } => {
					lir_block
						.contents
						.push(LIRInstruction::new(lower_mul(left, right, &lbcx)?));
				}
				InstrKind::Div { left, right } => {
					lir_block
						.contents
						.push(LIRInstruction::new(lower_div(left, right, &lbcx)?));
				}
				InstrKind::Mod { left, right } => {
					lir_block
						.contents
						.push(LIRInstruction::new(lower_mod(left, right, &lbcx)?));
				}
				InstrKind::Swap { left, right } => {
					lir_block
						.contents
						.push(LIRInstruction::new(lower_swap(left, right, &lbcx)?));
				}
			}
		}

		lir.functions.insert(interface, lir_block);
	}

	Ok(lir)
}

struct LowerBlockCx {
	registers: HashMap<Identifier, Register>,
	additional_reg_count: u32,
}

impl LowerBlockCx {
	fn new() -> Self {
		Self {
			registers: HashMap::new(),
			additional_reg_count: 0,
		}
	}

	fn new_additional_reg(&mut self) -> Identifier {
		let old_val = self.additional_reg_count;
		self.additional_reg_count += 1;
		Identifier::from(format!("__lir_lower_{old_val}"))
	}
}

fn lower_declare(
	left: MutableValue,
	right: DeclareBinding,
	lbcx: &mut LowerBlockCx,
) -> anyhow::Result<Vec<LIRInstruction>> {
	let mut out = Vec::new();

	let kind = match (
		left.get_ty(&lbcx.registers)?,
		right.get_ty(&lbcx.registers)?,
	) {
		(DataType::Score(..), DataType::Score(..)) => match right {
			DeclareBinding::Cast(ty, val) => {
				let val_ty = val.get_ty(&lbcx.registers)?;
				// If the cast is not trivial, we have to declare a new register,
				// initialize it with the cast, and then assign the result to our declaration
				let assign_val = if val_ty.is_trivially_castable(&ty) {
					val
				} else {
					// No non-trivial casts at the moment
					let new_reg = MutableValue::Register(lbcx.new_additional_reg());
					let instr = LIRInstruction::new(LIRInstrKind::SetScore(
						new_reg.clone(),
						Value::Mutable(val),
					));
					out.push(instr);
					new_reg
				};
				LIRInstrKind::SetScore(left, Value::Mutable(assign_val))
			}
			DeclareBinding::Value(val) => LIRInstrKind::SetScore(left, val),
		},
	};
	out.push(LIRInstruction::new(kind));

	Ok(out)
}

fn lower_assign(
	left: MutableValue,
	right: Value,
	lbcx: &LowerBlockCx,
) -> anyhow::Result<LIRInstrKind> {
	let types = get_op_tys(&left, &right, &lbcx.registers)?;
	let kind = match types {
		(DataType::Score(..), DataType::Score(..)) => LIRInstrKind::SetScore(left, right),
	};

	Ok(kind)
}

fn lower_add(
	left: MutableValue,
	right: Value,
	lbcx: &LowerBlockCx,
) -> anyhow::Result<LIRInstrKind> {
	let tys = get_op_tys(&left, &right, &lbcx.registers)?;
	let kind = match tys {
		(DataType::Score(..), DataType::Score(..)) => LIRInstrKind::AddScore(left, right),
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
	};

	Ok(kind)
}

fn lower_swap(
	left: MutableValue,
	right: MutableValue,
	lbcx: &LowerBlockCx,
) -> anyhow::Result<LIRInstrKind> {
	let kind = match (
		left.get_ty(&lbcx.registers)?,
		right.get_ty(&lbcx.registers)?,
	) {
		(DataType::Score(..), DataType::Score(..)) => LIRInstrKind::SwapScore(left, right),
	};

	Ok(kind)
}
