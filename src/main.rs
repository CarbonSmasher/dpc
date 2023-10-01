use dpc::common::{DeclareBinding, FunctionInterface, Identifier, MutableValue, Value};
use dpc::ir::{Block, InstrKind, Instruction, IR};
use dpc::linker::link;
use dpc::lower::ir_to_mir::lower_ir;
use dpc::lower::mir_to_lir::lower_mir;

use dpc::common::ty::{
	create_nbyte_array, DataType, DataTypeContents, NBTArrayType, NBTType, NBTTypeContents,
	ScoreType, ScoreTypeContents,
};
use dpc::passes::{run_ir_passes, run_lir_passes, run_mir_passes};
use rand::Rng;

fn main() {
	fuzz();
}

#[allow(dead_code)]
fn known() {
	let mut ir = IR::new();
	let mut block = Block::new();
	let reg_id = Identifier::from("foo");
	let reg2_id = Identifier::from("bar");
	let reg3_id = Identifier::from("baz");
	let reg4_id = Identifier::from("hello");
	let reg5_id = Identifier::from("there");
	let reg6_id = Identifier::from("swapl");
	let reg7_id = Identifier::from("swapr");
	let reg8_id = Identifier::from("arr");
	let reg9_id = Identifier::from("arridx");
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
	block.contents.push(Instruction::new(InstrKind::Declare {
		left: reg4_id.clone(),
		ty: DataType::NBT(NBTType::Long),
		right: DeclareBinding::Value(Value::Constant(DataTypeContents::NBT(
			NBTTypeContents::Long(-9219209999023734623),
		))),
	}));
	block.contents.push(Instruction::new(InstrKind::Assign {
		left: MutableValue::Register(reg4_id.clone()),
		right: Value::Constant(DataTypeContents::NBT(NBTTypeContents::Long(1289))),
	}));
	block.contents.push(Instruction::new(InstrKind::Abs {
		val: MutableValue::Register(reg2_id.clone()),
	}));
	block.contents.push(Instruction::new(InstrKind::Div {
		left: MutableValue::Register(reg_id.clone()),
		right: Value::Constant(DataTypeContents::Score(ScoreTypeContents::Bool(true))),
	}));
	block.contents.push(Instruction::new(InstrKind::Declare {
		left: reg5_id.clone(),
		ty: DataType::Score(ScoreType::UScore),
		right: DeclareBinding::Cast(
			DataType::Score(ScoreType::UScore),
			MutableValue::Register(reg4_id.clone()),
		),
	}));
	block.contents.push(Instruction::new(InstrKind::Mul {
		left: MutableValue::Register(reg2_id.clone()),
		right: Value::Constant(DataTypeContents::Score(ScoreTypeContents::Score(2))),
	}));
	block.contents.push(Instruction::new(InstrKind::Swap {
		left: MutableValue::Register(reg5_id.clone()),
		right: MutableValue::Register(reg2_id.clone()),
	}));
	block.contents.push(Instruction::new(InstrKind::Declare {
		left: reg6_id.clone(),
		ty: DataType::NBT(NBTType::Int),
		right: DeclareBinding::Value(Value::Constant(DataTypeContents::NBT(
			NBTTypeContents::Int(-9046),
		))),
	}));
	block.contents.push(Instruction::new(InstrKind::Declare {
		left: reg7_id.clone(),
		ty: DataType::NBT(NBTType::Int),
		right: DeclareBinding::Value(Value::Constant(DataTypeContents::NBT(
			NBTTypeContents::Int(1),
		))),
	}));
	block.contents.push(Instruction::new(InstrKind::Swap {
		left: MutableValue::Register(reg6_id.clone()),
		right: MutableValue::Register(reg7_id.clone()),
	}));
	block.contents.push(Instruction::new(InstrKind::Declare {
		left: reg8_id.clone(),
		ty: DataType::NBT(NBTType::Arr(NBTArrayType::Byte(6))),
		right: DeclareBinding::Value(Value::Constant(DataTypeContents::NBT(
			NBTTypeContents::Arr(create_nbyte_array(vec![5, 9, -2, 8, -121, 86])),
		))),
	}));
	block.contents.push(Instruction::new(InstrKind::Declare {
		left: reg9_id.clone(),
		ty: DataType::NBT(NBTType::Byte),
		right: DeclareBinding::Index {
			ty: DataType::NBT(NBTType::Byte),
			val: Value::Mutable(MutableValue::Register(reg8_id.clone())),
			index: Value::Constant(DataTypeContents::Score(ScoreTypeContents::UScore(3))),
		},
	}));

	ir.functions
		.insert(FunctionInterface::new("foo::main".into()), block);

	run(ir, true);
}

#[allow(dead_code)]
fn fuzz() {
	let instr_count = 1000000;
	let mut rng = rand::thread_rng();
	let mut reg_count = 0;

	let mut block = Block::new();

	let new_reg = reg_count;
	reg_count += 1;
	let kind = InstrKind::Declare {
		left: Identifier::from(format!("reg{new_reg}")),
		ty: DataType::Score(ScoreType::Score),
		right: DeclareBinding::Value(Value::Constant(DataTypeContents::Score(
			ScoreTypeContents::Score(0),
		))),
	};
	block.contents.push(Instruction::new(kind));

	for _ in 0..instr_count {
		let left_reg = rng.gen_range(0..reg_count);
		let left_reg = Identifier::from(format!("reg{left_reg}"));
		let right_val = match rng.gen_range(0..2) {
			0 => {
				let reg = rng.gen_range(0..reg_count);
				let reg = Identifier::from(format!("reg{reg}"));
				Value::Mutable(MutableValue::Register(reg))
			}
			1 => {
				let val = rng.gen_range(0..128);
				Value::Constant(DataTypeContents::Score(ScoreTypeContents::Score(val)))
			}
			_ => continue,
		};

		let instr = rng.gen_range(0..10);
		let kind = match instr {
			0 => {
				let new_reg = reg_count;
				reg_count += 1;
				InstrKind::Declare {
					left: Identifier::from(format!("reg{new_reg}")),
					ty: DataType::Score(ScoreType::Score),
					right: DeclareBinding::Value(right_val),
				}
			}
			1 => InstrKind::Assign {
				left: MutableValue::Register(left_reg),
				right: right_val,
			},
			2 => InstrKind::Add {
				left: MutableValue::Register(left_reg),
				right: right_val,
			},
			3 => InstrKind::Sub {
				left: MutableValue::Register(left_reg),
				right: right_val,
			},
			4 => InstrKind::Mul {
				left: MutableValue::Register(left_reg),
				right: right_val,
			},
			5 => InstrKind::Div {
				left: MutableValue::Register(left_reg),
				right: right_val,
			},
			6 => InstrKind::Mod {
				left: MutableValue::Register(left_reg),
				right: right_val,
			},
			7 => InstrKind::Min {
				left: MutableValue::Register(left_reg),
				right: right_val,
			},
			8 => InstrKind::Max {
				left: MutableValue::Register(left_reg),
				right: right_val,
			},
			9 => InstrKind::Abs {
				val: MutableValue::Register(left_reg),
			},
			_ => continue,
		};

		block.contents.push(Instruction::new(kind));
	}

	let mut ir = IR::new();
	ir.functions
		.insert(FunctionInterface::new("foo::main".into()), block);
	run(ir, false);
}

fn run(mut ir: IR, debug: bool) {
	if debug {
		println!("IR:");
		dbg!(&ir.functions);
	}
	run_ir_passes(&mut ir).expect("IR passes failed");

	let mut mir = lower_ir(ir).expect("Failed to lower IR");
	if debug {
		println!("MIR:");
		dbg!(&mir.functions);
	}
	run_mir_passes(&mut mir).expect("MIR passes failed");
	if debug {
		println!("Optimized MIR:");
		dbg!(&mir.functions);
	}

	let mut lir = lower_mir(mir).expect("Failed to lower MIR");
	if debug {
		println!("LIR:");
		dbg!(&lir.functions);
	}
	run_lir_passes(&mut lir).expect("LIR passes failed");
	if debug {
		println!("Optimized LIR:");
		dbg!(&lir.functions);
	}

	let datapack = link(lir).expect("Failed to link datapack");
	if debug {
		dbg!(datapack);
	}
}
