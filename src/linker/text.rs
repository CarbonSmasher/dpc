// Objectives
pub const REG_OBJECTIVE: &str = "_dpc.r";
pub const LIT_OBJECTIVE: &str = "_dpc.l";

// Storage
pub const REG_STORAGE_LOCATION: &str = "dpc:r";

pub fn format_reg_fake_player(num: u32) -> String {
	format!("%r{num}")
}

pub fn format_lit_fake_player(num: i32) -> String {
	format!("%l{num}")
}

pub fn format_local_storage_entry(num: u32) -> String {
	format!("s{num}")
}

pub fn format_local_storage_path(entry: &str) -> String {
	format!("loc.{entry}")
}
