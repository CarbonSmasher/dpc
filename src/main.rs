use std::io::{stdin, Read};
use std::path::PathBuf;
use std::process::ExitCode;

use anyhow::Context;
use clap::Parser;
use dpc::{codegen_ir, project::ProjectSettings, CodegenIRSettings};

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

		std::fs::read_to_string(path).context("Failed to read input file")?
	};

	// Parse the input
	let mut parse = dpc::parse::Parser::new();
	parse.parse(&contents).expect("Failed to parse input");
	let ir = parse.finish();

	let settings = CodegenIRSettings {
		debug: false,
		debug_functions: false,
		ir_passes: false,
		mir_passes: false,
		lir_passes: false,
	};
	let name = if let Some(name) = cli.name {
		name.clone()
	} else {
		"dpc".into()
	};

	// Run the codegen
	let datapack =
		codegen_ir(ir, &ProjectSettings::new(name), settings).expect("Failed to codegen input");
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
	/// The project name, which is used for namespacing things.
	/// Defaults to 'dpc'
	#[arg(short, long)]
	name: Option<String>,
	/// The file to read from
	file: Option<String>,
}
