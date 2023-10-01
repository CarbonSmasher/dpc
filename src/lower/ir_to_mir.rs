use crate::common::{DeclareBinding, MutableValue};
use crate::ir::{InstrKind, IR};
use crate::mir::{MIRBlock, MIRInstrKind, MIRInstruction, MIR};

macro_rules! lower_exact {
	($mir_block:expr, $kind:ident, $($arg:ident),+) => {
		$mir_block
			.contents
			.push(MIRInstruction::new(MIRInstrKind::$kind {$($arg),+}))
	};
}

/// Lower IR to MIR
pub fn lower_ir(ir: IR) -> anyhow::Result<MIR> {
	let mut mir = MIR::new();

	for (interface, block) in ir.functions {
		let mut mir_block = MIRBlock::new();

		for ir_instr in block.contents {
			match ir_instr.kind {
				InstrKind::Declare { left, ty, right } => {
					let left_clone = left.clone();
					lower_exact!(mir_block, Declare, left, ty);
					mir_block
						.contents
						.push(MIRInstruction::new(MIRInstrKind::Assign {
							left: MutableValue::Register(left_clone),
							right,
						}))
				}
				InstrKind::Assign { left, right } => {
					mir_block
						.contents
						.push(MIRInstruction::new(MIRInstrKind::Assign {
							left,
							right: DeclareBinding::Value(right),
						}))
				}
				InstrKind::Add { left, right } => lower_exact!(mir_block, Add, left, right),
				InstrKind::Sub { left, right } => lower_exact!(mir_block, Sub, left, right),
				InstrKind::Mul { left, right } => lower_exact!(mir_block, Mul, left, right),
				InstrKind::Div { left, right } => lower_exact!(mir_block, Div, left, right),
				InstrKind::Mod { left, right } => lower_exact!(mir_block, Mod, left, right),
				InstrKind::Min { left, right } => lower_exact!(mir_block, Min, left, right),
				InstrKind::Max { left, right } => lower_exact!(mir_block, Max, left, right),
				InstrKind::Swap { left, right } => lower_exact!(mir_block, Swap, left, right),
				InstrKind::Abs { val } => lower_exact!(mir_block, Abs, val),
				InstrKind::Use { val } => lower_exact!(mir_block, Use, val),
			}
		}

		mir.functions.insert(interface, mir_block);
	}

	Ok(mir)
}
