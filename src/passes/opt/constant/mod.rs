pub mod cond;
pub mod fold;
pub mod prop;

use anyhow::Context;
use rustc_hash::FxHashMap;

use crate::common::reg::GetUsedRegs;
use crate::common::ty::{DataType, DataTypeContents};
use crate::common::{val::MutableValue, val::Value, DeclareBinding, Identifier};
use crate::common::{Register, RegisterList};
use crate::mir::MIRInstrKind;
use crate::passes::{MIRPass, MIRPassData, Pass};

use self::cond::ConstConditionPass;
use self::fold::ConstFoldPass;
use self::prop::ConstPropPass;

/// Combines all of the constant passes and runs them
/// until no changes are made
pub struct ConstComboPass;

impl Pass for ConstComboPass {
	fn get_name(&self) -> &'static str {
		"const_combo"
	}
}

impl MIRPass for ConstComboPass {
	fn run_pass(&mut self, data: &mut MIRPassData) -> anyhow::Result<()> {
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

struct ConstAnalyzer {
	regs: RegisterList,
}

impl ConstAnalyzer {
	fn new() -> Self {
		Self {
			regs: RegisterList::default(),
		}
	}

	fn reset(&mut self) {
		self.regs.clear();
	}

	fn feed<'reg>(
		&mut self,
		kind: &'reg MIRInstrKind,
	) -> anyhow::Result<ConstAnalyzerResult<'reg>> {
		let out = match kind {
			MIRInstrKind::Declare { left, ty } => {
				self.regs.insert(
					left.clone(),
					Register {
						id: left.clone(),
						ty: ty.clone(),
					},
				);
				ConstAnalyzerResult::Other
			}
			MIRInstrKind::Assign {
				left: MutableValue::Reg(reg),
				right: DeclareBinding::Value(Value::Constant(val)),
			} => ConstAnalyzerResult::Add(reg.clone(), ConstAnalyzerValue::Value(val.clone())),
			MIRInstrKind::Assign { left, right, .. } => {
				let mut out = Vec::new();
				if let MutableValue::Reg(reg) = left {
					out.push(reg);
				}
				if let DeclareBinding::Value(Value::Mutable(MutableValue::Reg(reg))) = &right {
					out.push(reg);
				} else {
					let used = right.get_used_regs();
					out.extend(used)
				}
				ConstAnalyzerResult::Remove(out)
			}
			MIRInstrKind::Remove {
				val: MutableValue::Reg(reg),
			} => {
				let ty = &self.regs.get(reg).context("Register does not exist")?.ty;
				ConstAnalyzerResult::Add(reg.clone(), ConstAnalyzerValue::Reset(ty.clone()))
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
				if let MutableValue::Reg(reg) = left {
					ConstAnalyzerResult::Remove(vec![reg])
				} else {
					ConstAnalyzerResult::Other
				}
			}
			MIRInstrKind::Swap {
				left: MutableValue::Reg(left_reg),
				right: MutableValue::Reg(right_reg),
			} => ConstAnalyzerResult::Remove(vec![left_reg, right_reg]),
			MIRInstrKind::Abs {
				val: MutableValue::Reg(reg),
			}
			| MIRInstrKind::Use {
				val: MutableValue::Reg(reg),
			} => ConstAnalyzerResult::Remove(vec![reg]),
			other => {
				let used = other.get_used_regs();
				ConstAnalyzerResult::Remove(used)
			}
		};

		Ok(out)
	}
}

enum ConstAnalyzerResult<'reg> {
	Other,
	Add(Identifier, ConstAnalyzerValue),
	Remove(Vec<&'reg Identifier>),
}

pub enum ConstAnalyzerValue {
	/// The value is just the value
	Value(DataTypeContents),
	/// The value has been reset to not exist
	Reset(DataType),
}

/// Wrapper around ConstAnalyzer that stores its values
struct StoringConstAnalyzer {
	an: ConstAnalyzer,
	vals: FxHashMap<Identifier, ConstAnalyzerValue>,
}

impl StoringConstAnalyzer {
	fn new() -> Self {
		Self {
			an: ConstAnalyzer::new(),
			vals: FxHashMap::default(),
		}
	}

	fn feed(&mut self, kind: &MIRInstrKind) -> anyhow::Result<()> {
		let result = self.an.feed(kind)?;
		match result {
			ConstAnalyzerResult::Add(reg, val) => {
				self.vals.insert(reg, val);
			}
			ConstAnalyzerResult::Remove(regs) => {
				for reg in regs {
					self.vals.remove(reg);
				}
			}
			ConstAnalyzerResult::Other => {}
		}

		Ok(())
	}

	fn reset(&mut self) {
		self.an.reset();
		self.vals.clear();
	}
}
