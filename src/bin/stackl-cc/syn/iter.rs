use std::{cell::RefCell, rc::Rc};

use crate::tok::Token;

use crate::diag::syn;

#[derive(Default)]
pub struct TokenStack {
	stack: Box<[(usize, Token, usize)]>,
	index: usize,
}

impl TokenStack {
	fn next_token(&mut self) -> Option<syn::ResultTriple<Token, usize>> {
		if self.index == self.stack.len() {
			None
		} else {
			let index = self.index;
			self.index += 1;
			Some(Ok(self.stack[index].clone()))
		}
	}
}

pub struct TokenIter {
	pub stack_ref: Rc<RefCell<TokenStack>>,
}

impl From<Box<[(usize, Token, usize)]>> for TokenIter {
	fn from(value: Box<[(usize, Token, usize)]>) -> Self {
		let token_stack = TokenStack {
			stack: value,
			index: 0,
		};
		Self {
			stack_ref: Rc::new(RefCell::new(token_stack)),
		}
	}
}

impl Iterator for TokenIter {
	type Item = syn::ResultTriple<Token, usize>;
	fn next(&mut self) -> Option<Self::Item> {
		self.stack_ref.borrow_mut().next_token()
	}
}
