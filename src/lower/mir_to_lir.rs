use anyhow::{anyhow, bail, Context};

use crate::common::condition::Condition;
use crate::common::function::FunctionInterface;
use crate::common::mc::modifier::{
	IfModCondition, IfScoreCondition, IfScoreRangeEnd, Modifier, StoreModLocation,
};
use crate::common::ty::{
	get_op_tys, ArraySize, DataType, DataTypeContents, ScoreType, ScoreTypeContents,
};
use crate::common::{
	val::MutableNBTValue, val::MutableScoreValue, val::MutableValue, val::NBTValue,
	val::ScoreValue, val::Value, DeclareBinding, Identifier, Register, RegisterList,
};
use crate::lir::{LIRBlock, LIRInstrKind, LIRInstruction, LIR};
use crate::mir::{MIRInstrKind, MIR};

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
pub fn lower_mir(mut mir: MIR) -> anyhow::Result<LIR> {
	let mut lir = LIR::with_capacity(mir.functions.len(), mir.blocks.count());

	for (interface, block) in mir.functions {
		let block = mir
			.blocks
			.remove(&block)
			.ok_or(anyhow!("Block does not exist"))?;
		let mut lir_instrs = Vec::with_capacity(block.contents.len());

		let mut lbcx = LowerBlockCx::new(&mut lir);

		for mir_instr in block.contents {
			lower_kind(mir_instr.kind, &mut lir_instrs, &mut lbcx)?;
		}

		let mut lir_block = LIRBlock::new(lbcx.registers);
		lir_block.contents = lir_instrs;

		let id = lir.blocks.add(lir_block);
		lir.functions.insert(interface, id);
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
		MIRInstrKind::Get { value } => {
			lir_instrs.push(LIRInstruction::new(lower_get(value, lbcx)?));
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
		MIRInstrKind::Use { val } => lower!(lir_instrs, Use, val),
		MIRInstrKind::Call { call } => lower!(lir_instrs, Call, call.function),
		MIRInstrKind::Say { message } => lower!(lir_instrs, Say, message),
		MIRInstrKind::Tell { target, message } => lower!(lir_instrs, Tell, target, message),
		MIRInstrKind::Kill { target } => lower!(lir_instrs, Kill, target),
		MIRInstrKind::Reload => lower!(lir_instrs, Reload),
		MIRInstrKind::SetXP {
			target,
			amount,
			value,
		} => {
			lower!(lir_instrs, SetXP, target, amount, value)
		}
		MIRInstrKind::Pow { base, exp } => {
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
				// itself multiple times will yield incorrect results
				exp => {
					let base = base.clone().to_mutable_score_value()?;
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
					for _ in 0..exp - 1 {
						lower!(
							lir_instrs,
							MulScore,
							base.clone(),
							ScoreValue::Mutable(new_reg.clone())
						);
					}
				}
			}
			if exp == 0 {
			} else {
				for _ in 0..(exp - 1) {
					lower!(
						lir_instrs,
						MulScore,
						base.clone().to_mutable_score_value()?,
						ScoreValue::Mutable(base.clone().to_mutable_score_value()?)
					);
				}
			}
		}
		MIRInstrKind::Publish => lower!(lir_instrs, Publish),
		MIRInstrKind::Seed => lower!(lir_instrs, Seed),
		MIRInstrKind::GetDifficulty => lower!(lir_instrs, GetDifficulty),
		MIRInstrKind::StopServer => lower!(lir_instrs, StopServer),
		MIRInstrKind::StopSound => lower!(lir_instrs, StopSound),
		MIRInstrKind::Banlist => lower!(lir_instrs, Banlist),
		MIRInstrKind::WhitelistList => lower!(lir_instrs, WhitelistList),
		MIRInstrKind::WhitelistReload => lower!(lir_instrs, WhitelistReload),
		MIRInstrKind::WhitelistOn => lower!(lir_instrs, WhitelistOn),
		MIRInstrKind::WhitelistOff => lower!(lir_instrs, WhitelistOff),
		MIRInstrKind::ListPlayers => lower!(lir_instrs, ListPlayers),
		MIRInstrKind::Me { message } => lower!(lir_instrs, Me, message),
		MIRInstrKind::TeamMessage { message } => lower!(lir_instrs, TeamMessage, message),
		MIRInstrKind::BanPlayers { targets, reason } => {
			lower!(lir_instrs, BanPlayers, targets, reason)
		}
		MIRInstrKind::BanIP { target, reason } => {
			lower!(lir_instrs, BanIP, target, reason)
		}
		MIRInstrKind::PardonPlayers { targets } => {
			lower!(lir_instrs, PardonPlayers, targets)
		}
		MIRInstrKind::PardonIP { target } => {
			lower!(lir_instrs, PardonIP, target)
		}
		MIRInstrKind::Op { targets } => {
			lower!(lir_instrs, Op, targets)
		}
		MIRInstrKind::Deop { targets } => {
			lower!(lir_instrs, Deop, targets)
		}
		MIRInstrKind::WhitelistAdd { targets } => {
			lower!(lir_instrs, WhitelistAdd, targets)
		}
		MIRInstrKind::WhitelistRemove { targets } => {
			lower!(lir_instrs, WhitelistRemove, targets)
		}
		MIRInstrKind::Kick { targets, reason } => {
			lower!(lir_instrs, Kick, targets, reason)
		}
		MIRInstrKind::SetDifficulty { difficulty } => {
			lower!(lir_instrs, SetDifficulty, difficulty)
		}
		MIRInstrKind::Enchant {
			target,
			enchantment,
			level,
		} => {
			lower!(lir_instrs, Enchant, target, enchantment, level)
		}
		MIRInstrKind::SetBlock { data } => lower!(lir_instrs, SetBlock, data),
		MIRInstrKind::Fill { data } => lower!(lir_instrs, Fill, data),
		MIRInstrKind::Clone { data } => lower!(lir_instrs, Clone, data),
		MIRInstrKind::SetWeather { weather, duration } => {
			lower!(lir_instrs, SetWeather, weather, duration)
		}
		MIRInstrKind::AddTime { time } => lower!(lir_instrs, AddTime, time),
		MIRInstrKind::SetTime { time } => lower!(lir_instrs, SetTime, time),
		MIRInstrKind::SetTimePreset { time } => lower!(lir_instrs, SetTimePreset, time),
		MIRInstrKind::GetTime { query } => lower!(lir_instrs, GetTime, query),
		MIRInstrKind::AddTag { target, tag } => lower!(lir_instrs, AddTag, target, tag),
		MIRInstrKind::RemoveTag { target, tag } => {
			lower!(lir_instrs, RemoveTag, target, tag)
		}
		MIRInstrKind::ListTags { target } => lower!(lir_instrs, ListTags, target),
		MIRInstrKind::RideMount { target, vehicle } => {
			lower!(lir_instrs, RideMount, target, vehicle)
		}
		MIRInstrKind::RideDismount { target } => lower!(lir_instrs, RideDismount, target),
		MIRInstrKind::FillBiome { data } => lower!(lir_instrs, FillBiome, data),
		MIRInstrKind::Spectate { target, spectator } => {
			lower!(lir_instrs, Spectate, target, spectator)
		}
		MIRInstrKind::SpectateStop => lower!(lir_instrs, SpectateStop),
		MIRInstrKind::SetGamemode { target, gamemode } => {
			lower!(lir_instrs, SetGamemode, target, gamemode)
		}
		MIRInstrKind::DefaultGamemode { gamemode } => {
			lower!(lir_instrs, DefaultGamemode, gamemode)
		}
		MIRInstrKind::If { condition, body } => {
			let (prepend, condition) =
				lower_condition(condition, lbcx).context("Failed to lower condition")?;
			lir_instrs.extend(prepend);

			// Create the body
			let mut new_lir_instrs = Vec::new();
			lower_kind(*body, &mut new_lir_instrs, lbcx).context("Failed to lower if body")?;

			// Create a new function or just inline it
			let mut instr = if new_lir_instrs.len() == 1 {
				new_lir_instrs
					.first()
					.expect("If len is 1, instr should exist")
					.clone()
			} else {
				let mut lir_block = LIRBlock::new(RegisterList::new());
				lir_block.contents = new_lir_instrs;
				let block = lbcx.lir.blocks.add(lir_block);
				let interface = lbcx.new_if_body_fn();
				lbcx.lir.functions.insert(interface.clone(), block);
				LIRInstruction::new(LIRInstrKind::Call(interface.id))
			};
			instr.modifiers.insert(
				0,
				Modifier::If {
					condition: Box::new(condition),
					negate: false,
				},
			);
		}
		MIRInstrKind::TeleportToEntity { source, dest } => {
			lower!(lir_instrs, TeleportToEntity, source, dest)
		}
		MIRInstrKind::TeleportToLocation { source, dest } => {
			lower!(lir_instrs, TeleportToLocation, source, dest)
		}
		MIRInstrKind::TeleportWithRotation {
			source,
			dest,
			rotation,
		} => {
			lower!(lir_instrs, TeleportWithRotation, source, dest, rotation)
		}
		MIRInstrKind::TeleportFacingLocation {
			source,
			dest,
			facing,
		} => {
			lower!(lir_instrs, TeleportFacingLocation, source, dest, facing)
		}
		MIRInstrKind::TeleportFacingEntity {
			source,
			dest,
			facing,
		} => {
			lower!(lir_instrs, TeleportFacingEntity, source, dest, facing)
		}
		MIRInstrKind::GiveItem {
			target,
			item,
			amount,
		} => {
			lower!(lir_instrs, GiveItem, target, item, amount)
		}
		MIRInstrKind::AddScoreboardObjective {
			objective,
			criterion,
			display_name,
		} => {
			lower!(
				lir_instrs,
				AddScoreboardObjective,
				objective,
				criterion,
				display_name
			)
		}
		MIRInstrKind::RemoveScoreboardObjective { objective } => {
			lower!(lir_instrs, RemoveScoreboardObjective, objective)
		}
		MIRInstrKind::ListScoreboardObjectives => {
			lower!(lir_instrs, ListScoreboardObjectives)
		}
		MIRInstrKind::TriggerAdd { objective, amount } => {
			lower!(lir_instrs, TriggerAdd, objective, amount)
		}
		MIRInstrKind::TriggerSet { objective, amount } => {
			lower!(lir_instrs, TriggerSet, objective, amount)
		}
	}
	Ok(())
}

struct LowerBlockCx<'lir> {
	lir: &'lir mut LIR,
	registers: RegisterList,
	additional_reg_count: u32,
	if_body_count: u32,
}

impl<'lir> LowerBlockCx<'lir> {
	fn new(lir: &'lir mut LIR) -> Self {
		Self {
			lir,
			registers: RegisterList::new(),
			additional_reg_count: 0,
			if_body_count: 0,
		}
	}

	fn new_additional_reg(&mut self) -> Identifier {
		let old_val = self.additional_reg_count;
		self.additional_reg_count += 1;
		Identifier::from(format!("__lir_lower_{old_val}"))
	}

	fn new_if_body_fn(&mut self) -> FunctionInterface {
		let old_val = self.if_body_count;
		self.if_body_count += 1;
		FunctionInterface::new(format!("dpc::ifbody_{old_val}").into())
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
		DeclareBinding::Null => None,
		DeclareBinding::Value(val) => Some(val.clone()),
		DeclareBinding::Cast(ty, val) => {
			let val_ty = val.get_ty(&lbcx.registers)?;
			// If the cast is not trivial, we have to declare a new register,
			// initialize it with the cast, and then assign the result to our declaration
			let assign_val = if val_ty.is_trivially_castable(&ty) {
				Some(Value::Mutable(val.clone()))
			} else {
				// Run the cast
				let store_loc = match ty {
					DataType::Score(..) => StoreModLocation::from_mut_score_val(
						&left.clone().to_mutable_score_value()?,
					),
					DataType::NBT(..) => {
						StoreModLocation::from_mut_nbt_val(&left.clone().to_mutable_nbt_value()?)
					}
				};

				let get_instr = match val_ty {
					DataType::Score(..) => {
						LIRInstrKind::GetScore(val.clone().to_mutable_score_value()?)
					}
					DataType::NBT(..) => LIRInstrKind::GetData(val.clone().to_mutable_nbt_value()?),
				};
				out.push(LIRInstruction::with_modifiers(
					get_instr,
					vec![Modifier::StoreResult(store_loc)],
				));

				None
			};
			assign_val
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
						score: MutableScoreValue::Reg(new_reg.clone()),
						value: val.clone().to_nbt_value()?,
						index,
					}));
				}
				_ => bail!("Cannot use index declaration with these types"),
			}
			Some(Value::Mutable(MutableValue::Register(new_reg)))
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
	let tys = get_op_tys(&left, &right, &lbcx.registers)?;
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
	let tys = get_op_tys(&left, &right, &lbcx.registers)?;
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
	let tys = get_op_tys(&left, &right, &lbcx.registers)?;
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
	let tys = get_op_tys(&left, &right, &lbcx.registers)?;
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
	let tys = get_op_tys(&left, &right, &lbcx.registers)?;
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
	let tys = get_op_tys(&left, &right, &lbcx.registers)?;
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
	let tys = get_op_tys(&left, &right, &lbcx.registers)?;
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
		left.get_ty(&lbcx.registers)?,
		right.get_ty(&lbcx.registers)?,
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
	let kind = match val.get_ty(&lbcx.registers)? {
		DataType::Score(..) => LIRInstrKind::MulScore(
			val.clone().to_mutable_score_value()?,
			ScoreValue::Constant(ScoreTypeContents::Score(-1)),
		),
		_ => bail!("Instruction does not allow this type"),
	};

	let modifier = Modifier::If {
		condition: Box::new(IfModCondition::Score(IfScoreCondition::Range {
			score: val.to_mutable_score_value()?,
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

fn lower_get(value: MutableValue, lbcx: &LowerBlockCx) -> anyhow::Result<LIRInstrKind> {
	let kind = match value.get_ty(&lbcx.registers)? {
		DataType::Score(..) => LIRInstrKind::GetScore(value.to_mutable_score_value()?),
		DataType::NBT(..) => LIRInstrKind::GetData(value.to_mutable_nbt_value()?),
	};

	Ok(kind)
}

fn lower_merge(
	left: MutableValue,
	right: Value,
	lbcx: &LowerBlockCx,
) -> anyhow::Result<LIRInstrKind> {
	let tys = get_op_tys(&left, &right, &lbcx.registers)?;
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
	let tys = get_op_tys(&left, &right, &lbcx.registers)?;
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
	let tys = get_op_tys(&left, &right, &lbcx.registers)?;
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
	let tys = get_op_tys(&left, &right, &lbcx.registers)?;
	let kind = match tys {
		(DataType::NBT(..), DataType::NBT(..)) => {
			LIRInstrKind::InsertData(left.to_mutable_nbt_value()?, right.to_nbt_value()?, index)
		}
		_ => bail!("Instruction does not allow this type"),
	};

	Ok(kind)
}

/// Returns a list of instructions to add before where the
/// condition is used, and the condition to use for the if
/// modifier
fn lower_condition(
	condition: Condition,
	lbcx: &LowerBlockCx,
) -> anyhow::Result<(Vec<LIRInstruction>, IfModCondition)> {
	let out = match condition {
		Condition::Equal(l, r) => {
			let lty = l.get_ty(&lbcx.registers)?;
			let rty = r.get_ty(&lbcx.registers)?;
			let cond = match (lty, rty) {
				(DataType::Score(..), DataType::Score(..)) => {
					// Rearrange them so that the mutable value is on the left
					let cond = match (l.to_score_value()?, r.to_score_value()?) {
						(ScoreValue::Mutable(l), r) => {
							IfModCondition::Score(IfScoreCondition::Single { left: l, right: r })
						}
						(l @ ScoreValue::Constant(..), ScoreValue::Mutable(r)) => {
							IfModCondition::Score(IfScoreCondition::Single { left: r, right: l })
						}
						(ScoreValue::Constant(l), ScoreValue::Constant(r)) => {
							let result = l.get_i32() == r.get_i32();
							IfModCondition::Const(result)
						}
					};
					cond
				}
				_ => bail!("Condition does not allow these types"),
			};
			(Vec::new(), cond)
		}
		Condition::Exists(val) => {
			let cond = match val.get_ty(&lbcx.registers)? {
				DataType::Score(..) => match val.to_score_value()? {
					ScoreValue::Constant(..) => IfModCondition::Const(true),
					ScoreValue::Mutable(val) => IfModCondition::Score(IfScoreCondition::Range {
						score: val,
						left: IfScoreRangeEnd::Infinite,
						right: IfScoreRangeEnd::Infinite,
					}),
				},
				DataType::NBT(..) => match val.to_nbt_value()? {
					NBTValue::Constant(..) => IfModCondition::Const(true),
					NBTValue::Mutable(val) => IfModCondition::DataExists(val),
				},
			};

			(Vec::new(), cond)
		}
	};

	Ok(out)
}
