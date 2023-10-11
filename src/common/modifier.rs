use std::fmt::Debug;

use super::mc::{DoubleCoordinates, EntityTarget, Score};

use super::{Identifier, MutableScoreValue, ScoreValue};

/// A modifier to the context of a command
#[derive(Clone)]
pub enum Modifier {
	StoreResult(StoreModLocation),
	StoreSuccess(StoreModLocation),
	If {
		condition: Box<IfModCondition>,
		negate: bool,
	},
	Anchored(AnchorModLocation),
	As(EntityTarget),
	At(EntityTarget),
	In(String),
	On(EntityRelation),
	Positioned(DoubleCoordinates),
}

impl Modifier {
	pub fn get_used_regs(&self) -> Vec<&Identifier> {
		match self {
			Modifier::StoreResult(loc) | Modifier::StoreSuccess(loc) => loc.get_used_regs(),
			Modifier::If { condition, .. } => condition.get_used_regs(),
			_ => Vec::new(),
		}
	}
}

impl Debug for Modifier {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::StoreResult(loc) => write!(f, "str {loc:?}"),
			Self::StoreSuccess(loc) => write!(f, "sts {loc:?}"),
			Self::If { condition, negate } => {
				if *negate {
					write!(f, "if !{condition:?}")
				} else {
					write!(f, "if !{condition:?}")
				}
			}
			Self::Anchored(loc) => write!(f, "anc {loc:?}"),
			Self::As(target) => write!(f, "as {target:?}"),
			Self::At(target) => write!(f, "at {target:?}"),
			Self::In(dim) => write!(f, "in {dim}"),
			Self::On(rel) => write!(f, "on {rel:?}"),
			Self::Positioned(coords) => write!(f, "pos {coords:?}"),
		}
	}
}

#[derive(Clone)]
pub enum StoreModLocation {
	Reg(Identifier),
	Score(Score),
}

impl StoreModLocation {
	pub fn get_used_regs(&self) -> Vec<&Identifier> {
		match self {
			Self::Reg(reg) => vec![reg],
			Self::Score(..) => Vec::new(),
		}
	}
}

impl Debug for StoreModLocation {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Reg(reg) => write!(f, "{reg}"),
			Self::Score(score) => write!(f, "{score:?}"),
		}
	}
}

#[derive(Debug, Clone)]
pub enum AnchorModLocation {
	Eyes,
	Feet,
}

#[derive(Debug, Clone)]
pub enum EntityRelation {
	Attacker,
	Controller,
	Leasher,
	Origin,
	Owner,
	Passengers,
	Target,
	Vehicle,
}

#[derive(Clone)]
pub enum IfModCondition {
	Score(IfScoreCondition),
}

impl IfModCondition {
	pub fn get_used_regs(&self) -> Vec<&Identifier> {
		match self {
			Self::Score(cond) => match cond {
				IfScoreCondition::Single { left, right } => {
					[left.get_used_regs(), right.get_used_regs()].concat()
				}
				IfScoreCondition::Range { score, left, right } => [
					score.get_used_regs(),
					left.get_used_regs(),
					right.get_used_regs(),
				]
				.concat(),
			},
		}
	}
}

impl Debug for IfModCondition {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Score(condition) => write!(f, "sco {condition:?}"),
		}
	}
}

#[derive(Clone)]
pub enum IfScoreCondition {
	Single {
		left: MutableScoreValue,
		right: ScoreValue,
	},
	Range {
		score: MutableScoreValue,
		left: IfScoreRangeEnd,
		right: IfScoreRangeEnd,
	},
}

impl Debug for IfScoreCondition {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Single { left, right } => write!(f, "{left:?} = {right:?}"),
			Self::Range { score, left, right } => write!(f, "{score:?} {left:?}..{right:?}"),
		}
	}
}

#[derive(Clone)]
pub enum IfScoreRangeEnd {
	Infinite,
	Fixed { value: ScoreValue, inclusive: bool },
}

impl IfScoreRangeEnd {
	pub fn get_used_regs(&self) -> Vec<&Identifier> {
		match self {
			Self::Infinite => Vec::new(),
			Self::Fixed { value, .. } => value.get_used_regs(),
		}
	}
}

impl Debug for IfScoreRangeEnd {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		if let Self::Fixed { value, inclusive } = self {
			if *inclusive {
				write!(f, "=")?;
			}
			write!(f, "{value:?}")?;
		}
		Ok(())
	}
}
