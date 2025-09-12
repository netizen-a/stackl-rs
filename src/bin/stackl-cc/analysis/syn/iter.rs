use std::{cell::RefCell, rc::Rc};

use crate::analysis::tok::{Ident, Token, TokenKind, TokenTriple};
use crate::diagnostics as diag;
use crate::symtab::SymbolTable;

#[derive(Default)]
pub struct InnerIter {
	data: Box<[TokenTriple]>,
	pos: usize,
	pub is_typedef: bool,
	typename_table: SymbolTable<String, Ident>,
}
impl InnerIter {
	fn new(data: Box<[TokenTriple]>) -> Self {
		Self {
			data,
			..Default::default()
		}
	}
	pub fn push_type(&mut self, ident: Ident) {
		self.typename_table
			.insert(ident.name.clone(), ident)
			.expect("failed to insert into token symbol table");
	}
	#[inline]
	pub fn increase_scope(&mut self) {
		self.typename_table.increase_scope();
	}
	#[inline]
	pub fn decrease_scope(&mut self) {
		self.typename_table.decrease_scope();
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
			if let TokenKind::Ident(ident) = &mut self.data[pos].1.kind {
				ident.is_type = self.typename_table.lookup(&ident.name).is_some();
			}
			Some(self.data[pos].clone())
		}
	}
}

pub struct TokenIter {
	pub inner: Rc<RefCell<InnerIter>>,
}

impl From<Box<[TokenTriple]>> for TokenIter {
	fn from(value: Box<[TokenTriple]>) -> Self {
		let iter = InnerIter::new(value);
		Self {
			inner: Rc::new(RefCell::new(iter)),
		}
	}
}

impl Iterator for TokenIter {
	type Item = diag::ResultTriple<Token, usize>;
	fn next(&mut self) -> Option<Self::Item> {
		self.inner.borrow_mut().next().map(Ok)
	}
}
