use std::collections::HashMap;

use crate::common::ResourceLocation;

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
