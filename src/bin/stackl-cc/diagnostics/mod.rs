pub mod lex;
pub mod sem;
pub mod syn;

use std::result;
use crate::analysis::tok;

use lalrpop_util::ErrorRecovery;

#[derive(Debug)]
pub enum DiagLevel {
	Warning,
	Error,
}

pub struct DiagnosticEngine {
	diag_lex: Vec<lex::Diagnostic>,
	diag_syn: Vec<ErrorRecovery<usize, tok::Token, syn::Diagnostic>>,
	diag_sem: Vec<sem::Diagnostic>,
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
	pub fn push_syn(&mut self, diag: ErrorRecovery<usize, tok::Token, syn::Diagnostic>) {
		self.diag_syn.push(diag)
	}
	pub fn push_sem(&mut self, diag: sem::Diagnostic) {
		self.diag_sem.push(diag)
	}
	pub fn contains_error(&self) -> bool {
		for diag in self.diag_lex.iter() {
			if let DiagLevel::Error = diag.level {
				return true;
			}
		}
		if !self.diag_syn.is_empty() {
			return true;
		}
		for diag in self.diag_sem.iter() {
			if let DiagLevel::Error = diag.level {
				return true;
			}
		}
		false
	}
	pub fn print_errors(&self) {
		for diag in self.diag_lex.iter() {
			if let DiagLevel::Error = diag.level {
				eprintln!("error: {:?}", diag.kind);
			}
		}
		for syn_error in self.diag_syn.iter() {
			eprintln!("error: {:?}", syn_error);
		}
		for diag in self.diag_sem.iter() {
			if let DiagLevel::Error = diag.level {
				eprintln!("error: {:?}", diag.kind);
			}
		}
	}
	pub fn print_warnings(&self) {
		for diag in self.diag_sem.iter() {
			if let DiagLevel::Warning = diag.level {
				eprintln!("warning: {:?}", diag.kind);
			}
		}
	}
}

pub type ResultTriple<Tok, Loc, Err> = result::Result<(Loc, Tok, Loc), Err>;
