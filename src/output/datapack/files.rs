use std::path::Path;

use anyhow::Context;

use super::Datapack;

/// Outputs a datapack to a folder. Will remove all existing files in /data directory of that folder.
pub fn output_pack(pack: Datapack, path: &Path) -> anyhow::Result<()> {
	let data_path = path.join("data");
	// Clear the old data
	if data_path.exists() {
		std::fs::remove_dir_all(&data_path).context("Failed to remove data directory")?;
	}
	std::fs::create_dir_all(&data_path).context("Failed to recreate data directory")?;
	for (id, function) in pack.functions {
		let path = data_path
			.join(get_func_path(&id).context(format!("Failed to get function path {id}"))?);
		if let Some(parent) = path.parent() {
			let _ = std::fs::create_dir_all(parent);
		}
		std::fs::write(path, function.contents.join("\n"))
			.context(format!("Failed to write function file {id}"))?;
	}

	Ok(())
}

/// Gets the relative path of a function
pub fn get_func_path(loc: &str) -> anyhow::Result<String> {
	let (l, r) = loc.split_at(loc.find(':').context("No colon in resource location")?);
	let r = &r[1..];
	Ok(format!("{l}/functions/{r}.mcfunction"))
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_func_path() {
		assert_eq!(
			get_func_path("game:gen/main").unwrap(),
			String::from("game/functions/gen/main.mcfunction")
		);
	}
}
