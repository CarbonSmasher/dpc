use crate::mir::MIRInstrKind;

use super::val::{MutableValue, Value};

#[derive(Debug, Clone)]
pub enum Operation {
	Single(SingleOperation),
	Double(DoubleOperation),
}

impl Operation {
	pub fn to_instr(self) -> MIRInstrKind {
		match self {
			Self::Single(op) => op.to_instr(),
			Self::Double(op) => op.to_instr(),
		}
	}

	pub fn from_instr(instr: MIRInstrKind) -> Option<Self> {
		if let Some(instr) = SingleOperation::from_instr(instr.clone()) {
			Some(Self::Single(instr))
		} else {
			if let Some(instr) = DoubleOperation::from_instr(instr) {
				Some(Self::Double(instr))
			} else {
				None
			}
		}
	}

	pub fn set_lhs(&mut self, lhs: MutableValue) {
		match self {
			Self::Single(op) => op.val = lhs,
			Self::Double(op) => op.left = lhs,
		}
	}

	pub fn get_rhs(&self) -> Option<&Value> {
		match self {
			Self::Single(..) => None,
			Self::Double(op) => Some(&op.right),
		}
	}
}

#[derive(Debug, Clone)]
pub struct SingleOperation {
	pub val: MutableValue,
	pub kind: SingleOperationKind,
}

impl SingleOperation {
	pub fn to_instr(self) -> MIRInstrKind {
		match self.kind {
			SingleOperationKind::Abs => MIRInstrKind::Abs { val: self.val },
			SingleOperationKind::Not => MIRInstrKind::Not { value: self.val },
			SingleOperationKind::Pow(exp) => MIRInstrKind::Pow {
				base: self.val,
				exp,
			},
		}
	}

	pub fn from_instr(instr: MIRInstrKind) -> Option<Self> {
		match instr {
			MIRInstrKind::Abs { val } => Some(Self {
				val,
				kind: SingleOperationKind::Abs,
			}),
			MIRInstrKind::Not { value } => Some(Self {
				val: value,
				kind: SingleOperationKind::Not,
			}),
			MIRInstrKind::Pow { base, exp } => Some(Self {
				val: base,
				kind: SingleOperationKind::Pow(exp),
			}),
			_ => None,
		}
	}
}

#[derive(Debug, Clone)]
pub enum SingleOperationKind {
	Abs,
	Not,
	Pow(u8),
}

#[derive(Debug, Clone)]
pub struct DoubleOperation {
	pub left: MutableValue,
	pub right: Value,
	pub kind: DoubleOperationKind,
}

impl DoubleOperation {
	pub fn to_instr(self) -> MIRInstrKind {
		match self.kind {
			DoubleOperationKind::Add => MIRInstrKind::Add {
				left: self.left,
				right: self.right,
			},
			DoubleOperationKind::Sub => MIRInstrKind::Sub {
				left: self.left,
				right: self.right,
			},
			DoubleOperationKind::Mul => MIRInstrKind::Mul {
				left: self.left,
				right: self.right,
			},
			DoubleOperationKind::Div => MIRInstrKind::Div {
				left: self.left,
				right: self.right,
			},
			DoubleOperationKind::Mod => MIRInstrKind::Mod {
				left: self.left,
				right: self.right,
			},
			DoubleOperationKind::Min => MIRInstrKind::Min {
				left: self.left,
				right: self.right,
			},
			DoubleOperationKind::Max => MIRInstrKind::Max {
				left: self.left,
				right: self.right,
			},
			DoubleOperationKind::Merge => MIRInstrKind::Merge {
				left: self.left,
				right: self.right,
			},
			DoubleOperationKind::Push => MIRInstrKind::Push {
				left: self.left,
				right: self.right,
			},
			DoubleOperationKind::PushFront => MIRInstrKind::PushFront {
				left: self.left,
				right: self.right,
			},
			DoubleOperationKind::Insert(i) => MIRInstrKind::Insert {
				left: self.left,
				right: self.right,
				index: i,
			},
			DoubleOperationKind::Or => MIRInstrKind::Or {
				left: self.left,
				right: self.right,
			},
			DoubleOperationKind::And => MIRInstrKind::And {
				left: self.left,
				right: self.right,
			},
		}
	}

	pub fn from_instr(instr: MIRInstrKind) -> Option<Self> {
		match instr {
			MIRInstrKind::Add { left, right } => Some(Self {
				kind: DoubleOperationKind::Add,
				left,
				right,
			}),
			MIRInstrKind::Sub { left, right } => Some(Self {
				kind: DoubleOperationKind::Sub,
				left,
				right,
			}),
			MIRInstrKind::Mul { left, right } => Some(Self {
				kind: DoubleOperationKind::Mul,
				left,
				right,
			}),
			MIRInstrKind::Div { left, right } => Some(Self {
				kind: DoubleOperationKind::Div,
				left,
				right,
			}),
			MIRInstrKind::Mod { left, right } => Some(Self {
				kind: DoubleOperationKind::Mod,
				left,
				right,
			}),
			MIRInstrKind::Min { left, right } => Some(Self {
				kind: DoubleOperationKind::Min,
				left,
				right,
			}),
			MIRInstrKind::Max { left, right } => Some(Self {
				kind: DoubleOperationKind::Max,
				left,
				right,
			}),
			MIRInstrKind::Merge { left, right } => Some(Self {
				kind: DoubleOperationKind::Merge,
				left,
				right,
			}),
			MIRInstrKind::Push { left, right } => Some(Self {
				kind: DoubleOperationKind::Push,
				left,
				right,
			}),
			MIRInstrKind::PushFront { left, right } => Some(Self {
				kind: DoubleOperationKind::PushFront,
				left,
				right,
			}),
			MIRInstrKind::Insert { left, right, index } => Some(Self {
				kind: DoubleOperationKind::Insert(index),
				left,
				right,
			}),
			MIRInstrKind::Or { left, right } => Some(Self {
				kind: DoubleOperationKind::Or,
				left,
				right,
			}),
			MIRInstrKind::And { left, right } => Some(Self {
				kind: DoubleOperationKind::And,
				left,
				right,
			}),
			_ => None,
		}
	}
}

#[derive(Debug, Clone)]
pub enum DoubleOperationKind {
	Add,
	Sub,
	Mul,
	Div,
	Mod,
	Min,
	Max,
	Merge,
	Push,
	PushFront,
	Insert(i32),
	Or,
	And,
}
