use std::{collections::HashMap, path::Path};

use serde::{Deserialize, Serialize};

use crate::common::ResourceLocation;

use self::files::output_pack;

mod files;

#[derive(Debug, Clone)]
pub struct Datapack {
	pub functions: HashMap<ResourceLocation, Function>,
	pub function_tags: HashMap<ResourceLocation, Tag>,
}

impl Datapack {
	pub fn new() -> Self {
		Self {
			functions: HashMap::new(),
			function_tags: HashMap::new(),
		}
	}

	pub fn output(self, path: &Path) -> anyhow::Result<()> {
		output_pack(self, path)
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagInner {
	pub values: Vec<String>,
}
