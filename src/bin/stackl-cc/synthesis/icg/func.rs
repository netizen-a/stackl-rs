// Copyright (c) 2024-2026 Jonathan Thomason

use super::{
	Diagnostic,
	syn,
};

impl super::SSACodeGen<'_> {
	pub(super) fn function_definition(
		&mut self,
		def: &syn::FunctionDefinition,
	) -> Result<u32, Diagnostic> {
		let type_id = self.resolve_type(def.specifiers.layout.as_ref().unwrap());
		for decl in def.declaration_list.iter() {
			self.declaration(decl)?;
		}
		for block_item in def.compound_stmt.blocks.iter() {
			match block_item {
				syn::BlockItem::Declaration(decl) => {
					self.declaration(decl);
				}
				syn::BlockItem::Statement(stmt) => {
					self.statement(stmt);
				}
				_ => todo!(),
			}
		}
		Ok(0)
	}
}
