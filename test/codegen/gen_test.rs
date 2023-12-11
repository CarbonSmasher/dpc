mod common;

use std::{fs::File, io::Write, path::PathBuf};

use common::{get_control_comment, TEST_ENTRYPOINT};
use dpc::{codegen_ir, parse::Parser};

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
	let datapack = codegen_ir(ir, settings).expect("Failed to codegen input");

	// Check the test function
	let actual = datapack
		.functions
		.get(TEST_ENTRYPOINT.into())
		.expect("Test function does not exist");

	let mut out_file = File::create(test_dir.join(format!("{test_name}.mcfunction")))
		.expect("Failed to create output file");
	for cmd in &actual.contents {
		writeln!(&mut out_file, "{cmd}").expect("Failed to write command to output");
	}
}
