use anyhow::{anyhow, bail};

use crate::common::ty::get_op_tys;
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
		for (_, block) in &ir.functions {
			let block = ir
				.blocks
				.get_mut(block)
				.ok_or(anyhow!("Block does not exist"))?;
			let regs = RegisterList::new();
			for instr in &block.contents {
				match &instr.kind {
					InstrKind::Declare { left, ty, right } => {
						if regs.contains_key(left) {
							bail!("Redefinition of register {left}");
						}
						let right_ty = right.get_ty(&regs)?;
						if let Some(right_ty) = right_ty {
							if !right_ty.is_trivially_castable(ty) {
								bail!("Register type does not match value type");
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
						let (left_ty, right_ty) = get_op_tys(left, right, &regs)?;
						if !right_ty.is_trivially_castable(&left_ty) {
							bail!("Incompatible types in instruction");
						}
					}
					InstrKind::Swap { left, right } => {
						if !right
							.get_ty(&regs)?
							.is_trivially_castable(&left.get_ty(&regs)?)
						{
							bail!("Incompatible types in instruction");
						}
					}
					_ => {}
				}
			}
		}

		Ok(())
	}
}
