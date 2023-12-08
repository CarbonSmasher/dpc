use std::fmt::Debug;

use crate::{linker::codegen::Codegen, util::EqFloat};

#[derive(Clone, PartialEq, Eq)]
pub struct Time {
	pub amount: EqFloat,
	pub unit: TimeUnit,
}

impl Time {
	pub fn new(amount: f32, unit: TimeUnit) -> Self {
		Self {
			amount: EqFloat(amount),
			unit,
		}
	}

	pub fn new_ticks(amount: f32) -> Self {
		Self::new(amount, TimeUnit::Ticks)
	}

	pub fn instant() -> Self {
		Self::new_ticks(0.0)
	}
}

impl Debug for Time {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.amount.0)?;
		self.unit.fmt(f)
	}
}

impl Codegen for Time {
	fn gen_writer<F>(
		&self,
		f: &mut F,
		cbcx: &mut crate::linker::codegen::CodegenBlockCx,
	) -> anyhow::Result<()>
	where
		F: std::fmt::Write,
	{
		let _ = cbcx;
		write!(f, "{}", self.amount.0)?;
		match self.unit {
			TimeUnit::Days => write!(f, "d")?,
			TimeUnit::Seconds => write!(f, "s")?,
			// Ticks are always default and can be omitted
			TimeUnit::Ticks => {}
		}
		Ok(())
	}
}

#[derive(Clone, PartialEq, Eq)]
pub enum TimeUnit {
	Days,
	Seconds,
	Ticks,
}

impl Debug for TimeUnit {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Days => write!(f, "d"),
			Self::Seconds => write!(f, "s"),
			Self::Ticks => write!(f, "t"),
		}
	}
}

#[derive(Clone, PartialEq, Eq)]
pub enum TimePreset {
	Day,
	Night,
	Noon,
	Midnight,
}

impl Debug for TimePreset {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Day => write!(f, "day"),
			Self::Night => write!(f, "night"),
			Self::Noon => write!(f, "noon"),
			Self::Midnight => write!(f, "midnight"),
		}
	}
}

impl Codegen for TimePreset {
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
			// Using the number is shorter than the full name
			Self::Midnight => write!(f, "18000")?,
			other => write!(f, "{other:?}")?,
		}
		Ok(())
	}
}

#[derive(Clone, PartialEq, Eq)]
pub enum TimeQuery {
	Daytime,
	Gametime,
	Day,
}

impl Debug for TimeQuery {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Daytime => write!(f, "daytime"),
			Self::Gametime => write!(f, "gametime"),
			Self::Day => write!(f, "day"),
		}
	}
}

impl Codegen for TimeQuery {
	fn gen_writer<F>(
		&self,
		f: &mut F,
		cbcx: &mut crate::linker::codegen::CodegenBlockCx,
	) -> anyhow::Result<()>
	where
		F: std::fmt::Write,
	{
		let _ = cbcx;
		write!(f, "{self:?}")?;
		Ok(())
	}
}
