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

impl SingleCriterion {
	pub fn parse(string: &str) -> Option<Self> {
		match string {
			"air" => Some(Self::Air),
			"armor" => Some(Self::Armor),
			"death_count" => Some(Self::DeathCount),
			"dummy" => Some(Self::Dummy),
			"food" => Some(Self::Food),
			"health" => Some(Self::Health),
			"level" => Some(Self::Level),
			"player_kill_count" => Some(Self::PlayerKillCount),
			"total_kill_count" => Some(Self::TotalKillCount),
			"trigger" => Some(Self::Trigger),
			"xp" => Some(Self::XP),
			_ => None,
		}
	}
}
