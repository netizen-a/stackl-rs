use std::{cell::RefCell, rc::Rc};

use crate::analysis::tok::{Ident, Token, TokenTriple};
use crate::diagnostics::syn;

#[derive(Default)]
pub struct InnerIter {
	pub data: Box<[TokenTriple]>,
	pub pos: usize,
	pub is_typedef: bool,
	typenames: Vec<String>,
}
impl InnerIter {
	pub fn push_type(&mut self, ident: Ident) {
		eprintln!("TYPEDEF: {}", ident.name);
		self.typenames.push(ident.name);
	}
}

impl Iterator for InnerIter {
	type Item = TokenTriple;
	fn next(&mut self) -> Option<TokenTriple> {
		if self.pos == self.data.len() {
			None
		} else {
			let pos = self.pos;
			self.pos += 1;
			Some(self.data[pos].clone())
		}
	}
}

pub struct TokenIter {
	pub inner: Rc<RefCell<InnerIter>>,
}

impl From<Box<[TokenTriple]>> for TokenIter {
	fn from(value: Box<[TokenTriple]>) -> Self {
		let iter = InnerIter {
			data: value,
			pos: 0,
			is_typedef: false,
			typenames: vec![],
		};
		Self {
			inner: Rc::new(RefCell::new(iter)),
		}
	}
}

impl Iterator for TokenIter {
	type Item = syn::ResultTriple<Token, usize>;
	fn next(&mut self) -> Option<Self::Item> {
		self.inner.borrow_mut().next().map(Ok)
	}
}
