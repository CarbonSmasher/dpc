// Objectives
pub const REG_OBJECTIVE: &str = "_r";
pub const LIT_OBJECTIVE: &str = "_l";

// Storage
pub const REG_STORAGE_LOCATION: &str = "dpc:r";

pub fn format_reg_fake_player(num: u32, func_id: &str) -> String {
	format!("%r{func_id}{num}")
}

pub fn format_lit_fake_player(num: i32) -> String {
	format!("%l{num}")
}

pub fn format_arg_fake_player(num: u16, func_id: &str) -> String {
	format!("%a{func_id}{num}")
}

pub fn format_ret_fake_player(num: u16, func_id: &str) -> String {
	format!("%r{func_id}{num}")
}

pub fn format_local_storage_entry(num: u32, func_id: &str) -> String {
	format!("s{func_id}{num}")
}

pub fn format_arg_local_storage_entry(num: u16, func_id: &str) -> String {
	format!("a{func_id}{num}")
}

pub fn format_ret_local_storage_entry(num: u16, func_id: &str) -> String {
	format!("r{func_id}{num}")
}

pub fn format_local_storage_path(entry: &str) -> String {
	format!("{entry}")
}
