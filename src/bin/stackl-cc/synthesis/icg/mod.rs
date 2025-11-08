//! Intermediate Code Generation

mod data;
mod decl;
mod func;
mod layout;

use std::collections::HashSet;

use crate::analysis::syn;
use crate::diagnostics::{
	DiagKind,
	Diagnostic,
};
pub use layout::*;
use stackl::ssa::build::Builder;
use stackl::ssa::data::Module;

pub struct SSACodeGen {
	builder: Builder,
	data_layouts: HashSet<DataLayout>,
}

impl SSACodeGen {
	pub fn new(data_layouts: HashSet<(StorageClass, DataLayout)>) -> Self {
		for data in data_layouts {
			println!("{data:?}");
		}
		Self {
			builder: Builder::new(),
			data_layouts: HashSet::new(),
		}
	}
	pub fn build(mut self, unit: &[syn::ExternalDeclaration]) -> Result<Module, Diagnostic> {
		for external_decl in unit {
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
		Ok(self.builder.build())
	}
}
