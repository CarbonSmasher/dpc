pub mod entity_target;

use std::sync::Arc;

use crate::common::mc::block::{
	BlockData, BlockFilter, BlockProperties, BlockStateValue, BlockStates, CloneMaskMode,
	CloneMode, FillMode, SetBlockMode,
};
use crate::common::mc::entity::{SelectorSort, SelectorType};
use crate::common::mc::modifier::{AlignAxes, AnchorLocation};
use crate::common::mc::{
	DataLocation, Difficulty, FullDataLocation, Gamemode, Heightmap, Weather, XPValue,
};
use crate::common::val::{MutableNBTValue, MutableScoreValue, NBTValue, ScoreValue};

use super::util::{get_mut_nbt_val_loc, get_mut_score_val_score, get_score_val_score};
use super::{t::macros::cgwrite, Codegen, CodegenBlockCx};

macro_rules! cg_impl {
	($name:ty, $self:ident, $f:ident, $b:tt) => {
		impl Codegen for $name {
			fn gen_writer<F>(&$self, $f: &mut F, _cbcx: &mut CodegenBlockCx) -> anyhow::Result<()>
			where
				F: std::fmt::Write,
			{
				$b()
			}
		}
	};

	($name:ty, $self:ident, $f:ident, $cbcx:ident, $b:tt) => {
		impl Codegen for $name {
			fn gen_writer<F>(&$self, $f: &mut F, $cbcx: &mut CodegenBlockCx) -> anyhow::Result<()>
			where
				F: std::fmt::Write,
			{
				$b()
			}
		}
	};
}

macro_rules! impl_disp {
	($name:ty) => {
		cg_impl!(
			$name,
			self,
			f,
			cbcx,
			(|| {
				let _ = cbcx;
				write!(f, "{self}")?;
				Ok(())
			})
		);
	};
}

macro_rules! impl_dbg {
	($name:ty) => {
		cg_impl!(
			$name,
			self,
			f,
			cbcx,
			(|| {
				let _ = cbcx;
				write!(f, "{self:?}")?;
				Ok(())
			})
		);
	};
}

macro_rules! impl_cg_str {
	($name:ty) => {
		cg_impl!(
			$name,
			self,
			f,
			cbcx,
			(|| {
				let _ = cbcx;
				write!(f, "{}", self.codegen_str())?;
				Ok(())
			})
		);
	};
}

// Common types
impl_disp!(str);
impl_disp!(String);
impl_disp!(i32);
impl_disp!(Arc<str>);

// Value types
impl Codegen for ScoreValue {
	fn gen_writer<F>(&self, f: &mut F, cbcx: &mut CodegenBlockCx) -> anyhow::Result<()>
	where
		F: std::fmt::Write,
	{
		let (score, lit) = get_score_val_score(self, &cbcx.ra)?;
		cbcx.ccx.score_literals.extend(lit);
		score.gen_writer(f, cbcx)
	}
}

impl Codegen for MutableScoreValue {
	fn gen_writer<F>(&self, f: &mut F, cbcx: &mut CodegenBlockCx) -> anyhow::Result<()>
	where
		F: std::fmt::Write,
	{
		let score = get_mut_score_val_score(self, &cbcx.ra)?;
		score.gen_writer(f, cbcx)
	}
}

impl Codegen for NBTValue {
	fn gen_writer<F>(&self, f: &mut F, cbcx: &mut CodegenBlockCx) -> anyhow::Result<()>
	where
		F: std::fmt::Write,
	{
		match self {
			Self::Constant(val) => write!(f, "{}", val.get_literal_str())?,
			Self::Mutable(val) => val.gen_writer(f, cbcx)?,
		}
		Ok(())
	}
}

impl Codegen for MutableNBTValue {
	fn gen_writer<F>(&self, f: &mut F, cbcx: &mut CodegenBlockCx) -> anyhow::Result<()>
	where
		F: std::fmt::Write,
	{
		let loc = get_mut_nbt_val_loc(self, &cbcx.ra)?;
		loc.gen_writer(f, cbcx)?;

		Ok(())
	}
}

// Arg types
cg_impl!(
	DataLocation,
	self,
	f,
	cbcx,
	(|| {
		match self {
			Self::Block(pos) => cgwrite!(f, cbcx, "block ", pos)?,
			Self::Entity(target) => cgwrite!(f, cbcx, "entity ", target)?,
			Self::Storage(loc) => cgwrite!(f, cbcx, "storage ", loc)?,
		}
		Ok(())
	})
);
cg_impl!(
	FullDataLocation,
	self,
	f,
	cbcx,
	(|| {
		cgwrite!(f, cbcx, self.loc, " ", self.path)?;
		Ok(())
	})
);
impl_disp!(XPValue);
impl_disp!(Difficulty);
impl_disp!(Gamemode);
impl_dbg!(Heightmap);
impl_disp!(Weather);

// Selectors
impl_disp!(SelectorSort);
impl_cg_str!(SelectorType);

// Blocks
cg_impl!(
	BlockData,
	self,
	f,
	cbcx,
	(|| {
		write!(f, "{}", self.block)?;
		cgwrite!(f, cbcx, self.props)?;
		Ok(())
	})
);
cg_impl!(
	BlockFilter,
	self,
	f,
	cbcx,
	(|| {
		write!(f, "{}", self.block)?;
		cgwrite!(f, cbcx, self.props)?;
		Ok(())
	})
);
cg_impl!(
	BlockProperties,
	self,
	f,
	cbcx,
	(|| {
		if !self.states.is_empty() {
			cgwrite!(f, cbcx, self.states)?;
		}

		if !self.data.is_empty() {
			cgwrite!(f, cbcx, self.data.get_literal_str())?;
		}

		Ok(())
	})
);
impl_dbg!(SetBlockMode);
impl_dbg!(FillMode);
cg_impl!(
	CloneMaskMode,
	self,
	f,
	cbcx,
	(|| {
		if let Self::Filtered(filter) = self {
			cgwrite!(f, cbcx, "filtered ", filter)?;
		} else {
			write!(f, "{self:?}")?;
		}
		Ok(())
	})
);
impl_dbg!(CloneMode);
cg_impl!(
	BlockStates,
	self,
	f,
	(|| {
		write!(f, "[")?;

		for (i, (k, v)) in self.get().iter().enumerate() {
			write!(f, "\"{k}\"=")?;
			match v {
				BlockStateValue::String(string) => write!(f, "{string}")?,
			}
			if i != self.get().len() - 1 {
				write!(f, ",")?;
			}
		}

		write!(f, "]")?;
		Ok(())
	})
);

// Positions
impl_dbg!(AnchorLocation);
impl_dbg!(AlignAxes);
