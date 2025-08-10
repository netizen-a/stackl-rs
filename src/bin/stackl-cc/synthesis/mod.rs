// intermediate code generation
mod icg;
// optimizer
mod opt;
// output code generation
mod out;
// intermediate data representation
mod idr;

use crate::analysis::syn::ExternalDeclaration;

// TODO: fix Return type
pub fn parse(ast: &[ExternalDeclaration]) {
	let icg = icg::IntermediateCodeGen::new();
	icg.parse(ast)
}
