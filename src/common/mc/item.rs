use crate::common::ty::NBTCompoundTypeContents;
use crate::common::ResourceLocation;
use std::fmt::Debug;

#[derive(Clone, PartialEq, Eq)]
pub struct ItemData {
	pub item: ResourceLocation,
	pub nbt: NBTCompoundTypeContents,
}

impl Debug for ItemData {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{:?} {:?}", self.item, self.nbt)
	}
}
