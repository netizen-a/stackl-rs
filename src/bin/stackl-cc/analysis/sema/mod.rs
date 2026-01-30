// Copyright (c) 2024-2026 Jonathan A. Thomason

mod data;
mod decl;
mod expr;
mod func;
mod spec;
mod stmt;

use std::collections::HashSet;

use crate::analysis::syn;
use crate::cli;
use crate::data_type::DataType;
use crate::diagnostics::*;
use crate::symtab as sym;
use crate::synthesis::icg;

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

pub struct SemanticParser<'a> {
	label_table: sym::SymbolTable<String, LabelContext>,
	tag_table: sym::SymbolTable,
	ordinary_table: sym::SymbolTable,
	pub data_layouts: Option<HashSet<icg::DataLayout>>,
	diagnostics: &'a mut DiagnosticEngine,
	is_traced: bool,
	warn_lvl: cli::WarnLevel,
	print_ast: bool,
	tree_builder: ptree::TreeBuilder,
}

impl<'a> SemanticParser<'a> {
	pub fn new(diagnostics: &'a mut DiagnosticEngine, args: &cli::Args) -> Self {
		Self {
			label_table: sym::SymbolTable::new(),
			tag_table: sym::SymbolTable::new(),
			ordinary_table: sym::SymbolTable::new(),
			data_layouts: Some(HashSet::new()),
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
		self.ordinary_table.increase_scope();
	}
	pub(self) fn decrease_scope(&mut self) {
		// TODO: check if any types are incomplete
		// for (tag, entry) in self.tag_table.iter_current_scope().unwrap() {
		// 	if !entry.data_type.is_incomplete() {
		// 		self.data_types.insert()
		// 	}
		// }
		// for (k, v) in self.tag_table.iter_current_scope().unwrap() {
		// 	println!("tag {k}: {v:#?}")
		// }
		self.tag_table.decrease_scope();
		// for (k, v) in self.ordinary_table.iter_current_scope().unwrap() {
		// 	println!("ordinary {k}: {v:#?}")
		// }
		self.ordinary_table.decrease_scope();
	}
	pub fn print_errors(&mut self) {
		self.diagnostics.print_once();
	}
	pub fn contains_error(&self) -> bool {
		self.diagnostics.contains_error()
	}
}

impl Drop for SemanticParser<'_> {
	fn drop(&mut self) {
		self.decrease_scope();
	}
}
