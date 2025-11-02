//! Intermediate Code Generation

mod decl;
mod func;

use crate::analysis::syn;
use crate::diagnostics::{
	DiagKind,
	Diagnostic,
};
use stackl::ssa::build::Builder;
use stackl::ssa::data::Module;

pub struct SSACodeGen {
	builder: Builder,
}

impl SSACodeGen {
	pub fn new() -> Self {
		Self {
			builder: Builder::new(),
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
