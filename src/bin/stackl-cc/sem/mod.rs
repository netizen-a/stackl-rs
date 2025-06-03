mod decl;
mod expr;
mod stmt;

use crate::syn::*;

pub struct SemanticParser {}

impl SemanticParser {
	pub fn new() -> Self {
		Self {}
	}
	pub fn parse(&mut self, unit: Vec<ExternalDeclaration>) {
		for external_decl in unit {
			use ExternalDeclaration::*;
			match external_decl {
				FunctionDefinition(decl) => self.function_definition(decl),
				Declaration(decl) => self.declaration(decl),
			}
		}
	}
	fn function_definition(&mut self, decl: FunctionDefinition) {
		for specifier in decl.declaration_specifiers {
			self.declaration_specifier(specifier);
		}
		self.declarator(decl.declarator);
		for declaration in decl.declaration_list {
			self.declaration(declaration);
		}
		self.compound_stmt(decl.compound_stmt);
	}
	fn declaration(&mut self, decl: Declaration) {
		for specifier in decl.declaration_specifiers {
			self.declaration_specifier(specifier);
		}
		for init_declarator in decl.init_declarator_list {
			self.init_declarator(init_declarator);
		}
	}
}
