pub mod entity_target;

use std::sync::Arc;

use crate::common::mc::block::{
	BlockData, BlockFilter, BlockProperties, BlockStateValue, BlockStates, CloneMaskMode,
	CloneMode, FillMode, SetBlockMode,
};
use crate::common::mc::entity::{AttributeType, EffectDuration, SelectorSort, SelectorType, UUID};
use crate::common::mc::item::ItemData;
use crate::common::mc::modifier::{AlignAxes, AnchorLocation};
use crate::common::mc::pos::{AbsOrRelCoord, IntCoordinates, IntCoordinates2D, DoubleCoordinates, DoubleCoordinates2D};
use crate::common::mc::scoreboard_and_teams::SingleCriterion;
use crate::common::mc::{
	DataLocation, DataPath, DatapackListMode, DatapackOrder, DatapackPriority, Difficulty,
	FullDataLocation, Gamemode, Heightmap, Weather, XPValue,
};
use crate::common::val::{MutableNBTValue, MutableScoreValue, NBTValue, ScoreValue};

use super::util::{get_mut_nbt_val_loc, get_mut_score_val_score, get_score_val_score, cg_float};
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
impl_disp!(u8);
impl_disp!(u32);
impl_disp!(f32);
impl_disp!(f64);
impl_disp!(bool);
impl_disp!(Arc<str>);

// Value types
impl Codegen for ScoreValue {
	fn gen_writer<F>(&self, f: &mut F, cbcx: &mut CodegenBlockCx) -> anyhow::Result<()>
	where
		F: std::fmt::Write,
	{
		let (score, lit) = get_score_val_score(self, &cbcx.ra, &cbcx.func_id)?;
		cbcx.ccx.score_literals.extend(lit);
		score.gen_writer(f, cbcx)
	}
}

impl Codegen for MutableScoreValue {
	fn gen_writer<F>(&self, f: &mut F, cbcx: &mut CodegenBlockCx) -> anyhow::Result<()>
	where
		F: std::fmt::Write,
	{
		let score = get_mut_score_val_score(self, &cbcx.ra, &cbcx.func_id)?;
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
		let loc = get_mut_nbt_val_loc(self, &cbcx.ra, &cbcx.func_id)?;
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
		cgwrite!(f, cbcx, self.loc)?;
		if let DataPath::This = self.path {
		} else {
			cgwrite!(f, cbcx, " ", self.path)?;
		}
		Ok(())
	})
);
impl_dbg!(DataPath);
impl_disp!(XPValue);
impl_disp!(Difficulty);
impl_disp!(Gamemode);
impl_dbg!(Heightmap);
impl_disp!(Weather);
impl_dbg!(SingleCriterion);
impl_dbg!(UUID);
impl_dbg!(AttributeType);
impl_dbg!(DatapackPriority);
impl_dbg!(DatapackOrder);
impl_dbg!(DatapackListMode);
impl_dbg!(EffectDuration);

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

// Items
cg_impl!(
	ItemData,
	self,
	f,
	cbcx,
	(|| {
		cgwrite!(f, cbcx, self.item)?;

		if !self.nbt.is_empty() {
			cgwrite!(f, cbcx, self.nbt.get_literal_str())?;
		}

		Ok(())
	})
);

// Positions
cg_impl!(
	AbsOrRelCoord<f64>,
	self,
	f,
	(|| {
		match self {
			Self::Abs(val) => {
				cg_float(f, *val, false, true, true)?;
			}
			Self::Rel(val) => {
				write!(f, "~")?;
				cg_float(f, *val, true, true, true)?;
			}
		}

		Ok(())
	})
);

impl_dbg!(AbsOrRelCoord<i32>);

cg_impl!(
	DoubleCoordinates,
	self,
	f,
	cbcx,
	(|| {
		match self {
			Self::XYZ(x, y, z) => {
				x.gen_writer(f, cbcx)?;
				write!(f, " ")?;
				y.gen_writer(f, cbcx)?;
				write!(f, " ")?;
				z.gen_writer(f, cbcx)?;
			}
			Self::Local(a, b, c) => {
				write!(f, "^")?;
				cg_float(f, *a, true, true, true)?;
				write!(f, " ^")?;
				cg_float(f, *b, true, true, true)?;
				write!(f, " ^")?;
				cg_float(f, *c, true, true, true)?;
			}
		}
		Ok(())
	})
);

impl_dbg!(IntCoordinates);
impl_dbg!(IntCoordinates2D);

cg_impl!(
	DoubleCoordinates2D,
	self,
	f,
	cbcx,
	(|| {
		self.0.gen_writer(f, cbcx)?;
		write!(f, " ")?;
		self.1.gen_writer(f, cbcx)?;
		write!(f, " ")?;
		Ok(())
	})
);

impl_dbg!(AnchorLocation);
impl_dbg!(AlignAxes);
