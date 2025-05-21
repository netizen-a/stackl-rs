pub mod lex;
pub mod syn;

use std::sync::Mutex;

pub struct DiagnosticEngine {
	lexical_errors: Mutex<Vec<lex::Error>>,
	syntax_errors: Mutex<Vec<syn::Error>>,
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
