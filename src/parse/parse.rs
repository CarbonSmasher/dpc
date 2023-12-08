use anyhow::{bail, Context};

use crate::common::ty::{DataType, DataTypeContents, ScoreType, ScoreTypeContents};
use crate::common::val::{MutableValue, Value};
use crate::common::DeclareBinding;
use crate::ir::{InstrKind, Instruction};

use super::lex::{Token, TokenAndPos};

pub type UnparsedBody = Vec<TokenAndPos>;

pub fn parse_body(body: UnparsedBody) -> anyhow::Result<Vec<Instruction>> {
	let mut out = Vec::new();

	// Split into the tokens for each instruction
	let split = body.split(|x| matches!(x.0, Token::Semicolon));
	for instr in split {
		let instr = parse_instr(instr).context("Failed to parse instruction")?;
		out.extend(instr);
	}

	Ok(out)
}

// If rustfmt formats this it will continue to indent the blocks forever
#[rustfmt::skip]
macro_rules! consume {
	($toks:ident, $err:block) => {{
		let tok = $toks.nth(0);
		let Some(tok) = tok else {
			$err
		};
		tok
	}};
}

// If rustfmt formats this it will continue to indent the blocks forever
#[rustfmt::skip]
macro_rules! consume_expect {
	($toks:ident, $ty:ident, $err:block) => {{
		let tok = $toks.nth(0);
		let Some(tok) = tok else {
			$err
		};
		let Token::$ty = &tok.0 else {
			bail!("Unexpected token {:?} {}", tok.0, tok.1);
		};
		tok
	}};
}

// If rustfmt formats this it will continue to indent the blocks forever
#[rustfmt::skip]
macro_rules! consume_extract {
	($toks:ident, $ty:ident, $err:block) => {{
		let tok = $toks.nth(0);
		let Some(tok) = tok else {
			$err
		};
		let Token::$ty(tok) = &tok.0 else {
			bail!("Unexpected token {:?} {}", tok.0, tok.1);
		};
		tok
	}};
}

fn parse_instr(toks: &[TokenAndPos]) -> anyhow::Result<Option<Instruction>> {
	let mut iter = toks.iter();

	let instr = consume_extract!(iter, Ident, { return Ok(None) });

	let instr = match instr.as_str() {
		"let" => parse_let(&mut iter),
		other => bail!("Unknown instruction {other}"),
	}
	.context("Failed to parse instruction")?;
	Ok(Some(Instruction::new(instr)))
}

fn parse_let<'t>(toks: &mut impl Iterator<Item = &'t TokenAndPos>) -> anyhow::Result<InstrKind> {
	let reg = consume_extract!(toks, Ident, { bail!("Missing register name") });
	consume_expect!(toks, Colon, { bail!("Missing register name") });
	let ty = parse_ty(toks).context("Failed to parse register type")?;
	consume_expect!(toks, Equal, { bail!("Missing register name") });
	let binding = parse_decl_binding(toks).context("Failed to parse declare binding")?;
	Ok(InstrKind::Declare {
		left: reg.clone().into(),
		ty,
		right: binding,
	})
}

fn parse_ty<'t>(toks: &mut impl Iterator<Item = &'t TokenAndPos>) -> anyhow::Result<DataType> {
	let ty = consume_extract!(toks, Ident, { bail!("Missing type token") });
	// TODO: More types
	match ty.as_str() {
		"score" => Ok(DataType::Score(ScoreType::Score)),
		"uscore" => Ok(DataType::Score(ScoreType::UScore)),
		"bool" => Ok(DataType::Score(ScoreType::Bool)),
		other => bail!("Unknown type {other}"),
	}
}

fn parse_decl_binding<'t>(
	toks: &mut impl Iterator<Item = &'t TokenAndPos>,
) -> anyhow::Result<DeclareBinding> {
	let kind = consume_extract!(toks, Ident, { bail!("Missing declare binding token") });
	// TODO: More declare bindings
	match kind.as_str() {
		"val" => {
			let val = parse_val(toks).context("Failed to parse value")?;
			Ok(DeclareBinding::Value(val))
		}
		other => bail!("Unknown type {other}"),
	}
}

fn parse_val<'t>(toks: &mut impl Iterator<Item = &'t TokenAndPos>) -> anyhow::Result<Value> {
	let kind = consume_extract!(toks, Ident, { bail!("Missing value type token") });
	match kind.as_str() {
		"mut" => {
			// TODO: More mutable value types
			let reg_name = consume_extract!(toks, Ident, { bail!("Missing register name token") });
			Ok(Value::Mutable(MutableValue::Register(
				reg_name.clone().into(),
			)))
		}
		"const" => {
			let lit = parse_lit(toks).context("Failed to parse literal")?;
			Ok(Value::Constant(lit))
		}
		other => bail!("Unknown value type {other}"),
	}
}

fn parse_lit<'t>(
	toks: &mut impl Iterator<Item = &'t TokenAndPos>,
) -> anyhow::Result<DataTypeContents> {
	let (tok, pos) = consume!(toks, { bail!("Missing declare binding token") });
	match tok {
		// TODO: More num literals
		Token::Num(num) => Ok(DataTypeContents::Score(ScoreTypeContents::Score(*num))),
		other => bail!("Unexpected token {other:?} {pos}"),
	}
}
