use std::collections::HashMap;

use crate::common::ResourceLocation;

use super::{
	codegen::CodegenCx,
	datapack::Function,
	text::{format_lit_fake_player, LIT_OBJECTIVE, REG_OBJECTIVE, REG_STORAGE_LOCATION},
};

pub fn gen_fns(ccx: &CodegenCx) -> anyhow::Result<HashMap<ResourceLocation, Function>> {
	let mut out = HashMap::new();

	let init_fn = gen_init(ccx);
	out.insert(ResourceLocation::from("dpc::init"), init_fn);

	Ok(out)
}

fn gen_init(ccx: &CodegenCx) -> Function {
	let mut out = Function::new();

	if ccx.racx.get_reg_count() > 0 {
		let cmd = format!("scoreboard objectives add {REG_OBJECTIVE} dummy");
		out.contents.push(cmd);
	}
	if ccx.racx.get_local_count() > 0 {
		let cmd = format!("data merge storage {REG_STORAGE_LOCATION} {{}}");
		out.contents.push(cmd);
	}
	if ccx.score_literals.len() > 0 {
		let cmd = format!("scoreboard objectives add {LIT_OBJECTIVE} dummy");
		out.contents.push(cmd);
	}

	for lit in &ccx.score_literals {
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
