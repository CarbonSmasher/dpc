pub mod codegen;
pub mod datapack;
mod gen_fns;
pub mod ra;
pub mod text;

use crate::lir::LIR;

use self::codegen::{codegen_block, CodegenCx};
use self::datapack::{Datapack, Function};

pub fn link(lir: LIR) -> anyhow::Result<Datapack> {
	let mut out = Datapack::new();

	let mut ccx = CodegenCx::new();
	for (interface, block) in lir.functions {
		let mut fun = Function::new();
		let code = codegen_block(&block, &mut ccx)?;
		fun.contents = code;
		out.functions.insert(interface.id, fun);
	}

	let extra_fns = gen_fns::gen_fns(&ccx)?;
	out.functions.extend(extra_fns);

	Ok(out)
}
