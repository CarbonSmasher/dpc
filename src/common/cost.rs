use super::{
	condition::Condition,
	mc::{
		entity::{SelectorParameter, SelectorType, TargetSelector},
		modifier::{IfModCondition, Modifier},
		EntityTarget,
	},
	val::{MutableValue, Value},
};

/// Used to get the cost of some operation as a number for comparison with other costs
pub trait GetCost {
	/// Get the cost
	fn get_cost(&self) -> f32;
}

/// Used to get the cost of some operation relatively
pub trait GetRelativeCost: GetCost {
	const MAX: f32;

	/// Get the relative cost from 0-1 compared to the maximum
	/// cost of this item
	fn get_relative_cost(&self) -> f32 {
		let cost = self.get_cost() as f32;
		debug_assert!(cost < 1.0);
		cost / Self::MAX as f32
	}
}

impl GetCost for Condition {
	fn get_cost(&self) -> f32 {
		match self {
			Self::And(l, r) | Self::Or(l, r) => l.get_cost() + r.get_cost(),
			Self::Not(cond) => cond.get_cost(),
			Self::Entity(..) => 40.0,
			Self::Function(..) => 20.0,
			Self::Biome(..) | Self::Loaded(..) | Self::Dimension(..) => 18.0,
			Self::Predicate(..) => 12.0,
			Self::GreaterThan(l, r)
			| Self::GreaterThanOrEqual(l, r)
			| Self::LessThan(l, r)
			| Self::LessThanOrEqual(l, r) => (l.get_cost() + r.get_cost()) * 1.8,
			Self::Exists(val) => val.get_cost() * 1.8,
			Self::Equal(l, r) => (l.get_cost() + r.get_cost()) * 1.2,
			Self::Bool(val) | Self::NotBool(val) => val.get_cost() * 1.1,
		}
	}
}

impl GetCost for Value {
	fn get_cost(&self) -> f32 {
		match self {
			Self::Constant(..) => 0.1,
			Self::Mutable(val) => val.get_cost(),
		}
	}
}

impl GetCost for MutableValue {
	fn get_cost(&self) -> f32 {
		match self {
			Self::Data(..) => 4.0,
			Self::Score(..) => 1.1,
			Self::Reg(..)
			| Self::Arg(..)
			| Self::CallReturnValue(..)
			| Self::CallArg(..)
			| Self::ReturnValue(..) => 1.0,
			Self::Index(val, ..) => val.get_cost() + 0.25,
			Self::Property(val, ..) => val.get_cost() + 0.35,
		}
	}
}

impl GetCost for Modifier {
	fn get_cost(&self) -> f32 {
		match self {
			Self::If { condition, .. } => condition.get_cost(),
			Self::Summon(..) => 100.0,
			Self::In(..) => 80.0,
			Self::As(..) => 60.0,
			Self::At(..)
			| Self::PositionedAs(..)
			| Self::FacingEntity(..)
			| Self::RotatedAs(..)
			| Self::On(..) => 40.0,
			Self::Positioned(..)
			| Self::FacingPosition(..)
			| Self::Rotated(..)
			| Self::Align(..)
			| Self::Anchored(..)
			| Self::PositionedOver(..) => 20.0,
			Self::StoreResult(..) | Self::StoreSuccess(..) => 20.0,
		}
	}
}

impl GetCost for IfModCondition {
	fn get_cost(&self) -> f32 {
		match self {
			Self::Block(_, block) => {
				let mut cost = 32.0;
				if !block.props.states.is_empty() {
					cost += 5.0;
				}
				cost += block.props.states.len() as f32 * 0.25;
				if !block.props.data.is_empty() {
					cost += 30.0;
				}
				cost += block.props.data.0.len() as f32 * 0.25;
				cost
			}
			Self::Entity(..) => 50.0,
			Self::DataEquals(..) | Self::DataExists(..) => 40.0,
			Self::Biome(..) | Self::Loaded(..) | Self::Dimension(..) => 32.0,
			Self::Function(..) => 20.0,
			Self::Predicate(..) => 15.0,
			Self::Score(..) => 4.0,
			Self::Const(..) => 0.0,
		}
	}
}

impl GetCost for EntityTarget {
	fn get_cost(&self) -> f32 {
		match self {
			Self::Player(..) => 2.0,
			Self::Selector(sel) => sel.get_cost(),
		}
	}
}

impl GetCost for TargetSelector {
	fn get_cost(&self) -> f32 {
		let mut out = 5.0;
		out += self
			.params
			.iter()
			.fold(0.0, |accum, x| accum + x.get_cost());
		out += match self.selector {
			SelectorType::AllEntities => 20.0,
			SelectorType::AllPlayers => 10.0,
			SelectorType::NearestPlayer | SelectorType::RandomPlayer => 4.0,
			SelectorType::This => 1.0,
		};

		out
	}
}

impl GetCost for SelectorParameter {
	fn get_cost(&self) -> f32 {
		match self {
			Self::NBT { .. } => 20.0,
			Self::Distance { .. }
			| Self::Gamemode { .. }
			| Self::Name { .. }
			| Self::Tag { .. }
			| Self::Type { .. }
			| Self::Predicate { .. }
			| Self::NoTags => 7.0,
			Self::Sort(..) | Self::Limit(..) => 1.0,
		}
	}
}
