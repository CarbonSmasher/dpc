use std::collections::HashMap;

use anyhow::anyhow;
use dashmap::{DashMap, DashSet};

use crate::common::function::CallInterface;
use crate::common::val::MutableValue;
use crate::mir::MIRInstrKind;
use crate::passes::{MIRPass, MIRPassData, Pass};
use crate::util::remove_indices;

pub struct UnusedArgsPass;

impl Pass for UnusedArgsPass {
	fn get_name(&self) -> &'static str {
		"unused_args"
	}
}

impl MIRPass for UnusedArgsPass {
	fn run_pass(&mut self, data: &mut MIRPassData) -> anyhow::Result<()> {
		// First find unused arguments in the functions,
		// and remove those arguments from the functions
		let unused = DashMap::new();
		for (func_id, func) in &mut data.mir.functions {
			let block = data
				.mir
				.blocks
				.get_mut(&func.block)
				.ok_or(anyhow!("Block does not exist"))?;

			let used_args = DashSet::new();

			for instr in &mut block.contents {
				instr.kind.replace_mut_vals(|x| {
					if let MutableValue::Arg(a) = x {
						used_args.insert(*a);
					}
				})
			}

			// Remove unused args. We also have to remap the args in the function to be
			// correct
			let mut unused_args = Vec::new();
			let mut new_mapping = HashMap::new();
			let mut counter: u16 = 0;
			for i in 0..func.interface.sig.params.len() {
				if !used_args.contains(&(i as u16)) {
					unused_args.push(i);
					counter += 1;
				} else {
					new_mapping.insert(i as u16, i as u16 - counter);
				}
			}
			// Remap
			for instr in &mut block.contents {
				instr.kind.replace_mut_vals(|x| {
					if let MutableValue::Arg(a) = x {
						*a = *new_mapping
							.get(&a)
							.expect("New argument mapping should exist");
					}
				})
			}

			// dbg!(&func.interface.sig.params);
			remove_indices(&mut func.interface.sig.params, &unused_args);
			// dbg!(&func.interface.sig.params);

			// println!("{func_id}");
			// dbg!(&unused_args);
			unused.insert(func_id.clone(), unused_args);
		}

		// Remove unused args in calls
		let modify_call = |call: &mut CallInterface| {
			if let Some(unused) = unused.get(&call.function) {
				remove_indices(&mut call.args, unused.value());
			}
		};

		for func in data.mir.functions.values_mut() {
			let block = data
				.mir
				.blocks
				.get_mut(&func.block)
				.ok_or(anyhow!("Block does not exist"))?;

			for instr in &mut block.contents {
				match &mut instr.kind {
					// TODO: Make some sort of get_body function
					MIRInstrKind::Call { call } => modify_call(call),
					MIRInstrKind::If { body, .. }
					| MIRInstrKind::As { body, .. }
					| MIRInstrKind::At { body, .. }
					| MIRInstrKind::StoreResult { body, .. }
					| MIRInstrKind::StoreSuccess { body, .. }
					| MIRInstrKind::ReturnRun { body, .. } => match body.as_mut() {
						MIRInstrKind::Call { call } => modify_call(call),
						_ => {}
					},
					_ => {}
				}
			}
		}

		Ok(())
	}
}
