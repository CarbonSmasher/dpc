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
	Arr(NBTArrayType),
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
			Self::Arr(other_arr) => {
				matches!(self, Self::Arr(this_arr) if this_arr.is_trivially_castable(other_arr))
			}
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
			Self::Byte => "nbyte".to_string(),
			Self::Bool => "nbool".to_string(),
			Self::Short => "nshort".to_string(),
			Self::Int => "nint".to_string(),
			Self::Long => "nlong".to_string(),
			Self::Arr(arr) => format!("{arr:?}"),
		};
		write!(f, "{text}")
	}
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum NBTArrayType {
	Byte(ArraySize),
	Int(ArraySize),
	Long(ArraySize),
}

impl NBTArrayType {
	pub fn is_trivially_castable(&self, other: &NBTArrayType) -> bool {
		match other {
			Self::Byte(other_size) => {
				matches!(self, Self::Byte(this_size) if this_size == other_size)
			}
			Self::Int(other_size) => {
				matches!(self, Self::Byte(this_size) | Self::Int(this_size) if this_size == other_size)
			}
			Self::Long(other_size) => {
				matches!(self, Self::Byte(this_size) | Self::Int(this_size) | Self::Long(this_size) if this_size == other_size)
			}
		}
	}
}

impl Debug for NBTArrayType {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let text = match self {
			Self::Byte(size) => format!("nbyte[{size}]"),
			Self::Int(size) => format!("nint[{size}]"),
			Self::Long(size) => format!("nlong[{size}]"),
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
	Byte(Byte),
	Bool(bool),
	Short(Short),
	Int(Int),
	Long(Long),
	Arr(NBTArrayTypeContents),
}

impl NBTTypeContents {
	pub fn get_ty(&self) -> DataType {
		match self {
			Self::Byte(..) => DataType::NBT(NBTType::Byte),
			Self::Bool(..) => DataType::NBT(NBTType::Bool),
			Self::Short(..) => DataType::NBT(NBTType::Short),
			Self::Int(..) => DataType::NBT(NBTType::Int),
			Self::Long(..) => DataType::NBT(NBTType::Long),
			Self::Arr(arr) => arr.get_ty(),
		}
	}

	pub fn get_literal_str(&self) -> String {
		match self {
			Self::Byte(val) => format!("{val}b"),
			Self::Bool(val) => format!("{}b", *val as Byte),
			Self::Short(val) => format!("{val}s"),
			Self::Int(val) => format!("{val}"),
			Self::Long(val) => format!("{val}l"),
			Self::Arr(arr) => arr.get_literal_str(),
		}
	}
}

impl Debug for NBTTypeContents {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let text = match self {
			Self::Byte(val) => format!("{val}b"),
			Self::Bool(val) => format!("{}B", *val as Byte),
			Self::Short(val) => format!("{val}s"),
			Self::Int(val) => format!("{val}i"),
			Self::Long(val) => format!("{val}l"),
			Self::Arr(val) => format!("{val:?}"),
		};
		write!(f, "{text}")
	}
}

#[derive(Clone)]
pub enum NBTArrayTypeContents {
	Byte(Vec<Byte>, ArraySize),
	Int(Vec<Int>, ArraySize),
	Long(Vec<Long>, ArraySize),
}

impl NBTArrayTypeContents {
	pub fn get_ty(&self) -> DataType {
		match self {
			Self::Byte(_, len) => DataType::NBT(NBTType::Arr(NBTArrayType::Byte(*len))),
			Self::Int(_, len) => DataType::NBT(NBTType::Arr(NBTArrayType::Int(*len))),
			Self::Long(_, len) => DataType::NBT(NBTType::Arr(NBTArrayType::Long(*len))),
		}
	}

	pub fn get_literal_str(&self) -> String {
		match self {
			Self::Byte(val, ..) => format!("[B;{}]", fmt_arr(val.iter().map(|x| format!("{x}b")))),
			Self::Int(val, ..) => format!("[I;{}]", fmt_arr(val)),
			Self::Long(val, ..) => format!("[L;{}]", fmt_arr(val.iter().map(|x| format!("{x}l")))),
		}
	}

	pub fn const_index(&self, index: ArraySize) -> Option<String> {
		match self {
			Self::Byte(val, ..) => val.get(index).map(ToString::to_string),
			Self::Int(val, ..) => val.get(index).map(ToString::to_string),
			Self::Long(val, ..) => val.get(index).map(ToString::to_string),
		}
	}
}

impl Debug for NBTArrayTypeContents {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let text = match self {
			Self::Byte(val, ..) => format!("[B;{}]", fmt_arr(val)),
			Self::Int(val, ..) => format!("[I;{}]", fmt_arr(val)),
			Self::Long(val, ..) => format!("[L;{}]", fmt_arr(val)),
		};
		write!(f, "{text}")
	}
}

// Const type contents helpers

pub fn create_nbyte_array(contents: Vec<i8>) -> NBTArrayTypeContents {
	let len = contents.len();
	NBTArrayTypeContents::Byte(contents, len)
}

pub fn create_nint_array(contents: Vec<i32>) -> NBTArrayTypeContents {
	let len = contents.len();
	NBTArrayTypeContents::Int(contents, len)
}

pub fn create_nlong_array(contents: Vec<i64>) -> NBTArrayTypeContents {
	let len = contents.len();
	NBTArrayTypeContents::Long(contents, len)
}

// Types

pub type ArraySize = usize;
pub type Byte = i8;
pub type Short = i16;
pub type Int = i32;
pub type Long = i64;

fn fmt_arr<T: ToString>(arr: impl IntoIterator<Item = T>) -> String {
	arr.into_iter()
		.map(|x| x.to_string())
		.collect::<Vec<_>>()
		.join(",")
}

pub fn get_op_tys(
	left: &MutableValue,
	right: &Value,
	regs: &RegisterList,
) -> anyhow::Result<(DataType, DataType)> {
	Ok((left.get_ty(regs)?, right.get_ty(regs)?))
}
