// Copyright (c) 2024-2026 Jonathan A. Thomason

//! Intermediate Code Generation

mod data;
mod decl;
mod expr;
mod func;
mod layout;
mod stmt;

use std::collections::{
	HashMap,
	HashSet,
};

use crate::analysis::syn;
use crate::diagnostics::{
	DiagKind,
	Diagnostic,
	DiagnosticEngine,
};
use crate::symtab::SymbolTable;
pub use layout::*;
use stackl::ssa::{
	builder::Builder,
	data::Module,
};

#[derive(Debug)]
pub struct IrContext {
	pub layouts: HashSet<DataLayout>,
	pub unit: Box<[syn::ExternalDeclaration]>,
}

pub struct SSACodeGen<'a> {
	builder: Builder,
	type_map: HashMap<DataLayout, u32>,
	table: SymbolTable<String, u32>,
	diag_engine: &'a mut DiagnosticEngine,
	is_traced: bool,
}

impl<'a> SSACodeGen<'a> {
	pub fn new(diag_engine: &'a mut DiagnosticEngine, is_traced: bool) -> Self {
		Self {
			builder: Builder::new(),
			type_map: HashMap::new(),
			table: SymbolTable::new(),
			diag_engine,
			is_traced,
		}
	}
	pub fn build(mut self, input: IrContext) -> Result<Module, Diagnostic> {
		self.parse_types(input.layouts);
		for external_decl in input.unit.iter() {
			match external_decl {
				syn::ExternalDeclaration::FunctionDefinition(inner) => {
					self.function_definition(inner)?;
				}
				syn::ExternalDeclaration::Declaration(inner) => {
					self.declaration(inner)?;
				}
				syn::ExternalDeclaration::Pragma(_) => {
					todo!()
				}
				&syn::ExternalDeclaration::Error => {
					const kind: DiagKind = DiagKind::Internal("external declaration error");
					return Err(Diagnostic::fatal(kind, None));
				}
			}
		}
		// println!("{:#?}", self.builder);
		Ok(self.builder.build())
	}
}
