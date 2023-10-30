use crate::passes::{MIRPass, MIRPassData, Pass};

pub struct PrintBlocksPass;

impl Pass for PrintBlocksPass {
	fn get_name(&self) -> &'static str {
		"print_blocks"
	}
}

impl MIRPass for PrintBlocksPass {
	fn run_pass(&mut self, data: &mut MIRPassData) -> anyhow::Result<()> {
		dbg!(&data.mir.blocks);

		Ok(())
	}
}
