use crate::common::val::Value;
use crate::mir::MIRInstrKind;
use crate::passes::{MIRPass, MIRPassData, Pass};
use crate::project::{OptimizationLevel, ProjectSettings};

pub struct CleanupReturnPass;

impl Pass for CleanupReturnPass {
	fn get_name(&self) -> &'static str {
		"cleanup_return"
	}

	fn should_run(&self, proj: &ProjectSettings) -> bool {
		proj.op_level >= OptimizationLevel::More
	}
}

impl MIRPass for CleanupReturnPass {
	fn run_pass(&mut self, data: &mut MIRPassData) -> anyhow::Result<()> {
		for func in data.mir.functions.values_mut() {
			let block = &mut func.block;

			// Remove all instructions after an early return that is in the bare body
			let mut rem_pos = None;
			for (i, instr) in block.contents.iter().enumerate() {
				match &instr.kind {
					MIRInstrKind::Return { .. } | MIRInstrKind::ReturnRun { .. } => {
						rem_pos = Some(i);
						break;
					}
					_ => {}
				}
			}
			if let Some(rem_pos) = rem_pos {
				// Don't remove the return itself, we will do that next
				block.contents.truncate(rem_pos + 1);
			}

			// Remove a ret 1 at the end of the block
			let mut rem_last = false;
			if let Some(last) = block.contents.last() {
				if let MIRInstrKind::Return {
					value: Value::Constant(val),
				} = &last.kind
				{
					if let Some(1) = val.try_get_i32() {
						rem_last = true;
					}
				}
			}
			if rem_last {
				block.contents.pop();
			}
		}

		Ok(())
	}
}
