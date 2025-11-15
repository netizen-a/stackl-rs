use super::{
	syn,
	Diagnostic,
};

impl super::SSACodeGen<'_> {
	pub(super) fn declaration(&mut self, decl: &syn::Declaration) -> Result<Box<[u32]>, Diagnostic> {
		let type_id = self.resolve_type(decl.specifiers.layout.as_ref().unwrap());
		let storage_class = decl.specifiers.storage.as_ref().unwrap();
		let mut var_id_list = vec![];
		for init_decl in &decl.init_declarator_list {
			let var_id = self.builder.variable(type_id, storage_class.clone(), None);
			var_id_list.push(var_id);
		}
		Ok(var_id_list.into_boxed_slice())
	}
}
