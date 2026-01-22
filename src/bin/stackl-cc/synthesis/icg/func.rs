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
				self.ordinary_table.insert(def.ident.name.clone(), func_id);
				self.increase_scope();
				self.function_declarations(&def.declaration_list)
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
				self.ordinary_table.insert(def.ident.name.clone(), func_id);
				self.increase_scope();
				self.function_parameters(param_list);
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
		self.decrease_scope();
		self.builder.function_end();
		Ok(())
	}
	fn function_parameters(&mut self, params: &[syn::ParameterDeclaration]) {
		for param in params.iter() {
			let type_id = self.resolve_type(&param.specifiers.layout.as_ref().unwrap());
			let param_id = self.builder.function_parameter(type_id).unwrap();
			if let Some(param_ident) = param.ident.as_ref() {
				self.ordinary_table.insert(param_ident.name.clone(), param_id);
			}
		}
	}
	fn function_declarations(&mut self, decls: &[syn::Declaration]) {
		for decl in decls.iter() {
			let type_id = self.resolve_type(&decl.specifiers.layout.as_ref().unwrap());
			for init_decl in decl.init_declarator_list.iter() {
				let init_decl_id = self.builder.function_parameter(type_id).unwrap();
				self.ordinary_table.insert(init_decl.identifier.name.clone(), init_decl_id);
			}
		}
	}
}
