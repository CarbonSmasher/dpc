use std::collections::HashMap;

use anyhow::anyhow;

use crate::common::ty::DataType;
use crate::common::Identifier;
use crate::lir::{LIRBlock, LIRInstrKind};

use super::text::{format_local_storage_entry, format_reg_fake_player};

#[derive(Debug)]
pub struct RegAllocCx {
	reg_count: u32,
	local_count: u32,
}

impl RegAllocCx {
	pub fn new() -> Self {
		Self {
			reg_count: 0,
			local_count: 0,
		}
	}

	pub fn new_reg(&mut self) -> String {
		let old_count = self.reg_count;
		self.reg_count += 1;
		format_reg_fake_player(old_count)
	}

	pub fn new_local(&mut self) -> String {
		let old_count = self.local_count;
		self.local_count += 1;
		format_local_storage_entry(old_count)
	}

	pub fn get_reg_count(&self) -> u32 {
		self.reg_count
	}

	pub fn get_local_count(&self) -> u32 {
		self.reg_count
	}
}

/// Result from allocating registers for a block
pub struct RegAllocResult {
	pub regs: HashMap<Identifier, String>,
	pub locals: HashMap<Identifier, String>,
}

pub fn alloc_block_registers(
	block: &LIRBlock,
	racx: &mut RegAllocCx,
) -> anyhow::Result<RegAllocResult> {
	let mut out = RegAllocResult {
		regs: HashMap::new(),
		locals: HashMap::new(),
	};

	for instr in &block.contents {
		let used_regs = get_used_regs(&instr.kind);
		for reg_id in used_regs {
			let reg = block
				.regs
				.get(reg_id)
				.ok_or(anyhow!("Used register does not exist"))?;
			match reg.ty {
				DataType::Score(..) => {
					if !out.regs.contains_key(reg_id) {
						out.regs.insert(reg_id.clone(), racx.new_reg());
					}
				}
				DataType::NBT(..) => {
					if !out.locals.contains_key(reg_id) {
						out.locals.insert(reg_id.clone(), racx.new_local());
					}
				}
			}
		}
	}

	Ok(out)
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
		| LIRInstrKind::MaxScore(left, right)
		| LIRInstrKind::SetData(left, right) => [left.get_used_regs(), right.get_used_regs()].concat(),
		LIRInstrKind::SwapScore(left, right) => {
			[left.get_used_regs(), right.get_used_regs()].concat()
		}
	}
}
