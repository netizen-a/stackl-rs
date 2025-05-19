use std::{
	iter::Peekable,
	result,
	sync::mpsc::{self, Receiver},
};

use crate::{ast, tok};

pub enum SyntaxError {
	Unknown,
}

pub type Result<T> = result::Result<T, SyntaxError>;

pub struct SyntaxParser<'a> {
	iter: Peekable<mpsc::Iter<'a, tok::Token>>,
}

impl<'a> SyntaxParser<'a> {
	pub fn new(rcv_tokens: &'a Receiver<tok::Token>) -> Self {
		Self {
			iter: rcv_tokens.iter().peekable(),
		}
	}
	pub fn parse(&mut self) -> Result<ast::TranslationUnit> {
		let Some(_token) = self.iter.next() else {
			return Ok(ast::TranslationUnit::default());
		};

		todo!()
	}
}
