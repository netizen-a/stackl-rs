mod data;
mod decl;
mod expr;
mod func;
mod spec;
mod stmt;

use crate::analysis::syn;
use crate::cli;
use crate::data_type::DataType;
use crate::diagnostics::*;
use crate::symbol_table as sym;

#[derive(Clone, Copy, PartialEq, Eq)]
enum DeclType {
	Proto,
	FnDef,
	Decl,
}

#[derive(Clone)]
struct LabelContext {
	pub label: Option<Span>,
	pub gotos: Vec<Span>,
}

pub struct SemanticParser {
	label_table: sym::SymbolTable<String, LabelContext>,
	tag_table: sym::SymbolTable,
	member_table: sym::SymbolTable<Vec<String>>,
	ordinary_table: sym::SymbolTable,
	diagnostics: DiagnosticEngine,
	is_traced: bool,
	warn_lvl: cli::WarnLevel,
	print_ast: bool,
	tree_builder: ptree::TreeBuilder,
}

impl SemanticParser {
	pub fn new(diagnostics: DiagnosticEngine, args: &cli::Args) -> Self {
		Self {
			label_table: sym::SymbolTable::new(),
			tag_table: sym::SymbolTable::new(),
			member_table: sym::SymbolTable::new(),
			ordinary_table: sym::SymbolTable::new(),
			diagnostics,
			is_traced: args.is_traced,
			warn_lvl: args.warn_lvl,
			print_ast: args.ast,
			tree_builder: ptree::TreeBuilder::new("translation-unit".to_string()),
		}
	}
	pub fn parse(
		&mut self,
		mut unit: Vec<syn::ExternalDeclaration>,
	) -> Option<Box<[syn::ExternalDeclaration]>> {
		use syn::ExternalDeclaration::*;
		let mut is_valid = true;
		for external_decl in unit.iter_mut() {
			match external_decl {
				FunctionDefinition(decl) => is_valid &= self.function_definition(decl),
				Declaration(decl) => {
					is_valid &= self.declaration(decl, syn::StorageClass::Static, false)
				}
				Pragma(_) => {}
				Error => {
					self.tree_builder.add_empty_child("error".to_string());
					is_valid &= false;
				}
			}
		}
		match is_valid {
			true => Some(unit.into_boxed_slice()),
			false => None,
		}
	}
	pub fn build_tree(&mut self) -> ptree::item::StringItem {
		self.tree_builder.build()
	}
	pub(self) fn increase_scope(&mut self) {
		self.tag_table.increase_scope();
		self.member_table.increase_scope();
		self.ordinary_table.increase_scope();
	}
	pub(self) fn decrease_scope(&mut self) {
		if self.is_traced {
			let iter = self.ordinary_table.iter_current_scope().unwrap();
			let layer = self.ordinary_table.scope_count();
			for (name, symbol) in iter {
				eprintln!("[TRACE] ordinary table({layer}): {name:?} => {symbol:#?}");
			}
		}
		// TODO: check if any types are incomplete
		self.tag_table.decrease_scope();
		self.member_table.decrease_scope();
		self.ordinary_table.decrease_scope();
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
