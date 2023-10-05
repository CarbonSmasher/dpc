#[derive(Debug, Clone)]
pub struct TargetSelector {
	pub selector: SelectorType,
	pub params: Vec<SelectorParameter>,
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
