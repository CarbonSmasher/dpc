use std::path::Path;

use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};

use crate::common::ResourceLocation;

use self::files::output_pack;

mod files;

#[derive(Debug, Clone)]
pub struct Datapack {
	pub functions: FxHashMap<ResourceLocation, Function>,
	pub function_tags: FxHashMap<ResourceLocation, Tag>,
}

impl Datapack {
	pub fn new() -> Self {
		Self {
			functions: FxHashMap::default(),
			function_tags: FxHashMap::default(),
		}
	}

	pub fn output(self, path: &Path) -> anyhow::Result<()> {
		output_pack(self, path)
	}
}

impl Default for Datapack {
	fn default() -> Self {
		Self::new()
	}
}

#[derive(Debug, Clone)]
pub struct Function {
	pub contents: Vec<String>,
}

impl Function {
	pub fn new() -> Self {
		Self {
			contents: Vec::new(),
		}
	}
}

impl Default for Function {
	fn default() -> Self {
		Self::new()
	}
}

#[derive(Debug, Clone)]
pub struct Tag {
	pub inner: TagInner,
}

impl Tag {
	pub fn new() -> Self {
		Self {
			inner: TagInner { values: Vec::new() },
		}
	}
}

impl Default for Tag {
	fn default() -> Self {
		Self::new()
	}
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagInner {
	pub values: Vec<String>,
}
