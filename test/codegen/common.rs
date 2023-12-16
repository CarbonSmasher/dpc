use std::fmt::Write;

use anyhow::Context;
use dpc::output::datapack::Datapack;
use dpc::parse::lex::{lex, Token};
use dpc::CodegenIRSettings;
use itertools::Itertools;

pub static TEST_ENTRYPOINT: &str = "test:main";

pub fn get_control_comment(contents: &str) -> anyhow::Result<CodegenIRSettings> {
	let default = CodegenIRSettings {
		debug: false,
		ir_passes: false,
		mir_passes: false,
		lir_passes: false,
	};
	let lexed = lex(contents).context("Failed to lex text")?;
	let Some(first) = lexed.first() else { return Ok(default) };
	let Token::Comment(comment) = &first.0 else { return Ok(default) };

	let debug = comment.contains("debug");
	let ir_passes = comment.contains("ir_passes");
	let mir_passes = comment.contains("mir_passes");
	let lir_passes = comment.contains("lir_passes");

	let settings = CodegenIRSettings {
		debug,
		ir_passes,
		mir_passes,
		lir_passes,
	};

	Ok(settings)
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
