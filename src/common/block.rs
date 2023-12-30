use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct BlockAllocator<Block> {
	blocks: HashMap<BlockID, Block>,
	block_count: BlockID,
}

pub type BlockID = usize;

impl<Block> BlockAllocator<Block> {
	pub fn new() -> Self {
		Self {
			blocks: HashMap::new(),
			block_count: 0,
		}
	}

	pub fn with_capacity(capacity: usize) -> Self {
		Self {
			blocks: HashMap::with_capacity(capacity),
			block_count: 0,
		}
	}

	/// Creates a new block and returns its ID
	pub fn add(&mut self, block: Block) -> BlockID {
		let old_count = self.block_count;
		self.block_count += 1;
		self.blocks.insert(old_count, block);
		old_count
	}

	/// Gets a block
	pub fn get(&self, id: &BlockID) -> Option<&Block> {
		self.blocks.get(id)
	}

	/// Gets a block mutably
	pub fn get_mut(&mut self, id: &BlockID) -> Option<&mut Block> {
		self.blocks.get_mut(id)
	}

	/// Removes a block with an ID
	pub fn remove(&mut self, id: &BlockID) -> Option<Block> {
		self.blocks.remove(id).map(|x| {
			self.block_count -= 1;
			x
		})
	}

	/// Counts the number of blocks
	pub fn count(&self) -> BlockID {
		self.block_count
	}
}

impl<Block> Default for BlockAllocator<Block> {
	fn default() -> Self {
		Self::new()
	}
}

impl<BlockT: Block> BlockAllocator<BlockT> {
	pub fn instr_count(&self) -> usize {
		self.blocks.iter().fold(0, |sum, x| sum + x.1.instr_count())
	}
}

/// Trait for the different types of blocks we use at different
/// stages of IR. Includes some utility methods
pub trait Block {
	fn instr_count(&self) -> usize;

	fn get_children(&self) -> Vec<BlockID>;
}
