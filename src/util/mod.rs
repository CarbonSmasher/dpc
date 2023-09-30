/// `https://users.rust-lang.org/t/removing-multiple-indices-from-a-vector/65599/4`
pub fn remove_indices<T>(v: &mut Vec<T>, indices: &[usize]) {
	let mut i = 0;
	v.retain(|_| {
		let keep = !indices.contains(&i);
		i += 1;
		keep
	});
}
