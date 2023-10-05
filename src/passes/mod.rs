use std::sync::Arc;

use dashmap::{DashMap, DashSet};

use crate::common::block::BlockID;
use crate::lir::LIRBlock;
use crate::{ir::IR, lir::LIR, mir::MIR};

use self::analysis::ir::ValidatePass;
use self::opt::lir::SimplifyModifiersPass;
use self::opt::mir::{ConstPropPass, InstCombinePass, MIRSimplifyPass};
use self::opt::{lir::LIRSimplifyPass, mir::DSEPass};

pub mod analysis;
pub mod opt;

pub trait IRPass: Pass {
	fn run_pass(&mut self, ir: &mut IR) -> anyhow::Result<()>;
}

pub fn run_ir_passes(ir: &mut IR) -> anyhow::Result<()> {
	let passes = [
		Box::new(NullPass) as Box<dyn IRPass>,
		Box::new(ValidatePass),
	];

	for mut pass in passes {
		println!("Running pass {}", pass.get_name());
		pass.run_pass(ir)?;
	}

	Ok(())
}

pub trait MIRPass: Pass {
	fn run_pass(&mut self, mir: &mut MIR) -> anyhow::Result<()>;
}

pub fn run_mir_passes(mir: &mut MIR) -> anyhow::Result<()> {
	let passes = [
		Box::new(NullPass) as Box<dyn MIRPass>,
		Box::new(MIRSimplifyPass),
		Box::new(ConstPropPass),
		Box::new(MIRSimplifyPass),
		Box::new(DSEPass),
		Box::new(InstCombinePass),
	];

	for mut pass in passes {
		println!("Running pass {}", pass.get_name());
		pass.run_pass(mir)?;
	}

	Ok(())
}

pub trait LIRPass: Pass {
	fn run_pass(&mut self, lir: &mut LIR) -> anyhow::Result<()>;
}

pub fn run_lir_passes(lir: &mut LIR) -> anyhow::Result<()> {
	let passes = [
		Box::new(NullPass) as Box<dyn LIRPass>,
		Box::new(LIRSimplifyPass),
		Box::new(SimplifyModifiersPass),
	];

	for mut pass in passes {
		println!("Running pass {}", pass.get_name());
		pass.run_pass(lir)?;
	}

	Ok(())
}

pub trait LIRBlockPass: BlockPass {
	fn run_pass(
		&mut self,
		block: &mut LIRBlock,
		bpcx: &mut BlockPassCtx,
		pass_id: usize,
	) -> anyhow::Result<()>;
}

struct LIRBlockPassRunner;

impl Pass for LIRBlockPassRunner {
	fn get_name(&self) -> &'static str {
		"lir_block_passes"
	}
}

impl LIRPass for LIRBlockPassRunner {
	fn run_pass(&mut self, lir: &mut LIR) -> anyhow::Result<()> {
		let _ = lir;
		Ok(())
	}
}

struct NullPass;

impl IRPass for NullPass {
	fn run_pass(&mut self, ir: &mut IR) -> anyhow::Result<()> {
		let _ = ir;
		Ok(())
	}
}

impl MIRPass for NullPass {
	fn run_pass(&mut self, mir: &mut MIR) -> anyhow::Result<()> {
		let _ = mir;
		Ok(())
	}
}

impl LIRPass for NullPass {
	fn run_pass(&mut self, lir: &mut LIR) -> anyhow::Result<()> {
		let _ = lir;
		Ok(())
	}
}

impl LIRBlockPass for NullPass {
	fn run_pass(
		&mut self,
		block: &mut LIRBlock,
		bpcx: &mut BlockPassCtx,
		pass_id: usize,
	) -> anyhow::Result<()> {
		let _ = block;
		let _ = bpcx;
		let _ = pass_id;
		Ok(())
	}
}

impl Pass for NullPass {
	fn get_name(&self) -> &'static str {
		"null"
	}
}

impl BlockPass for NullPass {
	fn get_name(&self) -> &'static str {
		"null"
	}
}

pub trait Pass {
	fn get_name(&self) -> &'static str;
}

pub trait BlockPass {
	fn get_name(&self) -> &'static str;
}

pub struct BlockPassCtx {
	pub removed_instrs: Arc<DashMap<BlockID, DashSet<usize>>>,
	pub block_operation_progress: Arc<DashMap<BlockID, usize>>,
}
