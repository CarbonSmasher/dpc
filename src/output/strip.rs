use std::collections::HashMap;

use itertools::Itertools;

use crate::common::ResourceLocation;
use crate::lir::{LIRInstrKind, LIR};
use crate::project::ProjectSettings;

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

	for block in lir.functions.values() {
		let block = lir.blocks.get(block).expect("Block does not exist");
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
		let func = lir
			.functions
			.keys()
			.find(|x| &x.id == func_id)
			.expect("Function does not exist");
		if func.annotations.preserve || func.annotations.no_strip {
			out.0.insert(func_id.clone(), func_id.clone());
		} else {
			let mut name = get_stripped_name_unstable(idx);
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

fn get_stripped_name_unstable(idx: u32) -> String {
	if idx == 0 {
		return String::new();
	}
	let mut idx = idx as usize;
	let alphabet = [
		'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r',
		's', 't', 'u', 'v', 'w', 'x', 'y', 'z', '0', '1', '2', '3', '4', '5', '6', '7', '8', '9',
		'_', '-', '.',
	];
	let mut out = String::new();
	let mut is_first = true;
	// Add one and then subtract it on the first iteration
	// to bypass the while check
	// TODO: Make this actually good
	idx += 1;
	while idx != 0 {
		if is_first {
			idx -= 1;
		}
		let digit = idx % alphabet.len();
		out.push(alphabet[digit]);
		idx /= alphabet.len();
		is_first = false;
	}

	out
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_unstable_stripping() {
		assert_eq!(get_stripped_name_unstable(0), String::from(""));
		assert_eq!(get_stripped_name_unstable(1), String::from("b"));
		assert_eq!(get_stripped_name_unstable(38), String::from("."));
		assert_eq!(get_stripped_name_unstable(39), String::from("ab"));
	}
}
