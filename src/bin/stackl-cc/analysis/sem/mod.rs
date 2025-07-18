mod decl;
mod expr;
mod stmt;

use crate::analysis::syn::*;

pub struct SemanticParser {}

impl SemanticParser {
	pub fn new() -> Self {
		Self {}
	}
	pub fn parse(mut self, unit: &mut [ExternalDeclaration]) {
		use ExternalDeclaration::*;
		for external_decl in unit {
			match external_decl {
				FunctionDefinition(decl) => self.function_definition(decl),
				Declaration(decl) => self.declaration(decl),
			}
		}
	}
}
