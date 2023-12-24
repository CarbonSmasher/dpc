/// Settings for a generated project
pub struct ProjectSettings {
	pub(crate) name: String,
}

impl ProjectSettings {
	pub fn new(name: String) -> Self {
		Self { name }
	}
}
