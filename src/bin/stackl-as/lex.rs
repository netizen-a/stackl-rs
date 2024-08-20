// Copyright (c) 2024-2026 Jonathan A. Thomason

use logos::{
	Logos,
	SpannedIter,
};

use crate::tok::{
	LexicalError,
	Token,
};

type Spanned<Tok, Loc, Error> = Result<(Loc, Tok, Loc), Error>;

pub(super) struct Lexer<'input> {
	// instead of an iterator over characters, we have a token iterator
	token_stream: SpannedIter<'input, Token>,
}

impl<'input> Lexer<'input> {
	pub fn new(input: &'input str) -> Self {
		Self {
			token_stream: Token::lexer(input).spanned(),
		}
	}
}

impl Iterator for Lexer<'_> {
	type Item = Spanned<Token, usize, LexicalError>;

	fn next(&mut self) -> Option<Self::Item> {
		self.token_stream
			.next()
			.map(|(token, span)| Ok((span.start, token?, span.end)))
	}
}
