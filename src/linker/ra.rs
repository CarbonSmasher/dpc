use std::collections::HashMap;

use anyhow::anyhow;
use rayon::prelude::*;

use crate::common::Identifier;
use crate::lir::LIRBlock;
use crate::{common::ty::DataType, lir::LIRInstrKind};

use super::text::{format_local_storage_entry, format_reg_fake_player};

#[derive(Debug)]
pub struct RegAllocCx {
	regs: RegAllocator,
	locals: RegAllocator,
}

impl RegAllocCx {
	pub fn new() -> Self {
		Self {
			regs: RegAllocator::new(),
			locals: RegAllocator::new(),
		}
	}

	pub fn new_reg(&mut self) -> u32 {
		let reg = self.regs.alloc();
		// format_reg_fake_player(reg)
		reg
	}

	pub fn new_local(&mut self) -> u32 {
		let reg = self.locals.alloc();
		// format_local_storage_entry(reg)
		reg
	}

	pub fn finish_using_all(&mut self) {
		self.regs.finish_using_all();
		self.locals.finish_using_all();
	}

	pub fn get_reg_count(&self) -> u32 {
		self.regs.get_count()
	}

	pub fn get_local_count(&self) -> u32 {
		self.locals.get_count()
	}
}

/// An allocator for a single register type
#[derive(Debug)]
pub struct RegAllocator {
	count: u32,
	available: Vec<u32>,
}

impl RegAllocator {
	pub fn new() -> Self {
		Self {
			count: 0,
			available: Vec::new(),
		}
	}

	pub fn alloc(&mut self) -> u32 {
		if let Some(reg) = self.available.pop() {
			reg
		} else {
			let old_count = self.count;
			self.count += 1;
			old_count
		}
	}

	pub fn finish_using(&mut self, reg: u32) {
		if !self.available.contains(&reg) {
			self.available.push(reg);
		}
	}

	pub fn finish_using_all(&mut self) {
		self.available = Vec::new();
		self.count = 0;
	}

	pub fn get_count(&self) -> u32 {
		self.count
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
	let mut out_regs = HashMap::new();
	let mut out_locals = HashMap::new();

	for instr in &block.contents {
		if let LIRInstrKind::FinishUsing(reg_id) = &instr.kind {
			let reg = block
				.regs
				.get(reg_id)
				.ok_or(anyhow!("Used register does not exist"))?;
			match reg.ty {
				DataType::Score(..) => racx.regs.finish_using(
					*out_regs
						.get(reg_id)
						.ok_or(anyhow!("Register does not exist"))?,
				),
				DataType::NBT(..) => racx.locals.finish_using(
					*out_locals
						.get(reg_id)
						.ok_or(anyhow!("Register does not exist"))?,
				),
			}
			continue;
		}
		let used_regs = instr.kind.get_used_regs();
		for reg_id in used_regs {
			let reg = block
				.regs
				.get(reg_id)
				.ok_or(anyhow!("Used register does not exist"))?;
			match reg.ty {
				DataType::Score(..) => {
					if !out_regs.contains_key(reg_id) {
						out_regs.insert(reg_id.clone(), racx.new_reg());
					}
				}
				DataType::NBT(..) => {
					if !out_locals.contains_key(reg_id) {
						out_locals.insert(reg_id.clone(), racx.new_local());
					}
				}
			}
		}
	}

	// We are finished using all registers in this block
	racx.finish_using_all();

	let out = RegAllocResult {
		regs: out_regs
			.par_iter()
			.map(|(x, y)| (x.clone(), format_reg_fake_player(*y)))
			.collect(),
		locals: out_locals
			.par_iter()
			.map(|(x, y)| (x.clone(), format_local_storage_entry(*y)))
			.collect(),
	};

	Ok(out)
}
