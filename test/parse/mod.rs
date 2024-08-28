use std::panic::catch_unwind;

use anyhow::{bail, Context};
use dpc::common::function::FunctionInterface;
use dpc::common::ty::{DataType, DataTypeContents, ScoreType, ScoreTypeContents};
use dpc::common::{val::Value, DeclareBinding};
use dpc::ir::{Block, IRFunction, InstrKind, IR};
use dpc::parse::Parser;
use dpc::push_instrs;

struct Test {
	name: &'static str,
	input: &'static str,
	output: IR,
}

macro_rules! test {
	($name:literal, $output:block) => {
		Test {
			name: $name,
			input: include_str!(concat!("tests/", $name, ".dpc")),
			output: $output,
		}
	};
	($name:literal, $input:literal, $output:block) => {
		Test {
			name: $name,
			input: $input,
			output: $output,
		}
	};
}

fn main() {
	let tests = [test!("simple", {
		let mut ir = IR::new();
		let mut block = Block::new();
		push_instrs! {
			block,
			InstrKind::Declare {
				left: "x".into(),
				ty: DataType::Score(ScoreType::Score),
				right: DeclareBinding::Value(Value::Constant(DataTypeContents::Score(
					ScoreTypeContents::Score(7)
				)))
			}
		}

		ir.functions.insert(
			"test:main/main".into(),
			IRFunction {
				interface: FunctionInterface::new("test:main/main".into()),
				block,
			},
		);

		ir
	})];

	for test in tests {
		let name = test.name;
		catch_unwind(|| {
			println!("     - Running parse test '{name}'");
			run_test(test).expect("Test failed");
		})
		.unwrap_or_else(|_| panic!("Test {name} failed"));
	}
}

fn run_test(test: Test) -> anyhow::Result<()> {
	let mut parse = Parser::new();
	parse.parse(test.input).context("Failed to parse")?;
	let actual = parse.finish();
	for (func_id, func) in test.output.functions {
		let Some(actual_func) = actual.functions.get(&func_id) else {
			bail!("Function in output does not exist in input")
		};
		let expected_block = func.block;
		let actual_block = &actual_func.block;

		// Check the instructions
		assert_eq!(
			expected_block.contents.len(),
			actual_block.contents.len(),
			"Blocks are not same size"
		);
		for (i, (l, r)) in expected_block
			.contents
			.iter()
			.zip(actual_block.contents.iter())
			.enumerate()
		{
			assert_eq!(l, r, "Instruction {i} failed to match");
		}
	}

	Ok(())
}
