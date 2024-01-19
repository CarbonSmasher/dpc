use rustc_hash::FxHashSet;

use crate::common::ResourceLocation;
use crate::{ir::IR, lir::LIR, mir::MIR};

use self::analysis::inline_candidates::InlineCandidatesPass;
use self::analysis::ir::ValidatePass;
use self::opt::constant::{fold::ConstFoldPass, prop::ConstPropPass, ConstComboPass};
use self::opt::dataflow::get::DataflowGetPass;
use self::opt::dataflow::result::DataflowResultPass;
use self::opt::dce::DCEPass;
use self::opt::dse::DSEPass;
use self::opt::func::cleanup_return::CleanupReturnPass;
use self::opt::func::inline::SimpleInlinePass;
use self::opt::func::unused_args::UnusedArgsPass;
use self::opt::modifiers::merge::MergeModifiersPass;
use self::opt::modifiers::null::NullModifiersPass;
use self::opt::modifiers::simplify::SimplifyModifiersPass;
use self::opt::multifold::assign::MultifoldAssignPass;
use self::opt::multifold::combine::MultifoldCombinePass;
use self::opt::multifold::logic::MultifoldLogicPass;
use self::opt::order::conditions::ReorderConditionsPass;
use self::opt::simplify::cleanup::CleanupPass;
use self::opt::simplify::{lir::LIRSimplifyPass, mir::MIRSimplifyPass};
use self::opt::ty::TypeBasedOptimizationPass;

pub mod analysis;
pub mod opt;
pub mod util;

pub trait IRPass: Pass {
	fn run_pass(&mut self, ir: &mut IR) -> anyhow::Result<()>;
}

pub fn run_ir_passes(ir: &mut IR, debug: bool) -> anyhow::Result<()> {
	let passes = [
		Box::new(NullPass) as Box<dyn IRPass>,
		Box::new(ValidatePass),
	];

	for mut pass in passes {
		if debug {
			println!("Running pass {}", pass.get_name());
		}
		pass.run_pass(ir)?;
	}

	Ok(())
}

pub trait MIRPass: Pass {
	fn run_pass(&mut self, data: &mut MIRPassData) -> anyhow::Result<()>;
}

pub struct MIRPassData<'mir> {
	pub mir: &'mir mut MIR,
	pub inline_candidates: FxHashSet<ResourceLocation>,
}

pub fn run_mir_passes(mir: &mut MIR, debug: bool) -> anyhow::Result<()> {
	let passes = [
		Box::new(NullPass) as Box<dyn MIRPass>,
		Box::new(CleanupPass),
		Box::new(CleanupReturnPass),
		Box::new(DCEPass),
		Box::new(InlineCandidatesPass),
		Box::new(SimpleInlinePass),
		Box::new(DCEPass),
		Box::new(TypeBasedOptimizationPass),
		Box::new(UnusedArgsPass),
		Box::new(MIRSimplifyPass),
		Box::new(ConstComboPass),
		Box::new(MIRSimplifyPass),
		Box::new(DSEPass),
		Box::new(MultifoldCombinePass),
		Box::new(MultifoldAssignPass),
		Box::new(MultifoldLogicPass),
		Box::new(ConstPropPass::new()),
		Box::new(MIRSimplifyPass),
		Box::new(ConstFoldPass::new()),
		Box::new(ConstComboPass),
		Box::new(CleanupReturnPass),
		Box::new(InlineCandidatesPass),
		Box::new(SimpleInlinePass),
		Box::new(CleanupPass),
		Box::new(MultifoldAssignPass),
		Box::new(MultifoldLogicPass),
		Box::new(ConstComboPass),
		Box::new(DSEPass),
		Box::new(MIRSimplifyPass),
		Box::new(DCEPass),
		Box::new(ReorderConditionsPass),
		Box::new(UnusedArgsPass),
	];

	let mut data = MIRPassData {
		mir,
		inline_candidates: FxHashSet::default(),
	};

	for mut pass in passes {
		if debug {
			println!("Running pass {}", pass.get_name());
		}
		pass.run_pass(&mut data)?;
	}

	Ok(())
}

pub trait LIRPass: Pass {
	fn run_pass(&mut self, lir: &mut LIR) -> anyhow::Result<()>;
}

pub fn run_lir_passes(lir: &mut LIR, debug: bool) -> anyhow::Result<()> {
	let passes = [
		Box::new(NullPass) as Box<dyn LIRPass>,
		Box::new(LIRSimplifyPass),
		Box::new(DataflowResultPass),
		Box::new(MergeModifiersPass),
		Box::new(NullModifiersPass),
		Box::new(SimplifyModifiersPass),
		Box::new(MergeModifiersPass),
		Box::new(DataflowGetPass),
		Box::new(LIRSimplifyPass),
	];

	for mut pass in passes {
		if debug {
			println!("Running pass {}", pass.get_name());
		}
		pass.run_pass(lir)?;
	}

	Ok(())
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

impl Pass for NullPass {
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
