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
