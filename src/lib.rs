use anyhow::Context;
use ir::IR;
use output::datapack::Datapack;

use crate::lower::{ir_to_mir::lower_ir, mir_to_lir::lower_mir};
use crate::output::link;
use crate::passes::{run_ir_passes, run_lir_passes, run_mir_passes};

pub mod common;
pub mod ir;
pub mod lir;
pub mod lower;
pub mod macros;
pub mod mir;
pub mod output;
pub mod parse;
pub mod passes;
mod util;

/// Runs the full routine for lowering IR and producing a datapack
pub fn codegen_ir(mut ir: IR, settings: CodegenIRSettings) -> anyhow::Result<Datapack> {
	if settings.debug {
		println!("Functions:");
		dbg!(&ir.functions);
		println!("IR:");
		dbg!(&ir.blocks);
	}
	if settings.ir_passes {
		run_ir_passes(&mut ir).context("IR passes failed")?;
	}

	let mut mir = lower_ir(ir).context("Failed to lower IR")?;
	let init_count = mir.blocks.instr_count();
	if settings.debug {
		println!("MIR:");
		dbg!(&mir.blocks);
	}

	if settings.mir_passes {
		run_mir_passes(&mut mir).context("MIR passes failed")?;
		if settings.debug {
			println!("Optimized MIR:");
			dbg!(&mir.blocks);
		}
	}
	let final_count = mir.blocks.instr_count();
	let pct = if init_count == 0 {
		0.0
	} else {
		(1.0 - (final_count as f32 / init_count as f32)) * 100.0
	};
	if settings.debug {
		println!("Removed percent: {pct}%");
	}

	let mut lir = lower_mir(mir).context("Failed to lower MIR")?;
	let init_count = lir.blocks.instr_count();
	if settings.debug {
		println!("LIR:");
		dbg!(&lir.blocks);
	}
	if settings.lir_passes {
		run_lir_passes(&mut lir).context("LIR passes failed")?;
		if settings.debug {
			println!("Optimized LIR:");
			dbg!(&lir.blocks);
		}
	}
	let final_count = lir.blocks.instr_count();
	let pct = if init_count == 0 {
		0.0
	} else {
		(1.0 - (final_count as f32 / init_count as f32)) * 100.0
	};
	if settings.debug {
		println!("Removed percent: {pct}%");
	}

	if settings.debug {
		println!("Doing codegen...");
	}
	let datapack = link(lir).context("Failed to link datapack")?;
	if settings.debug {
		dbg!(&datapack);
	}

	Ok(datapack)
}

/// Settings for the codegen_ir utility function
pub struct CodegenIRSettings {
	pub debug: bool,
	pub ir_passes: bool,
	pub mir_passes: bool,
	pub lir_passes: bool,
}

impl CodegenIRSettings {
	/// Construct new settings
	pub fn new() -> Self {
		Self {
			debug: false,
			ir_passes: true,
			mir_passes: false,
			lir_passes: false,
		}
	}
}
