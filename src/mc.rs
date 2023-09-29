use crate::common::Identifier;

#[derive(Debug, Clone)]
pub enum TargetSelector {
	Player(String),
}

impl TargetSelector {
	pub fn codegen_str(&self) -> String {
		match self {
			Self::Player(player) => player.clone(),
		}
	}
}

#[derive(Debug, Clone)]
pub struct Score {
	pub holder: TargetSelector,
	pub score: Identifier,
}

impl Score {
	pub fn new(holder: TargetSelector, score: Identifier) -> Self {
		Self { holder, score }
	}
}

/// Value that a score can be operated on with
#[derive(Debug, Clone)]
pub enum ScoreOperand {
	Constant(i32),
	Score(Score),
}
