use anyhow::{anyhow, bail, Context};

use crate::common::function::Function;
use crate::common::mc::modifier::StoreModLocation;
use crate::common::ty::{get_op_tys, DataType};
use crate::common::{Register, RegisterList};
use crate::ir::{InstrKind, IR};
use crate::passes::{IRPass, Pass};

pub struct ValidatePass;

impl Pass for ValidatePass {
	fn get_name(&self) -> &'static str {
		"validate"
	}
}

impl IRPass for ValidatePass {
	fn run_pass(&mut self, ir: &mut IR) -> anyhow::Result<()> {
		for func in ir.functions.values() {
			let block = ir
				.blocks
				.get_mut(&func.block)
				.ok_or(anyhow!("Block does not exist"))?;
			let mut regs = RegisterList::default();
			for (i, instr) in block.contents.iter().enumerate() {
				validate_instr_kind(&instr.kind, &mut regs, func, &i)?;
			}
		}

		Ok(())
	}
}

fn validate_instr_kind(
	instr: &InstrKind,
	regs: &mut RegisterList,
	func: &Function,
	i: &usize,
) -> anyhow::Result<()> {
	match instr {
		InstrKind::Declare { left, ty, right } => {
			if regs.contains_key(left) {
				bail!("Redefinition of register {left} at {i}");
			}
			let right_ty = right.get_ty(regs, &func.interface.sig)?;
			if let Some(right_ty) = right_ty {
				if !right_ty.is_trivially_castable(ty) {
					bail!("Register type does not match value type at {i}");
				}
			}
			let reg = Register {
				id: left.clone(),
				ty: ty.clone(),
			};
			regs.insert(left.clone(), reg);
		}
		InstrKind::Assign { left, right }
		| InstrKind::Add { left, right }
		| InstrKind::Sub { left, right }
		| InstrKind::Mul { left, right }
		| InstrKind::Div { left, right }
		| InstrKind::Mod { left, right }
		| InstrKind::Min { left, right }
		| InstrKind::Max { left, right } => {
			let (left_ty, right_ty) = get_op_tys(left, right, regs, &func.interface.sig)?;
			if !right_ty.is_trivially_castable(&left_ty) {
				bail!("Incompatible types in instruction at {i}");
			}
		}
		InstrKind::Push { left, right }
		| InstrKind::PushFront { left, right }
		| InstrKind::Insert { left, right, .. } => {
			let (left, right) = get_op_tys(left, right, regs, &func.interface.sig)?;
			let (DataType::NBT(left), DataType::NBT(right)) = (left, right) else {
				bail!("Incompatible types in instruction at {i}");
			};
			if !left.can_contain(&right) {
				bail!("Incompatible types in instruction at {i}");
			}
		}
		InstrKind::Swap { left, right } => {
			if !right
				.get_ty(regs, &func.interface.sig)?
				.is_trivially_castable(&left.get_ty(regs, &func.interface.sig)?)
			{
				bail!("Incompatible types in instruction at {i}");
			}
		}
		InstrKind::Get { value, scale } => match value.get_ty(regs, &func.interface.sig)? {
			DataType::Score(..) => {
				if *scale != 1.0 {
					bail!("Scale that is not 1.0 cannot be used for getting a value of score type");
				}
			}
			_ => {}
		},
		InstrKind::StoreResult { location, body } => {
			validate_instr_kind(body, regs, func, i)?;
			if let StoreModLocation::Reg(reg, scale) = location {
				if let DataType::Score(..) = regs.get(reg).context("Register does not exist")?.ty {
					if *scale != 1.0 {
						bail!("Scale that is not 1.0 cannot be used for storing to a value of score type");
					}
				}
			}
		}
		InstrKind::If { body, .. }
		| InstrKind::As { body, .. }
		| InstrKind::At { body, .. }
		| InstrKind::StoreSuccess { body, .. }
		| InstrKind::Positioned { body, .. }
		| InstrKind::ReturnRun { body, .. } => {
			validate_instr_kind(body, regs, func, i)?;
		}
		_ => {}
	}

	Ok(())
}
