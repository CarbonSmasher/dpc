use crate::common::{val::MutableValue, DeclareBinding};
use crate::ir::{Block, InstrKind, IR};
use crate::mir::{MIRBlock, MIRFunction, MIRInstrKind, MIRInstruction, MIR};

use anyhow::Context;

/// Lower IR to MIR
pub fn lower_ir(ir: IR) -> anyhow::Result<MIR> {
	let mut mir = MIR::with_capacity(ir.functions.len());

	for (func_id, func) in ir.functions {
		let mir_block = lower_block(func.block)?;

		mir.functions.insert(
			func_id,
			MIRFunction {
				interface: func.interface,
				block: mir_block,
			},
		);
	}

	Ok(mir)
}

fn lower_block(block: Block) -> anyhow::Result<MIRBlock> {
	let mut mir_block = MIRBlock::with_capacity(block.contents.len());

	for ir_instr in block.contents {
		let instrs = lower_kind(ir_instr.kind).context("Failed to lower instruction")?;
		mir_block.contents.extend(instrs);
	}

	Ok(mir_block)
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

fn lower_kind(kind: InstrKind) -> anyhow::Result<Vec<MIRInstruction>> {
	let mut out = Vec::new();
	let kind = match kind {
		InstrKind::Declare { left, ty, right } => {
			let left_clone = left.clone();
			out.push(MIRInstruction::new(lower!(Declare, left, ty)));
			lower!(MIRInstrKind::Assign {
				left: MutableValue::Reg(left_clone),
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
		InstrKind::Not { value } => lower!(Not, value),
		InstrKind::And { left, right } => lower!(And, left, right),
		InstrKind::Or { left, right } => lower!(Or, left, right),
		InstrKind::Use { val } => lower!(Use, val),
		InstrKind::Call { call } => lower!(Call, call),
		InstrKind::CallExtern { func } => lower!(CallExtern, func),
		InstrKind::If { condition, body } => {
			let instrs = lower_block(*body).context("Failed to lower if body")?;
			MIRInstrKind::If {
				condition,
				body: Box::new(instrs),
			}
		}
		InstrKind::IfElse {
			condition,
			first,
			second,
		} => {
			let first = lower_block(*first).context("Failed to lower if else first body")?;
			let second = lower_block(*second).context("Failed to lower if else second body")?;
			MIRInstrKind::IfElse {
				condition,
				first: Box::new(first),
				second: Box::new(second),
			}
		}
		InstrKind::Modify { modifier, body } => {
			let instrs = lower_block(*body).context("Failed to lower mdf body")?;
			MIRInstrKind::Modify {
				modifier,
				body: Box::new(instrs),
			}
		}
		InstrKind::ReturnRun { body } => {
			let instrs = lower_block(*body).context("Failed to lower retr body")?;
			MIRInstrKind::ReturnRun {
				body: Box::new(instrs),
			}
		}
		InstrKind::ReturnValue { index, value } => lower!(ReturnValue, index, value),
		InstrKind::Return { value } => lower!(Return, value),
		InstrKind::Command { command } => lower!(Command, command),
		InstrKind::Comment { comment } => lower!(Comment, comment),
		InstrKind::MC(instr) => MIRInstrKind::MC(instr),
	};
	out.push(MIRInstruction::new(kind));

	Ok(out)
}
