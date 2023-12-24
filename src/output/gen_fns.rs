use std::collections::HashMap;

use itertools::Itertools;

use crate::common::ResourceLocation;

use super::codegen::CodegenCx;
use super::datapack::Function;
use super::text::{format_lit_fake_player, LIT_OBJECTIVE, REG_OBJECTIVE, REG_STORAGE_LOCATION};

pub fn gen_fns(ccx: &CodegenCx) -> anyhow::Result<HashMap<ResourceLocation, Function>> {
	let mut out = HashMap::new();

	let init_fn = gen_init(ccx);
	let loc = if ccx.project.name == "dpc" {
		"dpc:init".to_string()
	} else {
		ccx.project.name.clone() + ":dpc_init"
	};
	out.insert(ResourceLocation::from(loc), init_fn);

	Ok(out)
}

fn gen_init(ccx: &CodegenCx) -> Function {
	let mut out = Function::new();

	if ccx.racx.has_allocated_reg() {
		let cmd = format!("scoreboard objectives add {REG_OBJECTIVE} dummy");
		out.contents.push(cmd);
	}
	if ccx.racx.has_allocated_local() {
		let cmd = format!("data merge storage {REG_STORAGE_LOCATION} {{}}");
		out.contents.push(cmd);
	}
	if ccx.score_literals.len() > 0 {
		let cmd = format!("scoreboard objectives add {LIT_OBJECTIVE} dummy");
		out.contents.push(cmd);
	}

	for lit in ccx.score_literals.iter().sorted() {
		let cmd = format!(
			"scoreboard players set {} {} {}",
			format_lit_fake_player(*lit),
			LIT_OBJECTIVE,
			*lit,
		);
		out.contents.push(cmd);
	}

	out
}
