use crate::mc::{Score, TargetSelector};

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
	As(TargetSelector),
	At(TargetSelector),
	In(String),
	On(EntityRelation),
}

#[derive(Debug, Clone)]
pub enum StoreModLocation {
	Reg(Identifier),
	Score(Score),
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
