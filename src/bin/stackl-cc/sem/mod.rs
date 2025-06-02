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
	fn function_definition(&mut self, decl: syn::FunctionDefinition) {
		for specifier in decl.declaration_specifiers {
			self.declaration_specifier(specifier);
		}
		self.declarator(decl.declarator);
		for declaration in decl.declaration_list {
			self.declaration(declaration);
		}
		self.compound_stmt(decl.compound_stmt);
	}
	fn declaration(&mut self, _decl: syn::Declaration) {
		todo!("declaration")
	}
}
