mod decl;
mod expr;
mod stmt;

use crate::analysis::syn;

pub struct IntermediateCodeGen {}

impl IntermediateCodeGen {
	pub fn new() -> Self {
		Self {}
	}
	pub fn parse(
		mut self,
		mut unit: Vec<syn::ExternalDeclaration>,
	) -> Result<Vec<syn::ExternalDeclaration>, ()> {
		use syn::ExternalDeclaration::*;
		for external_decl in unit.iter_mut() {
			match external_decl {
				FunctionDefinition(decl) => self.function_definition(decl),
				Declaration(decl) => self.declaration(decl),
			}
		}
		Ok(unit)
	}
}
