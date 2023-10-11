use std::fmt::Debug;

#[derive(Clone)]
pub struct TargetSelector {
	pub selector: SelectorType,
	pub params: Vec<SelectorParameter>,
}

impl TargetSelector {
	pub fn new(selector: SelectorType) -> Self {
		Self::with_params(selector, Vec::new())
	}

	pub fn with_params(selector: SelectorType, params: Vec<SelectorParameter>) -> Self {
		Self { selector, params }
	}

	pub fn is_blank_this(&self) -> bool {
		matches!(self.selector, SelectorType::This) && self.params.is_empty()
	}
}

impl Debug for TargetSelector {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.selector.codegen_str())?;
		if !self.params.is_empty() {
			write!(f, "[")?;
			for (i, param) in self.params.iter().enumerate() {
				write!(f, "{param:?}")?;
				if i != self.params.len() - 1 {
					write!(f, ",")?;
				}
			}
			write!(f, "]")?;
		}
		Ok(())
	}
}

#[derive(Debug, Clone)]
pub enum SelectorType {
	This,
	NearestPlayer,
	RandomPlayer,
	AllPlayers,
	AllEntities,
}

impl SelectorType {
	pub fn codegen_str(&self) -> &'static str {
		match self {
			Self::This => "@s",
			Self::NearestPlayer => "@p",
			Self::RandomPlayer => "@r",
			Self::AllPlayers => "@a",
			Self::AllEntities => "@e",
		}
	}
}

#[derive(Debug, Clone)]
pub enum SelectorParameter {
	Type { ty: String, invert: bool },
	Tag { tag: String, invert: bool },
	NoTags,
	Predicate { predicate: String, invert: bool },
}
