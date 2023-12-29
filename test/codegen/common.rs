use std::fmt::Write;

use anyhow::Context;
use dpc::output::datapack::Datapack;
use dpc::output::strip::StripMode;
use dpc::parse::lex::{lex, Token};
use dpc::project::{ProjectSettings, ProjectSettingsBuilder};
use dpc::CodegenIRSettings;
use itertools::Itertools;

pub static TEST_ENTRYPOINT: &str = "test:main";

pub fn get_control_comment(contents: &str) -> anyhow::Result<(CodegenIRSettings, ProjectSettings)> {
	let default = CodegenIRSettings {
		debug: false,
		debug_functions: false,
		ir_passes: false,
		mir_passes: false,
		lir_passes: false,
	};
	let project = ProjectSettingsBuilder::new("dpc");

	let lexed = lex(contents).context("Failed to lex text")?;
	let Some(first) = lexed.first() else { return Ok((default, project.build())) };
	let Token::Comment(comment) = &first.0 else { return Ok((default, project.build())) };

	let debug = comment.contains("debug");
	let debug_functions = comment.contains("debug_functions");
	let ir_passes = comment.contains("ir_passes");
	let mir_passes = comment.contains("mir_passes");
	let lir_passes = comment.contains("lir_passes");
	let strip_mode = if comment.contains("strip_unstable") {
		StripMode::Unstable
	} else {
		StripMode::None
	};

	let settings = CodegenIRSettings {
		debug,
		debug_functions,
		ir_passes,
		mir_passes,
		lir_passes,
	};

	let project = project.strip_mode(strip_mode);

	Ok((settings, project.build()))
}

pub fn create_output(pack: Datapack) -> anyhow::Result<String> {
	let mut out = String::new();
	for (id, func) in pack.functions.iter().sorted_by_key(|x| x.0) {
		writeln!(&mut out, "# === {id} ===")?;
		for cmd in &func.contents {
			writeln!(&mut out, "{cmd}")?;
		}
		out.push('\n');
	}

	// Remove the final newline
	out.pop();

	Ok(out)
}
