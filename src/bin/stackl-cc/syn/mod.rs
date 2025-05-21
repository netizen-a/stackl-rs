use std::{
	iter::Peekable,
	sync::mpsc::{self, Receiver},
};

use crate::diag::syn;
use crate::{ast, tok};

pub struct SyntaxParser {
	iter: Peekable<mpsc::IntoIter<tok::Token>>,
}

impl SyntaxParser {
	pub fn new(rcv_tokens: Receiver<tok::Token>) -> Self {
		Self {
			iter: rcv_tokens.into_iter().peekable(),
		}
	}
}

impl Iterator for SyntaxParser {
	type Item = syn::Result<ast::ExternalDeclaration>;
	fn next(&mut self) -> Option<Self::Item> {
		Some(Err(syn::Error::Unknown))
	}
}
