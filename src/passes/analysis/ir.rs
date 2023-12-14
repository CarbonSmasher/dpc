use anyhow::{anyhow, bail};

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
		for (func, block) in &ir.functions {
			let block = ir
				.blocks
				.get_mut(block)
				.ok_or(anyhow!("Block does not exist"))?;
			let regs = RegisterList::new();
			for (i, instr) in block.contents.iter().enumerate() {
				match &instr.kind {
					InstrKind::Declare { left, ty, right } => {
						if regs.contains_key(left) {
							bail!("Redefinition of register {left} at {i}");
						}
						let right_ty = right.get_ty(&regs, &func.sig)?;
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
						let (left_ty, right_ty) = get_op_tys(left, right, &regs, &func.sig)?;
						if !right_ty.is_trivially_castable(&left_ty) {
							bail!("Incompatible types in instruction at {i}");
						}
					}
					InstrKind::Push { left, right }
					| InstrKind::PushFront { left, right }
					| InstrKind::Insert { left, right, .. } => {
						let (left, right) = get_op_tys(left, right, &regs, &func.sig)?;
						let (DataType::NBT(left), DataType::NBT(right)) = (left, right) else {
							bail!("Incompatible types in instruction at {i}");
						};
						if !left.can_contain(&right) {
							bail!("Incompatible types in instruction at {i}");
						}
					}
					InstrKind::Swap { left, right } => {
						if !right
							.get_ty(&regs, &func.sig)?
							.is_trivially_castable(&left.get_ty(&regs, &func.sig)?)
						{
							bail!("Incompatible types in instruction at {i}");
						}
					}
					_ => {}
				}
			}
		}

		Ok(())
	}
}
