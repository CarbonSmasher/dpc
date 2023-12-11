mod common;

use anyhow::{bail, Context};
use dpc::{codegen_ir, common::function::FunctionInterface, parse::Parser};
use include_dir::{include_dir, Dir};

use crate::common::{get_control_comment, TEST_ENTRYPOINT};

static TESTS: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/test/codegen/tests");

fn main() {
	let mut test_names = Vec::new();
	for file in TESTS.files() {
		let path = file.path();
		let file_name = path
			.file_name()
			.expect("Failed to get filename of file")
			.to_string_lossy();
		let file_stem = path
			.file_stem()
			.expect("Failed to get file stem of file")
			.to_string_lossy();
		if file_name.ends_with(".dpc") {
			test_names.push(file_stem);
		}
	}
	for test in test_names {
		println!("     - Running codegen test '{test}'");
		run_test(&test).expect(&format!("Test {test} failed"))
	}
}

fn run_test(test_name: &str) -> anyhow::Result<()> {
	let input_contents = TESTS
		.get_file(format!("{test_name}.dpc"))
		.expect("Input file does not exist")
		.contents_utf8()
		.context("Input file is not UTF-8")?;
	let output_contents = TESTS
		.get_file(format!("{test_name}.mcfunction"))
		.expect("Output file does not exist")
		.contents_utf8()
		.context("Output file is not UTF-8")?;

	// Parse the input
	let mut parse = Parser::new();
	parse
		.parse(input_contents)
		.context("Failed to parse test input")?;
	let mut ir = parse.finish();

	// Make sure the test function is marked as preserve
	let Some(mut actual) = ir.functions.remove_entry(&FunctionInterface::new(TEST_ENTRYPOINT.into())) else {
		bail!("Test function does not exist")
	};
	actual.0.annotations.preserve = true;
	ir.functions.insert(actual.0, actual.1);

	// Run the codegen
	let settings = get_control_comment(input_contents).expect("Failed to get control comment");
	let datapack = codegen_ir(ir, settings).context("Failed to codegen input")?;

	// Check the test function
	let Some(actual) = datapack.functions.get(TEST_ENTRYPOINT.into()) else {
		bail!("Test function does not exist")
	};
	assert_eq!(
		actual.contents.len(),
		output_contents.lines().count(),
		"Functions are of different lengths"
	);
	let expected = output_contents.lines();
	for (i, (l, r)) in expected.zip(actual.contents.iter()).enumerate() {
		assert_eq!(l, r, "Command mismatch at {i}");
	}

	Ok(())
}
