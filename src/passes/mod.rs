use crate::{ir::IR, lir::LIR, mir::MIR};

use self::analysis::ir::ValidatePass;
use self::opt::lir::InsertRegFinishesPass;
use self::opt::mir::{MIRSimplifyPass, ConstPropPass};
use self::opt::{lir::LIRSimplifyPass, mir::DSEPass};

pub mod analysis;
pub mod opt;

pub trait IRPass {
	fn run_pass(&mut self, ir: &mut IR) -> anyhow::Result<()>;
}

pub fn run_ir_passes(ir: &mut IR) -> anyhow::Result<()> {
	let passes = [
		Box::new(NullPass) as Box<dyn IRPass>,
		Box::new(ValidatePass),
	];

	for mut pass in passes {
		pass.run_pass(ir)?;
	}

	Ok(())
}

pub trait MIRPass {
	fn run_pass(&mut self, mir: &mut MIR) -> anyhow::Result<()>;
}

pub fn run_mir_passes(mir: &mut MIR) -> anyhow::Result<()> {
	let passes = [
		Box::new(NullPass) as Box<dyn MIRPass>,
		Box::new(MIRSimplifyPass),
		Box::new(ConstPropPass),
		Box::new(MIRSimplifyPass),
		Box::new(DSEPass),
	];

	for mut pass in passes {
		pass.run_pass(mir)?;
	}

	Ok(())
}

pub trait LIRPass {
	fn run_pass(&mut self, lir: &mut LIR) -> anyhow::Result<()>;
}

pub fn run_lir_passes(lir: &mut LIR) -> anyhow::Result<()> {
	let passes = [
		Box::new(NullPass) as Box<dyn LIRPass>,
		Box::new(LIRSimplifyPass),
		Box::new(InsertRegFinishesPass),
	];

	for mut pass in passes {
		pass.run_pass(lir)?;
	}

	Ok(())
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
