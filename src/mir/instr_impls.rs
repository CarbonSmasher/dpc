use std::iter;

use crate::common::reg::GetUsedRegs;
use crate::common::val::{MutableValue, Value};
use crate::common::{DeclareBinding, Identifier};

use super::{MIRBlock, MIRInstrKind};

impl MIRInstrKind {
	pub fn replace_regs<F: Fn(&mut Identifier)>(&mut self, f: &F) {
		match self {
			Self::Declare { left, .. } => {
				f(left);
			}
			Self::Assign { left, right } => {
				let right_regs: Box<dyn Iterator<Item = _>> = match right {
					DeclareBinding::Null => Box::new(std::iter::empty()),
					DeclareBinding::Value(val) => Box::new(val.get_used_regs_mut().into_iter()),
					DeclareBinding::Cast(_, val) => Box::new(val.get_used_regs_mut().into_iter()),
					DeclareBinding::Index { val, index, .. } => Box::new(
						val.get_used_regs_mut()
							.into_iter()
							.chain(index.get_used_regs_mut()),
					),
					DeclareBinding::Condition(cond) => cond.iter_used_regs_mut(),
				};
				for reg in left.get_used_regs_mut().into_iter().chain(right_regs) {
					f(reg);
				}
			}
			Self::Add { left, right }
			| Self::Sub { left, right }
			| Self::Mul { left, right }
			| Self::Div { left, right }
			| Self::Mod { left, right }
			| Self::Min { left, right }
			| Self::Max { left, right }
			| Self::Merge { left, right }
			| Self::Push { left, right }
			| Self::PushFront { left, right }
			| Self::Insert { left, right, .. }
			| Self::And { left, right }
			| Self::Or { left, right } => {
				for reg in left
					.get_used_regs_mut()
					.into_iter()
					.chain(right.get_used_regs_mut())
				{
					f(reg);
				}
			}
			Self::Swap { left, right } => {
				for reg in left
					.get_used_regs_mut()
					.into_iter()
					.chain(right.get_used_regs_mut())
				{
					f(reg);
				}
			}
			Self::Abs { val }
			| Self::Pow { base: val, .. }
			| Self::Get { value: val, .. }
			| Self::Use { val }
			| Self::Not { value: val }
			| Self::Remove { val } => {
				for reg in val.get_used_regs_mut() {
					f(reg);
				}
			}
			Self::Call { call } => {
				for reg in call.iter_used_regs_mut() {
					f(reg);
				}
			}
			Self::If { condition, body } => {
				for reg in condition.iter_used_regs_mut() {
					f(reg);
				}
				body.replace_regs(f);
			}
			Self::As { body, .. }
			| Self::At { body, .. }
			| Self::Positioned { body, .. }
			| Self::ReturnRun { body } => {
				body.replace_regs(f);
			}
			Self::StoreResult { location, body } | Self::StoreSuccess { location, body } => {
				location.replace_regs(&f);
				body.replace_regs(f);
			}
			Self::ReturnValue { value, .. } | Self::Return { value } => {
				for reg in value.get_used_regs_mut() {
					f(reg);
				}
			}
			Self::NoOp
			| Self::GetConst { .. }
			| Self::Command { .. }
			| Self::Comment { .. }
			| Self::CallExtern { .. }
			| Self::MC(..) => {}
		}
	}

	pub fn replace_mut_vals<F: Fn(&mut MutableValue)>(&mut self, f: &F) {
		match self {
			Self::Assign { left, right } => {
				let right_regs: Box<dyn Iterator<Item = _>> = match right {
					DeclareBinding::Null => Box::new(std::iter::empty()),
					DeclareBinding::Value(val) => Box::new(val.iter_mut_val().into_iter()),
					DeclareBinding::Cast(_, val) => Box::new(iter::once(val)),
					DeclareBinding::Index { val, index, .. } => {
						Box::new(val.iter_mut_val().into_iter().chain(index.iter_mut_val()))
					}
					DeclareBinding::Condition(cond) => cond.iter_mut_vals(),
				};
				for reg in iter::once(left).chain(right_regs) {
					f(reg);
				}
			}
			Self::Add { left, right }
			| Self::Sub { left, right }
			| Self::Mul { left, right }
			| Self::Div { left, right }
			| Self::Mod { left, right }
			| Self::Min { left, right }
			| Self::Max { left, right }
			| Self::Merge { left, right }
			| Self::Push { left, right }
			| Self::PushFront { left, right }
			| Self::Insert { left, right, .. }
			| Self::And { left, right }
			| Self::Or { left, right } => {
				for reg in iter::once(left).chain(right.iter_mut_val()) {
					f(reg);
				}
			}
			Self::Swap { left, right } => {
				f(left);
				f(right);
			}
			Self::Abs { val }
			| Self::Pow { base: val, .. }
			| Self::Get { value: val, .. }
			| Self::Use { val }
			| Self::Not { value: val }
			| Self::ReturnValue {
				value: Value::Mutable(val),
				..
			}
			| Self::Remove { val }
			| Self::Return {
				value: Value::Mutable(val),
			} => {
				f(val);
			}
			Self::Call { call } => {
				for val in &mut call.args {
					if let Value::Mutable(val) = val {
						f(val);
					}
				}
			}
			Self::If { condition, body } => {
				for val in condition.iter_mut_vals() {
					f(val);
				}
				body.replace_mut_vals(f);
			}
			Self::As { body, .. }
			| Self::At { body, .. }
			| Self::Positioned { body, .. }
			| Self::StoreResult { body, .. }
			| Self::StoreSuccess { body, .. }
			| Self::ReturnRun { body } => {
				body.replace_mut_vals(f);
			}
			Self::NoOp
			| Self::Declare { .. }
			| Self::GetConst { .. }
			| Self::Command { .. }
			| Self::Comment { .. }
			| Self::CallExtern { .. }
			| Self::Return {
				value: Value::Constant(..),
			}
			| Self::ReturnValue {
				value: Value::Constant(..),
				..
			}
			| Self::MC(..) => {}
		}
	}

	pub fn get_bodies(&self) -> Vec<&MIRBlock> {
		match self {
			Self::As { body, .. }
			| Self::At { body, .. }
			| Self::If { body, .. }
			| Self::StoreResult { body, .. }
			| Self::StoreSuccess { body, .. }
			| Self::Positioned { body, .. } => vec![body],
			_ => Vec::new(),
		}
	}

	pub fn get_bodies_mut<'a>(&'a mut self) -> Vec<&'a mut MIRBlock> {
		match self {
			Self::As { body, .. }
			| Self::At { body, .. }
			| Self::If { body, .. }
			| Self::StoreResult { body, .. }
			| Self::StoreSuccess { body, .. }
			| Self::Positioned { body, .. } => vec![body],
			_ => Vec::new(),
		}
	}

	pub fn get_op_lhs(&self) -> Option<&MutableValue> {
		match self {
			Self::Assign { left, .. }
			| Self::Add { left, .. }
			| Self::Sub { left, .. }
			| Self::Mul { left, .. }
			| Self::Div { left, .. }
			| Self::Mod { left, .. }
			| Self::Min { left, .. }
			| Self::Max { left, .. }
			| Self::Merge { left, .. }
			| Self::Insert { left, .. }
			| Self::Push { left, .. }
			| Self::PushFront { left, .. }
			| Self::Remove { val: left, .. }
			| Self::And { left, .. }
			| Self::Or { left, .. }
			| Self::Not { value: left, .. }
			| Self::Abs { val: left, .. } => Some(left),
			_ => None,
		}
	}
}

impl GetUsedRegs for MIRInstrKind {
	fn append_used_regs<'a>(&'a self, regs: &mut Vec<&'a Identifier>) {
		match self {
			Self::Assign { left, right } => {
				left.append_used_regs(regs);
				right.append_used_regs(regs);
			}
			Self::Add { left, right }
			| Self::Sub { left, right }
			| Self::Mul { left, right }
			| Self::Div { left, right }
			| Self::Mod { left, right }
			| Self::Min { left, right }
			| Self::Max { left, right }
			| Self::Merge { left, right }
			| Self::Push { left, right }
			| Self::PushFront { left, right }
			| Self::Insert { left, right, .. }
			| Self::And { left, right }
			| Self::Or { left, right } => {
				left.append_used_regs(regs);
				right.append_used_regs(regs);
			}
			Self::Swap { left, right } => {
				left.append_used_regs(regs);
				right.append_used_regs(regs);
			}
			Self::Abs { val } => val.append_used_regs(regs),
			Self::Not { value } => value.append_used_regs(regs),
			Self::Pow { base, .. } => base.append_used_regs(regs),
			Self::Get { value, .. } => value.append_used_regs(regs),
			Self::Use { val } => val.append_used_regs(regs),
			Self::Call { call } => call.append_used_regs(regs),
			Self::Remove { val } => val.append_used_regs(regs),
			Self::ReturnValue { value, .. } | Self::Return { value } => {
				value.append_used_regs(regs)
			}
			Self::If { condition, body } => {
				condition.append_used_regs(regs);
				body.append_used_regs(regs);
			}
			Self::As { body, .. }
			| Self::At { body, .. }
			| Self::Positioned { body, .. }
			| Self::ReturnRun { body } => body.append_used_regs(regs),
			Self::StoreResult { location, body } | Self::StoreSuccess { location, body } => {
				location.append_used_regs(regs);
				body.append_used_regs(regs);
			}
			Self::Declare { .. }
			| Self::NoOp
			| Self::GetConst { .. }
			| Self::Command { .. }
			| Self::Comment { .. }
			| Self::CallExtern { .. }
			| Self::MC(..) => {}
		}
	}
}
