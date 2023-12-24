use std::collections::HashMap;

use itertools::Itertools;

use crate::common::ResourceLocation;

use super::codegen::CodegenCx;
use super::datapack::{Function, Tag, TagInner};
use super::text::{format_lit_fake_player, LIT_OBJECTIVE, REG_OBJECTIVE, REG_STORAGE_LOCATION};

pub fn gen_fns(
	ccx: &CodegenCx,
) -> anyhow::Result<(
	HashMap<ResourceLocation, Function>,
	HashMap<ResourceLocation, Tag>,
)> {
	let mut fns = HashMap::new();
	let mut tags = HashMap::new();

	let init_fn = gen_init(ccx);
	if let Some(init_fn) = init_fn {
		let loc = if ccx.project.name == "dpc" {
			"dpc:init".to_string()
		} else {
			ccx.project.name.clone() + ":dpc_init"
		};
		fns.insert(ResourceLocation::from(loc.clone()), init_fn);

		let init_tag = Tag {
			inner: TagInner { values: vec![loc] },
		};
		tags.insert(ResourceLocation::from("minecraft:load"), init_tag);
	}

	Ok((fns, tags))
}

fn gen_init(ccx: &CodegenCx) -> Option<Function> {
	let mut out = Function::new();
	let mut function_needed = false;

	if ccx.racx.has_allocated_reg() {
		let cmd = format!("scoreboard objectives add {REG_OBJECTIVE} dummy");
		out.contents.push(cmd);
		function_needed = true;
	}
	if ccx.racx.has_allocated_local() {
		let cmd = format!("data merge storage {REG_STORAGE_LOCATION} {{}}");
		out.contents.push(cmd);
		function_needed = true;
	}
	if ccx.score_literals.len() > 0 {
		let cmd = format!("scoreboard objectives add {LIT_OBJECTIVE} dummy");
		out.contents.push(cmd);
		function_needed = true;
	}

	for lit in ccx.score_literals.iter().sorted() {
		let cmd = format!(
			"scoreboard players set {} {} {}",
			format_lit_fake_player(*lit),
			LIT_OBJECTIVE,
			*lit,
		);
		out.contents.push(cmd);
		function_needed = true;
	}

	if function_needed {
		Some(out)
	} else {
		None
	}
}
