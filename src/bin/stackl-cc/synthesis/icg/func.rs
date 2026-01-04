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
		let ret_type = self.resolve_type(def.specifiers.layout.as_ref().unwrap());
		let func_type: u32;
		match def.declarators.first().as_ref().unwrap() {
			syn::Declarator::ParamList(syn::ParamList{param_list, is_variadic}) => {
				let param_types: Vec<u32> = param_list.iter().map(|p| {
					self.resolve_type(p.specifiers.layout.as_ref().unwrap())
				}).collect();
				func_type = self.builder.type_function(ret_type, &param_types).unwrap()
			}
			_ => todo!()
		}
		for decl in def.declaration_list.iter() {
			self.declaration(decl)?;
		}
		self.builder.function_begin(func_type, 0);
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
		self.builder.function_end();
		Ok(0)
	}
}
