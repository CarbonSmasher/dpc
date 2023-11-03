#[derive(Clone, Debug)]
pub enum Token {
	Whitespace,
}

pub struct TokAndPos {
	pub tok: Token,
	pub pos: TextPos,
}

impl TokAndPos {
	fn new(tok: Token, pos: TextPos) -> Self {
		Self { tok, pos }
	}
}

pub struct TextPos {
	pub abs: usize,
}

impl TextPos {
	fn new(abs: usize) -> Self {
		Self { abs }
	}
}

pub type TokenList = Vec<TokAndPos>;

pub fn lex(text: &str) -> anyhow::Result<TokenList> {
	let mut out = Vec::new();
	let mut state = LexState::Root;

	for (i, c) in text.chars().enumerate() {
		let pos = TextPos::new(i);

		match state {
			LexState::Root => {
				if c.is_whitespace() {
					out.push(TokAndPos::new(Token::Whitespace, pos));
					continue;
				}
			}
		}
	}

	Ok(out)
}

enum LexState {
	Root,
}
