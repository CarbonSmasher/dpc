use std::iter;

use crate::common::{
	val::{MutableValue, Value},
	DeclareBinding, Identifier,
};

use super::MIRInstrKind;

impl MIRInstrKind {
	pub fn get_used_regs(&self) -> Vec<&Identifier> {
		match self {
			Self::Assign { left, right } => [left.get_used_regs(), right.get_used_regs()].concat(),
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
			| Self::Insert { left, right, .. } => [left.get_used_regs(), right.get_used_regs()].concat(),
			Self::Swap { left, right } => [left.get_used_regs(), right.get_used_regs()].concat(),
			Self::Abs { val } => val.get_used_regs(),
			Self::Pow { base, .. } => base.get_used_regs(),
			Self::Get { value, .. } => value.get_used_regs(),
			Self::Use { val } => val.get_used_regs(),
			Self::Call { call } => call.get_used_regs(),
			Self::Remove { val } => val.get_used_regs(),
			Self::ReturnValue { value, .. } | Self::Return { value } => value.get_used_regs(),
			Self::If { condition, body } => {
				[condition.get_used_regs(), body.get_used_regs()].concat()
			}
			Self::As { body, .. }
			| Self::At { body, .. }
			| Self::Positioned { body, .. }
			| Self::ReturnRun { body } => body.get_used_regs(),
			Self::StoreResult { location, body } | Self::StoreSuccess { location, body } => {
				[location.get_used_regs(), body.get_used_regs()].concat()
			}
			Self::Declare { .. }
			| Self::NoOp
			| Self::GetConst { .. }
			| Self::Command { .. }
			| Self::Comment { .. }
			| Self::CallExtern { .. }
			| Self::MC(..) => Vec::new(),
		}
	}

	pub fn replace_regs<F: Fn(&mut Identifier)>(&mut self, f: F) {
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
			| Self::Insert { left, right, .. } => {
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

	pub fn replace_mut_vals<F: Fn(&mut MutableValue)>(&mut self, f: F) {
		match self {
			Self::Assign { left, right } => {
				let right_regs: Box<dyn Iterator<Item = _>> = match right {
					DeclareBinding::Null => Box::new(std::iter::empty()),
					DeclareBinding::Value(val) => Box::new(val.iter_mut_val().into_iter()),
					DeclareBinding::Cast(_, val) => Box::new(iter::once(val)),
					DeclareBinding::Index { val, index, .. } => {
						Box::new(val.iter_mut_val().into_iter().chain(index.iter_mut_val()))
					}
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
			| Self::Insert { left, right, .. } => {
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
}
