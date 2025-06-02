use std::{cell::RefCell, rc::Rc};

use crate::tok::Token;

use crate::diag::syn;

#[derive(Default)]
pub struct TokenStack {
	stack: Vec<(usize, Token, usize)>,
}

impl TokenStack {
	fn next_token(&mut self) -> Option<syn::ResultTriple<Token, usize>> {
		self.stack.pop().map(Ok)
	}
}

pub struct TokenIter {
	pub stack_ref: Rc<RefCell<TokenStack>>,
}

impl From<Vec<(usize, Token, usize)>> for TokenIter {
	fn from(value: Vec<(usize, Token, usize)>) -> Self {
		let token_stack = TokenStack { stack: value };
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
