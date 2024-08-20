// Copyright (c) 2024-2026 Jonathan A. Thomason

use crate::synthesis::icg::DataLayout;
use crate::synthesis::icg::{
	self,
	FunctionLayout,
};

use super::{
	Diagnostic,
	syn,
};

impl super::SSACodeGen<'_> {
	pub(super) fn function_definition(
		&mut self,
		def: &syn::FunctionDefinition,
	) -> Result<(), Diagnostic> {
		let ret_layout = Box::new(def.specifiers.layout.clone().unwrap());
		match def.declarators.first().as_ref().unwrap() {
			syn::Declarator::IdentList(syn::IdentList { ident_list, .. }) => {
				let func_type = self.resolve_type(&DataLayout::Function(FunctionLayout {
					params: vec![],
					ret: ret_layout,
					is_variadic: true,
				}));
				let func_id = self.builder.function_begin(func_type, 0).unwrap();
				self.table.insert(def.ident.name.clone(), func_id);
				// for decl in def.declaration_list.iter() {
				// 	self.declaration(decl)?;
				// }
			}
			syn::Declarator::ParamList(syn::ParamList {
				param_list,
				is_variadic,
			}) => {
				let params: Vec<DataLayout> = param_list
					.iter()
					.map(|p| p.specifiers.layout.clone().unwrap())
					.collect();
				let func_type = self.resolve_type(&DataLayout::Function(FunctionLayout {
					params,
					ret: ret_layout,
					is_variadic: *is_variadic,
				}));
				let func_id = self.builder.function_begin(func_type, 0).unwrap();
				self.table.insert(def.ident.name.clone(), func_id);
			}
			_ => unreachable!(),
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
		self.builder.function_end();
		Ok(())
	}
}
