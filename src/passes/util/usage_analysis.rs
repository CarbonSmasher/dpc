use intset::GrowSet;
use rustc_hash::FxHashMap;

use crate::{common::{block::Block, val::MutableScoreValue}, lir::{LIRBlock, LIRInstrKind}};

use super::AnalysisResult;

/// Analyzes a block for locations where a value is modified after it is
/// copied
pub fn analyze_write_after_copy(block: &LIRBlock) -> GrowSet {
	let mut out = block.get_index_set();
	let mut copies = FxHashMap::default();

	for (i, instr) in block.contents.iter().enumerate() {
		if let LIRInstrKind::SetScore(MutableScoreValue::Local(loc), _) = &instr.kind {
			copies.insert(loc, i);
		}

		let modified = instr.kind.get_modified_locs();
		match modified {
			AnalysisResult::Known(modified) => {
				for modified in modified {
					if let Some(existing) = copies.remove(modified) {
						out.add(existing);
					}
				}
			}
			AnalysisResult::Unknown => {
				for (_, existing) in &copies {
					out.add(*existing);
				}
				copies.clear();
			}
		}
	}

	out
}
