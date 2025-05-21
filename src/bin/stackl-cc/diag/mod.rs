mod lex;

use std::sync::Mutex;

pub use lex::*;

pub struct DiagnosticEngine {
	lex_errors: Mutex<Vec<LexicalError>>,
}

impl DiagnosticEngine {
	pub fn new() -> Self {
		Self {
			lex_errors: Mutex::new(vec![]),
		}
	}
	pub fn push_lex(&mut self, error: LexicalError) {
		let mut guard = self.lex_errors.lock().unwrap();
		guard.push(error);
	}
}
