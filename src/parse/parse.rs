use anyhow::{bail, Context};

use crate::common::mc::Difficulty;
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
		let tok = consume!($toks, $err);
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
		let tok = consume!($toks, $err);
		let Token::$ty(tok) = &tok.0 else {
			bail!("Unexpected token {:?} {}", tok.0, tok.1);
		};
		tok
	}};
}

// If rustfmt formats this it will continue to indent the blocks forever
#[rustfmt::skip]
macro_rules! consume_optional {
	($toks:ident) => {{
		let tok = $toks.nth(0);
		tok
	}};
}

// If rustfmt formats this it will continue to indent the blocks forever
#[rustfmt::skip]
macro_rules! consume_optional_expect {
	($toks:ident, $ty:ident) => {{
		let tok = consume_optional!($toks);
		if let Some(tok) = &tok {
			let Token::$ty = &tok.0 else {
				bail!("Unexpected token {:?} {}", tok.0, tok.1);
			};
		}
	}};
}

// If rustfmt formats this it will continue to indent the blocks forever
#[rustfmt::skip]
macro_rules! consume_optional_extract {
	($toks:ident, $ty:ident) => {{
		let tok = consume_optional!($toks);
		if let Some(tok) = &tok {
			let Token::$ty(tok) = &tok.0 else {
				bail!("Unexpected token {:?} {}", tok.0, tok.1);
			};
			Some(tok)
		} else {
			None
		}
	}};
}

fn parse_instr(toks: &[TokenAndPos]) -> anyhow::Result<Option<Instruction>> {
	let mut iter = toks.iter();

	let instr = consume_extract!(iter, Ident, { return Ok(None) });

	let instr = match instr.as_str() {
		"let" => parse_let(&mut iter),
		"set" => {
			let (l, r) = parse_simple_op(&mut iter)?;
			Ok(InstrKind::Assign { left: l, right: r })
		}
		"add" => {
			let (l, r) = parse_simple_op(&mut iter)?;
			Ok(InstrKind::Add { left: l, right: r })
		}
		"sub" => {
			let (l, r) = parse_simple_op(&mut iter)?;
			Ok(InstrKind::Sub { left: l, right: r })
		}
		"mul" => {
			let (l, r) = parse_simple_op(&mut iter)?;
			Ok(InstrKind::Mul { left: l, right: r })
		}
		"div" => {
			let (l, r) = parse_simple_op(&mut iter)?;
			Ok(InstrKind::Div { left: l, right: r })
		}
		"mod" => {
			let (l, r) = parse_simple_op(&mut iter)?;
			Ok(InstrKind::Mod { left: l, right: r })
		}
		"min" => {
			let (l, r) = parse_simple_op(&mut iter)?;
			Ok(InstrKind::Min { left: l, right: r })
		}
		"max" => {
			let (l, r) = parse_simple_op(&mut iter)?;
			Ok(InstrKind::Max { left: l, right: r })
		}
		"swap" => {
			let (l, r) = parse_swap(&mut iter)?;
			Ok(InstrKind::Swap { left: l, right: r })
		}
		"abs" => {
			let reg = consume_extract!(iter, Ident, { bail!("Missing register name") });
			Ok(InstrKind::Abs {
				val: MutableValue::Register(reg.clone().into()),
			})
		}
		"pow" => {
			let (l, r) = parse_pow(&mut iter)?;
			Ok(InstrKind::Pow { base: l, exp: r })
		}
		"use" => {
			let reg = consume_extract!(iter, Ident, { bail!("Missing register name") });
			Ok(InstrKind::Use {
				val: MutableValue::Register(reg.clone().into()),
			})
		}
		"get" => {
			let reg = consume_extract!(iter, Ident, { bail!("Missing register name") });
			Ok(InstrKind::Get {
				value: MutableValue::Register(reg.clone().into()),
			})
		}
		"mrg" => {
			let (l, r) = parse_simple_op(&mut iter)?;
			Ok(InstrKind::Merge { left: l, right: r })
		}
		"psh" => {
			let (l, r) = parse_simple_op(&mut iter)?;
			Ok(InstrKind::Push { left: l, right: r })
		}
		"pshf" => {
			let (l, r) = parse_simple_op(&mut iter)?;
			Ok(InstrKind::PushFront { left: l, right: r })
		}
		"ins" => {
			let (l, r) = parse_simple_op(&mut iter)?;
			consume_expect!(iter, Comma, { bail!("Missing comma") });
			let idx = consume_extract!(iter, Num, { bail!("Missing exponent") });
			Ok(InstrKind::Insert {
				left: l,
				right: r,
				index: *idx,
			})
		}
		"say" => {
			let msg = consume_extract!(iter, Str, { bail!("Missing message") });
			Ok(InstrKind::Say {
				message: msg.clone(),
			})
		}
		"me" => {
			let msg = consume_extract!(iter, Str, { bail!("Missing message") });
			Ok(InstrKind::Me {
				message: msg.clone(),
			})
		}
		"tm" => {
			let msg = consume_extract!(iter, Str, { bail!("Missing message") });
			Ok(InstrKind::TeamMessage {
				message: msg.clone(),
			})
		}
		"banl" => Ok(InstrKind::Banlist),
		"bani" => {
			let tgt = consume_extract!(iter, Str, { bail!("Missing target") });
			consume_optional_expect!(iter, Comma);
			let reason = consume_optional_extract!(iter, Str);
			Ok(InstrKind::BanIP {
				target: tgt.clone(),
				reason: reason.cloned(),
			})
		}
		"pari" => {
			let tgt = consume_extract!(iter, Str, { bail!("Missing target") });
			Ok(InstrKind::PardonIP {
				target: tgt.clone(),
			})
		}
		"wlon" => Ok(InstrKind::WhitelistOn),
		"wloff" => Ok(InstrKind::WhitelistOff),
		"wlrl" => Ok(InstrKind::WhitelistReload),
		"wll" => Ok(InstrKind::WhitelistList),
		"lsp" => Ok(InstrKind::ListPlayers),
		"pub" => Ok(InstrKind::Publish),
		"rl" => Ok(InstrKind::Reload),
		"seed" => Ok(InstrKind::Seed),
		"stop" => Ok(InstrKind::StopServer),
		"stops" => Ok(InstrKind::StopSound),
		"diffg" => Ok(InstrKind::GetDifficulty),
		"diffs" => {
			let diff = consume_extract!(iter, Ident, { bail!("Missing difficulty") });
			let Some(diff) = Difficulty::parse(diff) else {
				bail!("Invalid difficulty");
			};
			Ok(InstrKind::SetDifficulty { difficulty: diff })
		}
		"specs" => Ok(InstrKind::SpectateStop),
		other => bail!("Unknown instruction {other}"),
	}
	.context("Failed to parse instruction")?;
	Ok(Some(Instruction::new(instr)))
}

fn parse_let<'t>(toks: &mut impl Iterator<Item = &'t TokenAndPos>) -> anyhow::Result<InstrKind> {
	let reg = consume_extract!(toks, Ident, { bail!("Missing register name") });
	consume_expect!(toks, Colon, { bail!("Missing comma") });
	let ty = parse_ty(toks).context("Failed to parse register type")?;
	consume_expect!(toks, Equal, { bail!("Missing equal sign") });
	let binding = parse_decl_binding(toks).context("Failed to parse declare binding")?;
	Ok(InstrKind::Declare {
		left: reg.clone().into(),
		ty,
		right: binding,
	})
}

fn parse_simple_op<'t>(
	toks: &mut impl Iterator<Item = &'t TokenAndPos>,
) -> anyhow::Result<(MutableValue, Value)> {
	let reg = consume_extract!(toks, Ident, { bail!("Missing register name") });
	consume_expect!(toks, Comma, { bail!("Missing comma") });
	let val = parse_val(toks).context("Failed to parse operator right hand side")?;
	Ok((MutableValue::Register(reg.clone().into()), val))
}

fn parse_swap<'t>(
	toks: &mut impl Iterator<Item = &'t TokenAndPos>,
) -> anyhow::Result<(MutableValue, MutableValue)> {
	let reg = consume_extract!(toks, Ident, { bail!("Missing register name") });
	consume_expect!(toks, Comma, { bail!("Missing comma") });
	let reg2 = consume_extract!(toks, Ident, { bail!("Missing register name") });
	Ok((
		MutableValue::Register(reg.clone().into()),
		MutableValue::Register(reg2.clone().into()),
	))
}

fn parse_pow<'t>(
	toks: &mut impl Iterator<Item = &'t TokenAndPos>,
) -> anyhow::Result<(MutableValue, u8)> {
	let reg = consume_extract!(toks, Ident, { bail!("Missing register name") });
	consume_expect!(toks, Comma, { bail!("Missing comma") });
	let exp = consume_extract!(toks, Num, { bail!("Missing exponent") });
	let exp: u8 = (*exp).try_into().context("Exponent is not a u8")?;
	Ok((MutableValue::Register(reg.clone().into()), exp))
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
