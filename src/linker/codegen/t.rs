use std::sync::Arc;

use super::CodegenBlockCx;

pub trait Codegen {
	fn gen_writer<F>(&self, f: &mut F, cbcx: &mut CodegenBlockCx) -> anyhow::Result<()>
	where
		F: std::fmt::Write;

	fn gen_str(&self, cbcx: &mut CodegenBlockCx) -> anyhow::Result<String> {
		let mut out = String::new();
		self.gen_writer(&mut out, cbcx)?;
		Ok(out)
	}
}

macro_rules! cg_impl {
	($name:ty) => {
		impl Codegen for $name {
			fn gen_writer<F>(&self, f: &mut F, cbcx: &mut CodegenBlockCx) -> anyhow::Result<()>
			where
				F: std::fmt::Write,
			{
				let _ = cbcx;
				write!(f, "{self}")?;
				Ok(())
			}
		}
	};
}

cg_impl!(&str);
cg_impl!(String);
cg_impl!(i32);
cg_impl!(Arc<str>);

pub mod macros {
	macro_rules! cgwrite {
		($f:expr, $cbcx:expr, $($t:expr),*) => {{
			$(
				$t.gen_writer($f, $cbcx)?;
			)*
			Ok::<(), anyhow::Error>(())
		}};
	}

	#[allow(unused_macros)]
	macro_rules! cgformat {
		($cbcx:expr, $($t:expr),*) => {{
			let mut out = String::new();
			$crate::linker::codegen::t::macros::cgwrite!(&mut out, $cbcx, $($t),*)?;
			Ok::<String, anyhow::Error>(out)
		}};
	}

	#[allow(unused_imports)]
	pub(crate) use cgformat;
	#[allow(unused_imports)]
	pub(crate) use cgwrite;
}
