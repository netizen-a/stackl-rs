// Copyright (c) 2024-2026 Jonathan A. Thomason

use super::{
	Diagnostic,
	syn,
};

impl super::SSACodeGen<'_> {
	pub(super) fn declaration(
		&mut self,
		decl: &syn::Declaration,
	) -> Result<Box<[u32]>, Diagnostic> {
		let type_id = self.resolve_type(decl.specifiers.layout.as_ref().unwrap());
		let storage_class = decl.specifiers.storage.as_ref().unwrap();
		let mut var_id_list = vec![];
		for init_decl in &decl.init_declarator_list {
			let init_id = init_decl.initializer.as_ref().map(|i| self.initializer(i));
			let var_id = self
				.builder
				.variable(type_id, storage_class.clone(), init_id);
			var_id_list.push(var_id);
		}
		Ok(var_id_list.into_boxed_slice())
	}
	fn initializer(&mut self, initializer: &syn::Initializer) -> u32 {
		match initializer {
			syn::Initializer::Expr(inner) => self.expr(inner),
			syn::Initializer::InitializerList(syn::InitializerList { list, .. }) => {
				for (_designator, initializer) in list.iter() {
					self.initializer(initializer);
				}
				todo!()
			}
		}
	}
}
