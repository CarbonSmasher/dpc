use crate::common::mc::entity::{SelectorParameter, TargetSelector};
use crate::common::mc::EntityTarget;

use super::super::t::macros::cgwrite;
use super::{Codegen, CodegenBlockCx};

impl Codegen for EntityTarget {
	fn gen_writer<F>(&self, f: &mut F, cbcx: &mut CodegenBlockCx) -> anyhow::Result<()>
	where
		F: std::fmt::Write,
	{
		match self {
			Self::Player(player) => write!(f, "{player}")?,
			Self::Selector(sel) => sel.gen_writer(f, cbcx)?,
		}

		Ok(())
	}
}

impl Codegen for TargetSelector {
	fn gen_writer<F>(&self, f: &mut F, cbcx: &mut CodegenBlockCx) -> anyhow::Result<()>
	where
		F: std::fmt::Write,
	{
		write!(f, "{}", self.selector.codegen_str())?;

		if !self.params.is_empty() {
			write!(f, "[")?;
			for (i, param) in self.params.iter().enumerate() {
				match param {
					SelectorParameter::Distance { range } => {
						cgwrite!(f, cbcx, "distance=", range)?;
					}
					SelectorParameter::Type { ty, invert } => {
						let invert = gen_invert_char(*invert);
						write!(f, "type={invert}{ty}")?;
					}
					SelectorParameter::Tag { tag, invert } => {
						let invert = gen_invert_char(*invert);
						write!(f, "tag={invert}{tag}")?;
					}
					SelectorParameter::NoTags => {
						write!(f, "tag=")?;
					}
					SelectorParameter::Predicate { predicate, invert } => {
						let invert = gen_invert_char(*invert);
						write!(f, "predicate={invert}{predicate}")?;
					}
					SelectorParameter::Gamemode { gamemode, invert } => {
						let invert = gen_invert_char(*invert);
						write!(f, "gamemode={invert}{gamemode}")?;
					}
					SelectorParameter::Name { name, invert } => {
						let invert = gen_invert_char(*invert);
						write!(f, "gamemode={invert}\"{name}\"")?;
					}
					SelectorParameter::NBT { nbt, invert } => {
						let invert = gen_invert_char(*invert);
						cgwrite!(f, cbcx, "nbt=", invert, nbt.get_literal_str())?;
					}
					SelectorParameter::Limit(limit) => {
						write!(f, "limit={limit}")?;
					}
					SelectorParameter::Sort(sort) => {
						write!(f, "sort={sort}")?;
					}
				}

				if i != self.params.len() - 1 {
					write!(f, ",")?;
				}
			}
			write!(f, "]")?;
		}

		Ok(())
	}
}

fn gen_invert_char(invert: bool) -> &'static str {
	if invert {
		"!"
	} else {
		""
	}
}
