pub mod codegen;
pub mod datapack;
mod gen_fns;
pub mod ra;
pub mod text;

use crate::common::block::BlockAllocator;
use crate::common::function::FunctionInterface;
use crate::lir::{LIRBlock, LIR};

use anyhow::{anyhow, Context};

use self::codegen::{codegen_block, CodegenCx};
use self::datapack::{Datapack, Function};

pub fn link(lir: LIR) -> anyhow::Result<Datapack> {
	let mut out = Datapack::new();

	let mut ccx = CodegenCx::new();
	for (interface, block) in lir.functions {
		let fun = codegen_fn(&interface, &lir.blocks, &mut ccx, &block)
			.with_context(|| format!("In function {:?}", interface))?;

		out.functions.insert(interface.id.clone(), fun);
	}

	let extra_fns = gen_fns::gen_fns(&ccx)?;
	out.functions.extend(extra_fns);

	Ok(out)
}

fn codegen_fn(
	interface: &FunctionInterface,
	blocks: &BlockAllocator<LIRBlock>,
	ccx: &mut CodegenCx,
	block: &usize,
) -> anyhow::Result<Function> {
	let block = blocks.get(&block).ok_or(anyhow!("Block does not exist"))?;
	let mut fun = Function::new();
	let code = codegen_block(&interface.id, &block, ccx)?;
	fun.contents = code;
	Ok(fun)
}
