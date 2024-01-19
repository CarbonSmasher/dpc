use crate::output::strip::StripMode;

/// Settings for a generated project
#[derive(Clone)]
pub struct ProjectSettings {
	pub(crate) name: String,
	pub(crate) strip_mode: StripMode,
	pub(crate) op_level: OptimizationLevel,
}

impl ProjectSettings {
	pub fn new(name: String) -> Self {
		Self {
			name,
			strip_mode: StripMode::None,
			op_level: OptimizationLevel::Basic,
		}
	}
}

pub struct ProjectSettingsBuilder {
	settings: ProjectSettings,
}

impl ProjectSettingsBuilder {
	pub fn new(name: &str) -> Self {
		Self {
			settings: ProjectSettings::new(name.to_string()),
		}
	}

	pub fn build(self) -> ProjectSettings {
		self.settings
	}

	pub fn strip_mode(mut self, mode: StripMode) -> Self {
		self.settings.strip_mode = mode;
		self
	}

	pub fn op_level(mut self, level: OptimizationLevel) -> Self {
		self.settings.op_level = level;
		self
	}
}

/// Different optimization levels that can be used
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum OptimizationLevel {
	/// No optimizations will occur. The only transformations that will
	/// happen to the IR will be necessary ones to make it correct.
	/// This is probably too slow even for debugging
	None,
	/// Some basic optimizations will happen to make the code
	/// run at a reasonable speed
	Basic,
	/// More intensive optimizations that make the code run even
	/// faster will be run. This will transform the code in ways that
	/// you might not expect, so it is not recommended for debugging
	More,
	/// All optimizations will be run. This can increase compile time
	/// by a lot, so it should only be used for final releases. Symbols
	/// will also be stripped from the generated pack.
	Full,
}
