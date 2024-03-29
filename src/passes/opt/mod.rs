use crate::common::function::CallInterface;
use crate::common::val::ArgRetIndex;
use crate::common::Identifier;
use crate::mir::{MIRBlock, MIRInstrKind};

pub mod constant;
pub mod dataflow;
pub mod dce;
pub mod dse;
pub mod func;
pub mod modifiers;
pub mod multifold;
pub mod order;
pub mod simplify;
pub mod ty;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum OptimizableValue {
	Reg(Identifier),
	Arg(ArgRetIndex),
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
