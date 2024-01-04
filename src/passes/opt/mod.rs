use crate::common::function::CallInterface;
use crate::common::val::{MutableScoreValue, MutableValue};
use crate::common::Identifier;
use crate::mir::MIRInstrKind;

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
			Self::Register(reg) => Some(OptimizableValue::Reg(reg.clone())),
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

pub fn get_instr_call(instr: &MIRInstrKind) -> Option<&CallInterface> {
	match instr {
		MIRInstrKind::Call { call } => Some(call),
		other => other.get_body().and_then(get_instr_call),
	}
}
