use anyhow::Context;
use common::IRType;
use ir::IR;
use output::datapack::Datapack;
use project::ProjectSettings;

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
pub mod project;
mod util;

/// Runs the full routine for lowering IR and producing a datapack
pub fn codegen_ir(
	mut ir: IR,
	project: &ProjectSettings,
	settings: CodegenIRSettings,
) -> anyhow::Result<Datapack> {
	if settings.debug {
		println!("IR:");
		dbg!(&ir.functions);
	}
	if settings.ir_passes {
		run_ir_passes(&mut ir, project, settings.debug).context("IR passes failed")?;
	}

	let mut mir = lower_ir(ir).context("Failed to lower IR")?;
	let init_count = mir.instr_count();
	if settings.debug {
		println!("MIR:");
		dbg!(&mir.functions);
	}

	if settings.mir_passes {
		run_mir_passes(&mut mir, project, settings.debug).context("MIR passes failed")?;
		if settings.debug {
			println!("Optimized MIR:");
			dbg!(&mir.functions);
		}
	}
	let final_count = mir.instr_count();
	let pct = if init_count == 0 {
		0.0
	} else {
		(1.0 - (final_count as f32 / init_count as f32)) * 100.0
	};
	if settings.debug {
		println!("Removed percent: {pct}%");
	}

	let mut lir = lower_mir(mir).context("Failed to lower MIR")?;
	let init_count = lir.instr_count();
	if settings.debug {
		println!("LIR:");
		dbg!(&lir.functions);
	}
	if settings.lir_passes {
		run_lir_passes(&mut lir, project, settings.debug).context("LIR passes failed")?;
		if settings.debug {
			println!("Optimized LIR:");
			dbg!(&lir.functions);
		}
	}

	let final_count = lir.instr_count();
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
	let datapack = link(lir, project).context("Failed to link datapack")?;
	if settings.debug {
		dbg!(&datapack);
	}

	Ok(datapack)
}

/// Settings for the codegen_ir utility function
#[derive(Clone)]
pub struct CodegenIRSettings {
	pub debug: bool,
	pub debug_functions: bool,
	pub ir_passes: bool,
	pub mir_passes: bool,
	pub lir_passes: bool,
}

impl CodegenIRSettings {
	/// Construct new settings
	pub fn new() -> Self {
		Self {
			debug: false,
			debug_functions: false,
			ir_passes: true,
			mir_passes: false,
			lir_passes: false,
		}
	}
}

impl Default for CodegenIRSettings {
	fn default() -> Self {
		Self::new()
	}
}
