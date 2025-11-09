use super::{
	syn,
	Diagnostic,
};

impl super::SSACodeGen<'_> {
	pub(super) fn function_definition(
		&mut self,
		decl: &syn::FunctionDefinition,
	) -> Result<u32, Diagnostic> {
		Ok(0)
	}
}
