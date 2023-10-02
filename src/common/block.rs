use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct BlockAllocator<Block> {
	blocks: HashMap<BlockID, Block>,
	block_count: BlockID,
}

pub type BlockID = u32;

impl<Block> BlockAllocator<Block> {
	pub fn new() -> Self {
		Self {
			blocks: HashMap::new(),
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
		self.blocks.remove(&id)
	}
}
