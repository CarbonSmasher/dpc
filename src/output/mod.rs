pub mod codegen;
pub mod datapack;
mod gen_fns;
pub mod ra;
pub mod strip;
pub mod text;

use crate::lir::{LIRFunction, LIR};
use crate::lower::cleanup_fn_id;
use crate::project::ProjectSettings;

use anyhow::Context;

use self::codegen::{codegen_block, CodegenCx};
use self::datapack::{Datapack, Function};

pub fn link(lir: LIR, project: &ProjectSettings) -> anyhow::Result<Datapack> {
	let mut out = Datapack::new();

	// Strip the LIR
	let mapping = self::strip::strip(&lir, project);

	let mut ccx = CodegenCx::new(project, mapping);
	for (func_id, func) in lir.functions {
		let mut func_id = func_id.clone();
		if let Some(mapping) = &ccx.func_mapping {
			if let Some(new_id) = mapping.0.get(&func_id) {
				func_id = new_id.clone();
			}
		}
		let fun = codegen_fn(&func_id, &func, &mut ccx)
			.with_context(|| format!("In function {:?}", func.interface))?;

		out.functions.insert(func_id, fun);
	}

	let (extra_fns, extra_tags) = gen_fns::gen_fns(&ccx)?;
	out.functions.extend(extra_fns);
	out.function_tags.extend(extra_tags);

	Ok(out)
}

fn codegen_fn(func_id: &str, func: &LIRFunction, ccx: &mut CodegenCx) -> anyhow::Result<Function> {
	let mut fun = Function::new();
	let cleaned_id = cleanup_fn_id(func_id);
	let code = codegen_block(&cleaned_id, &func.block, ccx)?;
	fun.contents = code;
	Ok(fun)
}
