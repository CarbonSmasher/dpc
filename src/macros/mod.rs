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
