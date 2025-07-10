use std::{cell::RefCell, rc::Rc};

use crate::analysis::tok::PPToken;

use super::lexer::Lexer;
use crate::analysis::prt::lex;

pub enum StackKind {
	Buffer(Vec<lex::ResultTriple<PPToken, usize>>),
	Lexer(Lexer),
}

#[derive(Default)]
pub struct PPTokenStack {
	stack: Vec<StackKind>,
}

impl PPTokenStack {
	pub fn push_lexer(&mut self, lexer: Lexer) {
		self.stack.push(StackKind::Lexer(lexer));
	}
	pub fn push_token(&mut self, hi: usize, pp_token: PPToken, lo: usize) {
		match self.stack.last_mut() {
			Some(StackKind::Buffer(buffer)) => buffer.push(Ok((hi, pp_token, lo))),
			_ => {
				let buffer = vec![Ok((hi, pp_token, lo))];
				self.stack.push(StackKind::Buffer(buffer))
			}
		}
	}
	fn next_token(&mut self) -> Option<lex::ResultTriple<PPToken, usize>> {
		while let Some(queue) = self.stack.last_mut() {
			if let StackKind::Buffer(buffer) = queue {
				if let Some(result) = buffer.pop() {
					return Some(result);
				}
			} else if let StackKind::Lexer(lexer) = queue {
				if let Some(result) = lexer.next() {
					return Some(result);
				}
			}
			self.stack.pop();
		}
		None
	}
}

pub struct PPTokenIter {
	pub stack_ref: Rc<RefCell<PPTokenStack>>,
}

impl From<Lexer> for PPTokenIter {
	fn from(value: Lexer) -> Self {
		let pp_token_stack = PPTokenStack {
			stack: vec![StackKind::Lexer(value)],
		};
		Self {
			stack_ref: Rc::new(RefCell::new(pp_token_stack)),
		}
	}
}

impl Iterator for PPTokenIter {
	type Item = lex::ResultTriple<PPToken, usize>;
	fn next(&mut self) -> Option<Self::Item> {
		self.stack_ref.borrow_mut().next_token()
	}
}
