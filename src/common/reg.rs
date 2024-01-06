use super::Identifier;

pub trait GetUsedRegs {
	fn append_used_regs<'this>(&'this self, regs: &mut Vec<&'this Identifier>);

	fn get_used_regs(&self) -> Vec<&Identifier> {
		let mut out = Vec::new();
		self.append_used_regs(&mut out);
		out
	}
}
