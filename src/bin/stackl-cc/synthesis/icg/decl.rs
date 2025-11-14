use super::{
	syn,
	Diagnostic,
};

impl super::SSACodeGen<'_> {
	pub(super) fn declaration(&mut self, decl: &syn::Declaration) -> Result<u32, Diagnostic> {
		let type_id = self.resolve_type(decl.specifiers.layout.as_ref().unwrap());
		let storage_class = decl.specifiers.storage.as_ref().unwrap();
		let var_id = self.builder.variable(type_id, storage_class.clone(), None);
		for init_decl in &decl.init_declarator_list {}
		Ok(var_id)
	}
}
