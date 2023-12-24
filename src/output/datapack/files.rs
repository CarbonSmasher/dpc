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
	for (id, tag) in pack.function_tags {
		let path = data_path
			.join(get_func_tag_path(&id).context(format!("Failed to get function tag path {id}"))?);
		if let Some(parent) = path.parent() {
			let _ = std::fs::create_dir_all(parent);
		}
		let contents = serde_json::to_string(&tag.inner)
			.context("Failed to serialize function tag contents")?;
		std::fs::write(path, contents)
			.context(format!("Failed to write function tag file {id}"))?;
	}

	Ok(())
}

/// Gets the relative path of a function
pub fn get_func_path(loc: &str) -> anyhow::Result<String> {
	get_resource_path(loc, "functions", "mcfunction")
}

/// Gets the relative path of a function tag
pub fn get_func_tag_path(loc: &str) -> anyhow::Result<String> {
	get_resource_path(loc, "tags/functions", "json")
}

/// Gets the relative path of a file from a resource
pub fn get_resource_path(loc: &str, ty: &str, extension: &str) -> anyhow::Result<String> {
	let (l, r) = loc.split_at(loc.find(':').context("No colon in resource location")?);
	let r = &r[1..];
	Ok(format!("{l}/{ty}/{r}.{extension}"))
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
