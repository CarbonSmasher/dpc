use std::collections::{HashMap, HashSet};

use anyhow::{anyhow, Context};

use crate::common::block::{BlockAllocator, BlockID};
use crate::common::function::{
	FunctionAnnotations, FunctionArgs, FunctionInterface, FunctionParams, FunctionSignature,
};
use crate::common::val::MutableValue;
use crate::common::{DeclareBinding, Identifier, Register, RegisterList, ResourceLocation};
use crate::lower::{cleanup_fn_id, fmt_lowered_arg};
use crate::mir::{MIRBlock, MIRInstrKind, MIRInstruction};
use crate::passes::{MIRPass, MIRPassData, Pass};
use crate::util::replace_and_expand_indices;

pub struct SimpleInlinePass;

impl Pass for SimpleInlinePass {
	fn get_name(&self) -> &'static str {
		"simple_inline"
	}
}

impl MIRPass for SimpleInlinePass {
	fn run_pass(&mut self, data: &mut MIRPassData) -> anyhow::Result<()> {
		let mut instrs_to_remove = Vec::new();
		let mut instrs_to_remove_set = HashSet::new();
		let cloned_funcs = data.mir.functions.clone();
		let cloned_blocks = data.mir.blocks.clone();
		for (func, block) in &mut data.mir.functions {
			let block = data
				.mir
				.blocks
				.get_mut(block)
				.ok_or(anyhow!("Block does not exist"))?;

			loop {
				instrs_to_remove.clear();
				let run_again = run_simple_inline_iter(
					&func,
					block,
					&mut instrs_to_remove,
					&mut instrs_to_remove_set,
					&data.inline_candidates,
					&cloned_funcs,
					&cloned_blocks,
				)?;

				block.contents =
					replace_and_expand_indices(block.contents.clone(), &instrs_to_remove);
				if !run_again {
					break;
				}
			}
		}

		Ok(())
	}
}

fn run_simple_inline_iter(
	interface: &FunctionInterface,
	block: &mut MIRBlock,
	instrs_to_remove: &mut Vec<(usize, Vec<MIRInstruction>)>,
	instrs_to_remove_set: &mut HashSet<usize>,
	inline_candidates: &HashSet<ResourceLocation>,
	cloned_funcs: &HashMap<FunctionInterface, BlockID>,
	cloned_blocks: &BlockAllocator<MIRBlock>,
) -> anyhow::Result<bool> {
	let mut run_again = false;

	let regs = RegisterList::new();
	for (i, instr) in block.contents.iter().enumerate() {
		if instrs_to_remove_set.contains(&i) {
			continue;
		}
		if let MIRInstrKind::Declare { left, ty } = &instr.kind {
			let reg = Register {
				id: left.clone(),
				ty: ty.clone(),
			};
			regs.insert(left.clone(), reg);
		}
		if let MIRInstrKind::Call { call } = &instr.kind {
			// Don't inline this function call if it is recursive
			if &call.function == &interface.id {
				continue;
			}
			if !inline_candidates.contains(&call.function) {
				continue;
			}
			let func = cloned_funcs
				.get(&FunctionInterface {
					id: call.function.clone(),
					sig: FunctionSignature::new(),
					annotations: FunctionAnnotations::new(),
				})
				.ok_or(anyhow!("Called function does not exist"))?;
			let inlined_block = cloned_blocks
				.get(func)
				.ok_or(anyhow!("Inlined block does not exist"))?;

			// Inline the block
			let mut inlined_contents = inlined_block.contents.clone();
			let func_id = cleanup_fn_id(&call.function);

			cleanup_fn(
				&func_id,
				&call.args,
				&mut inlined_contents,
				&regs,
				&interface.sig.params,
			)
			.context("Failed to clean up inlined function blocks")?;
			instrs_to_remove.push((i, inlined_contents));
			instrs_to_remove_set.insert(i);
			run_again = true;
		}
	}

	Ok(run_again)
}

/// Cleanup a function block so that it can be compatible when inlined
fn cleanup_fn(
	func_id: &str,
	args: &FunctionArgs,
	block: &mut Vec<MIRInstruction>,
	regs: &RegisterList,
	params: &FunctionParams,
) -> anyhow::Result<()> {
	// Set the arguments
	let mut prelude = Vec::new();
	for (i, arg) in args.iter().enumerate() {
		let reg = fmt_lowered_arg(func_id, i.try_into().expect("This should fit"));
		prelude.push(MIRInstruction::new(MIRInstrKind::Declare {
			left: reg.clone(),
			ty: arg.get_ty(regs, params)?,
		}));

		prelude.push(MIRInstruction::new(MIRInstrKind::Assign {
			left: MutableValue::Register(reg.clone()),
			right: DeclareBinding::Value(arg.clone()),
		}));
	}

	for instr in block.iter_mut() {
		instr.kind.replace_regs(|reg| {
			let new = fmt_inlined_reg(func_id, reg);
			*reg = new;
		});

		instr.kind.replace_mut_vals(|val| {
			if let MutableValue::Arg(idx) = val {
				*val = MutableValue::Register(fmt_lowered_arg(func_id, *idx));
			}
		});
	}

	*block = [prelude, block.clone()].concat();

	Ok(())
}

fn fmt_inlined_reg(func_id: &str, reg: &Identifier) -> Identifier {
	let reg = reg.to_string();
	let reg = format!("in_{func_id}_{reg}");
	reg.into()
}
