use std::collections::HashMap;

use crate::{
	common::Identifier,
	lir::{LIRBlock, LIRInstrKind},
};

use super::text::format_reg_fake_player;

#[derive(Debug)]
pub struct RegAllocCx {
	count: u32,
}

impl RegAllocCx {
	pub fn new() -> Self {
		Self { count: 0 }
	}

	pub fn new_reg(&mut self) -> String {
		let old_count = self.count;
		self.count += 1;
		format_reg_fake_player(old_count)
	}

	pub fn get_count(&self) -> u32 {
		self.count
	}
}

/// Result from allocating registers for a block
pub struct RegAllocResult {
	pub regs: HashMap<Identifier, String>,
}

pub fn alloc_block_registers(block: &LIRBlock, racx: &mut RegAllocCx) -> RegAllocResult {
	let mut out = RegAllocResult {
		regs: HashMap::new(),
	};

	for instr in &block.contents {
		let used_regs = get_used_regs(&instr.kind);
		for reg in used_regs {
			if !out.regs.contains_key(reg) {
				out.regs.insert(reg.clone(), racx.new_reg());
			}
		}
	}

	out
}

fn get_used_regs(kind: &LIRInstrKind) -> Vec<&Identifier> {
	match kind {
		LIRInstrKind::SetScore(left, right)
		| LIRInstrKind::AddScore(left, right)
		| LIRInstrKind::SubScore(left, right)
		| LIRInstrKind::MulScore(left, right)
		| LIRInstrKind::DivScore(left, right)
		| LIRInstrKind::ModScore(left, right)
		| LIRInstrKind::MinScore(left, right)
		| LIRInstrKind::MaxScore(left, right) => [left.get_used_regs(), right.get_used_regs()].concat(),
		LIRInstrKind::SwapScore(left, right) => {
			[left.get_used_regs(), right.get_used_regs()].concat()
		}
	}
}
