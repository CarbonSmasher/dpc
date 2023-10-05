use std::fmt::Display;

use num_traits::Num;

use crate::common::Identifier;
use crate::linker::codegen::{macros::cgformat, Codegen};

use super::target_selector::TargetSelector;

#[derive(Debug, Clone)]
pub enum EntityTarget {
	Player(String),
	Selector(TargetSelector),
}

#[derive(Debug, Clone)]
pub struct Score {
	pub holder: EntityTarget,
	pub objective: Identifier,
}

impl Score {
	pub fn new(holder: EntityTarget, objective: Identifier) -> Self {
		Self { holder, objective }
	}
}

#[derive(Debug, Clone)]
pub enum DataLocation {
	Entity(String),
	Storage(String),
}

#[derive(Debug, Clone)]
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

impl<T: Display + Num> Codegen for Coordinates<T> {
	fn gen_writer<F>(
		&self,
		f: &mut F,
		cbcx: &mut crate::linker::codegen::CodegenBlockCx,
	) -> anyhow::Result<()>
	where
		F: std::fmt::Write,
	{
		match self {
			Self::XYZ(x, y, z) => {
				cgformat!(cbcx, x, " ", y, " ", z)?;
			}
			Self::Local(a, b, c) => {
				write!(f, "^")?;
				if !a.is_zero() {
					write!(f, "{a}")?;
				}
				write!(f, " ^")?;
				if !b.is_zero() {
					write!(f, "{b}")?;
				}
				write!(f, " ^")?;
				if !c.is_zero() {
					write!(f, "{c}")?;
				}
			}
		}
		Ok(())
	}
}

pub type DoubleCoordinates = Coordinates<f64>;
pub type IntCoordinates = Coordinates<i64>;

#[derive(Debug, Clone)]
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

impl<T: Display + Num> Codegen for AbsOrRelCoord<T> {
	fn gen_writer<F>(
		&self,
		f: &mut F,
		cbcx: &mut crate::linker::codegen::CodegenBlockCx,
	) -> anyhow::Result<()>
	where
		F: std::fmt::Write,
	{
		let _ = cbcx;
		match self {
			Self::Abs(val) => write!(f, "{val}")?,
			Self::Rel(val) => {
				write!(f, "~")?;
				if !val.is_zero() {
					write!(f, "{val}")?;
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
