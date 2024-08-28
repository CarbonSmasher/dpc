pub mod usage_analysis;

use std::ops::Not;

/// Utility struct used to track changes made by a pass to determine whether it should be run again
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct RunAgain(bool);

impl RunAgain {
	pub fn new() -> Self {
		Self(false)
	}

	/// Merge another RunAgain with this one
	pub fn merge(&mut self, other: Self) {
		self.merge_bool(other.0)
	}

	/// Merge a bool with this RunAgain
	pub fn merge_bool(&mut self, other: bool) {
		if other {
			self.0 = true;
		}
	}

	/// Make the result yes
	pub fn yes(&mut self) {
		self.merge_bool(true)
	}

	/// Get the result
	pub fn result(&self) -> bool {
		self.0
	}
}

impl Not for RunAgain {
	type Output = bool;

	fn not(self) -> Self::Output {
		!self.0
	}
}

/// An enum for different analysis results
pub enum AnalysisResult<T> {
	/// Unknown result. Should invalidate everything
	Unknown,
	/// A known result
	Known(Vec<T>),
}

impl<T> AnalysisResult<T> {
	/// Creates a Known with no values
	pub fn known_empty() -> Self {
		Self::Known(Vec::new())
	}

	/// Creates a Known with one value
	pub fn known(val: T) -> Self {
		Self::Known(vec![val])
	}

	/// Merges this result with another one
	pub fn merge(&mut self, other: Self) {
		match (self, other) {
			(this @ Self::Unknown, _) | (this, Self::Unknown) => {
				*this = Self::Unknown;
			}
			(Self::Known(this), Self::Known(other)) => this.extend(other),
		}
	}
}

impl<T> From<Option<T>> for AnalysisResult<T> {
	fn from(value: Option<T>) -> Self {
		match value {
			Some(val) => Self::known(val),
			None => Self::known_empty(),
		}
	}
}
