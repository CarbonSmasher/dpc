use anyhow::{anyhow, bail};
use rustc_hash::{FxHashMap, FxHashSet};

use crate::common::Identifier;
use crate::common::{reg::GetUsedRegs, ty::DataType};
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

impl Default for RegAllocCx {
	fn default() -> Self {
		Self::new()
	}
}

/// An allocator for a single register type
#[derive(Debug)]
pub struct RegAllocator {
	count: u32,
	available: FxHashSet<u32>,
	available_ordered: Vec<u32>,
}

impl RegAllocator {
	pub fn new() -> Self {
		Self {
			count: 0,
			available: FxHashSet::default(),
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

impl Default for RegAllocator {
	fn default() -> Self {
		Self::new()
	}
}

/// Result from allocating registers for a block
#[derive(Debug)]
pub struct RegAllocResult {
	pub regs: FxHashMap<Identifier, String>,
	pub locals: FxHashMap<Identifier, String>,
}

impl RegAllocResult {
	pub fn new() -> Self {
		Self {
			regs: FxHashMap::default(),
			locals: FxHashMap::default(),
		}
	}
}

impl Default for RegAllocResult {
	fn default() -> Self {
		Self::new()
	}
}

pub fn alloc_block_registers(
	func_id: &str,
	block: &LIRBlock,
	racx: &mut RegAllocCx,
) -> anyhow::Result<RegAllocResult> {
	let func_id = func_id.to_string().replace([':', '/'], "_");
	let mut out_regs = FxHashMap::default();
	let mut out_locals = FxHashMap::default();

	let last_uses = analyze_last_register_uses(block);

	for (i, instr) in block.contents.iter().enumerate() {
		let used_regs = instr.get_used_regs();
		for reg_id in used_regs {
			let reg = block
				.regs
				.get(reg_id)
				.ok_or(anyhow!("Used register {reg_id} does not exist"))?;
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
				_ => bail!("Type not supported"),
			}
		}

		if let Some(regs) = last_uses.get(&i) {
			for reg_id in regs.iter() {
				let reg = block
					.regs
					.get(reg_id)
					.ok_or(anyhow!("Used register {reg_id} does not exist"))?;
				match reg.ty {
					DataType::Score(..) => racx.regs.finish_using(
						*out_regs
							.get(reg_id)
							.ok_or(anyhow!("Used register {reg_id} does not exist"))?,
					),
					DataType::NBT(..) => racx.locals.finish_using(
						*out_locals
							.get(reg_id)
							.ok_or(anyhow!("Used register {reg_id} does not exist"))?,
					),
					_ => bail!("Type not supported"),
				}
			}
		}
	}

	let out = RegAllocResult {
		regs: out_regs
			.iter()
			.map(|(x, y)| (x.clone(), format_reg_fake_player(*y, &func_id)))
			.collect(),
		locals: out_locals
			.iter()
			.map(|(x, y)| (x.clone(), format_local_storage_entry(*y, &func_id)))
			.collect(),
	};

	Ok(out)
}

fn analyze_last_register_uses(block: &LIRBlock) -> FxHashMap<usize, Vec<Identifier>> {
	let mut last_used_positions = FxHashMap::default();
	let mut already_spent = FxHashSet::default();
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
