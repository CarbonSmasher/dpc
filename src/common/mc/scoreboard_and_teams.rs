use std::fmt::Debug;

/// Criterion for a scoreboard objective
#[derive(Clone, PartialEq)]
pub enum Criterion {
	Single(SingleCriterion),
	Compound(String),
}

impl Debug for Criterion {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Single(val) => val.fmt(f),
			Self::Compound(val) => write!(f, "{val}"),
		}
	}
}

#[derive(Clone, PartialEq)]
pub enum SingleCriterion {
	Dummy,
	Trigger,
	DeathCount,
	PlayerKillCount,
	TotalKillCount,
	Health,
	XP,
	Level,
	Food,
	Air,
	Armor,
}

impl Debug for SingleCriterion {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(
			f,
			"{}",
			match self {
				Self::Air => "air",
				Self::Armor => "armor",
				Self::DeathCount => "deathCount",
				Self::Dummy => "dummy",
				Self::Food => "food",
				Self::Health => "health",
				Self::Level => "level",
				Self::PlayerKillCount => "playerKillCount",
				Self::TotalKillCount => "totalKillCount",
				Self::Trigger => "trigger",
				Self::XP => "xp",
			}
		)
	}
}
