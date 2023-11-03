mod lex;
mod parse;

use crate::ir::IR;

/// Parser for IR
pub struct Parser {
	ir: IR,
}

impl Parser {
	pub fn new() -> Self {
		Self { ir: IR::new() }
	}

	pub fn parse(&mut self, text: &str) -> anyhow::Result<()> {
		parse_definitions(&mut self.ir, text)
	}

	pub fn finish(self) -> IR {
		self.ir
	}
}

fn parse_definitions(ir: &mut IR, text: &str) -> anyhow::Result<()> {
	Ok(())
}
