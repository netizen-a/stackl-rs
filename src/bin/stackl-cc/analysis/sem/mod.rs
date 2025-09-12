mod decl;
mod expr;
mod stmt;

use crate::analysis::syn::{self, *};
use crate::data_types::DataType;
use crate::diagnostics::DiagnosticEngine;
use crate::symtab::SymbolTable;

#[derive(PartialEq, Eq, Hash)]
enum Namespace {
	Label(String),
	Tag(String),
	Member { tag: String, member: String },
	Ordinary(String),
}

pub enum StorageDuration {
	Static,
	Auto,
}

pub enum Linkage {
	None,
	External,
	Internal,
}

pub struct SymbolTableEntry {
	pub data_type: DataType,
	pub storage_duration: StorageDuration,
	pub linkage: Linkage,
	pub is_incomplete: bool,
}

pub struct SemanticParser<'a> {
	symtab: SymbolTable<Namespace, SymbolTableEntry>,
	diagnostics: &'a mut DiagnosticEngine,
}

impl<'a> SemanticParser<'a> {
	pub fn new(diagnostics: &'a mut DiagnosticEngine) -> Self {
		Self {
			symtab: SymbolTable::new(),
			diagnostics,
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
				Declaration(decl) => self.declaration(decl),
				Asm(stmt) => (),
				Error => {}
			}
		}
		Some(unit)
	}
}
