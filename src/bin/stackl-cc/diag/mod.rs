pub mod lex;
pub mod syn;

use std::{result, sync::Mutex};

pub struct DiagnosticEngine {
	pub lexical_errors: Mutex<Vec<lex::Error>>,
	pub syntax_errors: Mutex<Vec<syn::Error>>,
}

impl DiagnosticEngine {
	pub fn new() -> Self {
		Self {
			lexical_errors: Mutex::new(vec![]),
			syntax_errors: Mutex::new(vec![]),
		}
	}
	pub fn push_lex(&self, error: lex::Error) {
		let mut guard = self.lexical_errors.lock().unwrap();
		guard.push(error)
	}
	pub fn push_syn(&self, error: syn::Error) {
		let mut guard = self.syntax_errors.lock().unwrap();
		guard.push(error)
	}
}

pub type ResultTriple<Tok, Loc, Err> = result::Result<(Loc, Tok, Loc), Err>;
