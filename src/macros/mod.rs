#[macro_export]
macro_rules! def_compound {
	($($id:ident: $item:expr),*$(,)?) => {{
		let mut out = std::collections::HashMap::new();
		$(
			out.insert(stringify!($id).to_string(), $item);
		)*
		std::sync::Arc::new(out)
	}};
}

#[macro_export]
macro_rules! push_instrs {
	($block:expr, $($instr:expr);* $(;)?) => {
		$(
			$block.contents.push(Instruction::new($instr));
		)*
	};
}