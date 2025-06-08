mod decl;
mod expr;
mod stmt;

use crate::ir;
use crate::syn::*;
use stackl::dr;

pub struct SemanticParser {
	builder: ir::ModuleBuilder,
}

impl SemanticParser {
	pub fn new() -> Self {
		Self {
			builder: ir::ModuleBuilder::new(),
		}
	}
	pub fn parse(mut self, unit: Vec<ExternalDeclaration>) -> dr::Module {
		use ExternalDeclaration::*;
		for external_decl in unit {
			match external_decl {
				FunctionDefinition(decl) => self.function_definition(decl),
				Declaration(decl) => self.declaration(decl),
			}
		}
		self.builder.build()
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
}
