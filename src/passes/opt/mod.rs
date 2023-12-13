use crate::{
	common::{
		function::CallInterface,
		val::{MutableScoreValue, MutableValue},
		Identifier,
	},
	mir::MIRInstrKind,
};

pub mod const_passes;
pub mod dce;
pub mod dse;
pub mod inline;
pub mod inst_combine;
pub mod merge_modifiers;
pub mod scoreboard_dataflow;
pub mod simplify;
pub mod simplify_modifiers;

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
		MIRInstrKind::If { body, .. } => get_instr_call(body),
		_ => None,
	}
}
