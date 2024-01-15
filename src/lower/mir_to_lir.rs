use anyhow::{bail, Context};

use crate::common::condition::Condition;
use crate::common::function::{FunctionInterface, FunctionSignature};
use crate::common::mc::modifier::{
	IfModCondition, IfScoreCondition, IfScoreRangeEnd, Modifier, StoreDataType, StoreModLocation,
};
use crate::common::ty::{
	get_op_tys, ArraySize, DataType, DataTypeContents, Double, ScoreType, ScoreTypeContents,
};
use crate::common::ResourceLocation;
use crate::common::{
	val::MutableNBTValue, val::MutableScoreValue, val::MutableValue, val::NBTValue,
	val::ScoreValue, val::Value, DeclareBinding, Identifier, Register, RegisterList,
};
use crate::lir::{LIRBlock, LIRFunction, LIRInstrKind, LIRInstruction, LIR};
use crate::mir::{MIRBlock, MIRInstrKind, MIR};

macro_rules! lower {
	($instrs:expr, $kind:ident) => {
		lower!($instrs, LIRInstrKind::$kind)
	};

	($instrs:expr, $kind:ident, $($arg:expr),+) => {
		lower!($instrs, LIRInstrKind::$kind($($arg),+))
	};

	($instrs:expr, $val:expr) => {
		$instrs
			.push(LIRInstruction::new($val))
	};
}

/// Lower MIR to LIR
pub fn lower_mir(mir: MIR) -> anyhow::Result<LIR> {
	let mut lir = LIR::with_capacity(mir.functions.len());
	for (func_id, func) in mir.functions {
		let block = func.block;
		let mut lir_instrs = Vec::with_capacity(block.contents.len());

		let mut lbcx = LowerBlockCx::new(
			&mut lir,
			func.interface.id.clone(),
			func.interface.sig.clone(),
		);

		for mir_instr in block.contents {
			lower_kind(mir_instr.kind, &mut lir_instrs, &mut lbcx)?;
		}

		let mut lir_block = LIRBlock::new(lbcx.registers);
		lir_block.contents = lir_instrs;

		lir.functions.insert(
			func_id,
			LIRFunction {
				interface: func.interface,
				block: lir_block,
				parent: None,
			},
		);
	}

	Ok(lir)
}

fn lower_kind(
	kind: MIRInstrKind,
	lir_instrs: &mut Vec<LIRInstruction>,
	lbcx: &mut LowerBlockCx,
) -> anyhow::Result<()> {
	match kind {
		MIRInstrKind::Declare { left, ty } => {
			// Add as a register
			let reg = Register {
				id: left.clone(),
				ty,
			};
			lbcx.registers.insert(left.clone(), reg);
		}
		MIRInstrKind::Assign { left, right } => {
			lir_instrs.extend(lower_assign(left, right, lbcx)?);
		}
		MIRInstrKind::Add { left, right } => {
			lir_instrs.push(LIRInstruction::new(lower_add(left, right, lbcx)?));
		}
		MIRInstrKind::Sub { left, right } => {
			lir_instrs.push(LIRInstruction::new(lower_sub(left, right, lbcx)?));
		}
		MIRInstrKind::Mul { left, right } => {
			lir_instrs.push(LIRInstruction::new(lower_mul(left, right, lbcx)?));
		}
		MIRInstrKind::Div { left, right } => {
			lir_instrs.push(LIRInstruction::new(lower_div(left, right, lbcx)?));
		}
		MIRInstrKind::Mod { left, right } => {
			lir_instrs.push(LIRInstruction::new(lower_mod(left, right, lbcx)?));
		}
		MIRInstrKind::Min { left, right } => {
			lir_instrs.push(LIRInstruction::new(lower_min(left, right, lbcx)?));
		}
		MIRInstrKind::Max { left, right } => {
			lir_instrs.push(LIRInstruction::new(lower_max(left, right, lbcx)?));
		}
		MIRInstrKind::Swap { left, right } => {
			lir_instrs.extend(lower_swap(left, right, lbcx)?);
		}
		MIRInstrKind::Abs { val } => {
			lir_instrs.push(lower_abs(val, lbcx)?);
		}
		MIRInstrKind::Get { value, scale } => {
			lir_instrs.push(LIRInstruction::new(lower_get(value, scale, lbcx)?));
		}
		MIRInstrKind::Merge { left, right } => {
			lir_instrs.push(LIRInstruction::new(lower_merge(left, right, lbcx)?));
		}
		MIRInstrKind::Push { left, right } => {
			lir_instrs.push(LIRInstruction::new(lower_push(left, right, lbcx)?));
		}
		MIRInstrKind::PushFront { left, right } => {
			lir_instrs.push(LIRInstruction::new(lower_push_front(left, right, lbcx)?));
		}
		MIRInstrKind::Insert { left, right, index } => {
			lir_instrs.push(LIRInstruction::new(lower_insert(left, right, index, lbcx)?));
		}
		MIRInstrKind::Not { value } => {
			let mut instr = LIRInstruction::new(LIRInstrKind::NoOp);
			let store_loc =
				StoreModLocation::from_mut_val(value.clone(), &lbcx.registers, &lbcx.sig)?;
			instr.modifiers.push(Modifier::StoreSuccess(store_loc));
			instr.modifiers.push(Modifier::If {
				condition: Box::new(IfModCondition::Score(IfScoreCondition::Single {
					left: ScoreValue::Mutable(value.to_mutable_score_value()?),
					right: ScoreValue::Constant(ScoreTypeContents::Bool(false)),
				})),
				negate: false,
			});
			lir_instrs.push(instr);
		}
		MIRInstrKind::And { left, right } => {
			lower!(
				lir_instrs,
				MulScore,
				left.to_mutable_score_value()?,
				right.to_score_value()?
			);
		}
		MIRInstrKind::Or { left, right } => {
			lower!(
				lir_instrs,
				AddScore,
				left.clone().to_mutable_score_value()?,
				right.to_score_value()?
			);
			lower!(
				lir_instrs,
				DivScore,
				left.clone().to_mutable_score_value()?,
				ScoreValue::Mutable(left.to_mutable_score_value()?)
			);
		}
		MIRInstrKind::Use { val } => lower!(lir_instrs, Use, val),
		MIRInstrKind::GetConst { value } => lower!(lir_instrs, GetConst, value),
		MIRInstrKind::Call { call } => {
			// Set the arguments
			for (i, arg) in call.args.iter().enumerate() {
				let instrs = lower_assign(
					MutableValue::CallArg(
						i.try_into().expect("This should fit"),
						call.function.clone(),
						arg.get_ty(&lbcx.registers, &lbcx.sig)?,
					),
					DeclareBinding::Value(arg.clone()),
					lbcx,
				)
				.context("Failed to lower argument assignment")?;
				lir_instrs.extend(instrs);
			}
			lower!(lir_instrs, Call, call.function.clone());
			// Set the return values
			for (i, ret) in call.ret.iter().enumerate() {
				let instrs = lower_assign(
					ret.clone(),
					DeclareBinding::Value(Value::Mutable(MutableValue::CallReturnValue(
						i.try_into().expect("This should fit"),
						call.function.clone(),
						ret.get_ty(&lbcx.registers, &lbcx.sig)?,
					))),
					lbcx,
				)
				.context("Failed to lower return value assignment")?;
				lir_instrs.extend(instrs);
			}
		}
		MIRInstrKind::CallExtern { func } => lower!(lir_instrs, Call, func),
		MIRInstrKind::Remove { val } => lir_instrs.push(LIRInstruction::new(lower_rm(val, lbcx)?)),
		MIRInstrKind::Pow { base, exp } => {
			lower_pow(base, exp, lir_instrs, lbcx)?;
		}
		MIRInstrKind::If { condition, body } => {
			let (prepend, conditions) =
				lower_condition(condition, lbcx).context("Failed to lower condition")?;
			lir_instrs.extend(prepend);

			let mut instr = lower_subblock(*body, lbcx).context("Failed to lower if body")?;

			let prepend = conditions
				.into_iter()
				.map(|(condition, negate)| Modifier::If {
					condition: Box::new(condition),
					negate,
				});
			instr.modifiers = prepend.chain(instr.modifiers.into_iter()).collect();
			lir_instrs.push(instr);
		}
		MIRInstrKind::As { target, body } => {
			let mut instr = lower_subblock(*body, lbcx).context("Failed to lower as body")?;

			instr.modifiers.insert(0, Modifier::As(target));
			lir_instrs.push(instr);
		}
		MIRInstrKind::At { target, body } => {
			let mut instr = lower_subblock(*body, lbcx).context("Failed to lower at body")?;

			instr.modifiers.insert(0, Modifier::At(target));
			lir_instrs.push(instr);
		}
		MIRInstrKind::StoreResult { location, body } => {
			let mut instr = lower_subblock(*body, lbcx).context("Failed to lower str body")?;

			instr.modifiers.insert(0, Modifier::StoreResult(location));
			lir_instrs.push(instr);
		}
		MIRInstrKind::StoreSuccess { location, body } => {
			let mut instr = lower_subblock(*body, lbcx).context("Failed to lower sts body")?;

			instr.modifiers.insert(0, Modifier::StoreSuccess(location));
			lir_instrs.push(instr);
		}
		MIRInstrKind::Positioned { position, body } => {
			let mut instr = lower_subblock(*body, lbcx).context("Failed to lower pos body")?;

			instr.modifiers.insert(0, Modifier::Positioned(position));
			lir_instrs.push(instr);
		}
		MIRInstrKind::ReturnRun { body } => {
			let instr = lower_subblock(*body, lbcx).context("Failed to lower retr body")?;

			lir_instrs.push(LIRInstruction::new(LIRInstrKind::ReturnRun(Box::new(
				instr,
			))));
		}
		MIRInstrKind::ReturnValue { index, value } => {
			let instrs = lower_assign(
				MutableValue::ReturnValue(index),
				DeclareBinding::Value(value),
				lbcx,
			)
			.context("Failed to lower return value assignment")?;
			lir_instrs.extend(instrs);
		}
		MIRInstrKind::Return { value } => match value {
			Value::Constant(val) => {
				let Some(val) = val.try_get_i32() else {
					bail!("Value is not castable to an i32");
				};
				lower!(lir_instrs, ReturnValue, val)
			}
			Value::Mutable(val) => {
				let subinstr = lower_get(val, 1.0, lbcx).context("Failed to lower get")?;
				let subinstr = LIRInstruction::new(subinstr);
				lower!(lir_instrs, ReturnRun, Box::new(subinstr))
			}
		},
		MIRInstrKind::NoOp => {}
		MIRInstrKind::Command { command } => lower!(lir_instrs, Command, command),
		MIRInstrKind::Comment { comment } => lower!(lir_instrs, Comment, comment),
		MIRInstrKind::MC(instr) => lower!(lir_instrs, MC, instr),
	}
	Ok(())
}

struct LowerBlockCx<'lir> {
	lir: &'lir mut LIR,
	registers: RegisterList,
	additional_reg_count: u32,
	body_count: u32,
	func_id: ResourceLocation,
	sig: FunctionSignature,
}

impl<'lir> LowerBlockCx<'lir> {
	fn new(lir: &'lir mut LIR, func_id: ResourceLocation, sig: FunctionSignature) -> Self {
		Self {
			lir,
			registers: RegisterList::default(),
			additional_reg_count: 0,
			body_count: 0,
			func_id,
			sig,
		}
	}

	fn new_additional_reg(&mut self) -> Identifier {
		let old_val = self.additional_reg_count;
		self.additional_reg_count += 1;
		Identifier::from(format!("__lir_lower_{old_val}"))
	}

	fn new_body_fn(&mut self) -> FunctionInterface {
		let old_val = self.body_count;
		self.body_count += 1;
		FunctionInterface::new(format!("{}_body_{old_val}", self.func_id).into())
	}
}

fn lower_assign(
	left: MutableValue,
	right: DeclareBinding,
	lbcx: &mut LowerBlockCx,
) -> anyhow::Result<Vec<LIRInstruction>> {
	let mut out = Vec::new();

	let left_ty = left.get_ty(&lbcx.registers, &lbcx.sig)?;

	let right_val = match &right {
		DeclareBinding::Null => None,
		DeclareBinding::Value(val) => Some(val.clone()),
		DeclareBinding::Cast(ty, val) => {
			let val_ty = val.get_ty(&lbcx.registers, &lbcx.sig)?;
			// If the cast is not trivial, or they are not both
			// score types, we have to declare a new register,
			// initialize it with the cast, and then assign the result to our declaration

			if val_ty.is_trivially_castable(ty)
				|| matches!((&val_ty, &ty), (DataType::Score(..), DataType::Score(..)))
			{
				Some(Value::Mutable(val.clone()))
			} else {
				// Run the cast
				let store_loc = match ty {
					DataType::Score(..) => StoreModLocation::from_mut_score_val(
						&left.clone().to_mutable_score_value()?,
					)?,
					DataType::NBT(..) => {
						let DataType::NBT(ty) = ty else {
							bail!("Type is not a valid storage type");
						};
						let ty = StoreDataType::from_nbt_ty(ty)
							.context("Type is not a valid storage type")?;
						StoreModLocation::from_mut_nbt_val(
							&left.clone().to_mutable_nbt_value()?,
							ty,
							1.0,
						)?
					}
					_ => bail!("Type not supported"),
				};

				let get_instr = match val_ty {
					DataType::Score(..) => {
						LIRInstrKind::GetScore(val.clone().to_mutable_score_value()?)
					}
					DataType::NBT(..) => {
						LIRInstrKind::GetData(val.clone().to_mutable_nbt_value()?, 1.0)
					}
					_ => bail!("Type not supported"),
				};
				out.push(LIRInstruction::with_modifiers(
					get_instr,
					vec![Modifier::StoreResult(store_loc)],
				));

				None
			}
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
			match (val.get_ty(&lbcx.registers, &lbcx.sig)?, index) {
				(DataType::NBT(..), Value::Constant(DataTypeContents::Score(score))) => {
					let index = match score {
						ScoreTypeContents::Score(val) => *val as ArraySize,
						ScoreTypeContents::Bool(val) => *val as ArraySize,
					};
					out.push(LIRInstruction::new(LIRInstrKind::ConstIndexToScore {
						score: MutableScoreValue::Reg(new_reg.clone()),
						value: val.clone().to_nbt_value()?,
						index,
					}));
				}
				_ => bail!("Cannot use index declaration with these types"),
			}
			Some(Value::Mutable(MutableValue::Reg(new_reg)))
		}
		// Condition just becomes a simple execute store success {lhs} if {condition}
		DeclareBinding::Condition(cond) => {
			let store_loc = match left.get_ty(&lbcx.registers, &lbcx.sig)? {
				DataType::Score(..) => {
					StoreModLocation::from_mut_score_val(&left.clone().to_mutable_score_value()?)?
				}
				_ => bail!("Invalid type"),
			};
			let mut instr = LIRInstruction::new(LIRInstrKind::NoOp);
			instr.modifiers.push(Modifier::StoreSuccess(store_loc));
			let (prelude, conditions) = lower_condition(cond.clone(), lbcx)?;
			out.extend(prelude);
			for (condition, negate) in conditions {
				instr.modifiers.push(Modifier::If {
					condition: Box::new(condition),
					negate,
				});
			}
			out.push(instr);

			None
		}
	};

	if let Some(right_val) = right_val {
		let kind = match left_ty {
			DataType::Score(..) => {
				LIRInstrKind::SetScore(left.to_mutable_score_value()?, right_val.to_score_value()?)
			}
			DataType::NBT(..) => {
				LIRInstrKind::SetData(left.to_mutable_nbt_value()?, right_val.to_nbt_value()?)
			}
			_ => bail!("Type not supported"),
		};
		out.push(LIRInstruction::new(kind));
	}

	Ok(out)
}

fn lower_add(
	left: MutableValue,
	right: Value,
	lbcx: &LowerBlockCx,
) -> anyhow::Result<LIRInstrKind> {
	let tys = get_op_tys(&left, &right, &lbcx.registers, &lbcx.sig)?;
	let kind = match tys {
		(DataType::Score(..), DataType::Score(..)) => {
			LIRInstrKind::AddScore(left.to_mutable_score_value()?, right.to_score_value()?)
		}
		_ => bail!("Instruction does not allow this type"),
	};

	Ok(kind)
}

fn lower_sub(
	left: MutableValue,
	right: Value,
	lbcx: &LowerBlockCx,
) -> anyhow::Result<LIRInstrKind> {
	let tys = get_op_tys(&left, &right, &lbcx.registers, &lbcx.sig)?;
	let kind = match tys {
		(DataType::Score(..), DataType::Score(..)) => {
			LIRInstrKind::SubScore(left.to_mutable_score_value()?, right.to_score_value()?)
		}
		_ => bail!("Instruction does not allow this type"),
	};

	Ok(kind)
}

fn lower_mul(
	left: MutableValue,
	right: Value,
	lbcx: &LowerBlockCx,
) -> anyhow::Result<LIRInstrKind> {
	let tys = get_op_tys(&left, &right, &lbcx.registers, &lbcx.sig)?;
	let kind = match tys {
		(DataType::Score(..), DataType::Score(..)) => {
			LIRInstrKind::MulScore(left.to_mutable_score_value()?, right.to_score_value()?)
		}
		_ => bail!("Instruction does not allow this type"),
	};

	Ok(kind)
}

fn lower_div(
	left: MutableValue,
	right: Value,
	lbcx: &LowerBlockCx,
) -> anyhow::Result<LIRInstrKind> {
	let tys = get_op_tys(&left, &right, &lbcx.registers, &lbcx.sig)?;
	let kind = match tys {
		(DataType::Score(..), DataType::Score(..)) => {
			LIRInstrKind::DivScore(left.to_mutable_score_value()?, right.to_score_value()?)
		}
		_ => bail!("Instruction does not allow this type"),
	};

	Ok(kind)
}

fn lower_mod(
	left: MutableValue,
	right: Value,
	lbcx: &LowerBlockCx,
) -> anyhow::Result<LIRInstrKind> {
	let tys = get_op_tys(&left, &right, &lbcx.registers, &lbcx.sig)?;
	let kind = match tys {
		(DataType::Score(..), DataType::Score(..)) => {
			LIRInstrKind::ModScore(left.to_mutable_score_value()?, right.to_score_value()?)
		}
		_ => bail!("Instruction does not allow this type"),
	};

	Ok(kind)
}

fn lower_min(
	left: MutableValue,
	right: Value,
	lbcx: &LowerBlockCx,
) -> anyhow::Result<LIRInstrKind> {
	let tys = get_op_tys(&left, &right, &lbcx.registers, &lbcx.sig)?;
	let kind = match tys {
		(DataType::Score(..), DataType::Score(..)) => {
			LIRInstrKind::MinScore(left.to_mutable_score_value()?, right.to_score_value()?)
		}
		_ => bail!("Instruction does not allow this type"),
	};

	Ok(kind)
}

fn lower_max(
	left: MutableValue,
	right: Value,
	lbcx: &LowerBlockCx,
) -> anyhow::Result<LIRInstrKind> {
	let tys = get_op_tys(&left, &right, &lbcx.registers, &lbcx.sig)?;
	let kind = match tys {
		(DataType::Score(..), DataType::Score(..)) => {
			LIRInstrKind::MaxScore(left.to_mutable_score_value()?, right.to_score_value()?)
		}
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
		left.get_ty(&lbcx.registers, &lbcx.sig)?,
		right.get_ty(&lbcx.registers, &lbcx.sig)?,
	) {
		(DataType::Score(..), DataType::Score(..)) => {
			out.push(LIRInstruction::new(LIRInstrKind::SwapScore(
				left.to_mutable_score_value()?,
				right.to_mutable_score_value()?,
			)))
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
			let left = left.clone().to_mutable_nbt_value()?;
			let right = right.clone().to_mutable_nbt_value()?;
			out.push(LIRInstruction::new(LIRInstrKind::SetData(
				MutableNBTValue::Reg(temp_reg.clone()),
				NBTValue::Mutable(left.clone()),
			)));
			out.push(LIRInstruction::new(LIRInstrKind::SetData(
				left.clone(),
				NBTValue::Mutable(right.clone()),
			)));
			out.push(LIRInstruction::new(LIRInstrKind::SetData(
				right.clone(),
				NBTValue::Mutable(MutableNBTValue::Reg(temp_reg.clone())),
			)));
		}
		_ => bail!("Instruction does not allow this type"),
	};

	Ok(out)
}

fn lower_abs(val: MutableValue, lbcx: &LowerBlockCx) -> anyhow::Result<LIRInstruction> {
	let kind = match val.get_ty(&lbcx.registers, &lbcx.sig)? {
		DataType::Score(..) => LIRInstrKind::MulScore(
			val.clone().to_mutable_score_value()?,
			ScoreValue::Constant(ScoreTypeContents::Score(-1)),
		),
		_ => bail!("Instruction does not allow this type"),
	};

	let modifier = Modifier::If {
		condition: Box::new(IfModCondition::Score(IfScoreCondition::Range {
			score: ScoreValue::Mutable(val.to_mutable_score_value()?),
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

fn lower_get(
	value: MutableValue,
	scale: Double,
	lbcx: &LowerBlockCx,
) -> anyhow::Result<LIRInstrKind> {
	let kind = match value.get_ty(&lbcx.registers, &lbcx.sig)? {
		DataType::Score(..) => LIRInstrKind::GetScore(value.to_mutable_score_value()?),
		DataType::NBT(..) => LIRInstrKind::GetData(value.to_mutable_nbt_value()?, scale),
		_ => bail!("Type not supported"),
	};

	Ok(kind)
}

fn lower_merge(
	left: MutableValue,
	right: Value,
	lbcx: &LowerBlockCx,
) -> anyhow::Result<LIRInstrKind> {
	let tys = get_op_tys(&left, &right, &lbcx.registers, &lbcx.sig)?;
	let kind = match tys {
		(DataType::NBT(..), DataType::NBT(..)) => {
			LIRInstrKind::MergeData(left.to_mutable_nbt_value()?, right.to_nbt_value()?)
		}
		_ => bail!("Instruction does not allow this type"),
	};

	Ok(kind)
}

fn lower_push(
	left: MutableValue,
	right: Value,
	lbcx: &LowerBlockCx,
) -> anyhow::Result<LIRInstrKind> {
	let tys = get_op_tys(&left, &right, &lbcx.registers, &lbcx.sig)?;
	let kind = match tys {
		(DataType::NBT(..), DataType::NBT(..)) => {
			LIRInstrKind::PushData(left.to_mutable_nbt_value()?, right.to_nbt_value()?)
		}
		_ => bail!("Instruction does not allow this type"),
	};

	Ok(kind)
}

fn lower_push_front(
	left: MutableValue,
	right: Value,
	lbcx: &LowerBlockCx,
) -> anyhow::Result<LIRInstrKind> {
	let tys = get_op_tys(&left, &right, &lbcx.registers, &lbcx.sig)?;
	let kind = match tys {
		(DataType::NBT(..), DataType::NBT(..)) => {
			LIRInstrKind::PushFrontData(left.to_mutable_nbt_value()?, right.to_nbt_value()?)
		}
		_ => bail!("Instruction does not allow this type"),
	};

	Ok(kind)
}

fn lower_insert(
	left: MutableValue,
	right: Value,
	index: i32,
	lbcx: &LowerBlockCx,
) -> anyhow::Result<LIRInstrKind> {
	let tys = get_op_tys(&left, &right, &lbcx.registers, &lbcx.sig)?;
	let kind = match tys {
		(DataType::NBT(..), DataType::NBT(..)) => {
			LIRInstrKind::InsertData(left.to_mutable_nbt_value()?, right.to_nbt_value()?, index)
		}
		_ => bail!("Instruction does not allow this type"),
	};

	Ok(kind)
}

fn lower_rm(val: MutableValue, lbcx: &LowerBlockCx) -> anyhow::Result<LIRInstrKind> {
	let kind = match val.get_ty(&lbcx.registers, &lbcx.sig)? {
		DataType::Score(..) => LIRInstrKind::ResetScore(val.to_mutable_score_value()?),
		DataType::NBT(..) => LIRInstrKind::RemoveData(val.to_mutable_nbt_value()?),
		_ => bail!("Type not supported"),
	};

	Ok(kind)
}

fn lower_pow(
	base: MutableValue,
	exp: u8,
	lir_instrs: &mut Vec<LIRInstruction>,
	lbcx: &mut LowerBlockCx,
) -> anyhow::Result<()> {
	match exp {
		// x ^ 0 == 1
		0 => {
			lower!(
				lir_instrs,
				SetScore,
				base.clone().to_mutable_score_value()?,
				ScoreValue::Constant(ScoreTypeContents::Score(1))
			);
		}
		// x ^ 1 == x
		1 => {}
		// x ^ 2 == x * x
		2 => {
			let base = base.clone().to_mutable_score_value()?;
			lower!(
				lir_instrs,
				MulScore,
				base.clone(),
				ScoreValue::Mutable(base)
			);
		}
		// Now we have to use a temp register because just multiplying x by
		// itself multiple times will yield incorrect results. However,
		// we do a trick to reduce the number of instructions by splitting the exponent
		// into a highest power of two factor, so that we can chain simple multiplications by self first
		exp => {
			let base = base.clone().to_mutable_score_value()?;

			// First create the power of two section
			let largest_factor = highest_power_of_2_factor(exp);
			let two_power = (largest_factor as f32).log2() as u8;
			let second_exponent = exp / largest_factor;
			for _ in 0..two_power {
				lower!(
					lir_instrs,
					MulScore,
					base.clone(),
					ScoreValue::Mutable(base.clone())
				);
			}
			// Now create the section for the non-power of two, if it is needed
			if second_exponent == 1 {
				return Ok(());
			}
			let new_reg_id = lbcx.new_additional_reg();
			let new_reg = MutableScoreValue::Reg(new_reg_id.clone());
			// Declare the temp reg as the base
			lbcx.registers.insert(
				new_reg_id.clone(),
				Register {
					id: new_reg_id,
					ty: DataType::Score(ScoreType::Score),
				},
			);
			lower!(
				lir_instrs,
				SetScore,
				new_reg.clone(),
				ScoreValue::Mutable(base.clone())
			);
			// Do the multiplications
			for _ in 0..second_exponent - 1 {
				lower!(
					lir_instrs,
					MulScore,
					base.clone(),
					ScoreValue::Mutable(new_reg.clone())
				);
			}
		}
	}

	Ok(())
}

/// Utility for pow lowering to get the highest power of two that is a factor
/// of the given number
fn highest_power_of_2_factor(num: u8) -> u8 {
	num & (!(num - 1))
}

/// Returns a list of instructions to add before where the
/// condition is used, the conditions to use for one or more if
/// modifiers, and whether to negate each of those modifiers
fn lower_condition(
	condition: Condition,
	lbcx: &LowerBlockCx,
) -> anyhow::Result<(Vec<LIRInstruction>, Vec<(IfModCondition, bool)>)> {
	let mut prelude = Vec::new();
	let mut out = Vec::new();
	// Make this account for ScoreValue changes
	match condition {
		Condition::Equal(l, r) => {
			let lty = l.get_ty(&lbcx.registers, &lbcx.sig)?;
			let rty = r.get_ty(&lbcx.registers, &lbcx.sig)?;
			let cond = match (lty, rty) {
				(DataType::Score(..), DataType::Score(..)) => {
					IfModCondition::Score(IfScoreCondition::Single {
						left: l.to_score_value()?,
						right: r.to_score_value()?,
					})
				}
				_ => bail!("Condition does not allow these types"),
			};
			out.push((cond, false));
		}
		Condition::Exists(val) => {
			let cond = match val.get_ty(&lbcx.registers, &lbcx.sig)? {
				DataType::Score(..) => match val.to_score_value()? {
					ScoreValue::Constant(..) => IfModCondition::Const(true),
					ScoreValue::Mutable(val) => IfModCondition::Score(IfScoreCondition::Range {
						score: ScoreValue::Mutable(val),
						left: IfScoreRangeEnd::Infinite,
						right: IfScoreRangeEnd::Infinite,
					}),
				},
				DataType::NBT(..) => match val.to_nbt_value()? {
					NBTValue::Constant(..) => IfModCondition::Const(true),
					NBTValue::Mutable(val) => IfModCondition::DataExists(val),
				},
				_ => bail!("Type not supported"),
			};
			out.push((cond, false));
		}
		Condition::Not(condition) => {
			let (other_prelude, condition) =
				lower_condition(*condition, lbcx).context("Failed to lower not condition")?;
			// We can't do !(x and y) yet because the only way to do it is to do !x or !y,
			// and or is not supported yet
			if condition.len() != 1 {
				bail!("not and is not supported");
			}
			let (condition, negate) = condition.first().expect("Length is 1");
			prelude.extend(other_prelude);
			out.push((condition.clone(), !negate));
		}
		Condition::And(l, r) => {
			let (lp, lc) = lower_condition(*l, lbcx).context("Failed to lower and lhs")?;
			let (rp, rc) = lower_condition(*r, lbcx).context("Failed to lower and rhs")?;
			prelude.extend(lp);
			prelude.extend(rp);
			out.extend(lc);
			out.extend(rc);
		}
		Condition::GreaterThan(l, r) => {
			out.push((
				IfModCondition::Score(IfScoreCondition::Range {
					score: l.to_score_value()?,
					left: IfScoreRangeEnd::Fixed {
						value: r.to_score_value()?,
						inclusive: false,
					},
					right: IfScoreRangeEnd::Infinite,
				}),
				false,
			));
		}
		Condition::GreaterThanOrEqual(l, r) => {
			out.push((
				IfModCondition::Score(IfScoreCondition::Range {
					score: l.to_score_value()?,
					left: IfScoreRangeEnd::Fixed {
						value: r.to_score_value()?,
						inclusive: true,
					},
					right: IfScoreRangeEnd::Infinite,
				}),
				false,
			));
		}
		Condition::LessThan(l, r) => {
			out.push((
				IfModCondition::Score(IfScoreCondition::Range {
					score: l.to_score_value()?,
					left: IfScoreRangeEnd::Infinite,
					right: IfScoreRangeEnd::Fixed {
						value: r.to_score_value()?,
						inclusive: false,
					},
				}),
				false,
			));
		}
		Condition::LessThanOrEqual(l, r) => {
			out.push((
				IfModCondition::Score(IfScoreCondition::Range {
					score: l.to_score_value()?,
					left: IfScoreRangeEnd::Infinite,
					right: IfScoreRangeEnd::Fixed {
						value: r.to_score_value()?,
						inclusive: true,
					},
				}),
				false,
			));
		}
		Condition::Bool(val) => {
			out.push((lower_bool_cond(val, true, lbcx)?, false));
		}
		Condition::NotBool(val) => {
			out.push((lower_bool_cond(val, false, lbcx)?, false));
		}
		Condition::Entity(ent) => {
			out.push((IfModCondition::Entity(ent), false));
		}
		Condition::Predicate(pred) => {
			out.push((IfModCondition::Predicate(pred), false));
		}
		Condition::Biome(loc, biome) => {
			out.push((IfModCondition::Biome(loc, biome), false));
		}
		Condition::Loaded(loc) => {
			out.push((IfModCondition::Loaded(loc), false));
		}
		Condition::Dimension(dim) => {
			out.push((IfModCondition::Dimension(dim), false));
		}
	};

	Ok((prelude, out))
}

fn lower_bool_cond(val: Value, check: bool, lbcx: &LowerBlockCx) -> anyhow::Result<IfModCondition> {
	let ty = val.get_ty(&lbcx.registers, &lbcx.sig)?;
	match ty {
		DataType::Score(..) => Ok(IfModCondition::Score(IfScoreCondition::Single {
			left: val.to_score_value()?,
			right: ScoreValue::Constant(ScoreTypeContents::Bool(check)),
		})),
		_ => bail!("Condition does not allow this type"),
	}
}

fn lower_subblock(block: MIRBlock, lbcx: &mut LowerBlockCx) -> anyhow::Result<LIRInstruction> {
	let mut new_lir_instrs = Vec::new();
	for instr in block.contents {
		lower_kind(instr.kind, &mut new_lir_instrs, lbcx)
			.context("Failed to lower subinstruction body")?;
	}

	// Create a new function or just inline it
	let out = if new_lir_instrs.len() == 1 {
		new_lir_instrs
			.first()
			.expect("If len is 1, instr should exist")
			.clone()
	} else {
		let mut lir_block = LIRBlock::new(lbcx.registers.clone());
		lir_block.contents = new_lir_instrs;
		let interface = lbcx.new_body_fn();
		lbcx.lir.functions.insert(
			interface.id.clone(),
			LIRFunction {
				interface: interface.clone(),
				block: lir_block,
				parent: Some(lbcx.func_id.clone()),
			},
		);
		LIRInstruction::new(LIRInstrKind::Call(interface.id))
	};

	Ok(out)
}
