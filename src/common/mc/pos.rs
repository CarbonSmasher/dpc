use num_traits::Num;

use std::fmt::Debug;

use crate::output::codegen::{util::cg_float, Codegen, CodegenBlockCx};

#[derive(Clone, PartialEq, Eq)]
pub enum Coordinates<T> {
	XYZ(AbsOrRelCoord<T>, AbsOrRelCoord<T>, AbsOrRelCoord<T>),
	Local(T, T, T),
}

impl<T: Num> Coordinates<T> {
	/// Creates absolute coordinates at 0 0 0
	pub fn origin() -> Self {
		Self::absolute(T::zero(), T::zero(), T::zero())
	}

	/// Creates fully absolute coordinates
	pub fn absolute(x: T, y: T, z: T) -> Self {
		Self::XYZ(
			AbsOrRelCoord::Abs(x),
			AbsOrRelCoord::Abs(y),
			AbsOrRelCoord::Abs(z),
		)
	}

	/// Creates relative coordinates at ~ ~ ~
	pub fn here() -> Self {
		Self::relative(T::zero(), T::zero(), T::zero())
	}

	/// Creates fully relative coordinates
	pub fn relative(x: T, y: T, z: T) -> Self {
		Self::XYZ(
			AbsOrRelCoord::Rel(x),
			AbsOrRelCoord::Rel(y),
			AbsOrRelCoord::Rel(z),
		)
	}
}

impl<T: Num> Coordinates<T> {
	pub fn are_zero(&self) -> bool {
		matches!(self, Self::XYZ(x, y, z) if x.is_relative_zero() && y.is_relative_zero() && z.is_relative_zero())
			|| matches!(self, Self::Local(a, b, c) if a.is_zero() && b.is_zero() && c.is_zero())
	}
}

impl<T: Num + PartialEq + Eq> Coordinates<T> {
	pub fn is_value_eq(&self, other: &Self) -> bool {
		matches!((self, other), (Self::XYZ(x1, y1, z1), Self::XYZ(x2, y2, z2)) if x1.is_value_eq(x2) && y1.is_value_eq(y2) && z1.is_value_eq(z2))
			|| matches!((self, other), (Self::Local(x1, y1, z1), Self::Local(x2, y2, z2)) if x1 == x2 && y1 == y2 && z1 == z2)
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

pub type DoubleCoordinates = Coordinates<f64>;
pub type IntCoordinates = Coordinates<i64>;

#[derive(Clone, PartialEq)]
pub struct Coordinates2D<T>(pub AbsOrRelCoord<T>, pub AbsOrRelCoord<T>);

impl<T> Coordinates2D<T> {
	pub fn new(x: AbsOrRelCoord<T>, y: AbsOrRelCoord<T>) -> Self {
		Self(x, y)
	}
}

impl<T: Num> Coordinates2D<T> {
	pub fn are_zero(&self) -> bool {
		self.0.is_relative_zero() && self.1.is_relative_zero()
	}
}

impl<T: Num + PartialEq + Eq> Coordinates2D<T> {
	pub fn is_value_eq(&self, other: &Self) -> bool {
		self.0.is_value_eq(&other.0) && self.1.is_value_eq(&other.1)
	}
}

impl<T: Debug + Num> Debug for Coordinates2D<T> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{:?} {:?}", self.0, self.1)
	}
}

pub type DoubleCoordinates2D = Coordinates2D<f64>;
pub type IntCoordinates2D = Coordinates2D<i64>;

#[derive(Clone, PartialEq, Eq)]
pub enum AbsOrRelCoord<T> {
	Abs(T),
	Rel(T),
}

impl<T> AbsOrRelCoord<T> {
	pub fn is_rel(&self) -> bool {
		matches!(self, Self::Rel(..))
	}
}

impl<T: Num> AbsOrRelCoord<T> {
	/// Checks if this coordinate is relatively zero. Absolute zero will return false
	pub fn is_relative_zero(&self) -> bool {
		matches!(self, Self::Rel(val) if val.is_zero())
	}
}

impl<T: Num + PartialEq + Eq> AbsOrRelCoord<T> {
	pub fn is_value_eq(&self, other: &Self) -> bool {
		matches!((self, other), (Self::Abs(l), Self::Abs(r)) if l == r)
			|| matches!((self, other), (Self::Rel(l), Self::Rel(r)) if l == r)
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

#[derive(Debug, Clone, PartialEq)]
pub struct Angle {
	pub relative: bool,
	pub value: f32,
}

impl Angle {
	pub fn new_absolute(value: f32) -> Self {
		Self::new(false, value)
	}

	pub fn new_relative(value: f32) -> Self {
		Self::new(true, value)
	}

	pub fn new(relative: bool, value: f32) -> Self {
		Self { relative, value }
	}

	pub fn is_relative_zero(&self) -> bool {
		self.relative && self.value == 0.0
	}

	pub fn is_absolute_zero(&self) -> bool {
		!self.relative && self.value == 0.0
	}
}

impl Codegen for Angle {
	fn gen_writer<F>(&self, f: &mut F, cbcx: &mut CodegenBlockCx) -> anyhow::Result<()>
	where
		F: std::fmt::Write,
	{
		let _ = cbcx;

		if self.relative {
			write!(f, "~")?;
			cg_float(f, self.value as f64, true, true, true);
		} else {
			cg_float(f, self.value as f64, false, true, true);
		}

		Ok(())
	}
}
