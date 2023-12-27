mod common;

use std::{fmt::Debug, panic::catch_unwind};

use anyhow::{bail, Context};
use color_print::cprintln;
use dpc::{
	codegen_ir, common::function::FunctionInterface, parse::Parser, project::ProjectSettings,
};
use include_dir::{include_dir, Dir, DirEntry, File};

use crate::common::{create_output, get_control_comment, TEST_ENTRYPOINT};

static TESTS: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/test/codegen/tests");

/// Recursive function to collect tests from the include directory
/// since it isn't recursive by default
fn collect_tests<'dir>(dir: &'dir Dir<'dir>, out: &mut Vec<&'dir File<'dir>>) {
	for entry in dir.entries() {
		match entry {
			DirEntry::Dir(dir) => collect_tests(dir, out),
			DirEntry::File(file) => out.push(file),
		}
	}
}

fn main() {
	let mut tests = Vec::new();
	collect_tests(&TESTS, &mut tests);

	let mut test_names = Vec::new();
	for file in tests {
		let path = file.path();
		let file_name = path
			.file_name()
			.expect("Failed to get filename of file")
			.to_string_lossy();
		let full_path = path
			.strip_prefix(TESTS.path())
			.expect("Failed to make test path relative")
			.to_string_lossy();
		if file_name.ends_with(".dpc") {
			let full_path = full_path
				.strip_suffix(".dpc")
				.expect("File name should end with dpc")
				.to_string();
			test_names.push(full_path);
		}
	}

	println!("Running {} tests", test_names.len());
	for test in test_names {
		println!("     - Running codegen test '{test}'");
		let result =
			catch_unwind(|| run_test(&test).unwrap_or_else(|_| panic!("Test {test} failed")));
		match result {
			Ok(..) => cprintln!("     - <g>Test {test} successful"),
			Err(e) => {
				if let Some(e) = e.downcast_ref::<Box<dyn Debug>>() {
					cprintln!("     - <r>Test {test} failed with error:\n{e:#?}");
				} else {
					cprintln!("     - <r>Test {test} failed");
				}
				panic!("Test failed");
			}
		}
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
	let datapack = codegen_ir(ir, &ProjectSettings::new("dpc".into()), settings)
		.context("Failed to codegen input")?;

	// Check the test function
	let Some(..) = datapack.functions.get(TEST_ENTRYPOINT) else {
		bail!("Test function does not exist")
	};
	let actual = create_output(datapack).expect("Failed to create actual test output");
	assert_eq!(
		actual.lines().count(),
		output_contents.lines().count(),
		"Functions are of different lengths"
	);
	let expected = output_contents.lines();
	for (i, (l, r)) in expected.zip(actual.lines()).enumerate() {
		assert_eq!(l, r, "Command mismatch at {i}");
	}

	Ok(())
}
