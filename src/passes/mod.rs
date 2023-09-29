use crate::{ir::IR, lir::LIR};

use self::{analysis::ir::ValidatePass, opt::lir::SimplifyLIRMathPass};

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

pub trait LIRPass {
	fn run_pass(&mut self, lir: &mut LIR) -> anyhow::Result<()>;
}

pub fn run_lir_passes(lir: &mut LIR) -> anyhow::Result<()> {
	let passes = [
		Box::new(NullPass) as Box<dyn LIRPass>,
		Box::new(SimplifyLIRMathPass),
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

impl LIRPass for NullPass {
	fn run_pass(&mut self, lir: &mut LIR) -> anyhow::Result<()> {
		let _ = lir;
		Ok(())
	}
}
