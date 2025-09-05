mod decl;
mod expr;
mod stmt;

use crate::analysis::syn::{self, *};
use crate::diagnostics::DiagnosticEngine;
use crate::symtab::SymbolTable;

#[derive(PartialEq, Eq, Hash)]
enum Namespace {
	Label(String),
	Tag(String),
	Member{
		tag: String,
		member: String
	},
	Ordinary(String),
}

struct MemberType {
	name: String,
	data: DataType,
}

#[derive(Debug, Hash, Clone, PartialEq, Eq)]
enum Scalar {
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
}

#[derive(Debug, Hash, Clone, PartialEq, Eq)]
pub struct Array {
    pub component: Box<DataType>,
    pub length: u32,
}

#[derive(Debug, Hash, Clone, PartialEq, Eq)]
pub struct Pointer{
	pub is_const: bool,
	pub is_volatile: bool,
	pub is_restrict: bool,
	inner: Box<DataType>,
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

#[derive(Debug, Hash, Clone, PartialEq, Eq)]
pub struct FuncType {
	params: Vec<DataType>,
    ret: Box<DataType>,
	is_variadic: bool
}

#[non_exhaustive]
#[derive(Debug, Hash, Clone, PartialEq, Eq)]
pub enum DataType {
	Void,
	Scalar(Scalar),
	Struct(Vec<DataType>),
	Union(Vec<DataType>),
	Enum,
    Function(FuncType),
    Pointer(Pointer),
	Array(Array),
}

pub struct SymbolTableEntry {
    pub data_type: DataType,
    pub storage_duration: StorageDuration,
    pub linkage: Linkage,
	pub is_incomplete: bool
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
				Error => eprintln!("ERROR: external decl error"),
			}
		}
		Some(unit)
	}
}
