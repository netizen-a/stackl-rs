//! Intermediate Code Generation

use crate::analysis::syn;
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
	pub fn build(&mut self, unit: &[syn::ExternalDeclaration]) -> Module {
		for external_decl in unit {
			match external_decl {
				syn::ExternalDeclaration::FunctionDefinition(_) => {}
				syn::ExternalDeclaration::Declaration(_) => {}
				syn::ExternalDeclaration::Pragma(_) => {}
				&syn::ExternalDeclaration::Error => {}
			}
		}
		todo!()
	}
}
