mod common;

use std::{fs::File, io::Write, path::PathBuf};

use common::{create_output, generate_datapacks, get_control_comment};
use dpc::parse::Parser;

fn main() {
	let test_dir = PathBuf::from("./test/codegen/tests");
	let test_name = std::env::args().nth(1).expect("Test name argument missing");
	let input = std::fs::read_to_string(test_dir.join(format!("{test_name}.dpc")))
		.expect("Failed to open test file");

	// Parse the input
	let mut parse = Parser::new();
	parse.parse(&input).expect("Failed to parse test input");
	let ir = parse.finish();

	// Run the codegen
	let (settings, project, split) =
		get_control_comment(&input).expect("Failed to get control comment");
	let datapacks =
		generate_datapacks(ir, project, settings, split).expect("Failed to codegen input");

	let mut out_file = File::create(test_dir.join(format!("{test_name}.mcfunction")))
		.expect("Failed to create output file");
	let output = create_output(datapacks).expect("Failed to output generated test");
	write!(&mut out_file, "{output}").expect("Failed to write");
}
