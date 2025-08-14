mod decl;
mod expr;
mod stmt;

use crate::analysis::syn::*;
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

pub struct SemanticParser {
	symtab: SymbolTable<Namespace, DataType>,
}

impl SemanticParser {
	pub fn new() -> Self {
		Self {
			symtab: SymbolTable::new(),
		}
	}
	pub fn parse(mut self, mut unit: Vec<ExternalDeclaration>) -> Option<Vec<ExternalDeclaration>> {
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
