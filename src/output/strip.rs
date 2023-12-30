use std::collections::HashMap;

use itertools::Itertools;

use crate::common::ResourceLocation;
use crate::lir::{LIRInstrKind, LIR};
use crate::project::ProjectSettings;

use super::text::{get_stripped_name_unstable, RESOURCE_LOCATION_CHARSET};

/// Different modes for stripping symbols
#[derive(Debug, Clone, Copy)]
pub enum StripMode {
	/// Don't strip symbols
	None,
	/// Strip symbols with the best algorithm, that is also unstable
	Unstable,
}

/// Mapping of original function names to new, stripped ones
#[derive(Debug)]
pub struct FunctionMapping(pub HashMap<ResourceLocation, ResourceLocation>);

pub fn strip(lir: &LIR, project: &ProjectSettings) -> Option<FunctionMapping> {
	match &project.strip_mode {
		StripMode::None => None,
		StripMode::Unstable => Some(strip_unstable(lir, project)),
	}
}

fn strip_unstable(lir: &LIR, project: &ProjectSettings) -> FunctionMapping {
	// Sort functions by how many times they are called
	let mut counts = HashMap::new();

	for func in lir.functions.values() {
		let block = lir.blocks.get(&func.block).expect("Block does not exist");
		for instr in &block.contents {
			if let LIRInstrKind::Call(func) = &instr.kind {
				let entry = counts.entry(func);
				*entry.or_insert(0) += 1;
			}
		}
	}

	let mut out = FunctionMapping(HashMap::new());
	let mut idx: u32 = 0;
	// Sort by count, and then by function id, to ensure that the order between
	// multiple functions with the same count is stable
	for func_id in counts
		.into_iter()
		.sorted_by_key(|x| (x.1, std::cmp::Reverse(x.0)))
		.rev()
		.map(|x| x.0)
	{
		let func = lir.functions.values().find(|x| &x.interface.id == func_id);
		// If it doesn't exist, then its probably a extern call that we can ignore
		let Some(func) = func else {
			continue;
		};
		if func.interface.annotations.preserve || func.interface.annotations.no_strip {
			out.0.insert(func_id.clone(), func_id.clone());
		} else {
			let mut name = get_stripped_name_unstable(idx, &RESOURCE_LOCATION_CHARSET);
			name = format!("{}:s/{name}", project.name);
			// If there is no size reduction, don't strip
			if name.len() >= func_id.len() {
				name = func_id.to_string();
			} else {
				// We are using the stripped name, so increase the index
				idx += 1;
			}
			out.0.insert(func_id.clone(), name.into());
		}
	}

	out
}
