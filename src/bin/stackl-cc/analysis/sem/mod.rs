mod data;
mod decl;
mod expr;
mod func;
mod spec;
mod stmt;

use crate::analysis::syn;
use crate::cli;
use crate::data_types::DataType;
use crate::diagnostics::*;
use crate::symtab as sym;

#[derive(Clone, Copy, PartialEq, Eq)]
enum DeclType {
	Proto,
	FnDef,
	Decl,
}

pub struct SemanticParser {
	symtab: sym::SymbolTable,
	diagnostics: DiagnosticEngine,
	is_traced: bool,
	warn_lvl: cli::WarnLevel,
	tree_builder: ptree::TreeBuilder,
}

impl SemanticParser {
	pub fn new(diagnostics: DiagnosticEngine, args: &cli::Args) -> Self {
		Self {
			symtab: sym::SymbolTable::new(),
			diagnostics,
			is_traced: args.is_traced,
			warn_lvl: args.warn_lvl,
			tree_builder: ptree::TreeBuilder::new("translation-unit".to_string()),
		}
	}
	pub fn parse(
		&mut self,
		mut unit: Vec<syn::ExternalDeclaration>,
	) -> Option<Vec<syn::ExternalDeclaration>> {
		use syn::ExternalDeclaration::*;
		let mut is_valid = true;
		for external_decl in unit.iter_mut() {
			match external_decl {
				FunctionDefinition(decl) => is_valid &= self.function_definition(decl),
				Declaration(decl) => is_valid &= self.declaration(decl, syn::StorageClass::Static),
				Error => {
					self.tree_builder.add_empty_child("error".to_string());
					is_valid &= false;
				}
			}
		}
		match is_valid {
			true => Some(unit),
			false => None,
		}
	}
	pub fn build_tree(&mut self) -> ptree::item::StringItem {
		self.tree_builder.build()
	}
	pub(self) fn decrease_scope(&mut self) {
		if self.is_traced {
			let iter = self.symtab.iter_current_scope().unwrap();
			let layer = self.symtab.scope_count();
			for (name, symbol) in iter {
				eprintln!("[TRACE] symbol table({layer}): {name:?} => {symbol:#?}");
			}
		}
		self.symtab.decrease_scope();
	}
	pub fn print_errors(&mut self) {
		self.diagnostics.print_once();
	}
	pub fn contains_error(&self) -> bool {
		self.diagnostics.contains_error()
	}
}

impl Drop for SemanticParser {
	fn drop(&mut self) {
		self.decrease_scope();
	}
}
