use intset::GrowSet;
use rustc_hash::FxHashMap;
use tinyvec::TinyVec;

use crate::common::block::Block;
use crate::common::reg::GetUsedRegs;
use crate::common::ty::{DataTypeContents, ScoreTypeContents};
use crate::common::{val::MutableValue, val::Value, Identifier};
use crate::mir::{MIRBlock, MIRInstrKind};
use crate::passes::{MIRPass, MIRPassData, Pass};
use crate::util::remove_indices;

pub struct MultifoldCombinePass;

impl Pass for MultifoldCombinePass {
	fn get_name(&self) -> &'static str {
		"multifold_combine"
	}
}

impl MIRPass for MultifoldCombinePass {
	fn run_pass(&mut self, data: &mut MIRPassData) -> anyhow::Result<()> {
		for func in data.mir.functions.values_mut() {
			let block = &mut func.block;

			let mut removed_indices = block.get_index_set();
			loop {
				let run_again = run_iter(block, &mut removed_indices);
				if !run_again {
					break;
				}
			}
			remove_indices(&mut block.contents, &removed_indices);
		}

		Ok(())
	}
}

fn run_iter(block: &mut MIRBlock, removed_indices: &mut GrowSet) -> bool {
	let mut run_again = false;
	let mut add_subs = FxHashMap::<Identifier, AddSubCombiner>::default();
	let mut muls = FxHashMap::<Identifier, MulCombiner>::default();
	let mut mods = FxHashMap::<Identifier, ModCombiner>::default();
	let mut pows = FxHashMap::<Identifier, PowCombiner>::default();
	let mut nots = FxHashMap::<Identifier, NotCombiner>::default();

	for (i, instr) in block.contents.iter().enumerate() {
		// Even though this instruction hasn't actually been removed from the vec, we treat it
		// as if it has to prevent doing the same work over and over and actually iterating indefinitely
		if removed_indices.contains(i) {
			continue;
		}
		match &instr.kind {
			MIRInstrKind::Add {
				left: MutableValue::Reg(reg),
				right: Value::Constant(DataTypeContents::Score(score)),
			} => {
				if let Some(combiner) = add_subs.get_mut(reg) {
					combiner.feed(i, score.get_i32());
				} else {
					add_subs.insert(reg.clone(), AddSubCombiner::new(score.get_i32(), i));
				}
			}
			MIRInstrKind::Sub {
				left: MutableValue::Reg(reg),
				right: Value::Constant(DataTypeContents::Score(score)),
			} => {
				if let Some(combiner) = add_subs.get_mut(reg) {
					combiner.feed(i, -score.get_i32());
				} else {
					add_subs.insert(reg.clone(), AddSubCombiner::new(-score.get_i32(), i));
				}
			}
			MIRInstrKind::Mul {
				left: MutableValue::Reg(reg),
				right: Value::Constant(DataTypeContents::Score(score)),
			} => {
				if let Some(combiner) = muls.get_mut(reg) {
					combiner.feed(i, score.get_i32());
				} else {
					muls.insert(reg.clone(), MulCombiner::new(score.get_i32(), i));
				}
			}
			MIRInstrKind::Mod {
				left: MutableValue::Reg(reg),
				right: Value::Constant(DataTypeContents::Score(score)),
			} => {
				if let Some(combiner) = mods.get_mut(reg) {
					combiner.feed(i, score.get_i32());
				} else {
					mods.insert(reg.clone(), ModCombiner::new(score.get_i32(), i));
				}
			}
			MIRInstrKind::Not {
				value: MutableValue::Reg(reg),
			} => {
				if let Some(combiner) = nots.get_mut(reg) {
					combiner.feed(i, 0);
				} else {
					nots.insert(reg.clone(), NotCombiner::new(1, i));
				}
			}
			MIRInstrKind::Pow {
				base: MutableValue::Reg(reg),
				exp,
			} => {
				if let Some(combiner) = pows.get_mut(reg) {
					combiner.feed(i, *exp as i32);
				} else {
					pows.insert(reg.clone(), PowCombiner::new(*exp as i32, i));
				}
			}
			other => {
				let used_regs = other.get_used_regs();
				for reg in used_regs {
					// Mark any combiners that are combining this register as finished
					add_subs.get_mut(reg).map(|x| x.finished = true);
					muls.get_mut(reg).map(|x| x.finished = true);
					mods.get_mut(reg).map(|x| x.finished = true);
					nots.get_mut(reg).map(|x| x.finished = true);
					pows.get_mut(reg).map(|x| x.finished = true);
				}
			}
		}
	}

	if !add_subs.is_empty()
		|| !muls.is_empty()
		|| !mods.is_empty()
		|| !pows.is_empty()
		|| !nots.is_empty()
	{
		let mut positions_to_remove = Vec::new();
		for (reg, combiner) in add_subs {
			if let Some((pos, to_remove, instr)) = combiner.finish(reg) {
				block
					.contents
					.get_mut(pos)
					.expect("Instr at pos does not exist")
					.kind = instr;
				positions_to_remove.extend(to_remove);
			}
		}

		for (reg, combiner) in muls {
			if let Some((pos, to_remove, instr)) = combiner.finish(reg) {
				block
					.contents
					.get_mut(pos)
					.expect("Instr at pos does not exist")
					.kind = instr;
				positions_to_remove.extend(to_remove);
			}
		}

		for (reg, combiner) in mods {
			if let Some((pos, to_remove, instr)) = combiner.finish(reg) {
				block
					.contents
					.get_mut(pos)
					.expect("Instr at pos does not exist")
					.kind = instr;
				positions_to_remove.extend(to_remove);
			}
		}

		for (reg, combiner) in pows {
			if let Some((pos, to_remove, instr)) = combiner.finish(reg) {
				block
					.contents
					.get_mut(pos)
					.expect("Instr at pos does not exist")
					.kind = instr;
				positions_to_remove.extend(to_remove);
			}
		}

		for (reg, combiner) in nots {
			if let Some((pos, to_remove, instr)) = combiner.finish(reg) {
				block
					.contents
					.get_mut(pos)
					.expect("Instr at pos does not exist")
					.kind = instr;
				positions_to_remove.extend(to_remove);
			}
		}

		if !positions_to_remove.is_empty() {
			run_again = true;
		}

		for pos in positions_to_remove {
			removed_indices.add(pos);
		}
	}

	run_again
}

macro_rules! combiner {
	($name:ident, $self:ident, $pos:ident, $amt:ident, $feed:tt, $instr:ident) => {
		combiner!($name, $self, $pos, $amt, $feed, reg, (|| {
			if $self.to_remove.is_empty() {
				None
			} else {
				Some((
					$self.pos,
					$self.to_remove,
					MIRInstrKind::$instr {
						left: MutableValue::Reg(reg),
						right: Value::Constant(DataTypeContents::Score(
							ScoreTypeContents::Score($self.val),
						)),
					},
				))
			}
		}));
	};

	($name:ident, $self:ident, $pos:ident, $amt:ident, $feed:tt, $reg:ident, $instr:tt) => {
		#[derive(Debug)]
		struct $name {
			val: i32,
			pos: usize,
			to_remove: TinyVec<[usize; 5]>,
			finished: bool,
		}

		impl $name {
			fn new(start_amt: i32, pos: usize) -> Self {
				Self {
					val: start_amt,
					pos,
					to_remove: TinyVec::new(),
					finished: false,
				}
			}

			fn feed(&mut $self, $pos: usize, $amt: i32) {
				if $self.finished {
					return;
				}
				$feed()
			}

			fn finish($self, $reg: Identifier) -> Option<(usize, TinyVec<[usize; 5]>, MIRInstrKind)> {
				$instr()
			}
		}
	};
}

combiner!(
	AddSubCombiner,
	self,
	pos,
	amt,
	(|| {
		// We can in fact overflow this because it will wrap around to negative.
		// This ends up having the same behavior when it is added to the register
		self.val = self.val.wrapping_add(amt);
		self.to_remove.push(pos);
	}),
	Add
);

combiner!(
	MulCombiner,
	self,
	pos,
	amt,
	(|| {
		if let Some(total) = self.val.checked_mul(amt) {
			self.val = total;
			self.to_remove.push(pos);
		}
	}),
	Mul
);

combiner!(
	ModCombiner,
	self,
	pos,
	amt,
	(|| {
		if amt > self.val {
			self.val = amt;
		}
		self.to_remove.push(pos);
	}),
	Mod
);

combiner!(
	PowCombiner,
	self,
	pos,
	amt,
	(|| {
		if let Some(total) = self.val.checked_mul(amt) {
			self.val = total;
			self.to_remove.push(pos);
		}
	}),
	reg,
	(|| {
		if self.to_remove.is_empty() {
			None
		} else {
			Some((
				self.pos,
				self.to_remove,
				MIRInstrKind::Pow {
					base: MutableValue::Reg(reg),
					exp: (self.val as u8),
				},
			))
		}
	})
);

combiner!(
	NotCombiner,
	self,
	pos,
	amt,
	(|| {
		let _ = amt;
		self.val += 1;
		self.to_remove.push(pos);
	}),
	reg,
	(|| {
		if self.val % 2 == 0 {
			Some((self.pos, self.to_remove, MIRInstrKind::NoOp))
		} else {
			Some((
				self.pos,
				self.to_remove,
				MIRInstrKind::Not {
					value: MutableValue::Reg(reg),
				},
			))
		}
	})
);
