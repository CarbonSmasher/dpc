pub const REG_OBJECTIVE: &str = "_dpc_reg";
pub const LIT_OBJECTIVE: &str = "_dpc_lit";

pub fn format_reg_fake_player(num: u32) -> String {
	format!("$r{num}")
}

pub fn format_lit_fake_player(num: i32) -> String {
	format!("$l{num}")
}
