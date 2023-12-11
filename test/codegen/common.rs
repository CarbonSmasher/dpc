use anyhow::Context;
use dpc::parse::lex::{lex, Token};
use dpc::CodegenIRSettings;

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
