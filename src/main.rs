use dpc::common::{DeclareBinding, FunctionInterface, Identifier, MutableValue, Value};
use dpc::ir::{Block, InstrKind, Instruction, IR};
use dpc::linker::link;
use dpc::lower::ir_to_lir::lower_ir;

use dpc::common::ty::{DataType, DataTypeContents, ScoreType, ScoreTypeContents};
use dpc::passes::{run_ir_passes, run_lir_passes};

fn main() {
	let mut ir = IR::new();
	let mut block = Block::new();
	let reg_id = Identifier::from("foo");
	let reg2_id = Identifier::from("bar");
	let reg3_id = Identifier::from("baz");
	block.contents.push(Instruction::new(InstrKind::Declare {
		left: reg_id.clone(),
		ty: DataType::Score(ScoreType::Score),
		right: DeclareBinding::Value(Value::Constant(DataTypeContents::Score(
			ScoreTypeContents::Score(15),
		))),
	}));
	block.contents.push(Instruction::new(InstrKind::Assign {
		left: MutableValue::Register(reg_id.clone()),
		right: Value::Constant(DataTypeContents::Score(ScoreTypeContents::Score(93))),
	}));
	block.contents.push(Instruction::new(InstrKind::Declare {
		left: reg2_id.clone(),
		ty: DataType::Score(ScoreType::Score),
		right: DeclareBinding::Value(Value::Constant(DataTypeContents::Score(
			ScoreTypeContents::Score(-2139),
		))),
	}));
	block.contents.push(Instruction::new(InstrKind::Add {
		left: MutableValue::Register(reg_id.clone()),
		right: Value::Mutable(MutableValue::Register(reg2_id.clone())),
	}));
	block.contents.push(Instruction::new(InstrKind::Declare {
		left: reg3_id.clone(),
		ty: DataType::Score(ScoreType::UScore),
		right: DeclareBinding::Value(Value::Constant(DataTypeContents::Score(
			ScoreTypeContents::Score(0),
		))),
	}));
	block.contents.push(Instruction::new(InstrKind::Abs {
		val: MutableValue::Register(reg2_id.clone()),
	}));
	block.contents.push(Instruction::new(InstrKind::Div {
		left: MutableValue::Register(reg_id.clone()),
		right: Value::Constant(DataTypeContents::Score(ScoreTypeContents::Bool(true))),
	}));
	block.contents.push(Instruction::new(InstrKind::Mul {
		left: MutableValue::Register(reg2_id.clone()),
		right: Value::Constant(DataTypeContents::Score(ScoreTypeContents::Score(2))),
	}));
	ir.functions
		.insert(FunctionInterface::new("foo::main".into()), block);

	dbg!(&ir);
	run_ir_passes(&mut ir).expect("IR passes failed");

	let mut lir = lower_ir(ir).expect("Failed to lower IR");
	dbg!(&lir);
	run_lir_passes(&mut lir).expect("LIR passes failed");
	dbg!(&lir);

	let datapack = link(lir).expect("Failed to link datapack");
	dbg!(datapack);
}
