use anyhow::{bail, Context};

use crate::common::condition::Condition;
use crate::common::mc::{DataLocation, Difficulty, EntityTarget, FullDataLocation, Score, XPValue};
use crate::common::ty::{
	DataType, DataTypeContents, NBTType, NBTTypeContents, ScoreType, ScoreTypeContents,
};
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
		let instr = parse_instr(&mut instr.iter()).context("Failed to parse instruction")?;
		out.extend(instr.map(Instruction::new));
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

fn parse_instr<'t>(
	toks: &mut impl Iterator<Item = &'t TokenAndPos>,
) -> anyhow::Result<Option<InstrKind>> {
	let instr = consume_extract!(toks, Ident, { return Ok(None) });

	let instr = match instr.as_str() {
		"let" => parse_let(toks),
		"set" => {
			let (l, r) = parse_simple_op(toks)?;
			Ok(InstrKind::Assign { left: l, right: r })
		}
		"add" => {
			let (l, r) = parse_simple_op(toks)?;
			Ok(InstrKind::Add { left: l, right: r })
		}
		"sub" => {
			let (l, r) = parse_simple_op(toks)?;
			Ok(InstrKind::Sub { left: l, right: r })
		}
		"mul" => {
			let (l, r) = parse_simple_op(toks)?;
			Ok(InstrKind::Mul { left: l, right: r })
		}
		"div" => {
			let (l, r) = parse_simple_op(toks)?;
			Ok(InstrKind::Div { left: l, right: r })
		}
		"mod" => {
			let (l, r) = parse_simple_op(toks)?;
			Ok(InstrKind::Mod { left: l, right: r })
		}
		"min" => {
			let (l, r) = parse_simple_op(toks)?;
			Ok(InstrKind::Min { left: l, right: r })
		}
		"max" => {
			let (l, r) = parse_simple_op(toks)?;
			Ok(InstrKind::Max { left: l, right: r })
		}
		"swap" => {
			let (l, r) = parse_swap(toks)?;
			Ok(InstrKind::Swap { left: l, right: r })
		}
		"abs" => {
			let val = parse_mut_val(toks).context("Failed to parse value")?;
			Ok(InstrKind::Abs { val })
		}
		"pow" => {
			let (l, r) = parse_pow(toks)?;
			Ok(InstrKind::Pow { base: l, exp: r })
		}
		"use" => {
			let val = parse_mut_val(toks).context("Failed to parse value")?;
			Ok(InstrKind::Use { val })
		}
		"get" => {
			let val = parse_mut_val(toks).context("Failed to parse value")?;
			Ok(InstrKind::Get { value: val })
		}
		"mrg" => {
			let (l, r) = parse_simple_op(toks)?;
			Ok(InstrKind::Merge { left: l, right: r })
		}
		"psh" => {
			let (l, r) = parse_simple_op(toks)?;
			Ok(InstrKind::Push { left: l, right: r })
		}
		"pshf" => {
			let (l, r) = parse_simple_op(toks)?;
			Ok(InstrKind::PushFront { left: l, right: r })
		}
		"ins" => {
			let (l, r) = parse_simple_op(toks)?;
			consume_expect!(toks, Comma, { bail!("Missing comma") });
			let idx = consume_extract!(toks, Num, { bail!("Missing exponent") });
			let idx: i32 = (*idx).try_into().context("Index not within 32 bit range")?;
			Ok(InstrKind::Insert {
				left: l,
				right: r,
				index: idx,
			})
		}
		"say" => {
			let msg = consume_extract!(toks, Str, { bail!("Missing message") });
			Ok(InstrKind::Say {
				message: msg.clone(),
			})
		}
		"tell" => {
			let tgt = parse_entity_target(toks).context("Failed to parse target")?;
			consume_expect!(toks, Comma, { bail!("Missing comma") });
			let msg = consume_extract!(toks, Str, { bail!("Missing message") });
			Ok(InstrKind::Tell {
				target: tgt,
				message: msg.clone(),
			})
		}
		"me" => {
			let msg = consume_extract!(toks, Str, { bail!("Missing message") });
			Ok(InstrKind::Me {
				message: msg.clone(),
			})
		}
		"tm" => {
			let msg = consume_extract!(toks, Str, { bail!("Missing message") });
			Ok(InstrKind::TeamMessage {
				message: msg.clone(),
			})
		}
		"banl" => Ok(InstrKind::Banlist),
		"bani" => {
			let tgt = consume_extract!(toks, Str, { bail!("Missing target") });
			consume_optional_expect!(toks, Comma);
			let reason = consume_optional_extract!(toks, Str);
			Ok(InstrKind::BanIP {
				target: tgt.clone(),
				reason: reason.cloned(),
			})
		}
		"pari" => {
			let tgt = consume_extract!(toks, Str, { bail!("Missing target") });
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
			let diff = consume_extract!(toks, Ident, { bail!("Missing difficulty") });
			let Some(diff) = Difficulty::parse(diff) else {
				bail!("Invalid difficulty");
			};
			Ok(InstrKind::SetDifficulty { difficulty: diff })
		}
		"specs" => Ok(InstrKind::SpectateStop),
		"if" => parse_if(toks).context("Failed to parse if"),
		"kill" => {
			let tgt = parse_entity_target(toks).context("Failed to parse target")?;
			Ok(InstrKind::Kill { target: tgt })
		}
		"ench" => {
			let tgt = parse_entity_target(toks).context("Failed to parse target")?;
			consume_expect!(toks, Comma, { bail!("Missing comma") });
			let ench = consume_extract!(toks, Str, { bail!("Missing enchantment") });
			consume_expect!(toks, Comma, { bail!("Missing comma") });
			let lvl = consume_extract!(toks, Num, { bail!("Missing level") });
			let lvl: i32 = (*lvl).try_into().context("Level is not an i32")?;

			Ok(InstrKind::Enchant {
				target: tgt,
				enchantment: ench.clone().into(),
				level: lvl,
			})
		}
		"xps" => {
			let tgt = parse_entity_target(toks).context("Failed to parse target")?;
			consume_expect!(toks, Comma, { bail!("Missing comma") });
			let amt = consume_extract!(toks, Num, { bail!("Missing amount") });
			let amt: i32 = (*amt).try_into().context("Amount is not an i32")?;
			consume_expect!(toks, Comma, { bail!("Missing comma") });
			let kind = consume_extract!(toks, Ident, { bail!("Missing XP value") });
			let Some(kind) = XPValue::parse(kind) else {
				bail!("Invalid XP value");
			};
			Ok(InstrKind::SetXP {
				target: tgt,
				amount: amt,
				value: kind,
			})
		}
		"taga" => {
			let tgt = parse_entity_target(toks).context("Failed to parse target")?;
			consume_expect!(toks, Comma, { bail!("Missing comma") });
			let tag = consume_extract!(toks, Str, { bail!("Missing tag") });
			Ok(InstrKind::AddTag {
				target: tgt,
				tag: tag.clone().into(),
			})
		}
		"tagr" => {
			let tgt = parse_entity_target(toks).context("Failed to parse target")?;
			consume_expect!(toks, Comma, { bail!("Missing comma") });
			let tag = consume_extract!(toks, Str, { bail!("Missing tag") });
			Ok(InstrKind::RemoveTag {
				target: tgt,
				tag: tag.clone().into(),
			})
		}
		"tagl" => {
			let tgt = parse_entity_target(toks).context("Failed to parse target")?;
			Ok(InstrKind::ListTags { target: tgt })
		}
		"mnt" => {
			let tgt = parse_entity_target(toks).context("Failed to parse target")?;
			consume_expect!(toks, Comma, { bail!("Missing comma") });
			let vehicle = parse_entity_target(toks).context("Failed to parse vehicle target")?;
			Ok(InstrKind::RideMount {
				target: tgt,
				vehicle,
			})
		}
		"dmnt" => {
			let tgt = parse_entity_target(toks).context("Failed to parse target")?;
			Ok(InstrKind::RideDismount { target: tgt })
		}
		"spec" => {
			let tgt = parse_entity_target(toks).context("Failed to parse target")?;
			consume_expect!(toks, Comma, { bail!("Missing comma") });
			let spectator =
				parse_entity_target(toks).context("Failed to parse spectator target")?;
			Ok(InstrKind::Spectate {
				target: tgt,
				spectator,
			})
		}
		"sbor" => {
			let obj = consume_extract!(toks, Str, { bail!("Missing objective") });
			Ok(InstrKind::RemoveScoreboardObjective {
				objective: obj.clone(),
			})
		}
		"sbol" => Ok(InstrKind::ListScoreboardObjectives),
		"trga" => {
			let obj = consume_extract!(toks, Str, { bail!("Missing objective") });
			consume_expect!(toks, Comma, { bail!("Missing comma") });
			let amt = consume_extract!(toks, Num, { bail!("Missing amount") });
			let amt: i32 = (*amt).try_into().context("Amount is not an i32")?;
			Ok(InstrKind::TriggerAdd {
				objective: obj.clone(),
				amount: amt,
			})
		}
		"trgs" => {
			let obj = consume_extract!(toks, Str, { bail!("Missing objective") });
			consume_expect!(toks, Comma, { bail!("Missing comma") });
			let amt = consume_extract!(toks, Num, { bail!("Missing amount") });
			let amt: i32 = (*amt).try_into().context("Amount is not an i32")?;
			Ok(InstrKind::TriggerSet {
				objective: obj.clone(),
				amount: amt,
			})
		}
		"dpd" => {
			let pack = consume_extract!(toks, Str, { bail!("Missing pack") });
			Ok(InstrKind::DisableDatapack { pack: pack.clone() })
		}
		"dpe" => {
			let pack = consume_extract!(toks, Str, { bail!("Missing pack") });
			Ok(InstrKind::EnableDatapack { pack: pack.clone() })
		}
		"lspu" => Ok(InstrKind::ListPlayerUUIDs),
		other => bail!("Unknown instruction {other}"),
	}
	.context("Failed to parse instruction")?;
	Ok(Some(instr))
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
	let left = parse_mut_val(toks).context("Failed to parse operator left hand side")?;
	consume_expect!(toks, Comma, { bail!("Missing comma") });
	let right = parse_val(toks).context("Failed to parse operator right hand side")?;
	Ok((left, right))
}

fn parse_swap<'t>(
	toks: &mut impl Iterator<Item = &'t TokenAndPos>,
) -> anyhow::Result<(MutableValue, MutableValue)> {
	let l = parse_mut_val(toks).context("Failed to parse swap left hand side")?;
	consume_expect!(toks, Comma, { bail!("Missing comma") });
	let r = parse_mut_val(toks).context("Failed to parse swap right hand side")?;
	Ok((l, r))
}

fn parse_pow<'t>(
	toks: &mut impl Iterator<Item = &'t TokenAndPos>,
) -> anyhow::Result<(MutableValue, u8)> {
	let base = parse_mut_val(toks).context("Failed to parse base value")?;
	consume_expect!(toks, Comma, { bail!("Missing comma") });
	let exp = consume_extract!(toks, Num, { bail!("Missing exponent") });
	let exp: u8 = (*exp).try_into().context("Exponent is not a u8")?;
	Ok((base, exp))
}

fn parse_ty<'t>(toks: &mut impl Iterator<Item = &'t TokenAndPos>) -> anyhow::Result<DataType> {
	let ty = consume_extract!(toks, Ident, { bail!("Missing type token") });
	// TODO: More complex types
	match ty.as_str() {
		"score" => Ok(DataType::Score(ScoreType::Score)),
		"uscore" => Ok(DataType::Score(ScoreType::UScore)),
		"bool" => Ok(DataType::Score(ScoreType::Bool)),
		"nbyte" => Ok(DataType::NBT(NBTType::Byte)),
		"nbool" => Ok(DataType::NBT(NBTType::Bool)),
		"nshort" => Ok(DataType::NBT(NBTType::Short)),
		"nint" => Ok(DataType::NBT(NBTType::Int)),
		"nlong" => Ok(DataType::NBT(NBTType::Long)),
		"nstr" => Ok(DataType::NBT(NBTType::String)),
		other => bail!("Unknown type {other}"),
	}
}

fn parse_decl_binding<'t>(
	toks: &mut impl Iterator<Item = &'t TokenAndPos>,
) -> anyhow::Result<DeclareBinding> {
	let kind = consume_extract!(toks, Ident, { bail!("Missing declare binding token") });
	// TODO: Idx
	match kind.as_str() {
		"val" => {
			let val = parse_val(toks).context("Failed to parse value")?;
			Ok(DeclareBinding::Value(val))
		}
		"null" => Ok(DeclareBinding::Null),
		"cast" => {
			let ty = parse_ty(toks).context("Failed to parse cast type")?;
			let val = parse_mut_val(toks).context("Failed to parse cast value")?;
			Ok(DeclareBinding::Cast(ty, val))
		}
		other => bail!("Unknown type {other}"),
	}
}

fn parse_val<'t>(toks: &mut impl Iterator<Item = &'t TokenAndPos>) -> anyhow::Result<Value> {
	// Try both
	let first_tok = consume!(toks, { bail!("Missing first value token") });
	let val = parse_mut_val_impl(first_tok, toks).context("Failed to parse mutable value");
	match val {
		Ok(val) => Ok(Value::Mutable(val)),
		Err(..) => {
			let lit = parse_lit_impl(first_tok, toks).context("Failed to parse literal")?;
			Ok(Value::Constant(lit))
		}
	}
}

fn parse_mut_val<'t>(
	toks: &mut impl Iterator<Item = &'t TokenAndPos>,
) -> anyhow::Result<MutableValue> {
	let first_tok = consume!(toks, { bail!("Missing first mutable value token") });
	parse_mut_val_impl(first_tok, toks)
}

fn parse_mut_val_impl<'t>(
	first_tok: &TokenAndPos,
	toks: &mut impl Iterator<Item = &'t TokenAndPos>,
) -> anyhow::Result<MutableValue> {
	let (tok, pos) = first_tok;
	let out = match tok {
		Token::Percent => {
			let reg_name = consume_extract!(toks, Ident, { bail!("Missing register name token") });
			MutableValue::Register(reg_name.clone().into())
		}
		Token::Ident(ident) => match ident.as_str() {
			"sco" => {
				let score = parse_score(toks).context("Failed to parse score")?;
				MutableValue::Score(score)
			}
			other => {
				let location = impl_parse_full_data_location(other, toks)
					.context("Failed to parse data location")?;
				MutableValue::Data(location)
			}
		},
		other => bail!("Unexpected token {other:?} {pos}"),
	};
	Ok(out)
}

fn parse_score<'t>(toks: &mut impl Iterator<Item = &'t TokenAndPos>) -> anyhow::Result<Score> {
	let holder = parse_entity_target(toks).context("Failed to parse score holder")?;
	let objective = consume_extract!(toks, Str, { bail!("Missing score objective token") });
	Ok(Score::new(holder, objective.clone().into()))
}

#[allow(dead_code)]
fn parse_full_data_location<'t>(
	toks: &mut impl Iterator<Item = &'t TokenAndPos>,
) -> anyhow::Result<FullDataLocation> {
	let loc = parse_data_location(toks).context("Failed to parse data location")?;
	let path = consume_extract!(toks, Str, { bail!("Missing data path token") });
	Ok(FullDataLocation {
		loc,
		path: path.clone(),
	})
}

fn impl_parse_full_data_location<'t>(
	loc: &str,
	toks: &mut impl Iterator<Item = &'t TokenAndPos>,
) -> anyhow::Result<FullDataLocation> {
	let loc = impl_parse_data_location(loc, toks).context("Failed to parse data location")?;
	let path = consume_extract!(toks, Str, { bail!("Missing data path token") });
	Ok(FullDataLocation {
		loc,
		path: path.clone(),
	})
}

#[allow(dead_code)]
fn parse_data_location<'t>(
	toks: &mut impl Iterator<Item = &'t TokenAndPos>,
) -> anyhow::Result<DataLocation> {
	let loc = consume_extract!(toks, Ident, { bail!("Missing value type token") });
	impl_parse_data_location(loc, toks)
}

fn impl_parse_data_location<'t>(
	loc: &str,
	toks: &mut impl Iterator<Item = &'t TokenAndPos>,
) -> anyhow::Result<DataLocation> {
	match loc {
		"ent" => {
			let tgt = parse_entity_target(toks).context("Failed to parse entity target")?;
			Ok(DataLocation::Entity(tgt))
		}
		"stg" => {
			let loc = consume_extract!(toks, Str, { bail!("Missing storage location token") });
			Ok(DataLocation::Storage(loc.clone().into()))
		}
		other => bail!("Unknown data location type {other}"),
	}
}

#[allow(dead_code)]
fn parse_lit<'t>(
	toks: &mut impl Iterator<Item = &'t TokenAndPos>,
) -> anyhow::Result<DataTypeContents> {
	let first_tok = consume!(toks, { bail!("Missing first literal token") });
	parse_lit_impl(first_tok, toks)
}

fn parse_lit_impl<'t>(
	first_tok: &TokenAndPos,
	toks: &mut impl Iterator<Item = &'t TokenAndPos>,
) -> anyhow::Result<DataTypeContents> {
	macro_rules! num_lit {
		($val:expr, $kind:ident, $kind2:ident, $kind3:ident, $name:literal) => {{
			let num =
				(*$val)
					.try_into()
					.expect(concat!("Numeric value is not within", $name, "range"));
			DataTypeContents::$kind($kind2::$kind3(num))
		}};
	}

	macro_rules! score_lit {
		($val:expr, $kind:ident, $name:literal) => {{
			num_lit!($val, Score, ScoreTypeContents, $kind, $name)
		}};
	}

	macro_rules! nbt_lit {
		($val:expr, $kind:ident, $name:literal) => {{
			num_lit!($val, NBT, NBTTypeContents, $kind, $name)
		}};
	}

	let (tok, pos) = first_tok;

	match tok {
		Token::Num(num) => {
			let suffix = consume_extract!(toks, Ident, {
				bail!("Missing number literal suffix token")
			});

			Ok(match suffix.as_str() {
				"s" => score_lit!(num, Score, "score"),
				"u" => score_lit!(num, UScore, "uscore"),
				"nb" => nbt_lit!(num, Byte, "nbyte"),
				"ns" => nbt_lit!(num, Short, "nshort"),
				"ni" => nbt_lit!(num, Int, "nint"),
				"nl" => nbt_lit!(num, Int, "nlong"),
				other => bail!("Unknown numeric literal suffix {other}"),
			})
		}
		Token::Ident(ident) => match ident.as_str() {
			"true" => Ok(DataTypeContents::Score(ScoreTypeContents::Bool(true))),
			"false" => Ok(DataTypeContents::Score(ScoreTypeContents::Bool(false))),
			"truen" => Ok(DataTypeContents::NBT(NBTTypeContents::Bool(true))),
			"falsen" => Ok(DataTypeContents::NBT(NBTTypeContents::Bool(false))),
			other => bail!("Unknown data value {other}"),
		},
		Token::Str(string) => Ok(DataTypeContents::NBT(NBTTypeContents::String(
			string.clone().into(),
		))),
		other => bail!("Unexpected token {other:?} {pos}"),
	}
}

fn parse_entity_target<'t>(
	toks: &mut impl Iterator<Item = &'t TokenAndPos>,
) -> anyhow::Result<EntityTarget> {
	let player = consume_extract!(toks, Str, { bail!("Missing score holder token") });
	Ok(EntityTarget::Player(player.clone()))
}

fn parse_if<'t>(toks: &mut impl Iterator<Item = &'t TokenAndPos>) -> anyhow::Result<InstrKind> {
	let condition = parse_condition(toks).context("Failed to parse if condition")?;
	consume_expect!(toks, Colon, { bail!("Missing colon") });
	let instr = parse_instr(toks).context("Failed to parse if body instruction")?;
	let Some(instr) = instr else { bail!("If instruction missing") };
	Ok(InstrKind::If {
		condition,
		body: Box::new(instr),
	})
}

fn parse_condition<'t>(
	toks: &mut impl Iterator<Item = &'t TokenAndPos>,
) -> anyhow::Result<Condition> {
	let ty = consume_extract!(toks, Ident, { bail!("Missing condition type token") });
	match ty.as_str() {
		"not" => {
			let condition = parse_condition(toks).context("Failed to parse not condition")?;
			Ok(Condition::Not(Box::new(condition)))
		}
		"eq" => {
			let (l, r) = parse_simple_condition(toks).context("Failed to parse condition")?;
			Ok(Condition::Equal(l, r))
		}
		"exists" => {
			let val = parse_val(toks).context("Failed to parse exists value")?;
			Ok(Condition::Exists(val))
		}
		"gt" => {
			let (l, r) = parse_simple_condition(toks).context("Failed to parse condition")?;
			Ok(Condition::GreaterThan(l, r))
		}
		"gte" => {
			let (l, r) = parse_simple_condition(toks).context("Failed to parse condition")?;
			Ok(Condition::GreaterThanOrEqual(l, r))
		}
		"lt" => {
			let (l, r) = parse_simple_condition(toks).context("Failed to parse condition")?;
			Ok(Condition::LessThan(l, r))
		}
		"lte" => {
			let (l, r) = parse_simple_condition(toks).context("Failed to parse condition")?;
			Ok(Condition::LessThanOrEqual(l, r))
		}
		other => bail!("Unknown condition type {other}"),
	}
}

fn parse_simple_condition<'t>(
	toks: &mut impl Iterator<Item = &'t TokenAndPos>,
) -> anyhow::Result<(Value, Value)> {
	let left = parse_val(toks).context("Failed to parse condition left hand side")?;
	consume_expect!(toks, Comma, { bail!("Missing comma") });
	let right = parse_val(toks).context("Failed to parse condition right hand side")?;
	Ok((left, right))
}
