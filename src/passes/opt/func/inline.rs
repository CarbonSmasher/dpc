use anyhow::{anyhow, Context};
use rustc_hash::{FxHashMap, FxHashSet};

use crate::common::function::{CallInterface, FunctionArgs, FunctionInterface, FunctionSignature};
use crate::common::val::{MutableValue, Value};
use crate::common::{DeclareBinding, Identifier, Register, RegisterList, ResourceLocation};
use crate::lower::{cleanup_fn_id, fmt_lowered_arg};
use crate::mir::{MIRBlock, MIRFunction, MIRInstrKind, MIRInstruction};
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
		let mut instrs_to_remove_set = FxHashSet::default();
		let cloned_funcs = data.mir.functions.clone();
		for func in data.mir.functions.values_mut() {
			let block = &mut func.block;

			loop {
				instrs_to_remove.clear();
				let run_again = run_simple_inline_iter(
					&func.interface,
					block,
					&mut instrs_to_remove,
					&mut instrs_to_remove_set,
					&data.inline_candidates,
					&cloned_funcs,
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
	instrs_to_remove_set: &mut FxHashSet<usize>,
	inline_candidates: &FxHashSet<ResourceLocation>,
	cloned_funcs: &FxHashMap<ResourceLocation, MIRFunction>,
) -> anyhow::Result<bool> {
	let mut run_again = false;

	let mut regs = RegisterList::default();
	for (i, instr) in block.contents.iter_mut().enumerate() {
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
		// Inline simple blocks into modifying instruction bodies
		if let MIRInstrKind::Call { call } = &instr.kind {
			let block = get_inlined_block(call, inline_candidates, interface, cloned_funcs, &regs)?;
			if let Some(block) = block {
				instrs_to_remove.push((i, block));
				instrs_to_remove_set.insert(i);
				run_again = true;
			}
		}
		if instr.kind.get_body().is_some() {
			fn inner(
				instr: &mut MIRInstrKind,
				inline_candidates: &FxHashSet<ResourceLocation>,
				interface: &FunctionInterface,
				cloned_funcs: &FxHashMap<ResourceLocation, MIRFunction>,
				regs: &RegisterList,
			) -> anyhow::Result<()> {
				if let MIRInstrKind::Call { call } = instr {
					let block =
						get_inlined_block(call, inline_candidates, interface, cloned_funcs, &regs)?;
					if let Some(block) = block {
						// We can only inline blocks that are one instruction long
						if block.len() == 1 {
							let body_instr = block.first().expect("Length is 1");
							*instr = body_instr.kind.clone();
						}
					}
				} else {
					if let Some(body) = instr.get_body_mut() {
						inner(body, inline_candidates, interface, cloned_funcs, regs)?;
					}
				}

				Ok(())
			}

			inner(
				&mut instr.kind,
				inline_candidates,
				interface,
				cloned_funcs,
				&regs,
			)?;
		}
	}

	Ok(run_again)
}

fn get_inlined_block(
	call: &CallInterface,
	inline_candidates: &FxHashSet<ResourceLocation>,
	interface: &FunctionInterface,
	cloned_funcs: &FxHashMap<ResourceLocation, MIRFunction>,
	regs: &RegisterList,
) -> anyhow::Result<Option<Vec<MIRInstruction>>> {
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
	let inlined_block = func.block.clone();

	// Inline the block
	let mut inlined_contents = inlined_block.contents;
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

	Ok(Some(inlined_contents))
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
		instr.kind.replace_regs(|reg| {
			let new = fmt_inlined_reg(func_id, reg);
			*reg = new;
		});

		instr.kind.replace_mut_vals(|val| {
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
