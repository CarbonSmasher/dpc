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
	format!("%R{func_id}{num}")
}

pub fn format_local_storage_entry(num: u32, func_id: &str) -> String {
	format!("r{func_id}{num}")
}

pub fn format_arg_local_storage_entry(num: u16, func_id: &str) -> String {
	format!("a{func_id}{num}")
}

pub fn format_ret_local_storage_entry(num: u16, func_id: &str) -> String {
	format!("R{func_id}{num}")
}

pub fn format_local_storage_path(entry: &str) -> String {
	entry.to_string()
}

/// Used for getting stripped counters encoded in a base of characters,
/// with different character sets
pub fn get_stripped_name_unstable(idx: u32, charset: &[char]) -> String {
	if idx == 0 {
		return String::new();
	}
	let mut idx = idx as usize;

	let mut out = String::new();
	let mut is_first = true;
	// Add one and then subtract it on the first iteration
	// to bypass the while check
	// TODO: Make this actually good
	idx += 1;
	while idx != 0 {
		if is_first {
			idx -= 1;
		}
		let digit = idx % charset.len();
		out.push(charset[digit]);
		idx /= charset.len();
		is_first = false;
	}

	out
}

pub static RESOURCE_LOCATION_CHARSET: [char; 39] = [
	'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's',
	't', 'u', 'v', 'w', 'x', 'y', 'z', '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', '_', '-',
	'.',
];

pub static FAKE_PLAYER_CHARSET: [char; 37] = [
	'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's',
	't', 'u', 'v', 'w', 'x', 'y', 'z', '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', '_',
];

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_unstable_stripping() {
		assert_eq!(
			get_stripped_name_unstable(0, &RESOURCE_LOCATION_CHARSET),
			String::from("")
		);
		assert_eq!(
			get_stripped_name_unstable(1, &RESOURCE_LOCATION_CHARSET),
			String::from("b")
		);
		assert_eq!(
			get_stripped_name_unstable(38, &RESOURCE_LOCATION_CHARSET),
			String::from(".")
		);
		assert_eq!(
			get_stripped_name_unstable(39, &RESOURCE_LOCATION_CHARSET),
			String::from("ab")
		);
	}
}
