mod icg;
mod opt;
mod out;

use crate::analysis::syn::ExternalDeclaration;

// TODO: fix Return type
pub fn parse(ast: &[ExternalDeclaration]) {
	println!("{ast:#?}");
	let icg = icg::IntermediateCodeGen::new();
	icg.parse(ast)
}
