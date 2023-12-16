use std::{
	io::{stdin, Read},
	path::PathBuf, process::ExitCode,
};

use anyhow::Context;
use clap::Parser;
use dpc::{codegen_ir, CodegenIRSettings};

fn main() -> ExitCode {
	let cli = Cli::parse();
	let res = run(cli);
	if let Err(e) = res {
		eprintln!("{e:#?}");
		return ExitCode::FAILURE;
	}

	ExitCode::SUCCESS
}

fn run(cli: Cli) -> anyhow::Result<()> {
	let contents = if cli.stdin {
		let mut stdin = stdin();
		let mut text = String::new();
		stdin
			.read_to_string(&mut text)
			.context("Failed to read from stdin")?;
		text
	} else {
		let file = cli.file.context("No file specified")?;
		let path = PathBuf::from(file);
		let text = std::fs::read_to_string(path).context("Failed to read input file")?;
		text
	};

	// Parse the input
	let mut parse = dpc::parse::Parser::new();
	parse.parse(&contents).expect("Failed to parse input");
	let ir = parse.finish();

	let settings = CodegenIRSettings {
		debug: false,
		ir_passes: false,
		mir_passes: false,
		lir_passes: false,
	};

	// Run the codegen
	let datapack = codegen_ir(ir, settings).expect("Failed to codegen input");
	datapack
		.output(&PathBuf::from(cli.out))
		.context("Failed to output datapack")?;

	Ok(())
}

#[derive(Parser)]
pub struct Cli {
	/// Whether to take input from stdin
	#[arg(short, long)]
	stdin: bool,
	/// The output directory
	#[arg(short, long)]
	out: String,
	/// The file to read from
	file: Option<String>,
}
