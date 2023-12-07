use include_dir::{include_dir, Dir};

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
		let input_contents = TESTS
			.get_file(format!("{test}.dpc"))
			.expect("Input file does not exist")
			.contents_utf8()
			.expect("Input file is not UTF-8");
		let output_contents = TESTS
			.get_file(format!("{test}.mcfunction"))
			.expect("Output file does not exist")
			.contents_utf8()
			.expect("Output file is not UTF-8");

		// Run the codegen
	}
}
