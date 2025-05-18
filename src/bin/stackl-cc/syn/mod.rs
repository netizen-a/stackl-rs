use std::{result, sync::mpsc::Receiver};

use crate::{
	ast,
	tok::{self, Token},
};

pub enum SyntaxError {
	Unknown,
}

pub type Result<T> = result::Result<T, SyntaxError>;

pub struct SyntaxParser {
	rcv_tokens: Receiver<Token>,
}

impl SyntaxParser {
	pub fn new(rcv_tokens: Receiver<Token>) -> Self {
		Self { rcv_tokens }
	}
	pub fn parse(&self) -> Result<ast::TranslationUnit> {
		let mut iter = self.rcv_tokens.iter().peekable();
		let Some(token) = iter.next() else {
			return Ok(ast::TranslationUnit::default());
		};

		todo!()
	}
}
