use itertools::Itertools;
use rustc_hash::FxHashMap;

use crate::common::ResourceLocation;

use super::codegen::CodegenCx;
use super::datapack::{Function, Tag, TagInner};
use super::text::{format_lit_fake_player, LIT_OBJECTIVE, REG_OBJECTIVE};

pub fn gen_fns(
	ccx: &CodegenCx,
) -> anyhow::Result<(
	FxHashMap<ResourceLocation, Function>,
	FxHashMap<ResourceLocation, Tag>,
)> {
	let mut fns = FxHashMap::default();
	let mut tags = FxHashMap::default();

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
	if !ccx.score_literals.is_empty() {
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
