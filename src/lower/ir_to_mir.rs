use crate::common::function::Function;
use crate::common::{val::MutableValue, DeclareBinding};
use crate::ir::{InstrKind, IR};
use crate::mir::{MIRBlock, MIRInstrKind, MIRInstruction, MIR};

use anyhow::{anyhow, Context};

/// Lower IR to MIR
pub fn lower_ir(mut ir: IR) -> anyhow::Result<MIR> {
	let mut mir = MIR::with_capacity(ir.functions.len(), ir.blocks.count());

	for (func_id, func) in ir.functions {
		let block = ir
			.blocks
			.remove(&func.block)
			.ok_or(anyhow!("Block does not exist"))?;
		let mut mir_block = MIRBlock::with_capacity(block.contents.len());

		for ir_instr in block.contents {
			let (prelude, instr) =
				lower_kind(ir_instr.kind).context("Failed to lower instruction")?;
			mir_block.contents.extend(prelude);
			mir_block.contents.push(MIRInstruction::new(instr));
		}

		let block = mir.blocks.add(mir_block);
		mir.functions.insert(
			func_id,
			Function {
				interface: func.interface,
				block,
			},
		);
	}

	Ok(mir)
}

macro_rules! lower {
	($kind:ident) => {
		MIRInstrKind::$kind
	};

	($kind:ident, $($arg:ident),+) => {
		MIRInstrKind::$kind {$($arg),+}
	};

	($kind:expr) => {
		$kind
	}
}

fn lower_kind(kind: InstrKind) -> anyhow::Result<(Vec<MIRInstruction>, MIRInstrKind)> {
	let mut prelude = Vec::new();
	let kind = match kind {
		InstrKind::Declare { left, ty, right } => {
			let left_clone = left.clone();
			prelude.push(MIRInstruction::new(lower!(Declare, left, ty)));
			lower!(MIRInstrKind::Assign {
				left: MutableValue::Register(left_clone),
				right,
			})
		}
		InstrKind::Assign { left, right } => {
			lower!(MIRInstrKind::Assign {
				left,
				right: DeclareBinding::Value(right),
			})
		}
		InstrKind::Add { left, right } => lower!(Add, left, right),
		InstrKind::Sub { left, right } => lower!(Sub, left, right),
		InstrKind::Mul { left, right } => lower!(Mul, left, right),
		InstrKind::Div { left, right } => lower!(Div, left, right),
		InstrKind::Mod { left, right } => lower!(Mod, left, right),
		InstrKind::Min { left, right } => lower!(Min, left, right),
		InstrKind::Max { left, right } => lower!(Max, left, right),
		InstrKind::Swap { left, right } => lower!(Swap, left, right),
		InstrKind::Remove { val } => lower!(Remove, val),
		InstrKind::Abs { val } => lower!(Abs, val),
		InstrKind::Pow { base, exp } => lower!(Pow, base, exp),
		InstrKind::Get { value, scale } => lower!(Get, value, scale),
		InstrKind::Merge { left, right } => lower!(Merge, left, right),
		InstrKind::Push { left, right } => lower!(Push, left, right),
		InstrKind::PushFront { left, right } => lower!(PushFront, left, right),
		InstrKind::Insert { left, right, index } => {
			lower!(Insert, left, right, index)
		}
		InstrKind::Use { val } => lower!(Use, val),
		InstrKind::Call { call } => lower!(Call, call),
		InstrKind::CallExtern { func } => lower!(CallExtern, func),
		InstrKind::If { condition, body } => {
			let (new_prelude, instr) = lower_kind(*body).context("Failed to lower if body")?;
			prelude.extend(new_prelude);
			MIRInstrKind::If {
				condition,
				body: Box::new(instr),
			}
		}
		InstrKind::As { target, body } => {
			let (new_prelude, instr) = lower_kind(*body).context("Failed to lower as body")?;
			prelude.extend(new_prelude);
			MIRInstrKind::As {
				target,
				body: Box::new(instr),
			}
		}
		InstrKind::At { target, body } => {
			let (new_prelude, instr) = lower_kind(*body).context("Failed to lower at body")?;
			prelude.extend(new_prelude);
			MIRInstrKind::At {
				target,
				body: Box::new(instr),
			}
		}
		InstrKind::StoreResult { location, body } => {
			let (new_prelude, instr) = lower_kind(*body).context("Failed to lower at body")?;
			prelude.extend(new_prelude);
			MIRInstrKind::StoreResult {
				location,
				body: Box::new(instr),
			}
		}
		InstrKind::StoreSuccess { location, body } => {
			let (new_prelude, instr) = lower_kind(*body).context("Failed to lower at body")?;
			prelude.extend(new_prelude);
			MIRInstrKind::StoreSuccess {
				location,
				body: Box::new(instr),
			}
		}
		InstrKind::Positioned { position, body } => {
			let (new_prelude, instr) = lower_kind(*body).context("Failed to lower pos body")?;
			prelude.extend(new_prelude);
			MIRInstrKind::Positioned {
				position,
				body: Box::new(instr),
			}
		}
		InstrKind::ReturnRun { body } => {
			let (new_prelude, instr) = lower_kind(*body).context("Failed to lower retr body")?;
			prelude.extend(new_prelude);
			MIRInstrKind::ReturnRun {
				body: Box::new(instr),
			}
		}
		InstrKind::ReturnValue { index, value } => lower!(ReturnValue, index, value),
		InstrKind::Return { value } => lower!(Return, value),
		InstrKind::Command { command } => lower!(Command, command),
		InstrKind::Comment { comment } => lower!(Comment, comment),
		InstrKind::MC(instr) => MIRInstrKind::MC(instr),
	};

	Ok((prelude, kind))
}
