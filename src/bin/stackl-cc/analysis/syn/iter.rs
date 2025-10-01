use std::{cell::RefCell, rc::Rc};

use super::Identifier;
use crate::analysis::tok::{self, Token, TokenKind, TokenTriple};
use crate::diagnostics as diag;
use crate::symtab::SymbolTable;

#[derive(Default)]
pub struct TokenIter {
	data: Box<[TokenTriple]>,
	pos: usize,
	is_typedef: bool,
	typename_table: SymbolTable<String, ()>,
}

impl Iterator for TokenIter {
	type Item = TokenTriple;
	fn next(&mut self) -> Option<TokenTriple> {
		if self.pos == self.data.len() {
			return None;
		}

		let pos = self.pos;
		self.pos += 1;
		match &mut self.data[pos].1.kind {
			TokenKind::Ident(ident) => {
				if self.is_typedef {
					self.typename_table
						.insert(ident.name.clone(), ())
						.expect("failed to insert token into symbol table");
				}
				ident.is_type = self.typename_table.global_lookup(&ident.name).is_some();
			}
			TokenKind::Keyword(tok::Keyword::Typedef) => {
				self.is_typedef = true;
			}
			TokenKind::Punct(tok::Punct::SemiColon) => {
				self.is_typedef = false;
			}
			TokenKind::Punct(tok::Punct::LCurly) => {
				self.typename_table.increase_scope();
			}
			TokenKind::Punct(tok::Punct::RCurly) => {
				self.typename_table.decrease_scope();
			}
			_ => {}
		}
		Some(self.data[pos].clone())
	}
}

impl From<Box<[TokenTriple]>> for TokenIter {
	fn from(value: Box<[TokenTriple]>) -> Self {
		Self {
			data: value,
			..Default::default()
		}
	}
}
