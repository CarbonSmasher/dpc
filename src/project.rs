use crate::output::strip::StripMode;

/// Settings for a generated project
#[derive(Clone)]
pub struct ProjectSettings {
	pub(crate) name: String,
	pub(crate) strip_mode: StripMode,
}

impl ProjectSettings {
	pub fn new(name: String) -> Self {
		Self {
			name,
			strip_mode: StripMode::None,
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
}
