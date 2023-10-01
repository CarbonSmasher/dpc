/// `https://users.rust-lang.org/t/removing-multiple-indices-from-a-vector/65599/4`
pub fn remove_indices<T>(v: &mut Vec<T>, indices: &[usize]) {
	let mut i = 0;
	v.retain(|_| {
		let keep = !indices.contains(&i);
		i += 1;
		keep
	});
}

pub fn insert_indices<T: Clone>(v: Vec<T>, values: &[(usize, T)]) -> Vec<T> {
	let mut out = Vec::new();
	for i in 0..=v.len() {
		if let Some(insert) = values.iter().find(|x| x.0 == i) {
			out.push(insert.1.clone());
		}
		out.extend(v.get(i).cloned());
	}

	out
}
