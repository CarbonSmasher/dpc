use std::fmt::Write;
use std::{fmt::Debug, sync::Arc};

use itertools::Itertools;
use rustc_hash::FxHashMap;

use crate::output::codegen::util::cg_float;

use super::function::FunctionSignature;
use super::{MutableValue, RegisterList, Value};

#[derive(Clone, PartialEq, Eq)]
pub enum DataType {
	Score(ScoreType),
	NBT(NBTType),
	Macro(MacroType),
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
			DataType::Macro(other_mac) => match self {
				Self::Macro(this_mac) => this_mac.is_trivially_castable(other_mac),
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
			Self::Macro(mac) => mac.fmt(f),
		}
	}
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum ScoreType {
	Score,
	Bool,
}

impl ScoreType {
	pub fn is_trivially_castable(&self, other: &ScoreType) -> bool {
		match other {
			Self::Score => {
				matches!(self, Self::Score | Self::Bool)
			}
			Self::Bool => matches!(self, Self::Score | Self::Bool),
		}
	}
}

impl Debug for ScoreType {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let text = match self {
			Self::Score => "score",
			Self::Bool => "bool",
		};
		write!(f, "{text}")
	}
}

#[derive(Clone, PartialEq, Eq)]
pub enum NBTType {
	Byte,
	Bool,
	Short,
	Int,
	Long,
	Float,
	Double,
	String,
	Arr(NBTArrayType),
	List(Box<NBTType>),
	Compound(NBTCompoundType),
	Any,
}

pub type NBTCompoundType = Arc<FxHashMap<String, NBTType>>;

impl NBTType {
	pub fn is_trivially_castable(&self, other: &NBTType) -> bool {
		match other {
			// Anything can be trivially cast to NBT any
			Self::Any => true,
			Self::Byte => matches!(self, Self::Byte | Self::Bool),
			Self::Bool => matches!(self, Self::Byte | Self::Bool),
			Self::Short => matches!(self, Self::Byte | Self::Bool | Self::Short),
			Self::Int => matches!(self, Self::Byte | Self::Bool | Self::Short | Self::Int),
			Self::Long => matches!(
				self,
				Self::Byte | Self::Bool | Self::Short | Self::Int | Self::Long
			),
			Self::Float => matches!(self, Self::Float),
			Self::Double => matches!(self, Self::Float | Self::Double),
			Self::String => matches!(self, Self::String),
			Self::Arr(other_arr) => {
				matches!(self, Self::Arr(this_arr) if this_arr.is_trivially_castable(other_arr))
			}
			Self::List(other_list) => {
				matches!(self, Self::List(this_list) if this_list.is_trivially_castable(other_list))
			}
			Self::Compound(other_comp) => {
				if let Self::Compound(this_comp) = self {
					this_comp.iter().all(|(k, v)| {
						if let Some(other) = other_comp.get(k) {
							other.is_trivially_castable(v)
						} else {
							false
						}
					})
				} else {
					false
				}
			}
		}
	}

	pub fn can_contain(&self, ty: &NBTType) -> bool {
		match self {
			Self::Arr(arr) => arr.can_contain(ty),
			Self::List(list) => ty.is_trivially_castable(list),
			_ => false,
		}
	}

	pub fn is_int_type(&self) -> bool {
		matches!(
			self,
			Self::Byte | Self::Bool | Self::Short | Self::Int | Self::Long
		)
	}

	pub fn is_castable_to_score(&self, other: &ScoreType) -> bool {
		if self.is_int_type() {
			match other {
				ScoreType::Score => true,
				ScoreType::Bool => true,
			}
		} else {
			false
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
			Self::Float => "nfloat".to_string(),
			Self::Double => "ndouble".to_string(),
			Self::String => "nstr".to_string(),
			Self::Arr(arr) => format!("{arr:?}"),
			Self::List(ty) => format!("{ty:?}[]"),
			Self::Compound(tys) => format!("{tys:?}"),
			Self::Any => "any".to_string(),
		};
		write!(f, "{text}")
	}
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
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

	pub fn can_contain(&self, ty: &NBTType) -> bool {
		match self {
			Self::Byte(..) => ty.is_trivially_castable(&NBTType::Byte),
			Self::Int(..) => ty.is_trivially_castable(&NBTType::Int),
			Self::Long(..) => ty.is_trivially_castable(&NBTType::Long),
		}
	}
}

impl Debug for NBTArrayType {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let text = match self {
			Self::Byte(size) => format!("[nbyte;{size}]"),
			Self::Int(size) => format!("[nint;{size}]"),
			Self::Long(size) => format!("[nlong;{size}]"),
		};
		write!(f, "{text}")
	}
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum MacroType {
	Raw,
}

impl MacroType {
	pub fn is_trivially_castable(&self, other: &MacroType) -> bool {
		match other {
			Self::Raw => {
				matches!(self, Self::Raw)
			}
		}
	}
}

impl Debug for MacroType {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let text = match self {
			Self::Raw => "mraw",
		};
		write!(f, "{text}")
	}
}

#[derive(Clone, PartialEq)]
pub enum DataTypeContents {
	Score(ScoreTypeContents),
	NBT(NBTTypeContents),
	Macro(MacroTypeContents),
}

impl DataTypeContents {
	pub fn get_ty(&self) -> DataType {
		match self {
			Self::Score(score) => score.get_ty(),
			Self::NBT(nbt) => nbt.get_ty(),
			Self::Macro(mac) => mac.get_ty(),
		}
	}

	pub fn is_value_eq(&self, other: &Self) -> bool {
		matches!((self, other), (Self::Score(l), Self::Score(r)) if l.is_value_eq(r))
			|| matches!((self, other), (Self::NBT(l), Self::NBT(r)) if l.is_value_eq(r))
			|| matches!((self, other), (Self::Macro(l), Self::Macro(r)) if l.is_value_eq(r))
	}

	pub fn try_get_i32(&self) -> Option<i32> {
		match self {
			Self::Score(sco) => Some(sco.get_i32()),
			Self::NBT(nbt) => nbt.try_get_i32(),
			_ => None,
		}
	}
}

impl Debug for DataTypeContents {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Score(score) => score.fmt(f),
			Self::NBT(nbt) => nbt.fmt(f),
			Self::Macro(mac) => mac.fmt(f),
		}
	}
}

#[derive(Clone, PartialEq, Eq)]
pub enum ScoreTypeContents {
	Score(i32),
	Bool(bool),
}

impl ScoreTypeContents {
	pub fn get_ty(&self) -> DataType {
		match self {
			Self::Score(..) => DataType::Score(ScoreType::Score),
			Self::Bool(..) => DataType::Score(ScoreType::Bool),
		}
	}

	pub fn get_i32(&self) -> i32 {
		match self {
			Self::Score(score) => *score,
			Self::Bool(score) => *score as i32,
		}
	}

	pub fn get_literal_str(&self) -> String {
		self.get_i32().to_string()
	}

	pub fn is_value_eq(&self, other: &Self) -> bool {
		self.get_i32() == other.get_i32()
	}
}

impl Debug for ScoreTypeContents {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let text = match self {
			Self::Score(val) => format!("{val}s"),
			Self::Bool(val) => format!("{val}"),
		};
		write!(f, "{text}")
	}
}

#[derive(Clone, PartialEq)]
pub enum NBTTypeContents {
	Byte(Byte),
	Bool(bool),
	Short(Short),
	Int(Int),
	Long(Long),
	Float(Float),
	Double(Double),
	String(Arc<str>),
	Arr(NBTArrayTypeContents),
	List(NBTType, Vec<NBTTypeContents>),
	Compound(NBTCompoundType, NBTCompoundTypeContents),
}

impl NBTTypeContents {
	pub fn get_ty(&self) -> DataType {
		match self {
			Self::Byte(..) => DataType::NBT(NBTType::Byte),
			Self::Bool(..) => DataType::NBT(NBTType::Bool),
			Self::Short(..) => DataType::NBT(NBTType::Short),
			Self::Int(..) => DataType::NBT(NBTType::Int),
			Self::Long(..) => DataType::NBT(NBTType::Long),
			Self::Float(..) => DataType::NBT(NBTType::Float),
			Self::Double(..) => DataType::NBT(NBTType::Double),
			Self::String(..) => DataType::NBT(NBTType::String),
			Self::Arr(arr) => arr.get_ty(),
			Self::List(ty, ..) => DataType::NBT(NBTType::List(Box::new(ty.clone()))),
			Self::Compound(ty, ..) => DataType::NBT(NBTType::Compound(ty.clone())),
		}
	}

	pub fn get_literal_str(&self) -> String {
		match self {
			Self::Byte(val) => format!("{val}b"),
			Self::Bool(val) => format!("{}b", *val as Byte),
			Self::Short(val) => format!("{val}s"),
			Self::Int(val) => format!("{val}"),
			Self::Long(val) => format!("{val}l"),
			Self::Float(val) => {
				let mut out = String::new();
				cg_float(&mut out, (*val).into(), false, true, true);
				out + "f"
			}
			Self::Double(val) => {
				let mut out = String::new();
				cg_float(&mut out, *val, false, true, true);
				out
			}
			Self::String(string) => write_string(string.to_string()),
			Self::Arr(arr) => arr.get_literal_str(),
			Self::List(_, list) => {
				let arr = fmt_arr(list.iter().map(|x| x.get_literal_str()));
				format!("[{arr}]")
			}
			Self::Compound(_, comp) => comp.get_literal_str(),
		}
	}

	pub fn is_value_eq(&self, other: &Self) -> bool {
		matches!((self, other), (Self::Byte(l), Self::Byte(r)) if l == r)
			|| matches!((self, other), (Self::Bool(l), Self::Bool(r)) if l == r)
			|| matches!((self, other), (Self::Short(l), Self::Short(r)) if l == r)
			|| matches!((self, other), (Self::Int(l), Self::Int(r)) if l == r)
			|| matches!((self, other), (Self::Long(l), Self::Long(r)) if l == r)
			|| matches!((self, other), (Self::Float(l), Self::Float(r)) if l == r)
			|| matches!((self, other), (Self::Double(l), Self::Double(r)) if l == r)
			|| matches!((self, other), (Self::Arr(l), Self::Arr(r)) if l.is_value_eq(r))
			|| matches!((self, other), (Self::String(l), Self::String(r)) if l == r)
			|| matches!((self, other), (Self::List(lt, l), Self::List(rt, r)) if lt == rt && l.iter().zip(r).all(|(l, r)| l.is_value_eq(r)))
			|| matches!((self, other), (Self::Compound(lt, l), Self::Compound(rt, r)) if lt == rt && l == r)
	}

	pub fn try_get_i32(&self) -> Option<i32> {
		match self {
			Self::Byte(n) => Some(*n as i32),
			Self::Bool(n) => Some(*n as i32),
			Self::Short(n) => Some(*n as i32),
			Self::Int(n) => Some(*n as i32),
			Self::Long(n) => (*n).try_into().ok(),
			_ => None,
		}
	}
}

impl Debug for NBTTypeContents {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Byte(val) => write!(f, "{val}b")?,
			Self::Bool(val) => write!(f, "{}B", *val as Byte)?,
			Self::Short(val) => write!(f, "{val}s")?,
			Self::Int(val) => write!(f, "{val}i")?,
			Self::Long(val) => write!(f, "{val}l")?,
			Self::Float(val) => write!(f, "{val}f")?,
			Self::Double(val) => write!(f, "{val}d")?,
			Self::String(val) => write!(f, "\"{val}\"")?,
			Self::Arr(val) => write!(f, "{val:?}")?,
			Self::List(_, list) => write!(f, "{}", fmt_arr(list.iter().map(|x| format!("{x:?}"))))?,
			Self::Compound(_, comp) => write!(f, "{comp:?}")?,
		};

		Ok(())
	}
}

#[derive(Clone, PartialEq, Eq)]
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

	pub fn get_size(&self) -> &ArraySize {
		let (Self::Byte(_, size) | Self::Int(_, size) | Self::Long(_, size)) = self;
		size
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

	pub fn is_value_eq(&self, other: &Self) -> bool {
		matches!((self, other), (Self::Byte(l, ..), Self::Byte(r, ..)) if l == r)
			|| matches!((self, other), (Self::Int(l, ..), Self::Int(r, ..)) if l == r)
			|| matches!((self, other), (Self::Long(l, ..), Self::Long(r, ..)) if l == r)
	}

	pub fn rectify_size(&mut self) {
		match self {
			Self::Byte(vec, len) => *len = vec.len(),
			Self::Int(vec, len) => *len = vec.len(),
			Self::Long(vec, len) => *len = vec.len(),
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

#[derive(Clone)]
pub struct NBTCompoundTypeContents(pub Arc<FxHashMap<String, NBTTypeContents>>);

impl NBTCompoundTypeContents {
	pub fn new() -> Self {
		Self(Arc::new(FxHashMap::default()))
	}

	pub fn is_empty(&self) -> bool {
		self.0.is_empty()
	}
}

impl Default for NBTCompoundTypeContents {
	fn default() -> Self {
		Self::new()
	}
}

impl PartialEq for NBTCompoundTypeContents {
	fn eq(&self, other: &Self) -> bool {
		self.0.iter().all(|(k, v)| {
			if let Some(other) = other.0.get(k) {
				other.is_value_eq(v)
			} else {
				false
			}
		})
	}
}

impl Eq for NBTCompoundTypeContents {}

impl From<Arc<FxHashMap<String, NBTTypeContents>>> for NBTCompoundTypeContents {
	fn from(value: Arc<FxHashMap<String, NBTTypeContents>>) -> Self {
		Self(value)
	}
}

impl Debug for NBTCompoundTypeContents {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		fmt_compound_dbg(f, &self.0)
	}
}

impl NBTCompoundTypeContents {
	pub fn get_literal_str(&self) -> String {
		let mut string = String::new();
		let _ = fmt_compound(&mut string, &self.0, |f, i| {
			write!(f, "{}", i.get_literal_str())
		});
		string
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
pub type Float = f32;
pub type Double = f64;

fn fmt_arr<T: ToString>(arr: impl IntoIterator<Item = T>) -> String {
	arr.into_iter()
		.map(|x| x.to_string())
		.collect::<Vec<_>>()
		.join(",")
}

fn fmt_compound_dbg<W: std::fmt::Write, I: Debug>(
	f: &mut W,
	vals: &FxHashMap<String, I>,
) -> std::fmt::Result {
	fmt_compound(f, vals, |f, i| {
		write!(f, "{i:?}")?;
		Ok(())
	})
}

fn fmt_compound<W: std::fmt::Write, I, F: Fn(&mut W, &I) -> std::fmt::Result>(
	f: &mut W,
	vals: &FxHashMap<String, I>,
	fun: F,
) -> std::fmt::Result {
	write!(f, "{{")?;
	for (i, (k, v)) in vals.iter().sorted_by_key(|x| x.0).enumerate() {
		write!(f, "{k}:")?;
		fun(f, v)?;
		if i != vals.len() - 1 {
			write!(f, ",")?;
		}
	}
	write!(f, "}}")?;

	Ok(())
}

#[derive(Clone, PartialEq, Eq)]
pub enum MacroTypeContents {
	Raw(String),
}

impl MacroTypeContents {
	pub fn get_ty(&self) -> DataType {
		match self {
			Self::Raw(..) => DataType::Macro(MacroType::Raw),
		}
	}

	pub fn get_literal_str(&self) -> String {
		match self {
			Self::Raw(val) => val.clone(),
		}
	}

	pub fn is_value_eq(&self, other: &Self) -> bool {
		matches!((self, other), (Self::Raw(l), Self::Raw(r)) if l == r)
	}
}

impl Debug for MacroTypeContents {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let text = match self {
			Self::Raw(val) => format!("\"{val}\"mraw"),
		};
		write!(f, "{text}")
	}
}

fn write_string(string: String) -> String {
	let escaped = string.replace('\"', "\\\"");
	format!("\"{escaped}\"")
}

pub fn get_op_tys(
	left: &MutableValue,
	right: &Value,
	regs: &RegisterList,
	sig: &FunctionSignature,
) -> anyhow::Result<(DataType, DataType)> {
	Ok((left.get_ty(regs, sig)?, right.get_ty(regs, sig)?))
}
