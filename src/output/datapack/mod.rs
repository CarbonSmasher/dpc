use std::{collections::HashMap, path::Path};

use crate::common::ResourceLocation;

use self::files::output_pack;

mod files;

#[derive(Debug, Clone)]
pub struct Datapack {
	pub functions: HashMap<ResourceLocation, Function>,
}

impl Datapack {
	pub fn new() -> Self {
		Self {
			functions: HashMap::new(),
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
