pub mod codegen;
pub mod datapack;
pub mod ra;

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

	Ok(out)
}
