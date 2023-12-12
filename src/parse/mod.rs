pub mod lex;
mod parse;

use std::collections::HashMap;

use anyhow::{bail, Context};

use crate::common::function::{FunctionAnnotations, FunctionInterface, FunctionSignature};
use crate::ir::{Block, IR};
use crate::parse::lex::{Side, Token};
use crate::parse::parse::{parse_body, parse_simple_ty, UnparsedBody};

use self::lex::{lex, reduce_tokens};

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
	enum State {
		Root,
		LookingForAnnotationOrFunctionName {
			state: AnnotationState,
			annotations: FunctionAnnotations,
		},
		LookingForOpeningCurly(FunctionInterface),
		Body(FunctionInterface, UnparsedBody),
	}

	enum AnnotationState {
		LookingForAt,
		LookingForName,
	}

	let mut unparsed_defs = HashMap::new();

	let mut state = State::Root;

	let lexed = lex(text).context("Failed to lex text")?;
	for (tok, pos) in reduce_tokens(lexed.iter()) {
		match &mut state {
			State::Root => match tok {
				Token::At => {
					state = State::LookingForAnnotationOrFunctionName {
						state: AnnotationState::LookingForName,
						annotations: FunctionAnnotations::new(),
					}
				}
				Token::Str(name) => {
					state =
						State::LookingForOpeningCurly(FunctionInterface::new(name.clone().into()));
				}
				_ => bail!("Unexpected token {tok:?} {pos}"),
			},
			State::LookingForAnnotationOrFunctionName {
				state: ann_state,
				annotations,
			} => match ann_state {
				AnnotationState::LookingForAt => match tok {
					Token::At => *ann_state = AnnotationState::LookingForName,
					// Function start
					Token::Str(name) => {
						state = State::LookingForOpeningCurly(FunctionInterface::with_all(
							name.clone().into(),
							FunctionSignature::new(),
							std::mem::take(annotations),
						));
					}
					_ => bail!("Unexpected token {tok:?} {pos}"),
				},
				AnnotationState::LookingForName => match tok {
					Token::Ident(name) => {
						match name.as_str() {
							"preserve" => annotations.preserve = true,
							"no_inline" => annotations.no_inline = true,
							other => bail!("Unknown annotation {other}"),
						};
						*ann_state = AnnotationState::LookingForAt;
					}
					_ => bail!("Unexpected token {tok:?} {pos}"),
				},
			},
			State::LookingForOpeningCurly(interface) => match tok {
				Token::Ident(ident) => {
					let ty = parse_simple_ty(ident).context("Failed to parse parameter type")?;
					interface.sig.params.push(ty);
				}
				Token::Curly(Side::Left) => {
					state = State::Body(std::mem::take(interface), UnparsedBody::new())
				}
				_ => bail!("Unexpected token {tok:?} {pos}"),
			},
			State::Body(interface, body) => match tok {
				Token::Curly(Side::Right) => {
					unparsed_defs.insert(std::mem::take(interface), std::mem::take(body));
					state = State::Root;
				}
				other => body.push((other.clone(), pos.clone())),
			},
		}
	}

	// Parse function bodies
	for (interface, body) in unparsed_defs {
		let body = parse_body(body).context("Failed to parse function body")?;
		let mut block = Block::new();
		block.contents = body;
		let block = ir.blocks.add(block);
		ir.functions.insert(interface, block);
	}

	Ok(())
}
