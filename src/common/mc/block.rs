use std::fmt::Debug;

#[derive(Clone)]
pub enum SetblockMode {
	Destroy,
	Keep,
	Replace,
}

impl Debug for SetblockMode {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Destroy => write!(f, "destroy"),
			Self::Keep => write!(f, "keep"),
			Self::Replace => write!(f, "replace"),
		}
	}
}
