use std::sync::Arc;

use anyhow::{bail, Context};
use rustc_hash::FxHashMap;

use crate::common::condition::Condition;
use crate::common::function::CallInterface;
use crate::common::mc::entity::{EffectDuration, SelectorParameter, SelectorType, TargetSelector};
use crate::common::mc::instr::MinecraftInstr;
use crate::common::mc::item::ItemData;
use crate::common::mc::modifier::{StoreDataType, StoreModLocation};
use crate::common::mc::pos::{
	AbsOrRelCoord, Angle, DoubleCoordinates, DoubleCoordinates2D, IntCoordinates,
};
use crate::common::mc::scoreboard_and_teams::{Criterion, SingleCriterion};
use crate::common::mc::time::{Time, TimeUnit};
use crate::common::mc::{
	DataLocation, DataPath, Difficulty, EntityTarget, FullDataLocation, Location, Score,
	SoundSource, XPValue,
};
use crate::common::ty::{
	ArraySize, DataType, DataTypeContents, Double, NBTArrayType, NBTArrayTypeContents,
	NBTCompoundType, NBTCompoundTypeContents, NBTType, NBTTypeContents, ScoreType,
	ScoreTypeContents,
};
use crate::common::val::{MutableValue, Value};
use crate::common::DeclareBinding;
use crate::ir::{InstrKind, Instruction};

use super::lex::{Side, Token, TokenAndPos};

pub type UnparsedBody = Vec<TokenAndPos>;

pub fn parse_body(body: UnparsedBody) -> anyhow::Result<Vec<Instruction>> {
	let mut out = Vec::new();

	// Split into the tokens for each instruction
	let split = body.split(|x| matches!(x.0, Token::Semicolon));
	for (i, instr) in split.enumerate() {
		let instr = parse_instr(&mut instr.iter())
			.with_context(|| format!("Failed to parse instruction {i}"))?;
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
		consume_expect!($toks, Token::$ty, $err)
	}};

	($toks:ident, $ty:pat, $err:block) => {{
		let tok = consume!($toks, $err);
		let $ty = &tok.0 else {
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
			true
		} else {
			false
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
		"and" => {
			let (l, r) = parse_simple_op(toks)?;
			Ok(InstrKind::And { left: l, right: r })
		}
		"or" => {
			let (l, r) = parse_simple_op(toks)?;
			Ok(InstrKind::Or { left: l, right: r })
		}
		"swap" => {
			let (l, r) = parse_swap(toks)?;
			Ok(InstrKind::Swap { left: l, right: r })
		}
		"rm" => {
			let val = parse_mut_val(toks).context("Failed to parse value")?;
			Ok(InstrKind::Remove { val })
		}
		"abs" => {
			let val = parse_mut_val(toks).context("Failed to parse value")?;
			Ok(InstrKind::Abs { val })
		}
		"not" => {
			let val = parse_mut_val(toks).context("Failed to parse value")?;
			Ok(InstrKind::Not { value: val })
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
			consume_expect!(toks, Comma, { bail!("Missing comma") });
			let scale = consume_extract!(toks, Decimal, { bail!("Missing exponent") });
			Ok(InstrKind::Get {
				value: val,
				scale: *scale,
			})
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
			Ok(InstrKind::MC(MinecraftInstr::Say {
				message: msg.clone(),
			}))
		}
		"tell" => {
			let tgt = parse_entity_target(toks).context("Failed to parse target")?;
			consume_expect!(toks, Comma, { bail!("Missing comma") });
			let msg = consume_extract!(toks, Str, { bail!("Missing message") });
			Ok(InstrKind::MC(MinecraftInstr::Tell {
				target: tgt,
				message: msg.clone(),
			}))
		}
		"me" => {
			let msg = consume_extract!(toks, Str, { bail!("Missing message") });
			Ok(InstrKind::MC(MinecraftInstr::Me {
				message: msg.clone(),
			}))
		}
		"tm" => {
			let msg = consume_extract!(toks, Str, { bail!("Missing message") });
			Ok(InstrKind::MC(MinecraftInstr::TeamMessage {
				message: msg.clone(),
			}))
		}
		"banl" => Ok(InstrKind::MC(MinecraftInstr::Banlist)),
		"bani" => {
			let tgt = consume_extract!(toks, Str, { bail!("Missing target") });
			consume_optional_expect!(toks, Comma);
			let reason = consume_optional_extract!(toks, Str);
			Ok(InstrKind::MC(MinecraftInstr::BanIP {
				target: tgt.clone(),
				reason: reason.cloned(),
			}))
		}
		"pari" => {
			let tgt = consume_extract!(toks, Str, { bail!("Missing target") });
			Ok(InstrKind::MC(MinecraftInstr::PardonIP {
				target: tgt.clone(),
			}))
		}
		"wlon" => Ok(InstrKind::MC(MinecraftInstr::WhitelistOn)),
		"wloff" => Ok(InstrKind::MC(MinecraftInstr::WhitelistOff)),
		"wlrl" => Ok(InstrKind::MC(MinecraftInstr::WhitelistReload)),
		"wll" => Ok(InstrKind::MC(MinecraftInstr::WhitelistList)),
		"lsp" => Ok(InstrKind::MC(MinecraftInstr::ListPlayers)),
		"rl" => Ok(InstrKind::MC(MinecraftInstr::Reload)),
		"seed" => Ok(InstrKind::MC(MinecraftInstr::Seed)),
		"stop" => Ok(InstrKind::MC(MinecraftInstr::StopServer)),
		"stops" => Ok(InstrKind::MC(MinecraftInstr::StopSound)),
		"diffg" => Ok(InstrKind::MC(MinecraftInstr::GetDifficulty)),
		"diffs" => {
			let diff = consume_extract!(toks, Ident, { bail!("Missing difficulty") });
			let Some(diff) = Difficulty::parse(diff) else {
				bail!("Invalid difficulty");
			};
			Ok(InstrKind::MC(MinecraftInstr::SetDifficulty {
				difficulty: diff,
			}))
		}
		"specs" => Ok(InstrKind::MC(MinecraftInstr::SpectateStop)),
		"if" => parse_if(toks).context("Failed to parse if"),
		"kill" => {
			let tgt = parse_entity_target(toks).context("Failed to parse target")?;
			Ok(InstrKind::MC(MinecraftInstr::Kill { target: tgt }))
		}
		"ench" => {
			let tgt = parse_entity_target(toks).context("Failed to parse target")?;
			consume_expect!(toks, Comma, { bail!("Missing comma") });
			let ench = consume_extract!(toks, Str, { bail!("Missing enchantment") });
			consume_expect!(toks, Comma, { bail!("Missing comma") });
			let lvl = consume_extract!(toks, Num, { bail!("Missing level") });
			let lvl: i32 = (*lvl).try_into().context("Level is not an i32")?;

			Ok(InstrKind::MC(MinecraftInstr::Enchant {
				target: tgt,
				enchantment: ench.clone().into(),
				level: lvl,
			}))
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
			Ok(InstrKind::MC(MinecraftInstr::SetXP {
				target: tgt,
				amount: amt,
				value: kind,
			}))
		}
		"xpa" => {
			let tgt = parse_entity_target(toks).context("Failed to parse target")?;
			consume_expect!(toks, Comma, { bail!("Missing comma") });
			let amt = consume_extract!(toks, Num, { bail!("Missing amount") });
			let amt: i32 = (*amt).try_into().context("Amount is not an i32")?;
			consume_expect!(toks, Comma, { bail!("Missing comma") });
			let kind = consume_extract!(toks, Ident, { bail!("Missing XP value") });
			let Some(kind) = XPValue::parse(kind) else {
				bail!("Invalid XP value");
			};
			Ok(InstrKind::MC(MinecraftInstr::AddXP {
				target: tgt,
				amount: amt,
				value: kind,
			}))
		}
		"xpg" => {
			let tgt = parse_entity_target(toks).context("Failed to parse target")?;
			consume_expect!(toks, Comma, { bail!("Missing comma") });
			let kind = consume_extract!(toks, Ident, { bail!("Missing XP value") });
			let Some(kind) = XPValue::parse(kind) else {
				bail!("Invalid XP value");
			};
			Ok(InstrKind::MC(MinecraftInstr::GetXP {
				target: tgt,
				value: kind,
			}))
		}
		"taga" => {
			let tgt = parse_entity_target(toks).context("Failed to parse target")?;
			consume_expect!(toks, Comma, { bail!("Missing comma") });
			let tag = consume_extract!(toks, Str, { bail!("Missing tag") });
			Ok(InstrKind::MC(MinecraftInstr::AddTag {
				target: tgt,
				tag: tag.clone().into(),
			}))
		}
		"tagr" => {
			let tgt = parse_entity_target(toks).context("Failed to parse target")?;
			consume_expect!(toks, Comma, { bail!("Missing comma") });
			let tag = consume_extract!(toks, Str, { bail!("Missing tag") });
			Ok(InstrKind::MC(MinecraftInstr::RemoveTag {
				target: tgt,
				tag: tag.clone().into(),
			}))
		}
		"tagl" => {
			let tgt = parse_entity_target(toks).context("Failed to parse target")?;
			Ok(InstrKind::MC(MinecraftInstr::ListTags { target: tgt }))
		}
		"mnt" => {
			let tgt = parse_entity_target(toks).context("Failed to parse target")?;
			consume_expect!(toks, Comma, { bail!("Missing comma") });
			let vehicle = parse_entity_target(toks).context("Failed to parse vehicle target")?;
			Ok(InstrKind::MC(MinecraftInstr::RideMount {
				target: tgt,
				vehicle,
			}))
		}
		"dmnt" => {
			let tgt = parse_entity_target(toks).context("Failed to parse target")?;
			Ok(InstrKind::MC(MinecraftInstr::RideDismount { target: tgt }))
		}
		"spec" => {
			let tgt = parse_entity_target(toks).context("Failed to parse target")?;
			consume_expect!(toks, Comma, { bail!("Missing comma") });
			let spectator =
				parse_entity_target(toks).context("Failed to parse spectator target")?;
			Ok(InstrKind::MC(MinecraftInstr::Spectate {
				target: tgt,
				spectator,
			}))
		}
		"sbor" => {
			let obj = consume_extract!(toks, Str, { bail!("Missing objective") });
			Ok(InstrKind::MC(MinecraftInstr::RemoveScoreboardObjective {
				objective: obj.clone(),
			}))
		}
		"sbol" => Ok(InstrKind::MC(MinecraftInstr::ListScoreboardObjectives)),
		"wbg" => Ok(InstrKind::MC(MinecraftInstr::WorldBorderGet)),
		"cmd" => {
			let cmd = consume_extract!(toks, Str, { bail!("Missing command") });
			Ok(InstrKind::Command {
				command: cmd.clone(),
			})
		}
		"cmt" => {
			let cmt = consume_extract!(toks, Str, { bail!("Missing comment") });
			Ok(InstrKind::Comment {
				comment: cmt.clone(),
			})
		}
		"trga" => {
			let obj = consume_extract!(toks, Str, { bail!("Missing objective") });
			consume_expect!(toks, Comma, { bail!("Missing comma") });
			let amt = consume_extract!(toks, Num, { bail!("Missing amount") });
			let amt: i32 = (*amt).try_into().context("Amount is not an i32")?;
			Ok(InstrKind::MC(MinecraftInstr::TriggerAdd {
				objective: obj.clone(),
				amount: amt,
			}))
		}
		"trgs" => {
			let obj = consume_extract!(toks, Str, { bail!("Missing objective") });
			consume_expect!(toks, Comma, { bail!("Missing comma") });
			let amt = consume_extract!(toks, Num, { bail!("Missing amount") });
			let amt: i32 = (*amt).try_into().context("Amount is not an i32")?;
			Ok(InstrKind::MC(MinecraftInstr::TriggerSet {
				objective: obj.clone(),
				amount: amt,
			}))
		}
		"dpd" => {
			let pack = consume_extract!(toks, Str, { bail!("Missing pack") });
			Ok(InstrKind::MC(MinecraftInstr::DisableDatapack {
				pack: pack.clone(),
			}))
		}
		"dpe" => {
			let pack = consume_extract!(toks, Str, { bail!("Missing pack") });
			Ok(InstrKind::MC(MinecraftInstr::EnableDatapack {
				pack: pack.clone(),
			}))
		}
		"lspu" => Ok(InstrKind::MC(MinecraftInstr::ListPlayerUUIDs)),
		"call" => {
			// Return
			let mut ret = Vec::new();
			loop {
				let first_tok = consume_optional!(toks);
				if let Some(first_tok) = first_tok {
					let val = match &first_tok.0 {
						Token::Comma => {
							parse_mut_val(toks).context("Failed to parse call return value")?
						}
						Token::Ident(string) => {
							if let "run" = string.as_str() {
								break;
							} else {
								parse_mut_val_impl(first_tok, toks)
									.context("Failed to parse call return value")?
							}
						}
						_ => parse_mut_val_impl(first_tok, toks)
							.context("Failed to parse call return value")?,
					};
					ret.push(val);
				} else {
					break;
				}
			}

			let func = consume_extract!(toks, Str, { bail!("Missing function to call") });

			// Args
			let mut args = Vec::new();
			loop {
				let first_tok = consume_optional!(toks);
				if let Some(first_tok) = first_tok {
					let val = if let Token::Comma = &first_tok.0 {
						parse_val(toks).context("Failed to parse argument value")?
					} else {
						parse_val_impl(first_tok, toks).context("Failed to parse argument value")?
					};
					args.push(val);
				} else {
					break;
				}
			}
			Ok(InstrKind::Call {
				call: CallInterface {
					function: func.clone().into(),
					args,
					ret,
				},
			})
		}
		"callx" => {
			let func = consume_extract!(toks, Str, { bail!("Missing function to call") });

			Ok(InstrKind::CallExtern {
				func: func.clone().into(),
			})
		}
		"sboa" => {
			let obj = consume_extract!(toks, Str, { bail!("Missing objective") });
			consume_expect!(toks, Comma, { bail!("Missing comma") });
			let (tok, pos) = consume!(toks, { bail!("Missing criterion") });
			let criterion = match tok {
				Token::Str(string) => Criterion::Compound(string.clone()),
				Token::Ident(ident) => {
					let criterion =
						SingleCriterion::parse(ident).context("Unknown criterion type")?;
					Criterion::Single(criterion)
				}
				other => bail!("Unexpected token {other:?} {pos}"),
			};
			let expect_display_name = consume_optional_expect!(toks, Comma);
			let display_name = if expect_display_name {
				Some(consume_extract!(toks, Str, {
					bail!("Missing display name")
				}))
			} else {
				None
			};
			Ok(InstrKind::MC(MinecraftInstr::AddScoreboardObjective {
				objective: obj.clone(),
				criterion,
				display_name: display_name.cloned(),
			}))
		}
		"sws" => {
			let pos = parse_int_coords(toks).context("Failed to parse position")?;
			consume_expect!(toks, Comma, { bail!("Missing comma") });
			let angle = parse_angle(toks).context("Failed to parse angle")?;
			Ok(InstrKind::MC(MinecraftInstr::SetWorldSpawn { pos, angle }))
		}
		"ssp" => {
			let target = parse_entity_target(toks).context("Failed to parse target")?;
			consume_expect!(toks, Comma, { bail!("Missing comma") });
			let pos = parse_int_coords(toks).context("Failed to parse position")?;
			consume_expect!(toks, Comma, { bail!("Missing comma") });
			let angle = parse_angle(toks).context("Failed to parse angle")?;
			Ok(InstrKind::MC(MinecraftInstr::SetSpawnpoint {
				targets: vec![target],
				pos,
				angle,
			}))
		}
		"smn" => {
			let entity = consume_extract!(toks, Str, { bail!("Missing entity") });
			consume_expect!(toks, Comma, { bail!("Missing comma") });
			let pos = parse_double_coords(toks).context("Failed to parse position")?;
			consume_expect!(toks, Comma, { bail!("Missing comma") });
			consume_expect!(toks, Token::Curly(Side::Left), {
				bail!("Missing NBT opening")
			});
			let (_, nbt) = parse_compound_lit(toks).context("Failed to parse NBT")?;
			Ok(InstrKind::MC(MinecraftInstr::SummonEntity {
				entity: entity.clone().into(),
				pos,
				nbt,
			}))
		}
		"itmg" => {
			let target = parse_entity_target(toks).context("Failed to parse target")?;
			consume_expect!(toks, Comma, { bail!("Missing comma") });
			let item = parse_item_data(toks).context("Failed to parse item")?;
			consume_expect!(toks, Comma, { bail!("Missing comma") });
			let amt = consume_extract!(toks, Num, { bail!("Missing amount") });
			let amt: u32 = (*amt).try_into().context("Amount is not a u32")?;
			Ok(InstrKind::MC(MinecraftInstr::GiveItem {
				target,
				item,
				amount: amt,
			}))
		}
		"itmc" => {
			// TODO: Flesh this out
			let target = parse_entity_target(toks).context("Failed to parse target")?;
			Ok(InstrKind::MC(MinecraftInstr::ClearItems {
				targets: vec![target],
				item: None,
				max_count: None,
			}))
		}
		"effc" => {
			let target = parse_entity_target(toks).context("Failed to parse target")?;
			consume_optional_expect!(toks, Comma);
			let effect = consume_optional_extract!(toks, Str);
			Ok(InstrKind::MC(MinecraftInstr::ClearEffect {
				target,
				effect: effect.map(|x| x.clone().into()),
			}))
		}
		"effg" => {
			let target = parse_entity_target(toks).context("Failed to parse target")?;
			consume_expect!(toks, Comma, { bail!("Missing comma") });
			let effect = consume_extract!(toks, Str, { bail!("Missing effect name") });
			consume_expect!(toks, Comma, { bail!("Missing comma") });
			let (tok, pos) = consume!(toks, { bail!("Missing effect duration") });
			let duration = match tok {
				Token::Ident(ident) => match ident.as_str() {
					"infinite" => EffectDuration::Infinite,
					other => bail!("Unknown effect duration {other}"),
				},
				Token::Num(num) => {
					let num = (*num)
						.try_into()
						.context("Effect duration is not within i32 range")?;
					EffectDuration::Seconds(num)
				}
				other => bail!("Unexpected token {other:?} {pos}"),
			};
			consume_expect!(toks, Comma, { bail!("Missing comma") });
			let amp = consume_extract!(toks, Num, { bail!("Missing amount") });
			let amp: u8 = (*amp).try_into().context("Amplifier is not a u8")?;
			consume_expect!(toks, Comma, { bail!("Missing comma") });
			let hide_particles =
				parse_bool(toks).context("Failed to parse hide particles setting")?;
			Ok(InstrKind::MC(MinecraftInstr::GiveEffect {
				target,
				effect: effect.clone().into(),
				duration,
				amplifier: amp,
				hide_particles,
			}))
		}
		"tima" => {
			let time = parse_time(toks).context("Failed to parse time")?;
			Ok(InstrKind::MC(MinecraftInstr::AddTime { time }))
		}
		"tims" => {
			let time = parse_time(toks).context("Failed to parse time")?;
			Ok(InstrKind::MC(MinecraftInstr::SetTime { time }))
		}
		"retv" => {
			let idx = consume_extract!(toks, Num, { bail!("Missing return index") });
			let idx: u16 = (*idx).try_into().context("Return index is not a u16")?;
			consume_expect!(toks, Comma, { bail!("Missing comma") });
			let val = parse_val(toks).context("Failed to parse return value")?;
			Ok(InstrKind::ReturnValue {
				index: idx,
				value: val,
			})
		}
		"tpe" => {
			let src = parse_entity_target(toks).context("Failed to parse source target")?;
			consume_expect!(toks, Comma, { bail!("Missing comma") });
			let dest = parse_entity_target(toks).context("Failed to parse dest target")?;

			Ok(InstrKind::MC(MinecraftInstr::TeleportToEntity {
				source: src,
				dest,
			}))
		}
		"tpl" => {
			let src = parse_entity_target(toks).context("Failed to parse source target")?;
			consume_expect!(toks, Comma, { bail!("Missing comma") });
			let dest = parse_double_coords(toks).context("Failed to parse dest location")?;

			Ok(InstrKind::MC(MinecraftInstr::TeleportToLocation {
				source: src,
				dest,
			}))
		}
		"tpr" => {
			let src = parse_entity_target(toks).context("Failed to parse source target")?;
			consume_expect!(toks, Comma, { bail!("Missing comma") });
			let dest = parse_double_coords(toks).context("Failed to parse dest location")?;
			consume_expect!(toks, Comma, { bail!("Missing comma") });
			let rot = parse_double_coords_2d(toks).context("Failed to parse rotation")?;

			Ok(InstrKind::MC(MinecraftInstr::TeleportWithRotation {
				source: src,
				dest,
				rotation: rot,
			}))
		}
		"tpfl" => {
			let src = parse_entity_target(toks).context("Failed to parse source target")?;
			consume_expect!(toks, Comma, { bail!("Missing comma") });
			let dest = parse_double_coords(toks).context("Failed to parse dest location")?;
			consume_expect!(toks, Comma, { bail!("Missing comma") });
			let face = parse_double_coords(toks).context("Failed to parse facing location")?;

			Ok(InstrKind::MC(MinecraftInstr::TeleportFacingLocation {
				source: src,
				dest,
				facing: face,
			}))
		}
		"tpfe" => {
			let src = parse_entity_target(toks).context("Failed to parse source target")?;
			consume_expect!(toks, Comma, { bail!("Missing comma") });
			let dest = parse_double_coords(toks).context("Failed to parse dest location")?;
			consume_expect!(toks, Comma, { bail!("Missing comma") });
			let face = parse_entity_target(toks).context("Failed to parse facing target")?;

			Ok(InstrKind::MC(MinecraftInstr::TeleportFacingEntity {
				source: src,
				dest,
				facing: face,
			}))
		}
		"grsb" => {
			let rule = consume_extract!(toks, Str, { bail!("Missing rule") });
			consume_expect!(toks, Comma, { bail!("Missing comma") });
			let value = parse_bool(toks).context("Missing value")?;
			Ok(InstrKind::MC(MinecraftInstr::SetGameruleBool {
				rule: rule.clone(),
				value,
			}))
		}
		"grsi" => {
			let rule = consume_extract!(toks, Str, { bail!("Missing rule") });
			consume_expect!(toks, Comma, { bail!("Missing comma") });
			let value = consume_extract!(toks, Num, { bail!("Missing value") });
			let value: i32 = (*value).try_into().context("Value is not an i32")?;

			Ok(InstrKind::MC(MinecraftInstr::SetGameruleInt {
				rule: rule.clone(),
				value,
			}))
		}
		"wba" => {
			let dist = consume_extract!(toks, Decimal, { bail!("Missing distance") });
			consume_expect!(toks, Comma, { bail!("Missing comma") });
			let time = consume_extract!(toks, Num, { bail!("Missing time") });
			let time: i32 = (*time).try_into().context("Time is not an i32")?;

			Ok(InstrKind::MC(MinecraftInstr::WorldBorderAdd {
				dist: *dist,
				time,
			}))
		}
		"wbs" => {
			let dist = consume_extract!(toks, Decimal, { bail!("Missing distance") });
			consume_expect!(toks, Comma, { bail!("Missing comma") });
			let time = consume_extract!(toks, Num, { bail!("Missing time") });
			let time: i32 = (*time).try_into().context("Time is not an i32")?;

			Ok(InstrKind::MC(MinecraftInstr::WorldBorderSet {
				dist: *dist,
				time,
			}))
		}
		"wbc" => {
			let pos = parse_int_coords(toks).context("Failed to parse pos")?;

			Ok(InstrKind::MC(MinecraftInstr::WorldBorderCenter { pos }))
		}
		"wbd" => {
			let val = consume_extract!(toks, Decimal, { bail!("Missing damage") });

			Ok(InstrKind::MC(MinecraftInstr::WorldBorderDamage {
				damage: *val,
			}))
		}
		"wbb" => {
			let val = consume_extract!(toks, Decimal, { bail!("Missing buffer") });

			Ok(InstrKind::MC(MinecraftInstr::WorldBorderBuffer {
				buffer: *val,
			}))
		}
		"wbwd" => {
			let val = consume_extract!(toks, Decimal, { bail!("Missing distance") });

			Ok(InstrKind::MC(MinecraftInstr::WorldBorderWarningDistance {
				dist: *val,
			}))
		}
		"wbwt" => {
			let time = consume_extract!(toks, Num, { bail!("Missing time") });
			let time: i32 = (*time).try_into().context("Time is not an i32")?;

			Ok(InstrKind::MC(MinecraftInstr::WorldBorderWarningTime {
				time,
			}))
		}
		"grg" => {
			let rule = consume_extract!(toks, Str, { bail!("Missing rule") });

			Ok(InstrKind::MC(MinecraftInstr::GetGamerule {
				rule: rule.clone(),
			}))
		}
		"loc" => {
			let ty = consume_extract!(toks, Ident, { bail!("Missing location type") });
			let ty = Location::parse(ty).context("Invalid location type")?;
			consume_expect!(toks, Comma, { bail!("Missing comma") });
			let location = consume_extract!(toks, Str, { bail!("Missing location") });

			Ok(InstrKind::MC(MinecraftInstr::Locate {
				location_type: ty,
				location: location.clone().into(),
			}))
		}
		"ply" => {
			let sound = consume_extract!(toks, Str, { bail!("Missing location") });
			consume_expect!(toks, Comma, { bail!("Missing comma") });
			let src = consume_extract!(toks, Ident, { bail!("Missing sound source") });
			let src = SoundSource::parse(src).context("Invalid sound source")?;
			consume_expect!(toks, Comma, { bail!("Missing comma") });
			let target = parse_entity_target(toks).context("Failed to parse entity target")?;
			consume_expect!(toks, Comma, { bail!("Missing comma") });
			let pos = parse_double_coords(toks).context("Failed to parse position")?;
			consume_expect!(toks, Comma, { bail!("Missing comma") });
			let volume = consume_extract!(toks, Decimal, { bail!("Missing volume") });
			consume_expect!(toks, Comma, { bail!("Missing comma") });
			let pitch = consume_extract!(toks, Decimal, { bail!("Missing pitch") });
			consume_expect!(toks, Comma, { bail!("Missing comma") });
			let min_volume = consume_extract!(toks, Decimal, { bail!("Missing min volume") });

			Ok(InstrKind::MC(MinecraftInstr::PlaySound {
				sound: sound.clone().into(),
				source: src,
				target,
				pos,
				volume: *volume as f32,
				pitch: *pitch as f32,
				min_volume: *min_volume as f32,
			}))
		}
		"as" => {
			let target = parse_entity_target(toks).context("Failed to parse entity target")?;
			consume_expect!(toks, Colon, { bail!("Missing colon") });
			let instr = parse_instr(toks).context("Failed to parse as body instruction")?;
			let Some(instr) = instr else { bail!("As instruction missing") };
			Ok(InstrKind::As {
				target,
				body: Box::new(instr),
			})
		}
		"at" => {
			let target = parse_entity_target(toks).context("Failed to parse entity target")?;
			consume_expect!(toks, Colon, { bail!("Missing colon") });
			let instr = parse_instr(toks).context("Failed to parse at body instruction")?;
			let Some(instr) = instr else { bail!("At instruction missing") };
			Ok(InstrKind::At {
				target,
				body: Box::new(instr),
			})
		}
		"str" => {
			let loc = parse_storage_location(toks).context("Failed to parse storage location")?;
			consume_expect!(toks, Colon, { bail!("Missing colon") });
			let instr = parse_instr(toks).context("Failed to parse str body instruction")?;
			let Some(instr) = instr else { bail!("Str instruction missing") };
			Ok(InstrKind::StoreResult {
				location: loc,
				body: Box::new(instr),
			})
		}
		"sts" => {
			let loc = parse_storage_location(toks).context("Failed to parse storage location")?;
			consume_expect!(toks, Colon, { bail!("Missing colon") });
			let instr = parse_instr(toks).context("Failed to parse sts body instruction")?;
			let Some(instr) = instr else { bail!("Sts instruction missing") };
			Ok(InstrKind::StoreSuccess {
				location: loc,
				body: Box::new(instr),
			})
		}
		"pos" => {
			let pos = parse_double_coords(toks).context("Failed to parse position")?;
			consume_expect!(toks, Colon, { bail!("Missing colon") });
			let instr = parse_instr(toks).context("Failed to parse pos body instruction")?;
			let Some(instr) = instr else { bail!("Pos instruction missing") };
			Ok(InstrKind::Positioned {
				position: pos,
				body: Box::new(instr),
			})
		}
		"retr" => {
			consume_expect!(toks, Colon, { bail!("Missing colon") });
			let instr = parse_instr(toks).context("Failed to parse retr body instruction")?;
			let Some(instr) = instr else { bail!("Retr instruction missing") };
			Ok(InstrKind::ReturnRun {
				body: Box::new(instr),
			})
		}
		"ret" => {
			let value = parse_val(toks).context("Failed to parse value")?;
			Ok(InstrKind::Return { value })
		}
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
	let first_tok = consume!(toks, { bail!("Missing first value token") });
	parse_ty_impl(first_tok, toks)
}

fn parse_ty_impl<'t>(
	first_tok: &TokenAndPos,
	toks: &mut impl Iterator<Item = &'t TokenAndPos>,
) -> anyhow::Result<DataType> {
	let (tok, pos) = first_tok;

	match tok {
		Token::Ident(ident) => parse_simple_ty(ident),
		// Array and list types
		Token::Square(Side::Left) => {
			let ty = consume_extract!(toks, Ident, { bail!("Missing array/list type token") });
			let ty = parse_simple_ty(ty).context("Failed to parse array/list internal type")?;
			let (tok, pos) = consume!(toks, { bail!("Missing next array/list token") });
			match tok {
				Token::Square(Side::Right) => {
					let DataType::NBT(ty) = ty else { bail!("Type cannot be used in a list") };
					Ok(DataType::NBT(NBTType::List(Box::new(ty))))
				}
				Token::Comma => {
					let len = consume_extract!(toks, Num, { bail!("Missing array length") });
					let len: ArraySize = (*len)
						.try_into()
						.context("Array length is not an array size")?;
					consume_expect!(toks, Token::Square(Side::Right), {
						bail!("Missing closing bracket")
					});
					let DataType::NBT(ty) = ty else { bail!("Type cannot be used in an array") };
					let ty = match ty {
						NBTType::Byte => NBTArrayType::Byte(len),
						NBTType::Int => NBTArrayType::Int(len),
						NBTType::Long => NBTArrayType::Long(len),
						other => bail!("Type {other:?} cannot be used in an array"),
					};
					Ok(DataType::NBT(NBTType::Arr(ty)))
				}
				other => bail!("Unexpected token {other:?} {pos}"),
			}
		}
		Token::Curly(Side::Left) => {
			let comp = parse_compound_ty(toks).context("Failed to parse compound type")?;
			Ok(DataType::NBT(comp))
		}
		other => bail!("Unexpected token {other:?} {pos}"),
	}
}

pub fn parse_simple_ty(ty: &str) -> anyhow::Result<DataType> {
	match ty {
		"score" => Ok(DataType::Score(ScoreType::Score)),
		"bool" => Ok(DataType::Score(ScoreType::Bool)),
		"nany" => Ok(DataType::NBT(NBTType::Any)),
		"nbyte" => Ok(DataType::NBT(NBTType::Byte)),
		"nbool" => Ok(DataType::NBT(NBTType::Bool)),
		"nshort" => Ok(DataType::NBT(NBTType::Short)),
		"nint" => Ok(DataType::NBT(NBTType::Int)),
		"nlong" => Ok(DataType::NBT(NBTType::Long)),
		"nfloat" => Ok(DataType::NBT(NBTType::Float)),
		"ndouble" => Ok(DataType::NBT(NBTType::Double)),
		"nstr" => Ok(DataType::NBT(NBTType::String)),
		other => bail!("Unknown type {other}"),
	}
}

// Does the rest of the compound ty parsing after the first bracket
fn parse_compound_ty<'t>(
	toks: &mut impl Iterator<Item = &'t TokenAndPos>,
) -> anyhow::Result<NBTType> {
	let mut out = FxHashMap::default();

	loop {
		let first_tok = consume_optional!(toks);
		if let Some(first_tok) = first_tok {
			if let Token::Curly(Side::Right) = first_tok.0 {
				break;
			}

			let Token::Str(key) = &first_tok.0 else {
				bail!("Unexpected token {:?} {}", first_tok.0, first_tok.1);
			};

			consume_expect!(toks, Token::Colon, { bail!("Missing colon") });

			let ty = parse_ty(toks).context("Failed to parse type")?;

			let DataType::NBT(ty) = ty else {
				bail!("Non-NBT types cannot be used in a compound");
			};

			out.insert(key.clone(), ty);

			let next = consume_optional!(toks);
			if let Some(next) = next {
				match &next.0 {
					Token::Comma => {}
					Token::Curly(Side::Right) => {
						break;
					}
					other => bail!("Unexpected token {other:?} {}", next.1),
				}
			}
		} else {
			break;
		}
	}

	Ok(NBTType::Compound(Arc::new(out)))
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
	let first_tok = consume!(toks, { bail!("Missing first value token") });
	parse_val_impl(first_tok, toks)
}

fn parse_val_impl<'t>(
	first_tok: &TokenAndPos,
	toks: &mut impl Iterator<Item = &'t TokenAndPos>,
) -> anyhow::Result<Value> {
	// Try both
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
			MutableValue::Reg(reg_name.clone().into())
		}
		Token::Ampersand => {
			let index = consume_extract!(toks, Num, { bail!("Missing argument index") });
			let index = (*index).try_into().context("Argument index is not a u16")?;
			MutableValue::Arg(index)
		}
		Token::Ident(ident) => match ident.as_str() {
			"sco" => {
				let score = parse_score(toks).context("Failed to parse score")?;
				MutableValue::Score(score)
			}
			"prop" => {
				let prop = consume_extract!(toks, Str, { bail!("Missing access property") });
				let val = parse_mut_val(toks).context("Failed to parse mutable value to access")?;
				MutableValue::Property(Box::new(val), prop.clone())
			}
			"idx" => {
				let index = consume_extract!(toks, Num, { bail!("Missing index") });
				let index = (*index)
					.try_into()
					.context("Argument index is not a usize")?;
				let val = parse_mut_val(toks).context("Failed to parse mutable value to index")?;
				MutableValue::Index(Box::new(val), index)
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
	let path = parse_data_path(toks).context("Failed to parse data path")?;
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
	let path = parse_data_path(toks).context("Failed to parse data path")?;
	Ok(FullDataLocation {
		loc,
		path: path.clone(),
	})
}

fn parse_data_path<'t>(
	toks: &mut impl Iterator<Item = &'t TokenAndPos>,
) -> anyhow::Result<DataPath> {
	let (tok, pos) = consume!(toks, { bail!("Missing first data path token") });
	match tok {
		Token::Str(path) => Ok(DataPath::String(path.clone())),
		Token::Ident(ident) => match ident.as_str() {
			"this" => Ok(DataPath::This),
			other => bail!("Unexpected identifier {other:?} {pos}"),
		},
		other => bail!("Unexpected token {other:?} {pos}"),
	}
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
		"blk" => {
			let block = parse_int_coords(toks).context("Failed to parse block coordinates")?;
			Ok(DataLocation::Block(block))
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
					.expect(concat!("Numeric value is not within ", $name, " range"));
			DataTypeContents::$kind($kind2::$kind3(num))
		}};

		($val:expr, as $ty:ty, $kind:ident, $kind2:ident, $kind3:ident, $name:literal) => {{
			let num = (*$val as $ty);
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
				"nb" => nbt_lit!(num, Byte, "nbyte"),
				"ns" => nbt_lit!(num, Short, "nshort"),
				"ni" => nbt_lit!(num, Int, "nint"),
				"nl" => nbt_lit!(num, Long, "nlong"),
				other => bail!("Unknown numeric literal suffix {other}"),
			})
		}
		Token::Decimal(num) => {
			let suffix = consume_extract!(toks, Ident, {
				bail!("Missing number literal suffix token")
			});

			Ok(match suffix.as_str() {
				"nf" => num_lit!(num, as f32, NBT, NBTTypeContents, Float, "nfloat"),
				"nd" => num_lit!(num, as f64, NBT, NBTTypeContents, Double, "ndouble"),
				other => bail!("Unknown numeric literal suffix {other}"),
			})
		}
		Token::Ident(ident) => match ident.as_str() {
			"true" => Ok(DataTypeContents::Score(ScoreTypeContents::Bool(true))),
			"false" => Ok(DataTypeContents::Score(ScoreTypeContents::Bool(false))),
			"truen" => Ok(DataTypeContents::NBT(NBTTypeContents::Bool(true))),
			"falsen" => Ok(DataTypeContents::NBT(NBTTypeContents::Bool(false))),
			"b" => {
				consume_expect!(toks, Token::Square(Side::Left), {
					bail!("Missing opening bracket")
				});
				Ok(DataTypeContents::NBT(NBTTypeContents::Arr(
					parse_array_lit(NBTArrayTypeContents::Byte(Vec::new(), 0), toks)?,
				)))
			}
			"i" => {
				consume_expect!(toks, Token::Square(Side::Left), {
					bail!("Missing opening bracket")
				});
				Ok(DataTypeContents::NBT(NBTTypeContents::Arr(
					parse_array_lit(NBTArrayTypeContents::Int(Vec::new(), 0), toks)?,
				)))
			}
			"l" => {
				consume_expect!(toks, Token::Square(Side::Left), {
					bail!("Missing opening bracket")
				});
				Ok(DataTypeContents::NBT(NBTTypeContents::Arr(
					parse_array_lit(NBTArrayTypeContents::Long(Vec::new(), 0), toks)?,
				)))
			}
			other => bail!("Unknown data value {other}"),
		},
		Token::Str(string) => Ok(DataTypeContents::NBT(NBTTypeContents::String(
			string.clone().into(),
		))),
		Token::Square(Side::Left) => {
			let list = parse_list_lit(toks).context("Failed to parse list literal")?;
			Ok(DataTypeContents::NBT(list))
		}
		Token::Curly(Side::Left) => {
			let comp = parse_compound_lit(toks).context("Failed to parse compound literal")?;
			Ok(DataTypeContents::NBT(NBTTypeContents::Compound(
				comp.0, comp.1,
			)))
		}
		other => bail!("Unexpected token {other:?} {pos}"),
	}
}

// Does the rest of the array lit parsing after the first ident and bracket
fn parse_array_lit<'t>(
	mut contents: NBTArrayTypeContents,
	toks: &mut impl Iterator<Item = &'t TokenAndPos>,
) -> anyhow::Result<NBTArrayTypeContents> {
	loop {
		let first_tok = consume_optional!(toks);
		if let Some(first_tok) = first_tok {
			if let Token::Square(Side::Right) = first_tok.0 {
				break;
			}

			let val = parse_lit_impl(first_tok, toks).context("Failed to parse argument value")?;
			match (&mut contents, val) {
				(
					NBTArrayTypeContents::Byte(vec, _),
					DataTypeContents::NBT(NBTTypeContents::Byte(val)),
				) => vec.push(val),
				(
					NBTArrayTypeContents::Int(vec, _),
					DataTypeContents::NBT(NBTTypeContents::Int(val)),
				) => vec.push(val),
				(
					NBTArrayTypeContents::Long(vec, _),
					DataTypeContents::NBT(NBTTypeContents::Long(val)),
				) => vec.push(val),
				_ => bail!("Incompatible types in NBT array literal"),
			}

			let next = consume_optional!(toks);
			if let Some(next) = next {
				match &next.0 {
					Token::Comma => {}
					Token::Square(Side::Right) => {
						break;
					}
					other => bail!("Unexpected token {other:?} {}", next.1),
				}
			}
		} else {
			break;
		}
	}

	contents.rectify_size();

	Ok(contents)
}

// Does the rest of the list lit parsing after the first bracket
fn parse_list_lit<'t>(
	toks: &mut impl Iterator<Item = &'t TokenAndPos>,
) -> anyhow::Result<NBTTypeContents> {
	let mut out = Vec::new();
	// First figure out the type
	let ty = parse_ty(toks).context("Failed to parse list literal type")?;
	let DataType::NBT(ty) = ty else {
		bail!("Non-NBT types cannot be used in a list");
	};
	consume_expect!(toks, Token::Square(Side::Right), {
		bail!("Missing closing type bracket")
	});
	consume_expect!(toks, Token::Square(Side::Left), {
		bail!("Missing list opening bracket")
	});

	loop {
		let first_tok = consume_optional!(toks);
		if let Some(first_tok) = first_tok {
			if let Token::Square(Side::Right) = first_tok.0 {
				break;
			}

			let val = parse_lit_impl(first_tok, toks).context("Failed to parse argument value")?;

			let val_ty = val.get_ty();
			let DataType::NBT(val_ty) = val_ty else {
				bail!("Non-NBT types cannot be used in a list");
			};
			if !val_ty.is_trivially_castable(&ty) {
				bail!("List item is incompatible with list type");
			}

			let DataTypeContents::NBT(val) = val else {
				bail!("Non-NBT types cannot be used in a list");
			};
			out.push(val);

			let next = consume_optional!(toks);
			if let Some(next) = next {
				match &next.0 {
					Token::Comma => {}
					Token::Square(Side::Right) => {
						break;
					}
					other => bail!("Unexpected token {other:?} {}", next.1),
				}
			}
		} else {
			break;
		}
	}

	Ok(NBTTypeContents::List(ty, out))
}

// Does the rest of the compound lit parsing after the first bracket
fn parse_compound_lit<'t>(
	toks: &mut impl Iterator<Item = &'t TokenAndPos>,
) -> anyhow::Result<(NBTCompoundType, NBTCompoundTypeContents)> {
	let mut ty_out = FxHashMap::default();
	let mut out = FxHashMap::default();

	loop {
		let first_tok = consume_optional!(toks);
		if let Some(first_tok) = first_tok {
			if let Token::Curly(Side::Right) = first_tok.0 {
				break;
			}

			let Token::Str(key) = &first_tok.0 else {
				bail!("Unexpected token {:?} {}", first_tok.0, first_tok.1);
			};

			consume_expect!(toks, Token::Colon, { bail!("Missing colon") });

			let val = parse_lit(toks).context("Failed to parse compound inner literal")?;

			let DataType::NBT(ty) = val.get_ty() else {
				bail!("Non-NBT types cannot be used in a compound");
			};

			ty_out.insert(key.clone(), ty);

			let DataTypeContents::NBT(val) = val else {
				bail!("Non-NBT types cannot be used in a compound");
			};
			out.insert(key.clone(), val);

			let next = consume_optional!(toks);
			if let Some(next) = next {
				match &next.0 {
					Token::Comma => {}
					Token::Curly(Side::Right) => {
						break;
					}
					other => bail!("Unexpected token {other:?} {}", next.1),
				}
			}
		} else {
			break;
		}
	}

	Ok((Arc::new(ty_out), NBTCompoundTypeContents(Arc::new(out))))
}

fn parse_entity_target<'t>(
	toks: &mut impl Iterator<Item = &'t TokenAndPos>,
) -> anyhow::Result<EntityTarget> {
	let (tok, pos) = consume!(toks, { bail!("Missing first literal token") });
	match tok {
		Token::At => {
			let ty = consume_extract!(toks, Ident, { bail!("Missing selector type token") });
			let sel = match ty.as_str() {
				"s" => SelectorType::This,
				"p" => SelectorType::NearestPlayer,
				"r" => SelectorType::RandomPlayer,
				"a" => SelectorType::AllPlayers,
				"e" => SelectorType::AllEntities,
				other => bail!("Unknown selector type {other}"),
			};
			// Parameters
			consume_expect!(toks, Token::Square(Side::Left), {
				bail!("Missing selector parameters opening bracket token");
			});
			let params =
				parse_selector_parameters(toks).context("Failed to parse selector parameters")?;
			Ok(EntityTarget::Selector(TargetSelector::with_params(
				sel, params,
			)))
		}
		Token::Str(player) => Ok(EntityTarget::Player(player.clone())),
		other => bail!("Unexpected token {other:?} {pos}"),
	}
}

fn parse_selector_parameters<'t>(
	toks: &mut impl Iterator<Item = &'t TokenAndPos>,
) -> anyhow::Result<Vec<SelectorParameter>> {
	let mut out = Vec::new();
	loop {
		let first_tok = consume_optional!(toks);
		if let Some(first_tok) = first_tok {
			if let Token::Square(Side::Right) = first_tok.0 {
				break;
			}

			let Token::Ident(key) = &first_tok.0 else {
				bail!("Unexpected token {:?} {}", first_tok.0, first_tok.1);
			};
			consume_expect!(toks, Token::Equal, {
				bail!("Missing selector parameter equal");
			});
			let param = match key.as_str() {
				// TODO: Inversion
				"type" => {
					let ty = consume_extract!(toks, Str, { bail!("Missing type token") });
					SelectorParameter::Type {
						ty: ty.clone(),
						invert: false,
					}
				}
				"tag" => {
					let tag = consume_extract!(toks, Str, { bail!("Missing tag token") });
					SelectorParameter::Tag {
						tag: tag.clone(),
						invert: false,
					}
				}
				"no_tags" => SelectorParameter::NoTags,
				"pred" => {
					let pred = consume_extract!(toks, Str, { bail!("Missing predicate token") });
					SelectorParameter::Predicate {
						predicate: pred.clone(),
						invert: false,
					}
				}
				"name" => {
					let name = consume_extract!(toks, Str, { bail!("Missing name token") });
					SelectorParameter::Name {
						name: name.clone(),
						invert: false,
					}
				}
				"nbt" => {
					consume_expect!(toks, Token::Curly(Side::Left), {
						bail!("Missing opening bracket")
					});
					let (_, nbt) = parse_compound_lit(toks).context("Failed to parse item data")?;
					SelectorParameter::NBT { nbt, invert: false }
				}
				"limit" => {
					let limit = consume_extract!(toks, Num, { bail!("Missing limit token") });
					let limit = (*limit).try_into().context("Limit is not a u32")?;
					SelectorParameter::Limit(limit)
				}
				other => bail!("Unknown selector parameter {other}"),
			};
			out.push(param);

			let next = consume_optional!(toks);
			if let Some(next) = next {
				match &next.0 {
					Token::Comma => {}
					Token::Square(Side::Right) => {
						break;
					}
					other => bail!("Unexpected token {other:?} {}", next.1),
				}
			}
		} else {
			break;
		}
	}

	Ok(out)
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
		"exi" => {
			let val = parse_val(toks).context("Failed to parse exists value")?;
			Ok(Condition::Exists(val))
		}
		"bool" => {
			let val = parse_val(toks).context("Failed to parse exists value")?;
			Ok(Condition::Bool(val))
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
		"ent" => {
			let ent = parse_entity_target(toks).context("Failed to parse target")?;
			Ok(Condition::Entity(ent))
		}
		"pred" => {
			let pred = consume_extract!(toks, Str, { bail!("Missing predicate") });
			Ok(Condition::Predicate(pred.clone().into()))
		}
		"dim" => {
			let dim = consume_extract!(toks, Str, { bail!("Missing dimension") });
			Ok(Condition::Dimension(dim.clone().into()))
		}
		"bio" => {
			let loc = parse_int_coords(toks).context("Failed to parse location")?;
			consume_expect!(toks, Comma, { bail!("Missing comma") });
			let biome = consume_extract!(toks, Str, { bail!("Missing biome") });
			Ok(Condition::Biome(loc, biome.clone().into()))
		}
		"load" => {
			let loc = parse_int_coords(toks).context("Failed to parse location")?;
			Ok(Condition::Loaded(loc))
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

fn parse_double_coords<'t>(
	toks: &mut impl Iterator<Item = &'t TokenAndPos>,
) -> anyhow::Result<DoubleCoordinates> {
	let x = parse_coord_part_double(toks).context("Failed to parse first part of coordinate")?;
	let y = parse_coord_part_double(toks).context("Failed to parse second part of coordinate")?;
	let z = parse_coord_part_double(toks).context("Failed to parse third part of coordinate")?;
	Ok(DoubleCoordinates::XYZ(x, y, z))
}

fn parse_double_coords_2d<'t>(
	toks: &mut impl Iterator<Item = &'t TokenAndPos>,
) -> anyhow::Result<DoubleCoordinates2D> {
	let x = parse_coord_part_double(toks).context("Failed to parse first part of coordinate")?;
	let y = parse_coord_part_double(toks).context("Failed to parse second part of coordinate")?;
	Ok(DoubleCoordinates2D::new(x, y))
}

fn parse_int_coords<'t>(
	toks: &mut impl Iterator<Item = &'t TokenAndPos>,
) -> anyhow::Result<IntCoordinates> {
	let x = parse_coord_part_int(toks).context("Failed to parse first part of coordinate")?;
	let y = parse_coord_part_int(toks).context("Failed to parse second part of coordinate")?;
	let z = parse_coord_part_int(toks).context("Failed to parse third part of coordinate")?;
	Ok(IntCoordinates::XYZ(x, y, z))
}

fn parse_angle<'t>(toks: &mut impl Iterator<Item = &'t TokenAndPos>) -> anyhow::Result<Angle> {
	let (rel, tok) = parse_coord_part_inner(toks).context("Failed to parse inner angle")?;
	if let Token::Decimal(val) = &tok.0 {
		let val = *val as f32;
		let out = if rel {
			Angle::new_relative(val)
		} else {
			Angle::new_absolute(val)
		};
		Ok(out)
	} else {
		bail!("Unexpected token {:?} {}", tok.0, tok.1);
	}
}

fn parse_coord_part_int<'t>(
	toks: &mut impl Iterator<Item = &'t TokenAndPos>,
) -> anyhow::Result<AbsOrRelCoord<i64>> {
	let (rel, tok) = parse_coord_part_inner(toks).context("Failed to parse coordinate")?;
	if let Token::Num(val) = &tok.0 {
		let val: i64 = (*val)
			.try_into()
			.context("Coordinate value is not an i32")?;
		let out = if rel {
			AbsOrRelCoord::Rel(val)
		} else {
			AbsOrRelCoord::Abs(val)
		};
		Ok(out)
	} else {
		bail!("Unexpected token {:?} {}", tok.0, tok.1);
	}
}

fn parse_coord_part_double<'t>(
	toks: &mut impl Iterator<Item = &'t TokenAndPos>,
) -> anyhow::Result<AbsOrRelCoord<Double>> {
	let (rel, tok) = parse_coord_part_inner(toks).context("Failed to parse coordinate")?;
	if let Token::Decimal(val) = &tok.0 {
		let out = if rel {
			AbsOrRelCoord::Rel(*val)
		} else {
			AbsOrRelCoord::Abs(*val)
		};
		Ok(out)
	} else {
		bail!("Unexpected token {:?} {}", tok.0, tok.1);
	}
}

fn parse_coord_part_inner<'t>(
	toks: &mut impl Iterator<Item = &'t TokenAndPos>,
) -> anyhow::Result<(bool, TokenAndPos)> {
	let first = consume!(toks, { bail!("Missing first coordinate part token") });
	if let Token::Tilde = first.0 {
		let next = consume!(toks, { bail!("Missing last coordinate part token") });
		Ok((true, next.clone()))
	} else {
		Ok((false, first.clone()))
	}
}

fn parse_item_data<'t>(
	toks: &mut impl Iterator<Item = &'t TokenAndPos>,
) -> anyhow::Result<ItemData> {
	let item = consume_extract!(toks, Str, { bail!("Missing item ID") });
	consume_expect!(toks, Token::Curly(Side::Left), {
		bail!("Missing opening bracket")
	});
	let (_, nbt) = parse_compound_lit(toks).context("Failed to parse item data")?;
	Ok(ItemData {
		item: item.clone().into(),
		nbt,
	})
}

fn parse_bool<'t>(toks: &mut impl Iterator<Item = &'t TokenAndPos>) -> anyhow::Result<bool> {
	let ident = consume_extract!(toks, Ident, { bail!("Missing boolean token") });
	match ident.as_str() {
		"true" => Ok(true),
		"false" => Ok(false),
		other => bail!("Unknown boolean value {other}"),
	}
}

fn parse_time<'t>(toks: &mut impl Iterator<Item = &'t TokenAndPos>) -> anyhow::Result<Time> {
	let num = consume_extract!(toks, Decimal, { bail!("Missing time token") });
	let num = *num as f32;
	let suffix = consume_extract!(toks, Ident, { bail!("Missing time suffix token") });
	let unit = match suffix.as_str() {
		"t" => TimeUnit::Ticks,
		"s" => TimeUnit::Seconds,
		"d" => TimeUnit::Days,
		other => bail!("Unknown time unit suffix {other}"),
	};

	Ok(Time::new(num, unit))
}

fn parse_storage_location<'t>(
	toks: &mut impl Iterator<Item = &'t TokenAndPos>,
) -> anyhow::Result<StoreModLocation> {
	let ty = consume_extract!(toks, Ident, {
		bail!("Missing storage location type token")
	});
	match ty.as_str() {
		"reg" => {
			let reg = consume_extract!(toks, Ident, { bail!("Missing storage register token") });
			consume_expect!(toks, Token::Comma, { bail!("Missing comma") });
			let scale = consume_extract!(toks, Decimal, { bail!("Missing scale token") });
			Ok(StoreModLocation::Reg(reg.clone().into(), *scale))
		}
		"data" => {
			let loc =
				parse_full_data_location(toks).context("Failed to parse full data location")?;
			consume_expect!(toks, Token::Comma, { bail!("Missing comma") });
			let ty = parse_storage_ty(toks).context("Failed to parse storage type")?;
			consume_expect!(toks, Token::Comma, { bail!("Missing comma") });
			let scale = consume_extract!(toks, Decimal, { bail!("Missing scale token") });
			Ok(StoreModLocation::Data(loc, ty, *scale))
		}
		"sco" => {
			let score = parse_score(toks).context("Failed to parse score")?;
			Ok(StoreModLocation::Score(score))
		}
		other => bail!("Unknown storage location type {other}"),
	}
}

fn parse_storage_ty<'t>(
	toks: &mut impl Iterator<Item = &'t TokenAndPos>,
) -> anyhow::Result<StoreDataType> {
	let ty = consume_extract!(toks, Ident, { bail!("Missing storage type token") });
	match ty.as_str() {
		"byte" => Ok(StoreDataType::Byte),
		"short" => Ok(StoreDataType::Short),
		"int" => Ok(StoreDataType::Int),
		"long" => Ok(StoreDataType::Long),
		"float" => Ok(StoreDataType::Float),
		"double" => Ok(StoreDataType::Double),
		other => bail!("Unknown storage type {other}"),
	}
}
