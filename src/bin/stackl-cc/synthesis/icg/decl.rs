use super::{
	syn,
	Diagnostic,
};

impl super::SSACodeGen<'_> {
	pub(super) fn declaration(&mut self, decl: &syn::Declaration) -> Result<u32, Diagnostic> {
		Ok(0)
	}
}
