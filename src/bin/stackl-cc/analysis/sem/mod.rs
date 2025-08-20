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
	Member{
		tag: String,
		members: Vec<String>
	},
	Ordinary(String),
}

struct MemberType {
	name: String,
	data: DataType,
}

enum DataType {
	Void,
	Bool,
	I8,
	U8,
	I16,
	U16,
	I32,
	U32,
	I64,
	U64,
	I128,
	U128,
	Float,
	Double,
	LongDouble,
	Enum,
	Struct(Vec<MemberType>),
	Union(Vec<MemberType>),
	Array{
		data: Box<DataType>,
		size: u32
	},
	Func,
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
