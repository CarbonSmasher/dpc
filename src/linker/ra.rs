use std::collections::HashMap;

use anyhow::anyhow;
use dashmap::{DashMap, DashSet};

use crate::common::ty::DataType;
use crate::common::Identifier;
use crate::lir::LIRBlock;

use super::text::{format_local_storage_entry, format_reg_fake_player};

#[derive(Debug)]
pub struct RegAllocCx {
	regs: RegAllocator,
	locals: RegAllocator,
	has_allocated_reg: bool,
	has_allocated_local: bool,
}

impl RegAllocCx {
	pub fn new() -> Self {
		Self {
			regs: RegAllocator::new(),
			locals: RegAllocator::new(),
			has_allocated_reg: false,
			has_allocated_local: false,
		}
	}

	pub fn new_reg(&mut self) -> u32 {
		self.has_allocated_reg = true;
		self.regs.alloc()
	}

	pub fn new_local(&mut self) -> u32 {
		self.has_allocated_local = true;
		self.locals.alloc()
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

	pub fn has_allocated_reg(&self) -> bool {
		self.has_allocated_reg
	}

	pub fn has_allocated_local(&self) -> bool {
		self.has_allocated_local
	}
}

/// An allocator for a single register type
#[derive(Debug)]
pub struct RegAllocator {
	count: u32,
	available: DashSet<u32>,
	available_ordered: Vec<u32>,
}

impl RegAllocator {
	pub fn new() -> Self {
		Self {
			count: 0,
			available: DashSet::new(),
			available_ordered: Vec::new(),
		}
	}

	pub fn alloc(&mut self) -> u32 {
		// If an existing register number is available because it was finished,
		// then we use that. Otherwise, increase the count.
		if let Some(reg) = self.available_ordered.pop() {
			self.available.remove(&reg);
			reg
		} else {
			let old_count = self.count;
			self.count += 1;
			old_count
		}
	}

	pub fn finish_using(&mut self, reg: u32) {
		let not_already_in = self.available.insert(reg);
		if not_already_in {
			self.available_ordered.push(reg);
		}
	}

	pub fn finish_using_all(&mut self) {
		self.available.clear();
		self.available_ordered.clear();
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

impl RegAllocResult {
	pub fn new() -> Self {
		Self {
			regs: HashMap::new(),
			locals: HashMap::new(),
		}
	}
}

pub fn alloc_block_registers(
	block: &LIRBlock,
	racx: &mut RegAllocCx,
) -> anyhow::Result<RegAllocResult> {
	let mut out_regs = HashMap::new();
	let mut out_locals = HashMap::new();

	let last_uses = analyze_last_register_uses(block);

	for (i, instr) in block.contents.iter().enumerate() {
		let used_regs = instr.get_used_regs();
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

		if let Some(regs) = last_uses.get(&i) {
			for reg_id in regs.iter() {
				let reg = block
					.regs
					.get(reg_id)
					.ok_or(anyhow!("Used register does not exist"))?;
				match reg.ty {
					DataType::Score(..) => racx.regs.finish_using(
						*out_regs
							.get(reg_id)
							.ok_or(anyhow!("Used register does not exist"))?,
					),
					DataType::NBT(..) => racx.locals.finish_using(
						*out_locals
							.get(reg_id)
							.ok_or(anyhow!("Used register does not exist"))?,
					),
				}
			}
		}
	}

	let out = RegAllocResult {
		regs: out_regs
			.iter()
			.map(|(x, y)| (x.clone(), format_reg_fake_player(*y)))
			.collect(),
		locals: out_locals
			.iter()
			.map(|(x, y)| (x.clone(), format_local_storage_entry(*y)))
			.collect(),
	};

	Ok(out)
}

fn analyze_last_register_uses(block: &LIRBlock) -> DashMap<usize, Vec<Identifier>> {
	let last_used_positions = DashMap::new();
	let mut already_spent = DashSet::new();
	for (i, instr) in block.contents.iter().enumerate().rev() {
		let used_regs = instr.get_used_regs();
		last_used_positions.insert(
			i,
			used_regs
				.iter()
				.filter(|x| !already_spent.contains(*x))
				.map(|x| (*x).clone())
				.collect(),
		);
		already_spent.extend(used_regs);
	}

	last_used_positions
}
