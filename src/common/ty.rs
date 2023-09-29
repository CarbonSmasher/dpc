use super::{MutableValue, RegisterList, Value};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataType {
	Score(ScoreType),
}

impl DataType {
	pub fn is_trivially_castable(&self, other: &DataType) -> bool {
		match other {
			DataType::Score(other_score) => match self {
				Self::Score(this_score) => this_score.is_trivially_castable(other_score),
			},
		}
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScoreType {
	Score,
	UScore,
	Bool,
}

impl ScoreType {
	pub fn is_trivially_castable(&self, other: &ScoreType) -> bool {
		match other {
			ScoreType::Score => {
				matches!(self, ScoreType::Score | ScoreType::UScore | ScoreType::Bool)
			}
			ScoreType::UScore => matches!(self, ScoreType::Score | ScoreType::UScore),
			ScoreType::Bool => matches!(self, ScoreType::Bool),
		}
	}
}

#[derive(Debug, Clone)]
pub enum DataTypeContents {
	Score(ScoreTypeContents),
}

impl DataTypeContents {
	pub fn get_ty(&self) -> DataType {
		match self {
			Self::Score(score) => score.get_ty(),
		}
	}
}

#[derive(Debug, Clone)]
pub enum ScoreTypeContents {
	Score(i32),
	UScore(u16),
	Bool(bool),
}

impl ScoreTypeContents {
	pub fn get_ty(&self) -> DataType {
		match self {
			Self::Score(..) => DataType::Score(ScoreType::Score),
			Self::UScore(..) => DataType::Score(ScoreType::UScore),
			Self::Bool(..) => DataType::Score(ScoreType::Bool),
		}
	}

	pub fn get_i32(&self) -> i32 {
		match self {
			ScoreTypeContents::Score(score) => *score,
			ScoreTypeContents::UScore(score) => *score as i32,
			ScoreTypeContents::Bool(score) => *score as i32,
		}
	}

	pub fn get_literal_str(&self) -> String {
		self.get_i32().to_string()
	}
}

pub fn get_op_tys(
	left: &MutableValue,
	right: &Value,
	regs: &RegisterList,
) -> anyhow::Result<(DataType, DataType)> {
	Ok((left.get_ty(regs)?, right.get_ty(regs)?))
}
