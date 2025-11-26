// Copyright (c) 2024-2025 Jonathan Thomason

use crate::{
	analysis::syn,
	diagnostics::Diagnostic,
};

impl super::SSACodeGen<'_> {
	pub(super) fn statement(&mut self, stmt: &syn::Stmt) -> Result<u32, Diagnostic> {
		Ok(0)
	}
}
