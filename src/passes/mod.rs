use std::collections::HashSet;
use std::sync::Arc;

use dashmap::{DashMap, DashSet};

use crate::common::block::BlockID;
use crate::common::ResourceLocation;
use crate::lir::LIRBlock;
use crate::{ir::IR, lir::LIR, mir::MIR};

use self::analysis::inline_candidates::InlineCandidatesPass;
use self::analysis::ir::ValidatePass;
use self::opt::constant::{fold::ConstFoldPass, prop::ConstPropPass, ConstComboPass};
use self::opt::dce::DCEPass;
use self::opt::dse::DSEPass;
use self::opt::inline::SimpleInlinePass;
use self::opt::inst_combine::InstCombinePass;
use self::opt::modifiers::merge::MergeModifiersPass;
use self::opt::modifiers::simplify::SimplifyModifiersPass;
use self::opt::scoreboard_dataflow::ScoreboardDataflowPass;
use self::opt::simplify::{lir::LIRSimplifyPass, mir::MIRSimplifyPass};

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
	fn run_pass(&mut self, data: &mut MIRPassData) -> anyhow::Result<()>;
}

pub struct MIRPassData<'mir> {
	pub mir: &'mir mut MIR,
	pub inline_candidates: HashSet<ResourceLocation>,
}

pub fn run_mir_passes(mir: &mut MIR) -> anyhow::Result<()> {
	let passes = [
		Box::new(NullPass) as Box<dyn MIRPass>,
		Box::new(DCEPass),
		Box::new(InlineCandidatesPass),
		Box::new(SimpleInlinePass),
		Box::new(DCEPass),
		Box::new(MIRSimplifyPass),
		Box::new(ConstComboPass),
		Box::new(MIRSimplifyPass),
		Box::new(DSEPass),
		Box::new(InstCombinePass),
		Box::new(ConstPropPass::new()),
		Box::new(MIRSimplifyPass),
		Box::new(ConstFoldPass::new()),
		Box::new(ConstComboPass),
		Box::new(InlineCandidatesPass),
		Box::new(SimpleInlinePass),
		Box::new(ConstComboPass),
		Box::new(DSEPass),
		Box::new(MIRSimplifyPass),
		Box::new(DCEPass),
	];

	let mut data = MIRPassData {
		mir,
		inline_candidates: HashSet::new(),
	};

	for mut pass in passes {
		println!("Running pass {}", pass.get_name());
		pass.run_pass(&mut data)?;
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
		Box::new(ScoreboardDataflowPass),
		Box::new(MergeModifiersPass),
		Box::new(SimplifyModifiersPass),
		Box::new(MergeModifiersPass),
		Box::new(LIRSimplifyPass),
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
	fn run_pass(&mut self, data: &mut MIRPassData) -> anyhow::Result<()> {
		let _ = data;
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

	fn made_changes(&self) -> bool {
		false
	}
}

pub trait BlockPass {
	fn get_name(&self) -> &'static str;
}

pub struct BlockPassCtx {
	pub removed_instrs: Arc<DashMap<BlockID, DashSet<usize>>>,
	pub block_operation_progress: Arc<DashMap<BlockID, usize>>,
}
