use crate::ir::IR;

pub mod analysis;

pub trait IRPass {
	fn run_pass(ir: &mut IR) -> anyhow::Result<()>;
}
