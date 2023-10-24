use std::fmt::{Debug, Display};

use num_traits::Num;

use crate::common::Identifier;
use crate::linker::codegen::Codegen;
use crate::linker::codegen::CodegenBlockCx;

use super::target_selector::TargetSelector;

#[derive(Clone)]
pub enum EntityTarget {
	Player(String),
	Selector(TargetSelector),
}

impl EntityTarget {
	pub fn is_blank_this(&self) -> bool {
		matches!(self, EntityTarget::Selector(sel) if sel.is_blank_this())
	}

	pub fn is_value_eq(&self, other: &Self) -> bool {
		matches!((self, other), (Self::Player(l), Self::Player(r)) if l == r)
			|| matches!((self, other), (Self::Selector(l), Self::Selector(r)) if l.is_value_eq(r))
	}
}

impl Debug for EntityTarget {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Player(player) => write!(f, "{player}"),
			Self::Selector(sel) => write!(f, "{sel:?}"),
		}
	}
}

#[derive(Clone)]
pub struct Score {
	pub holder: EntityTarget,
	pub objective: Identifier,
}

impl Score {
	pub fn new(holder: EntityTarget, objective: Identifier) -> Self {
		Self { holder, objective }
	}

	pub fn is_value_eq(&self, other: &Self) -> bool {
		self.holder.is_value_eq(&other.holder) && self.objective == other.objective
	}
}

impl Debug for Score {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{:?} {}", self.holder, self.objective)
	}
}

#[derive(Debug, Clone)]
pub enum DataLocation {
	Entity(String),
	Storage(String),
}

#[derive(Clone)]
pub enum Coordinates<T> {
	XYZ(AbsOrRelCoord<T>, AbsOrRelCoord<T>, AbsOrRelCoord<T>),
	Local(T, T, T),
}

impl<T: Num> Coordinates<T> {
	pub fn are_zero(&self) -> bool {
		matches!(self, Self::XYZ(x, y, z) if x.is_zero() && y.is_zero() && z.is_zero())
			|| matches!(self, Self::Local(a, b, c) if a.is_zero() && b.is_zero() && c.is_zero())
	}
}

impl<T: Debug + Num> Debug for Coordinates<T> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::XYZ(x, y, z) => {
				write!(f, "{x:?} {y:?} {z:?}")?;
			}
			Self::Local(a, b, c) => {
				write!(f, "^")?;
				if !a.is_zero() {
					write!(f, "{a:?}")?;
				}
				write!(f, " ^")?;
				if !b.is_zero() {
					write!(f, "{b:?}")?;
				}
				write!(f, " ^")?;
				if !c.is_zero() {
					write!(f, "{c:?}")?;
				}
			}
		}
		Ok(())
	}
}

impl<T: Debug + Num> Codegen for Coordinates<T> {
	fn gen_writer<F>(&self, f: &mut F, cbcx: &mut CodegenBlockCx) -> anyhow::Result<()>
	where
		F: std::fmt::Write,
	{
		let _ = cbcx;
		write!(f, "{self:?}")?;
		Ok(())
	}
}

pub type DoubleCoordinates = Coordinates<f64>;
pub type IntCoordinates = Coordinates<i64>;

#[derive(Clone)]
pub enum AbsOrRelCoord<T> {
	Abs(T),
	Rel(T),
}

impl<T: Num> AbsOrRelCoord<T> {
	/// Checks if this coordinate is relatively zero. Absolute zero will return false
	pub fn is_zero(&self) -> bool {
		matches!(self, Self::Rel(val) if val.is_zero())
	}
}

impl<T: Debug + Num> Debug for AbsOrRelCoord<T> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Abs(val) => write!(f, "{val:?}")?,
			Self::Rel(val) => {
				write!(f, "~")?;
				if !val.is_zero() {
					write!(f, "{val:?}")?;
				}
			}
		}

		Ok(())
	}
}

impl<T: Debug + Num> Codegen for AbsOrRelCoord<T> {
	fn gen_writer<F>(&self, f: &mut F, cbcx: &mut CodegenBlockCx) -> anyhow::Result<()>
	where
		F: std::fmt::Write,
	{
		let _ = cbcx;
		write!(f, "{self:?}")?;

		Ok(())
	}
}

#[derive(Debug, Clone)]
pub enum Axis {
	X,
	Y,
	Z,
}

impl Axis {
	pub fn codegen_str(&self) -> &'static str {
		match self {
			Self::X => "x",
			Self::Y => "y",
			Self::Z => "z",
		}
	}
}

#[derive(Debug, Clone)]
pub enum XPValue {
	Points,
	Levels,
}

impl Display for XPValue {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(
			f,
			"{}",
			match self {
				Self::Points => "points",
				Self::Levels => "levels",
			}
		)
	}
}

impl Codegen for XPValue {
	fn gen_writer<F>(&self, f: &mut F, cbcx: &mut CodegenBlockCx) -> anyhow::Result<()>
	where
		F: std::fmt::Write,
	{
		let _ = cbcx;
		write!(f, "{self}")?;
		Ok(())
	}
}
