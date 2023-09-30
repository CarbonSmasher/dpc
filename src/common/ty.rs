use std::fmt::Debug;

use super::{MutableValue, RegisterList, Value};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum DataType {
	Score(ScoreType),
	NBT(NBTType),
}

impl DataType {
	pub fn is_trivially_castable(&self, other: &DataType) -> bool {
		match other {
			DataType::Score(other_score) => match self {
				Self::Score(this_score) => this_score.is_trivially_castable(other_score),
				_ => false,
			},
			DataType::NBT(other_nbt) => match self {
				Self::NBT(this_nbt) => this_nbt.is_trivially_castable(other_nbt),
				_ => false,
			},
		}
	}
}

impl Debug for DataType {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Score(score) => score.fmt(f),
			Self::NBT(nbt) => nbt.fmt(f),
		}
	}
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ScoreType {
	Score,
	UScore,
	Bool,
}

impl ScoreType {
	pub fn is_trivially_castable(&self, other: &ScoreType) -> bool {
		match other {
			Self::Score => {
				matches!(self, Self::Score | Self::UScore | Self::Bool)
			}
			Self::UScore => matches!(self, Self::Score | Self::UScore),
			Self::Bool => matches!(self, Self::Bool),
		}
	}
}

impl Debug for ScoreType {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let text = match self {
			Self::Score => "score",
			Self::UScore => "uscore",
			Self::Bool => "bool",
		};
		write!(f, "{text}")
	}
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum NBTType {
	Byte,
	Bool,
	Short,
	Int,
	Long,
}

impl NBTType {
	pub fn is_trivially_castable(&self, other: &NBTType) -> bool {
		match other {
			Self::Byte => matches!(self, Self::Byte | Self::Bool),
			Self::Bool => matches!(self, Self::Bool),
			Self::Short => matches!(self, Self::Byte | Self::Bool | Self::Short),
			Self::Int => matches!(self, Self::Byte | Self::Bool | Self::Short | Self::Int),
			Self::Long => matches!(
				self,
				Self::Byte | Self::Bool | Self::Short | Self::Int | Self::Long
			),
		}
	}

	pub fn is_castable_to_score(&self, other: &ScoreType) -> bool {
		match other {
			ScoreType::Score => true,
			ScoreType::UScore => true,
			ScoreType::Bool => true,
		}
	}
}

impl Debug for NBTType {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let text = match self {
			Self::Byte => "nbyte",
			Self::Bool => "nbool",
			Self::Short => "nshort",
			Self::Int => "nint",
			Self::Long => "nlong",
		};
		write!(f, "{text}")
	}
}

#[derive(Clone)]
pub enum DataTypeContents {
	Score(ScoreTypeContents),
	NBT(NBTTypeContents),
}

impl DataTypeContents {
	pub fn get_ty(&self) -> DataType {
		match self {
			Self::Score(score) => score.get_ty(),
			Self::NBT(nbt) => nbt.get_ty(),
		}
	}
}

impl Debug for DataTypeContents {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Score(score) => score.fmt(f),
			Self::NBT(nbt) => nbt.fmt(f),
		}
	}
}

#[derive(Clone)]
pub enum ScoreTypeContents {
	Score(i32),
	UScore(u16),
	Bool(bool),
}

impl ScoreTypeContents {
	pub fn get_ty(&self) -> DataType {
		match self {
			Self::Score(..) => DataType::Score(ScoreType::Score),
			Self::UScore(..) => DataType::Score(ScoreType::UScore),
			Self::Bool(..) => DataType::Score(ScoreType::Bool),
		}
	}

	pub fn get_i32(&self) -> i32 {
		match self {
			ScoreTypeContents::Score(score) => *score,
			ScoreTypeContents::UScore(score) => *score as i32,
			ScoreTypeContents::Bool(score) => *score as i32,
		}
	}

	pub fn get_literal_str(&self) -> String {
		self.get_i32().to_string()
	}
}

impl Debug for ScoreTypeContents {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let text = match self {
			Self::Score(val) => format!("{val}.s"),
			Self::UScore(val) => format!("{val}.u"),
			Self::Bool(val) => format!("{val}"),
		};
		write!(f, "{text}")
	}
}

#[derive(Clone)]
pub enum NBTTypeContents {
	Byte(i8),
	Bool(bool),
	Short(i16),
	Int(i32),
	Long(i64),
}

impl NBTTypeContents {
	pub fn get_ty(&self) -> DataType {
		match self {
			Self::Byte(..) => DataType::NBT(NBTType::Byte),
			Self::Bool(..) => DataType::NBT(NBTType::Bool),
			Self::Short(..) => DataType::NBT(NBTType::Short),
			Self::Int(..) => DataType::NBT(NBTType::Int),
			Self::Long(..) => DataType::NBT(NBTType::Long),
		}
	}

	pub fn get_i64(&self) -> i64 {
		match self {
			Self::Byte(val) => *val as i64,
			Self::Bool(val) => *val as i64,
			Self::Short(val) => *val as i64,
			Self::Int(val) => *val as i64,
			Self::Long(val) => *val as i64,
		}
	}

	pub fn get_literal_str(&self) -> String {
		match self {
			Self::Byte(val) => format!("{val}b"),
			Self::Bool(val) => format!("{}b", *val as i8),
			Self::Short(val) => format!("{val}s"),
			Self::Int(val) => format!("{val}"),
			Self::Long(val) => format!("{val}l"),
		}
	}
}

impl Debug for NBTTypeContents {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let text = match self {
			Self::Byte(val) => format!("{val}b"),
			Self::Bool(val) => format!("{}B", *val as i8),
			Self::Short(val) => format!("{val}s"),
			Self::Int(val) => format!("{val}i"),
			Self::Long(val) => format!("{val}l"),
		};
		write!(f, "{text}")
	}
}

pub fn get_op_tys(
	left: &MutableValue,
	right: &Value,
	regs: &RegisterList,
) -> anyhow::Result<(DataType, DataType)> {
	Ok((left.get_ty(regs)?, right.get_ty(regs)?))
}
