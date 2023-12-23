use std::fmt::Debug;

use crate::output::codegen::{util::cg_float, Codegen};

#[derive(Clone, PartialEq)]
pub struct Time {
	pub amount: f32,
	pub unit: TimeUnit,
}

impl Time {
	pub fn new(amount: f32, unit: TimeUnit) -> Self {
		Self { amount, unit }
	}

	pub fn new_ticks(amount: f32) -> Self {
		Self::new(amount, TimeUnit::Ticks)
	}

	pub fn instant() -> Self {
		Self::new_ticks(0.0)
	}

	pub fn convert(mut self, unit: TimeUnit) -> Self {
		let to_ticks = self.unit.get_tick_conversion_factor();
		self.amount *= to_ticks;
		let from_ticks = unit.get_tick_conversion_factor();
		self.amount /= from_ticks;
		self.unit = unit;
		self
	}

	fn codegen_str(self) -> anyhow::Result<String> {
		let mut amount = String::new();
		cg_float(&mut amount, self.amount.into(), false, true, true)?;

		let suffix = match self.unit {
			TimeUnit::Days => "d",
			TimeUnit::Seconds => "s",
			// Ticks are always default and can be omitted
			TimeUnit::Ticks => "",
		};
		Ok(format!("{}{suffix}", amount))
	}
}

impl Debug for Time {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.amount)?;
		self.unit.fmt(f)
	}
}

impl Codegen for Time {
	fn gen_writer<F>(
		&self,
		f: &mut F,
		cbcx: &mut crate::output::codegen::CodegenBlockCx,
	) -> anyhow::Result<()>
	where
		F: std::fmt::Write,
	{
		let _ = cbcx;
		// Try different formats and pick the shortest one
		let ticks = self.clone().convert(TimeUnit::Ticks).codegen_str()?;
		let seconds = self.clone().convert(TimeUnit::Seconds).codegen_str()?;
		let days = self.clone().convert(TimeUnit::Days).codegen_str()?;

		let shortest = [ticks, seconds, days]
			.iter()
			.min_by_key(|x| x.len())
			.expect("Iterator is not empty")
			.clone();
		write!(f, "{shortest}")?;

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

impl TimeUnit {
	pub fn get_tick_conversion_factor(&self) -> f32 {
		match self {
			Self::Ticks => 1.0,
			Self::Seconds => 20.0,
			Self::Days => 24000.0,
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
		cbcx: &mut crate::output::codegen::CodegenBlockCx,
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
		cbcx: &mut crate::output::codegen::CodegenBlockCx,
	) -> anyhow::Result<()>
	where
		F: std::fmt::Write,
	{
		let _ = cbcx;
		write!(f, "{self:?}")?;
		Ok(())
	}
}
