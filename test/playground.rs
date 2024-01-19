//! This is just a helpful place to do random tests and stuff

use anyhow::Context;
use dpc::common::function::{
	CallInterface, FunctionAnnotations, FunctionInterface, FunctionSignature,
};
use dpc::common::mc::block::{BlockData, BlockProperties, SetBlockData, SetBlockMode};
use dpc::common::mc::entity::{SelectorType, TargetSelector};
use dpc::common::mc::instr::MinecraftInstr;
use dpc::common::mc::pos::Coordinates;
use dpc::common::mc::{EntityTarget, XPValue};
use dpc::common::val::{MutableValue, Value};
use dpc::common::{DeclareBinding, IRType, Identifier};
use dpc::ir::{Block, IRFunction, InstrKind, Instruction, IR};
use dpc::lower::ir_to_mir::lower_ir;
use dpc::lower::mir_to_lir::lower_mir;
use dpc::output::link;
use dpc::project::ProjectSettings;
use dpc::{def_compound, push_instrs};

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
	push_instrs! {
		block,
		InstrKind::Call {
			call: CallInterface {
				function: "foo:bar".into(),
				args: Vec::new(),
				ret: Vec::new(),
			},
		};
	}

	let mut annotations = FunctionAnnotations::new();
	annotations.preserve = true;
	ir.functions.insert(
		"foo_baz".into(),
		IRFunction {
			interface: FunctionInterface::with_all(
				"foo:baz".into(),
				FunctionSignature::new(),
				annotations,
			),
			block,
		},
	);

	let mut block = Block::new();

	let reg_id = Identifier::from("foo");
	let reg2_id = Identifier::from("bar");
	let reg3_id = Identifier::from("baz");
	let reg4_id = Identifier::from("hello");
	let reg5_id = Identifier::from("there");
	let reg6_id = Identifier::from("swapl");
	let reg7_id = Identifier::from("swapr");
	let reg8_id = Identifier::from("arr");
	let reg10_id = Identifier::from("data");

	let cmp1 = def_compound! {
		foo: NBTType::Int,
		bar: NBTType::Bool,
	};

	let cmp2 = def_compound! {
		foo: NBTType::Compound(cmp1.clone()),
		bar: NBTType::Short,
	};

	let cmp2_cont = def_compound! {
		foo: NBTTypeContents::Compound(cmp1.clone(), def_compound! {
			foo: NBTTypeContents::Int(5),
			bar: NBTTypeContents::Bool(false),
		}.into()),
		bar: NBTTypeContents::Short(123),
	};

	push_instrs! {
		block,
		InstrKind::Declare {
			left: reg_id.clone(),
			ty: DataType::Score(ScoreType::Score),
			right: DeclareBinding::Value(Value::Constant(DataTypeContents::Score(
				ScoreTypeContents::Score(15),
			))),
		};
		InstrKind::Assign {
			left: MutableValue::Reg(reg_id.clone()),
			right: Value::Constant(DataTypeContents::Score(ScoreTypeContents::Score(93))),
		};
		InstrKind::Declare {
			left: reg2_id.clone(),
			ty: DataType::Score(ScoreType::Score),
			right: DeclareBinding::Value(Value::Constant(DataTypeContents::Score(
				ScoreTypeContents::Score(-2139),
			))),
		};
		InstrKind::Add {
			left: MutableValue::Reg(reg_id.clone()),
			right: Value::Mutable(MutableValue::Reg(reg2_id.clone())),
		};
		InstrKind::Add {
			left: MutableValue::Reg(reg_id.clone()),
			right: Value::Constant(DataTypeContents::Score(ScoreTypeContents::Score(7))),
		};
		InstrKind::Add {
			left: MutableValue::Reg(reg_id.clone()),
			right: Value::Constant(DataTypeContents::Score(ScoreTypeContents::Score(-15))),
		};
		InstrKind::Sub {
			left: MutableValue::Reg(reg_id.clone()),
			right: Value::Constant(DataTypeContents::Score(ScoreTypeContents::Score(-1290))),
		};
		InstrKind::Declare {
			left: reg3_id.clone(),
			ty: DataType::Score(ScoreType::Score),
			right: DeclareBinding::Value(Value::Constant(DataTypeContents::Score(
				ScoreTypeContents::Score(0),
			))),
		};
		InstrKind::Declare {
			left: reg4_id.clone(),
			ty: DataType::NBT(NBTType::Long),
			right: DeclareBinding::Value(Value::Constant(DataTypeContents::NBT(
				NBTTypeContents::Long(-9219209999023734623),
			))),
		};
		InstrKind::Assign {
			left: MutableValue::Reg(reg4_id.clone()),
			right: Value::Constant(DataTypeContents::NBT(NBTTypeContents::Long(1289))),
		};
		InstrKind::Abs {
			val: MutableValue::Reg(reg2_id.clone()),
		};
		InstrKind::Div {
			left: MutableValue::Reg(reg_id.clone()),
			right: Value::Constant(DataTypeContents::Score(ScoreTypeContents::Bool(true))),
		};
		InstrKind::Declare {
			left: reg5_id.clone(),
			ty: DataType::Score(ScoreType::Score),
			right: DeclareBinding::Cast(
				DataType::Score(ScoreType::Score),
				MutableValue::Reg(reg4_id.clone()),
			),
		};
		InstrKind::Mul {
			left: MutableValue::Reg(reg2_id.clone()),
			right: Value::Constant(DataTypeContents::Score(ScoreTypeContents::Score(2))),
		};
		InstrKind::Swap {
			left: MutableValue::Reg(reg5_id.clone()),
			right: MutableValue::Reg(reg2_id.clone()),
		};
		InstrKind::Declare {
			left: reg6_id.clone(),
			ty: DataType::NBT(NBTType::Int),
			right: DeclareBinding::Value(Value::Constant(DataTypeContents::NBT(
				NBTTypeContents::Int(-9046),
			))),
		};
		InstrKind::Declare {
			left: reg7_id.clone(),
			ty: DataType::NBT(NBTType::Int),
			right: DeclareBinding::Value(Value::Constant(DataTypeContents::NBT(
				NBTTypeContents::Int(1),
			))),
		};
		InstrKind::Swap {
			left: MutableValue::Reg(reg6_id.clone()),
			right: MutableValue::Reg(reg7_id.clone()),
		};
		InstrKind::Declare {
			left: reg8_id.clone(),
			ty: DataType::NBT(NBTType::Arr(NBTArrayType::Byte(6))),
			right: DeclareBinding::Value(Value::Constant(DataTypeContents::NBT(
				NBTTypeContents::Arr(create_nbyte_array(vec![5, 9, -2, 8, -121, 86])),
			))),
		};
		InstrKind::Call {
			call: CallInterface {
				function: "foo:baz".into(),
				args: Vec::new(),
				ret: Vec::new(),
			},
		};
		InstrKind::Declare {
			left: reg10_id.clone(),
			ty: DataType::NBT(NBTType::Compound(cmp2.clone())),
			right: DeclareBinding::Value(Value::Constant(DataTypeContents::NBT(
				NBTTypeContents::Compound(cmp2, cmp2_cont.into()),
			))),
		};
	}

	ir.functions.insert(
		"foo:bar".into(),
		IRFunction {
			interface: FunctionInterface::new("foo:bar".into()),
			block,
		},
	);

	let mut block = Block::new();
	push_instrs! {
		block,
		InstrKind::Declare {
			left: reg2_id.clone(),
			ty: DataType::Score(ScoreType::Score),
			right: DeclareBinding::Value(Value::Constant(DataTypeContents::Score(
				ScoreTypeContents::Score(7),
			))),
		};
		InstrKind::Use {
			val: MutableValue::Reg(reg2_id.clone()),
		};
		InstrKind::Call {
			call: CallInterface {
				function: "foo:bar".into(),
				args: Vec::new(),
				ret: Vec::new(),
			},
		};
	}

	ir.functions.insert(
		"foo:main".into(),
		IRFunction {
			interface: FunctionInterface::with_all(
				"foo:main".into(),
				FunctionSignature::new(),
				FunctionAnnotations::new(),
			),
			block,
		},
	);

	let res = run(ir, true);
	if let Err(e) = res {
		eprintln!("{e:?}");
	}
}

#[allow(dead_code)]
fn fuzz() {
	let instr_count = 35;
	let fn_count = 1000;
	let debug = false;

	let mut rng = rand::thread_rng();

	let mut ir = IR::new();
	for fn_i in 0..fn_count {
		let instr_count = rng.gen_range(0..instr_count);
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
					Value::Mutable(MutableValue::Reg(reg))
				}
				1 => {
					let val = rng.gen_range(-128..128);
					Value::Constant(DataTypeContents::Score(ScoreTypeContents::Score(val)))
				}
				_ => continue,
			};

			let instr = rng.gen_range(0..13);
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
					left: MutableValue::Reg(left_reg),
					right: right_val,
				},
				2 => InstrKind::Add {
					left: MutableValue::Reg(left_reg),
					right: right_val,
				},
				3 => InstrKind::Sub {
					left: MutableValue::Reg(left_reg),
					right: right_val,
				},
				4 => InstrKind::Mul {
					left: MutableValue::Reg(left_reg),
					right: right_val,
				},
				5 => InstrKind::Div {
					left: MutableValue::Reg(left_reg),
					right: right_val,
				},
				6 => InstrKind::Mod {
					left: MutableValue::Reg(left_reg),
					right: right_val,
				},
				7 => InstrKind::Min {
					left: MutableValue::Reg(left_reg),
					right: right_val,
				},
				8 => InstrKind::Max {
					left: MutableValue::Reg(left_reg),
					right: right_val,
				},
				9 => InstrKind::Abs {
					val: MutableValue::Reg(left_reg),
				},
				10 => {
					let amount = rng.gen_range(0..1024);
					InstrKind::MC(MinecraftInstr::SetXP {
						target: EntityTarget::Selector(TargetSelector::new(SelectorType::This)),
						amount,
						value: XPValue::Points,
					})
				}
				11 => {
					let func = rng.gen_range(0..fn_count);
					if func == fn_i {
						continue;
					}
					InstrKind::Call {
						call: CallInterface {
							function: format!("foo:{func}").into(),
							args: Vec::new(),
							ret: Vec::new(),
						},
					}
				}
				12 => InstrKind::MC(MinecraftInstr::SetBlock {
					data: SetBlockData {
						pos: Coordinates::here(),
						block: BlockData::new("stone".into(), BlockProperties::new()),
						mode: SetBlockMode::Replace,
					},
				}),
				_ => continue,
			};

			block.contents.push(Instruction::new(kind));
		}

		let func_id = format!("foo:{fn_i}");
		let mut annotations = FunctionAnnotations::new();
		annotations.preserve = true;
		let func = if fn_i == 0 {
			FunctionInterface::with_all(
				func_id.clone().into(),
				FunctionSignature::new(),
				annotations,
			)
		} else {
			FunctionInterface::new(func_id.clone().into())
		};
		ir.functions.insert(
			func_id.into(),
			IRFunction {
				interface: func,
				block,
			},
		);
	}
	let res = run(ir, debug);
	if let Err(e) = res {
		eprintln!("{e:?}");
	}
}

fn run(mut ir: IR, debug: bool) -> anyhow::Result<()> {
	let proj = ProjectSettings::new("dpc".into());
	if debug {
		println!("IR:");
		dbg!(&ir.functions);
	}
	run_ir_passes(&mut ir, debug).context("IR passes failed")?;

	let mut mir = lower_ir(ir).context("Failed to lower IR")?;
	let init_count = mir.instr_count();
	if debug {
		println!("MIR:");
		dbg!(&mir.functions);
	}

	run_mir_passes(&mut mir, &proj, debug).context("MIR passes failed")?;
	if debug {
		println!("Optimized MIR:");
		dbg!(&mir.functions);
	}
	let final_count = mir.instr_count();
	let pct = if init_count == 0 {
		0.0
	} else {
		(1.0 - (final_count as f32 / init_count as f32)) * 100.0
	};
	println!("Removed percent: {pct}%");

	let mut lir = lower_mir(mir).context("Failed to lower MIR")?;
	let init_count = lir.instr_count();
	if debug {
		println!("LIR:");
		dbg!(&lir.functions);
	}
	run_lir_passes(&mut lir, &proj, debug).context("LIR passes failed")?;
	if debug {
		println!("Optimized LIR:");
		dbg!(&lir.functions);
	}
	let final_count = lir.instr_count();
	let pct = if init_count == 0 {
		0.0
	} else {
		(1.0 - (final_count as f32 / init_count as f32)) * 100.0
	};
	println!("Removed percent: {pct}%");

	println!("Doing codegen...");
	let datapack = link(lir, &proj).context("Failed to link datapack")?;
	if debug {
		dbg!(datapack);
	}

	Ok(())
}
