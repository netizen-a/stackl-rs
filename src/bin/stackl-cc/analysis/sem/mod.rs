mod decl;
mod expr;
mod stmt;

use crate::analysis::syn::*;
use crate::diagnostics::DiagnosticEngine;
use crate::symtab::SymbolTable;

#[derive(PartialEq, Eq, Hash)]
enum Namespace {
	Label(String),
	Tag(String),
	Member(Vec<String>),
	Ordinary(String),
}

enum DataType {
	Bool,
	Int,
	LongInt,
	LongLongInt,
	Float,
	Double,
	LongDouble,
	Enum,
	Struct,
	Union,
}

pub struct SemanticParser<'a> {
	symtab: SymbolTable<Namespace, DataType>,
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
				Error => todo!("external decl error"),
			}
		}
		Some(unit)
	}
}
