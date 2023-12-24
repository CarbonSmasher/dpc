pub mod cond;
pub mod fold;
pub mod prop;

use anyhow::Context;
use dashmap::DashMap;

use crate::common::ty::DataTypeContents;
use crate::common::{val::MutableValue, val::Value, DeclareBinding, Identifier};
use crate::mir::MIRInstrKind;
use crate::passes::{MIRPass, MIRPassData, Pass};

use self::cond::ConstConditionPass;
use self::fold::ConstFoldPass;
use self::prop::ConstPropPass;

/// Combines the ConstProp and ConstFold passes and runs them both
/// until no changes are made
pub struct ConstComboPass;

impl Pass for ConstComboPass {
	fn get_name(&self) -> &'static str {
		"const_combo"
	}
}

impl MIRPass for ConstComboPass {
	fn run_pass(&mut self, data: &mut MIRPassData) -> anyhow::Result<()> {
		// dbg!(&data.mir);
		loop {
			let mut prop = ConstPropPass::new();
			prop.run_pass(data).context("Const Prop pass failed")?;
			let mut fold = ConstFoldPass::new();
			fold.run_pass(data).context("Const Fold pass failed")?;
			let mut cond = ConstConditionPass::new();
			cond.run_pass(data).context("Const Condition pass failed")?;
			if !prop.made_changes() && !fold.made_changes && !cond.made_changes {
				break;
			}
		}

		Ok(())
	}
}

struct ConstAnalyzer<'cont> {
	vals: DashMap<Identifier, &'cont DataTypeContents>,
	store_self: bool,
}

impl<'cont> ConstAnalyzer<'cont> {
	fn new() -> Self {
		Self {
			vals: DashMap::new(),
			store_self: true,
		}
	}

	fn new_dont_store() -> Self {
		Self {
			vals: DashMap::with_capacity(0),
			store_self: false,
		}
	}

	fn feed(&mut self, kind: &'cont MIRInstrKind) -> ConstAnalyzerResult {
		match kind {
			MIRInstrKind::Assign {
				left: MutableValue::Register(reg),
				right: DeclareBinding::Value(Value::Constant(val)),
			} => {
				if self.store_self {
					self.vals.insert(reg.clone(), val);
				}
				ConstAnalyzerResult::Add(reg.clone(), val)
			}
			MIRInstrKind::Assign { left, right, .. } => {
				let mut out = Vec::new();
				if let MutableValue::Register(reg) = left {
					out.push(reg.clone());
				}
				if let DeclareBinding::Value(Value::Mutable(MutableValue::Register(reg))) = &right {
					out.push(reg.clone());
				} else {
					let used = right.get_used_regs();
					out.extend(used.into_iter().cloned())
				}
				if self.store_self {
					for reg in &out {
						self.vals.remove(reg);
					}
				}
				ConstAnalyzerResult::Remove(out)
			}
			MIRInstrKind::Add { left, .. }
			| MIRInstrKind::Sub { left, .. }
			| MIRInstrKind::Mul { left, .. }
			| MIRInstrKind::Div { left, .. }
			| MIRInstrKind::Mod { left, .. }
			| MIRInstrKind::Min { left, .. }
			| MIRInstrKind::Max { left, .. }
			| MIRInstrKind::Pow { base: left, .. }
			| MIRInstrKind::Merge { left, .. }
			| MIRInstrKind::Push { left, .. }
			| MIRInstrKind::PushFront { left, .. }
			| MIRInstrKind::Insert { left, .. } => {
				if let MutableValue::Register(reg) = left {
					if self.store_self {
						self.vals.remove(reg);
					}
					ConstAnalyzerResult::Remove(vec![reg.clone()])
				} else {
					ConstAnalyzerResult::Other
				}
			}
			MIRInstrKind::Swap {
				left: MutableValue::Register(left_reg),
				right: MutableValue::Register(right_reg),
			} => {
				if self.store_self {
					self.vals.remove(left_reg);
					self.vals.remove(right_reg);
				}
				ConstAnalyzerResult::Remove(vec![left_reg.clone(), right_reg.clone()])
			}
			MIRInstrKind::Abs {
				val: MutableValue::Register(reg),
			}
			| MIRInstrKind::Use {
				val: MutableValue::Register(reg),
			}
			| MIRInstrKind::Remove {
				val: MutableValue::Register(reg),
			} => {
				if self.store_self {
					self.vals.remove(reg);
				}
				ConstAnalyzerResult::Remove(vec![reg.clone()])
			}
			other => {
				let used = other.get_used_regs();
				if self.store_self {
					for used in &used {
						self.vals.remove(*used);
					}
				}
				ConstAnalyzerResult::Remove(used.into_iter().cloned().collect())
			}
		}
	}
}

enum ConstAnalyzerResult<'cont> {
	Other,
	Add(Identifier, &'cont DataTypeContents),
	Remove(Vec<Identifier>),
}
