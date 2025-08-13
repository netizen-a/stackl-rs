pub mod lex;
pub mod syn;
pub mod sem;

use std::result;

pub struct DiagnosticEngine {
	diag_lex: Vec<lex::Diagnostic>,
	diag_syn: Vec<syn::Diagnostic>,
	diag_sem: Vec<sem::Diagnostic>
}

impl DiagnosticEngine {
	pub fn new() -> Self {
		Self {
			diag_lex: vec![],
			diag_syn: vec![],
			diag_sem: vec![],
		}
	}
	pub fn push_lex(&mut self, diag: lex::Diagnostic) {
		self.diag_lex.push(diag)
	}
	pub fn push_syn(&mut self, diag: syn::Diagnostic) {
		self.diag_syn.push(diag)
	}
	pub fn push_sem(&mut self, diag: sem::Diagnostic) {
		self.diag_sem.push(diag)
	}
}

pub type ResultTriple<Tok, Loc, Err> = result::Result<(Loc, Tok, Loc), Err>;
