use super::mc::{DoubleCoordinates, EntityTarget, Score};

use super::{Identifier, MutableScoreValue, ScoreValue};

/// A modifier to the context of a command
#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
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
