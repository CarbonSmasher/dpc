use intset::GrowSet;

use super::reg::GetUsedRegs;

/// Trait for the different types of blocks we use at different
/// stages of IR. Includes some utility methods
pub trait Block {
	type InstrType;
	type InstrKindType;

	fn contents(&self) -> &Vec<Self::InstrType>;
	fn contents_mut(&mut self) -> &mut Vec<Self::InstrType>;

	fn instr_count(&self) -> usize {
		self.contents().len()
	}

	// Get a growable set of instruction indices
	fn get_index_set(&self) -> GrowSet {
		GrowSet::with_capacity(self.instr_count())
	}
}

impl<B: Block> GetUsedRegs for B
where
	B::InstrType: GetUsedRegs,
{
	fn append_used_regs<'this>(&'this self, regs: &mut Vec<&'this super::Identifier>) {
		for instr in self.contents() {
			instr.append_used_regs(regs);
		}
	}
}
