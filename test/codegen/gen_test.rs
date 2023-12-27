mod common;

use std::{fs::File, io::Write, path::PathBuf};

use common::{create_output, get_control_comment, TEST_ENTRYPOINT};
use dpc::{codegen_ir, parse::Parser, project::ProjectSettings};

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
	let settings = get_control_comment(&input).expect("Failed to get control comment");
	let datapack = codegen_ir(ir, &ProjectSettings::new("dpc".into()), settings)
		.expect("Failed to codegen input");

	// Check the test function
	datapack
		.functions
		.get(TEST_ENTRYPOINT)
		.expect("Test function does not exist");

	let mut out_file = File::create(test_dir.join(format!("{test_name}.mcfunction")))
		.expect("Failed to create output file");
	let output = create_output(datapack).expect("Failed to output generated test");
	write!(&mut out_file, "{output}").expect("Failed to write");
}
