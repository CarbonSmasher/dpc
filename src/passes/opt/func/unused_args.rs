use std::cell::RefCell;
use std::collections::HashMap;

use rustc_hash::{FxHashMap, FxHashSet};

use crate::common::function::CallInterface;
use crate::common::val::MutableValue;
use crate::passes::opt::get_instr_call_mut;
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
		let mut unused = FxHashMap::default();
		for (func_id, func) in &mut data.mir.functions {
			let block = &mut func.block;

			// Just a simple refcell for this closure
			let used_args = RefCell::new(FxHashSet::default());

			for instr in &mut block.contents {
				instr.kind.replace_mut_vals(|x| {
					if let MutableValue::Arg(a) = x {
						used_args.borrow_mut().insert(*a);
					}
				})
			}

			let used_args = used_args.take();

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
							.get(a)
							.expect("New argument mapping should exist");
					}
				})
			}

			remove_indices(&mut func.interface.sig.params, &unused_args);

			unused.insert(func_id.clone(), unused_args);
		}

		// Remove unused args in calls
		let modify_call = |call: &mut CallInterface| {
			if let Some(unused) = unused.get(&call.function) {
				remove_indices(&mut call.args, unused);
			}
		};

		for func in data.mir.functions.values_mut() {
			let block = &mut func.block;

			for instr in &mut block.contents {
				if let Some(call) = get_instr_call_mut(&mut instr.kind) {
					modify_call(call);
				}
			}
		}

		Ok(())
	}
}
