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
	pub objective: Identifier,
}

impl Score {
	pub fn new(holder: TargetSelector, objective: Identifier) -> Self {
		Self { holder, objective }
	}

	pub fn codegen_str(&self) -> String {
		format!("{} {}", self.holder.codegen_str(), self.objective)
	}
}

#[derive(Debug, Clone)]
pub enum DataLocation {
	Entity(String),
	Storage(String),
}

#[derive(Debug, Clone)]
pub enum Coordinates<T> {
	XYZ(AbsOrRelCoord<T>, AbsOrRelCoord<T>, AbsOrRelCoord<T>),
	Local(T, T, T),
}

impl<T: ToString> Coordinates<T> {
	pub fn codegen_str(&self) -> String {
		match self {
			Self::XYZ(x, y, z) => format!(
				"{} {} {}",
				x.codegen_str(),
				y.codegen_str(),
				z.codegen_str()
			),
			Self::Local(a, b, c) => {
				format!("^{} ^{} ^{}", a.to_string(), b.to_string(), c.to_string())
			}
		}
	}
}

pub type DoubleCoordinates = Coordinates<f64>;
pub type IntCoordinates = Coordinates<i64>;

#[derive(Debug, Clone)]
pub enum AbsOrRelCoord<T> {
	Abs(T),
	Rel(T),
}

impl<T: ToString> AbsOrRelCoord<T> {
	pub fn codegen_str(&self) -> String {
		match self {
			Self::Abs(val) => val.to_string(),
			Self::Rel(val) => format!("~{}", val.to_string()),
		}
	}
}

#[derive(Debug, Clone)]
pub enum Axis {
	X,
	Y,
	Z,
}

impl Axis {
	pub fn codegen_str(&self) -> &'static str {
		match self {
			Self::X => "x",
			Self::Y => "y",
			Self::Z => "z",
		}
	}
}
