// Copyright (c) 2024-2026 Jonathan A. Thomason

use crate::analysis::tok;
use crate::symtab::SymbolTable;

#[derive(Default)]
pub struct TokenIter {
	data: Box<[tok::TokenTriple]>,
	pos: usize,
	is_typedef: bool,
	typename_table: SymbolTable<String, ()>,
}

impl Iterator for TokenIter {
	type Item = tok::TokenTriple;
	fn next(&mut self) -> Option<tok::TokenTriple> {
		if self.pos == self.data.len() {
			return None;
		}

		let pos = self.pos;
		self.pos += 1;
		match &mut self.data[pos].1.kind {
			tok::TokenKind::Ident(ident) => {
				if self.is_typedef {
					self.typename_table
						.insert(ident.name.clone(), ())
						.expect("failed to insert token into symbol table");
				}
				ident.is_type = self.typename_table.global_lookup(&ident.name).is_some();
			}
			tok::TokenKind::Keyword(tok::Keyword::Typedef) => {
				self.is_typedef = true;
			}
			tok::TokenKind::Punct(tok::Punct::SemiColon)
			| tok::TokenKind::Punct(tok::Punct::Equal) => {
				self.is_typedef = false;
			}
			tok::TokenKind::Punct(tok::Punct::LCurly) => {
				self.typename_table.increase_scope();
			}
			tok::TokenKind::Punct(tok::Punct::RCurly) => {
				self.typename_table.decrease_scope();
			}
			_ => {}
		}
		Some(self.data[pos].clone())
	}
}

impl From<Box<[tok::TokenTriple]>> for TokenIter {
	fn from(value: Box<[tok::TokenTriple]>) -> Self {
		Self {
			data: value,
			..Default::default()
		}
	}
}
