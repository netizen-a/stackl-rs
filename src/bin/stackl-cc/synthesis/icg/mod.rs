//! Intermediate Code Generation

use crate::analysis::syn;
use crate::diagnostics::{DiagKind, Diagnostic};
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
	pub fn build(self, unit: &[syn::ExternalDeclaration]) -> Result<Module, Diagnostic> {
		for external_decl in unit {
			let result = match external_decl {
				syn::ExternalDeclaration::FunctionDefinition(_) => {todo!()}
				syn::ExternalDeclaration::Declaration(_) => {todo!()}
				syn::ExternalDeclaration::Pragma(_) => {todo!()}
				&syn::ExternalDeclaration::Error => {
					let kind = DiagKind::Internal("external declaration error".to_string());
					Err(Diagnostic::fatal(kind, None))
				}
			};
			if result.is_err() {
				return result;
			}
		}
		Ok(self.builder.build())
	}
}
