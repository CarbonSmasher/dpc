use intset::GrowSet;
use rustc_hash::FxHashSet;

use crate::common::mc::modifier::Modifier;
use crate::lir::{LIRInstrKind, LIR};
use crate::passes::{LIRPass, Pass};
use crate::util::{remove_indices, GetSetOwned};

use super::{Dependency, Modified, ModifierContext};

pub struct NullModifiersPass;

impl Pass for NullModifiersPass {
	fn get_name(&self) -> &'static str {
		"null_modifiers"
	}
}

impl LIRPass for NullModifiersPass {
	fn run_pass(&mut self, lir: &mut LIR) -> anyhow::Result<()> {
		for func in lir.functions.values_mut() {
			let block = &mut func.block;

			for instr in &mut block.contents {
				let modifier_count = instr.modifiers.len();

				let instr_deps = <LIRInstrKind as GetSetOwned<Dependency>>::get_set(&instr.kind);
				if instr_deps.contains(&Dependency(ModifierContext::Everything)) {
					continue;
				}
				let mut mods_to_remove = GrowSet::with_capacity(modifier_count);

				let mut mod_deps = FxHashSet::default();
				'outer: for (i, modifier) in instr.modifiers.iter_mut().enumerate().rev() {
					// If neither the instruction nor any of the modifiers after this one
					// needs the context of this modifier, then it can be removed
					let this_mod_ctx = <Modifier as GetSetOwned<Modified>>::get_set(modifier);

					let mut dont_push_deps = false;

					// We also need to check for side effects
					if !modifier.has_extra_side_efects() {
						// Can't do if either since that can break the chain
						if !matches!(modifier, Modifier::If { .. }) {
							if !this_mod_ctx.iter().any(|x| {
								let dep = x.clone().to_dep();
								instr_deps.contains(&dep) | mod_deps.contains(&dep)
							}) {
								mods_to_remove.add(i);
								dont_push_deps = true;
							}
						}
					}

					if !dont_push_deps {
						<Modifier as GetSetOwned<Dependency>>::append_set(modifier, &mut mod_deps);
					}
					if mod_deps.contains(&Dependency(ModifierContext::Everything)) {
						break 'outer;
					}
				}

				remove_indices(&mut instr.modifiers, &mods_to_remove);
			}
		}

		Ok(())
	}
}
