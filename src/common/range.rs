use std::fmt::Display;

use num_traits::{Bounded, NumOps, One, PrimInt, Signed};

use crate::output::codegen::Codegen;

use super::ty::{Float, Int};

// I love traits
pub trait Rangeable:
	Sized + NumOps + Bounded + PartialEq + PartialOrd + Signed + Display + Copy
{
}
impl<S> Rangeable for S where
	S: Sized + NumOps + Bounded + PartialEq + PartialOrd + Signed + Display + Copy
{
}

pub type IntRange = Range<Int>;
pub type FloatRange = Range<Float>;

/// An inclusive integer range with the ability to have
/// an infinite value on both sides
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Range<N: Rangeable> {
	left: RangeEnd<N>,
	right: RangeEnd<N>,
}

impl<N: Rangeable> Range<N> {
	/// Canonicalize this range so that the smaller number is on the left
	#[must_use]
	pub fn canonicalize(self) -> Self {
		if self.left.larger(&self.right) {
			self
		} else {
			self.flip()
		}
	}

	/// Prepare this range for codegen by changing a double infinite into a single infinite
	#[must_use]
	pub fn make_single_infinite(self) -> Self {
		if self.is_infinite() {
			Self {
				left: RangeEnd::Infinite,
				right: RangeEnd::infinite_val(),
			}
		} else {
			self
		}
	}

	/// Flip this range
	#[must_use]
	pub fn flip(self) -> Self {
		Self {
			left: self.right,
			right: self.left,
		}
	}

	/// Tries to get a single number from this range if it is exactly one long
	pub fn get_single(&self) -> Option<N> {
		if self.left == self.right {
			Some(self.left.get_val(RangeSide::Left))
		} else {
			None
		}
	}

	/// Checks if this range is fully infinite
	pub fn is_infinite(&self) -> bool {
		self.left.is_infinite() && self.right.is_infinite()
	}
}

impl<N: Rangeable + PrimInt + One> Range<N> {
	/// Get the amount of integers in this range
	pub fn len(&self) -> N {
		// Add one since its inclusive on both sides
		(self.right.get_val(RangeSide::Right) - self.left.get_val(RangeSide::Left) + N::one()).abs()
	}
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RangeEnd<N: Rangeable> {
	Value(N),
	Infinite,
}

impl<N: Rangeable> RangeEnd<N> {
	/// Creates a new RangeEnd with the infinite value
	pub fn infinite_val() -> Self {
		Self::Value(N::max_value())
	}

	/// Checks if larger than the other end. This end must be on the right
	pub fn larger(&self, other: &Self) -> bool {
		match (self, other) {
			(RangeEnd::Value(..) | RangeEnd::Infinite, RangeEnd::Infinite) => true,
			(RangeEnd::Value(r), RangeEnd::Value(l)) => r > l,
			(RangeEnd::Infinite, ..) => true,
		}
	}

	/// Gets the value of this end
	pub fn get_val(&self, side: RangeSide) -> N {
		match self {
			Self::Value(val) => *val,
			Self::Infinite => N::max_value() * side.sign(),
		}
	}

	/// Checks whether this end is infinite or effectively infinite
	pub fn is_infinite(&self) -> bool {
		match self {
			Self::Infinite => true,
			Self::Value(val) => val.abs() == N::max_value(),
		}
	}
}

/// Utility to pass to RangeEnd methods so that they know which side they are on
#[derive(Copy, Clone)]
pub enum RangeSide {
	Left,
	Right,
}

impl RangeSide {
	/// Gets the sign of this side for infinite
	pub fn sign<N: Rangeable + One>(&self) -> N {
		match self {
			Self::Left => N::one().neg(),
			Self::Right => N::one(),
		}
	}
}

impl<N: Rangeable> Codegen for RangeEnd<N> {
	fn gen_writer<F>(
		&self,
		f: &mut F,
		cbcx: &mut crate::output::codegen::CodegenBlockCx,
	) -> anyhow::Result<()>
	where
		F: std::fmt::Write,
	{
		let _ = cbcx;
		match self {
			Self::Value(val) => write!(f, "{val}")?,
			Self::Infinite => {}
		}

		Ok(())
	}
}

impl<N: Rangeable> Codegen for Range<N> {
	fn gen_writer<F>(
		&self,
		f: &mut F,
		cbcx: &mut crate::output::codegen::CodegenBlockCx,
	) -> anyhow::Result<()>
	where
		F: std::fmt::Write,
	{
		let range = self.clone().canonicalize().make_single_infinite();
		if let Some(single) = range.get_single() {
			write!(f, "{single}")?;
		} else {
			self.left.gen_writer(f, cbcx)?;
			write!(f, "..")?;
			self.right.gen_writer(f, cbcx)?;
		}

		Ok(())
	}
}
