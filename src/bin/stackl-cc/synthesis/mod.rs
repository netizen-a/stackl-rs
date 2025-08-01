mod icg;
mod opt;
mod out;

use crate::analysis::syn::ExternalDeclaration;

// TODO: fix Return error
pub fn parse(ast: &[ExternalDeclaration]) {
	let icg = icg::IntermediateCodeGen::new();
	icg.parse(ast)
}
