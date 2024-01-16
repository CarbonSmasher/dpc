use crate::common::function::CallInterface;
use crate::common::val::{MutableScoreValue, MutableValue};
use crate::common::Identifier;
use crate::mir::{MIRInstrKind, MIRBlock};

pub mod constant;
pub mod dce;
pub mod dse;
pub mod func;
pub mod modifiers;
pub mod multifold;
pub mod scoreboard_dataflow;
pub mod simplify;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum OptimizableValue {
	Reg(Identifier),
	Arg(u16),
}

impl MutableValue {
	pub(self) fn to_optimizable_value(&self) -> Option<OptimizableValue> {
		match self {
			Self::Reg(reg) => Some(OptimizableValue::Reg(reg.clone())),
			Self::Arg(arg) => Some(OptimizableValue::Arg(*arg)),
			_ => None,
		}
	}
}

impl MutableScoreValue {
	pub(self) fn to_optimizable_value(&self) -> Option<OptimizableValue> {
		match self {
			Self::Reg(reg) => Some(OptimizableValue::Reg(reg.clone())),
			Self::Arg(arg) => Some(OptimizableValue::Arg(*arg)),
			_ => None,
		}
	}
}

pub fn get_instr_calls(instr: &MIRInstrKind) -> Vec<&CallInterface> {
	match instr {
		MIRInstrKind::Call { call } => vec![call],
		other => {
			let bodies = other.get_bodies();
			let mut out = Vec::new();

			for body in bodies {
				for instr in &body.contents {
					out.extend(get_instr_calls(&instr.kind));
				}
			}

			out
		}
	}
}

pub fn get_instr_calls_mut(instr: &mut MIRInstrKind) -> Vec<&mut CallInterface> {
	match instr {
		MIRInstrKind::Call { call } => vec![call],
		other => {
			let bodies = other.get_bodies_mut();
			let mut out = Vec::new();

			for body in bodies {
				for instr in &mut body.contents {
					out.extend(get_instr_calls_mut(&mut instr.kind));
				}
			}

			out
		}
	}
}

pub fn are_blocks_equivalent(block1: &MIRBlock, block2: &MIRBlock) -> bool {
	block1 == block2
}
