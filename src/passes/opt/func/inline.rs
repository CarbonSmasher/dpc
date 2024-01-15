use anyhow::{anyhow, Context};
use intset::GrowSet;
use rustc_hash::{FxHashMap, FxHashSet};

use crate::common::function::{CallInterface, FunctionArgs, FunctionInterface, FunctionSignature};
use crate::common::val::{MutableValue, Value};
use crate::common::{DeclareBinding, Identifier, Register, RegisterList, ResourceLocation};
use crate::lower::{cleanup_fn_id, fmt_lowered_arg};
use crate::mir::{MIRBlock, MIRFunction, MIRInstrKind, MIRInstruction};
use crate::passes::util::RunAgain;
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
		let cloned_funcs = data.mir.functions.clone();
		for func in data.mir.functions.values_mut() {
			run_block(
				&func.interface,
				&data.inline_candidates,
				&mut func.block,
				&cloned_funcs,
				true,
			)?;
		}

		Ok(())
	}
}

fn run_block(
	interface: &FunctionInterface,
	inline_candidates: &FxHashSet<ResourceLocation>,
	block: &mut MIRBlock,
	cloned_funcs: &FxHashMap<ResourceLocation, MIRFunction>,
	is_root: bool,
) -> anyhow::Result<RunAgain> {
	let mut out = RunAgain::new();
	let mut instrs_to_remove = Vec::new();
	let mut instrs_to_remove_set = GrowSet::with_capacity(block.contents.len());

	loop {
		instrs_to_remove.clear();
		let run_again = run_iter(
			interface,
			block,
			&mut instrs_to_remove,
			&mut instrs_to_remove_set,
			inline_candidates,
			&cloned_funcs,
			is_root,
		)?;
		out.merge(run_again);

		if run_again.result() {
			block.contents = replace_and_expand_indices(block.contents.clone(), &instrs_to_remove);
		} else {
			break;
		}
	}

	Ok(out)
}

fn run_iter(
	interface: &FunctionInterface,
	block: &mut MIRBlock,
	instrs_to_remove: &mut Vec<(usize, Vec<MIRInstruction>)>,
	instrs_to_remove_set: &mut GrowSet,
	inline_candidates: &FxHashSet<ResourceLocation>,
	cloned_funcs: &FxHashMap<ResourceLocation, MIRFunction>,
	is_root: bool,
) -> anyhow::Result<RunAgain> {
	let mut run_again = RunAgain::new();

	let mut regs = RegisterList::default();
	for (i, instr) in block.contents.iter_mut().enumerate() {
		if instrs_to_remove_set.contains(i) {
			continue;
		}
		if let MIRInstrKind::Declare { left, ty } = &instr.kind {
			let reg = Register {
				id: left.clone(),
				ty: ty.clone(),
			};
			regs.insert(left.clone(), reg);
		}
		// Inline simple blocks into modifying instruction bodies
		if let MIRInstrKind::Call { call } = &instr.kind {
			let block = get_call_block(call, inline_candidates, interface, cloned_funcs)?;
			if let Some(block) = block {
				// If we aren't at the root, then inlining blocks that are more than 1 long
				// will just create a bunch of identical copies of functions because the blocks will be inlined
				// and just lowered to functions again
				// We may want to relax this for special cases in the future that allow certain folds to be run
				if !(!is_root && block.contents.len() != 1) {
					let block = get_inlined_block(call, interface, block, &regs)?;
					instrs_to_remove.push((i, block));
					instrs_to_remove_set.add(i);
					run_again.yes();
				}
			}
		}
		for body in instr.kind.get_bodies_mut() {
			run_block(interface, inline_candidates, body, cloned_funcs, false)?;
		}
	}

	Ok(run_again)
}

fn get_inlined_block(
	call: &CallInterface,
	interface: &FunctionInterface,
	call_block: &MIRBlock,
	regs: &RegisterList,
) -> anyhow::Result<Vec<MIRInstruction>> {
	// Inline the block
	let mut inlined_contents = call_block.contents.clone();
	let func_id = cleanup_fn_id(&call.function);

	cleanup_fn(
		&func_id,
		&call.args,
		&mut inlined_contents,
		&regs,
		&interface.sig,
		&call.ret,
	)
	.context("Failed to clean up inlined function blocks")?;

	Ok(inlined_contents)
}

/// Get the block from the function we are calling, to possibly be inlined
fn get_call_block<'f>(
	call: &CallInterface,
	inline_candidates: &FxHashSet<ResourceLocation>,
	interface: &FunctionInterface,
	cloned_funcs: &'f FxHashMap<ResourceLocation, MIRFunction>,
) -> anyhow::Result<Option<&'f MIRBlock>> {
	// Don't inline this function call if it is recursive
	if call.function == interface.id {
		return Ok(None);
	}
	if !inline_candidates.contains(&call.function) {
		return Ok(None);
	}
	let func = cloned_funcs
		.get(&call.function)
		.ok_or(anyhow!("Called function does not exist"))?;
	Ok(Some(&func.block))
}

/// Cleanup a function block so that it can be compatible when inlined
fn cleanup_fn(
	func_id: &str,
	args: &FunctionArgs,
	block: &mut Vec<MIRInstruction>,
	regs: &RegisterList,
	sig: &FunctionSignature,
	ret_destinations: &[MutableValue],
) -> anyhow::Result<()> {
	// Set the arguments
	let mut prelude = Vec::new();
	for (i, arg) in args.iter().enumerate() {
		let reg = fmt_lowered_arg(func_id, i.try_into().expect("This should fit"));
		prelude.push(MIRInstruction::new(MIRInstrKind::Declare {
			left: reg.clone(),
			ty: arg.get_ty(regs, sig)?,
		}));

		prelude.push(MIRInstruction::new(MIRInstrKind::Assign {
			left: MutableValue::Reg(reg.clone()),
			right: DeclareBinding::Value(arg.clone()),
		}));
	}

	for instr in block.iter_mut() {
		instr.kind.replace_regs(&|reg| {
			let new = fmt_inlined_reg(func_id, reg);
			*reg = new;
		});

		instr.kind.replace_mut_vals(&|val| {
			if let MutableValue::Arg(idx) = val {
				*val = MutableValue::Reg(fmt_lowered_arg(func_id, *idx));
			}
		});

		if let MIRInstrKind::ReturnValue { index, value } = &instr.kind {
			if let Some(dest) = ret_destinations.get(*index as usize) {
				// We want to replace regs and args on the right hand side, but not on
				// the destination since its part of the calling function
				let mut value = value.clone();
				if let Value::Mutable(MutableValue::Arg(idx)) = &mut value {
					value = Value::Mutable(MutableValue::Reg(fmt_lowered_arg(func_id, *idx)));
				}
				instr.kind = MIRInstrKind::Assign {
					left: dest.clone(),
					right: DeclareBinding::Value(value),
				}
			} else {
				instr.kind = MIRInstrKind::NoOp;
			}
		}
	}

	if !prelude.is_empty() {
		*block = [prelude, block.clone()].concat();
	}

	Ok(())
}

fn fmt_inlined_reg(func_id: &str, reg: &Identifier) -> Identifier {
	let reg = format!("in_{func_id}_{reg}");
	reg.into()
}
