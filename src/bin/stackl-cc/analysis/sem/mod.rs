mod decl;
mod expr;
mod stmt;

use crate::analysis::syn::{self, *};
use crate::data_types::DataType;
use crate::diagnostics::DiagnosticEngine;
use crate::symtab::SymbolTable;

#[derive(Debug, PartialEq, Eq, Hash)]
enum Namespace {
	Label(String),
	Tag(String),
	Member { tag: String, member: String },
	Ordinary(String),
}

#[derive(Debug, Clone, Copy)]
pub enum Linkage {
	None,
	External,
	Internal,
}

#[derive(Debug)]
pub struct SymbolTableEntry {
	pub data_type: DataType,
	pub storage: StorageClass,
	pub linkage: Linkage,
	pub is_incomplete: bool,
}

pub struct SemanticParser<'a> {
	symtab: SymbolTable<Namespace, SymbolTableEntry>,
	diagnostics: &'a mut DiagnosticEngine,
	is_traced: bool,
	warn_lvl: crate::WarnLevel,
}

impl<'a> SemanticParser<'a> {
	pub fn new(diagnostics: &'a mut DiagnosticEngine, args: &crate::Args) -> Self {
		Self {
			symtab: SymbolTable::new(),
			diagnostics,
			is_traced: args.is_traced,
			warn_lvl: args.warn_lvl,
		}
	}
	pub fn parse(
		&mut self,
		mut unit: Vec<ExternalDeclaration>,
	) -> Option<Vec<ExternalDeclaration>> {
		use ExternalDeclaration::*;
		for external_decl in unit.iter_mut() {
			match external_decl {
				FunctionDefinition(decl) => self.function_definition(decl),
				Declaration(decl) => self.declaration(decl, StorageClass::Static),
				Asm(stmt) => (),
				Error => {}
			}
		}
		Some(unit)
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
}

impl Drop for SemanticParser<'_> {
	fn drop(&mut self) {
		self.decrease_scope();
	}
}
