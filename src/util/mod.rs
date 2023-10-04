use std::hash::Hash;

use dashmap::DashSet;

/// A container that can hold values
pub trait Container<T> {
	fn contains_val(&self, val: &T) -> bool;
}

impl<T> Container<T> for DashSet<T>
where
	T: Hash + Eq,
{
	fn contains_val(&self, val: &T) -> bool {
		self.contains(val)
	}
}

impl<T> Container<T> for [T]
where
	T: PartialEq,
{
	fn contains_val(&self, val: &T) -> bool {
		self.contains(val)
	}
}

impl<T> Container<T> for Vec<T>
where
	T: PartialEq,
{
	fn contains_val(&self, val: &T) -> bool {
		self.contains(val)
	}
}

/// `https://users.rust-lang.org/t/removing-multiple-indices-from-a-vector/65599/4`
#[allow(dead_code)]
pub fn remove_indices<T>(v: &mut Vec<T>, indices: &impl Container<usize>) {
	let mut i = 0;
	v.retain(|_| {
		let keep = !indices.contains_val(&i);
		i += 1;
		keep
	});
}

#[allow(dead_code)]
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
