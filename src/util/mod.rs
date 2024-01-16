use std::collections::HashSet;
use std::hash::Hash;
use std::ops::{Deref, DerefMut};

use intset::GrowSet;
use rustc_hash::FxHashSet;

/// A container that can hold values
pub trait Container<T> {
	fn contains_val(&self, val: &T) -> bool;

	fn empty(&self) -> bool;
}

impl<T> Container<T> for HashSet<T>
where
	T: Hash + Eq,
{
	fn contains_val(&self, val: &T) -> bool {
		self.contains(val)
	}

	fn empty(&self) -> bool {
		self.is_empty()
	}
}

impl<T> Container<T> for [T]
where
	T: PartialEq,
{
	fn contains_val(&self, val: &T) -> bool {
		self.contains(val)
	}

	fn empty(&self) -> bool {
		self.is_empty()
	}
}

impl<T> Container<T> for Vec<T>
where
	T: PartialEq,
{
	fn contains_val(&self, val: &T) -> bool {
		self.contains(val)
	}

	fn empty(&self) -> bool {
		self.is_empty()
	}
}

impl Container<usize> for GrowSet {
	fn contains_val(&self, val: &usize) -> bool {
		self.contains(*val)
	}

	fn empty(&self) -> bool {
		self.len() == 0
	}
}

/// `https://users.rust-lang.org/t/removing-multiple-indices-from-a-vector/65599/4`
#[allow(dead_code)]
pub fn remove_indices<T>(v: &mut Vec<T>, indices: &impl Container<usize>) {
	if indices.empty() {
		return;
	}

	let mut i = 0;
	v.retain(|_| {
		let keep = !indices.contains_val(&i);
		i += 1;
		keep
	});
}

#[allow(dead_code)]
pub fn insert_indices<T: Clone>(v: Vec<T>, values: &[(usize, T)]) -> Vec<T> {
	if values.is_empty() {
		return v;
	}

	let mut out = Vec::with_capacity(v.len() + values.len());
	for i in 0..=v.len() {
		if let Some(insert) = values.iter().find(|x| x.0 == i) {
			out.push(insert.1.clone());
		}
		out.extend(v.get(i).cloned());
	}

	out
}

#[allow(dead_code)]
pub fn replace_and_expand_indices<T: Clone>(v: Vec<T>, values: &[(usize, Vec<T>)]) -> Vec<T> {
	if values.is_empty() {
		return v;
	}

	let mut out = Vec::with_capacity(v.len() + values.len());
	for i in 0..=v.len() {
		if let Some(insert) = values.iter().find(|x| x.0 == i) {
			out.extend(insert.1.clone());
		} else {
			out.extend(v.get(i).cloned());
		}
	}

	out
}

/// Wrapper around a DashSet that keeps track of whether any elements were inserted or not.
/// This allows an efficient empty() implementation
pub struct HashSetEmptyTracker<T> {
	inner: FxHashSet<T>,
	is_empty: bool,
}

impl<T> HashSetEmptyTracker<T>
where
	T: Eq + Hash,
{
	pub fn new() -> Self {
		Self {
			inner: FxHashSet::default(),
			is_empty: true,
		}
	}

	/// Override of insert that marks the container as not empty
	#[allow(dead_code)]
	pub fn insert(&mut self, val: T) -> bool {
		self.is_empty = false;
		self.inner.insert(val)
	}
}

impl<T> Deref for HashSetEmptyTracker<T> {
	type Target = FxHashSet<T>;

	fn deref(&self) -> &Self::Target {
		&self.inner
	}
}

impl<T> DerefMut for HashSetEmptyTracker<T> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		// Just to make sure
		self.is_empty = false;
		&mut self.inner
	}
}

impl<T> Container<T> for HashSetEmptyTracker<T>
where
	T: Eq + Hash,
{
	fn contains_val(&self, val: &T) -> bool {
		self.inner.contains(val)
	}

	fn empty(&self) -> bool {
		self.is_empty
	}
}

/// Float with Eq
#[derive(PartialEq, Clone)]
pub struct EqFloat(pub f32);

impl Eq for EqFloat {}

/// Trait for a container to get the first element only if the container
/// is exactly one element long
pub trait Only {
	type Item;

	fn only(&self) -> Option<&Self::Item>;
	fn only_mut(&mut self) -> Option<&mut Self::Item>;
}

impl<T> Only for Vec<T> {
	type Item = T;

	fn only(&self) -> Option<&Self::Item> {
		if self.len() == 1 {
			unsafe { Some(self.first().unwrap_unchecked()) }
		} else {
			None
		}
	}

	fn only_mut(&mut self) -> Option<&mut Self::Item> {
		if self.len() == 1 {
			unsafe { Some(self.first_mut().unwrap_unchecked()) }
		} else {
			None
		}
	}
}

/// Utility trait for getting a set of something in a custom type.
/// Uses overrides of the append method so that a new vec doesnt have to be
/// allocated for every subitem
pub trait GetSetOwned<T> {
	fn append_set(&self, set: &mut FxHashSet<T>);

	fn get_set(&self) -> FxHashSet<T> {
		let mut out = FxHashSet::default();
		self.append_set(&mut out);
		out
	}
}
