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
		for external_decl in unit.as_mut() {
			match external_decl {
				FunctionDefinition(decl) => self.function_definition(decl),
				Declaration(decl) => self.declaration(decl),
			}
		}
	}
	fn function_definition(&mut self, decl: &mut FunctionDefinition) {
		for ref mut specifier in decl.declaration_specifiers.iter_mut() {
			self.declaration_specifier(specifier);
		}
		self.declarator(&mut decl.declarator);
		for declaration in decl.declaration_list.iter_mut() {
			self.declaration(declaration);
		}
		self.compound_stmt(&mut decl.compound_stmt);
	}
}
