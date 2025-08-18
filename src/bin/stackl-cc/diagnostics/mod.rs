mod kind;
pub mod lex;
mod sem;

use crate::analysis::tok;
use std::result;

use lalrpop_util::ErrorRecovery;

pub use kind::*;

#[derive(Debug)]
pub enum DiagLevel {
	Warning,
	Error,
}

#[derive(Debug)]
pub struct Diagnostic {
	pub level: DiagLevel,
	pub kind: kind::DiagKind,
	pub loc: (usize, usize),
}

impl Diagnostic {
	pub fn error(kind: kind::DiagKind, loc: (usize, usize)) -> Self {
		Self {
			level: DiagLevel::Error,
			kind,
			loc,
		}
	}
	pub fn warn(kind: kind::DiagKind, loc: (usize, usize)) -> Self {
		Self {
			level: DiagLevel::Warning,
			kind,
			loc,
		}
	}
}

#[derive(Default)]
pub struct DiagnosticEngine {
	diag_lex: Vec<Diagnostic>,
	diag_syn: Vec<ErrorRecovery<usize, tok::Token, Diagnostic>>,
	diag_sem: Vec<Diagnostic>,
}

impl DiagnosticEngine {
	pub fn new() -> Self {
		Self::default()
	}
	pub fn push_lex(&mut self, diag: Diagnostic) {
		self.diag_lex.push(diag)
	}
	pub fn push_syn(&mut self, diag: ErrorRecovery<usize, tok::Token, Diagnostic>) {
		self.diag_syn.push(diag)
	}
	pub fn push_sem(&mut self, diag: Diagnostic) {
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
				lex::print_error(diag)
			}
		}
		for syn_error in self.diag_syn.iter() {
			eprintln!("error: {:?}", syn_error);
		}
		for diag in self.diag_sem.iter() {
			if let DiagLevel::Error = diag.level {
				sem::print_error(diag)
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

pub type ResultTriple<Tok, Loc> = result::Result<(Loc, Tok, Loc), Diagnostic>;
pub type Result<T> = result::Result<T, Diagnostic>;
