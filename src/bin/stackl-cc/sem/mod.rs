mod decl;
mod expr;
mod stmt;

use crate::syn;

pub struct SemanticParser {}

impl SemanticParser {
	pub fn new() -> Self {
		Self {}
	}
	pub fn parse(&mut self, unit: Vec<syn::ExternalDeclaration>) {
		for external_decl in unit {
			match external_decl {
				syn::ExternalDeclaration::FunctionDefinition(decl) => {
					self.function_definition(decl)
				}
				syn::ExternalDeclaration::Declaration(decl) => self.declaration(decl),
			}
		}
	}
	fn function_definition(&mut self, _decl: syn::FunctionDefinition) {
		todo!()
	}
	fn declaration(&mut self, _decl: syn::Declaration) {
		todo!()
	}
}
