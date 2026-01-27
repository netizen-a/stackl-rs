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
	label_table: SymbolTable<String, u32>,
	tag_table: SymbolTable<String, u32>,
	ordinary_table: SymbolTable<String, u32>,
	diag_engine: &'a mut DiagnosticEngine,
	is_traced: bool,
}

impl<'a> SSACodeGen<'a> {
	pub fn new(diag_engine: &'a mut DiagnosticEngine, is_traced: bool) -> Self {
		Self {
			builder: Builder::new(),
			type_map: HashMap::new(),
			label_table: SymbolTable::new(),
			tag_table: SymbolTable::new(),
			ordinary_table: SymbolTable::new(),
			diag_engine,
			is_traced,
		}
	}
	pub(self) fn increase_scope(&mut self) {
		self.label_table.increase_scope();
		self.tag_table.increase_scope();
		self.ordinary_table.increase_scope();
	}
	pub(self) fn decrease_scope(&mut self) {
		self.label_table.decrease_scope();
		self.tag_table.decrease_scope();
		self.ordinary_table.decrease_scope();
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
