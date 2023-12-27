use crate::common::Identifier;

pub mod ir_to_mir;
pub mod mir_to_lir;

pub fn fmt_lowered_arg(func_id: &str, arg_num: u16) -> Identifier {
	let reg = format!("in_arg_{func_id}_{arg_num}");
	reg.into()
}

pub fn cleanup_fn_id(func_id: &str) -> String {
	func_id.to_string().replace([':', '/'], "_")
}
