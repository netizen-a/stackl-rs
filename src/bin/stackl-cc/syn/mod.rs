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
	fn declaration_specifiers(&mut self) -> Vec<ast::DeclarationSpecifier> {
		// peeked token must be an identifier or keyword
		// let Some(peeked_token) = self.iter.peek() else {
		// 	return vec![];
		// };

		// if let tok::Token::Keyword(kw) = peeked_token {
		// 	//
		// } else if let tok::Token::Identifier(ident) = peeked_token {
		// 	//
		// }
		if let Some(tok::Token::Keyword(tok::Keyword { term, .. })) =
			self.iter.next_if(tok::Token::is_keyword)
		{
			use tok::KeywordTerminal as Term;
			match term {
				// storage-class-specifiers
				Term::Typedef | Term::Extern | Term::Static | Term::Auto | Term::Register => {
					todo!("storage-class-specifiers")
				}
				// type-specifier
				Term::Void
				| Term::Char
				| Term::Short
				| Term::Int
				| Term::Long
				| Term::Float
				| Term::Double
				| Term::Signed
				| Term::Unsigned
				| Term::Bool
				| Term::Struct
				| Term::Union
				| Term::Enum => {
					todo!("type-specifier")
				}
				// type-qualifier
				Term::Const | Term::Restrict | Term::Volatile => todo!("type-qualifier"),
				Term::Inline => todo!("function-specifier"),
				_ => todo!(),
			}
		} else if let Some(tok::Token::Identifier(_)) = self.iter.next_if(tok::Token::is_identifier)
		{
			todo!("type-specifier")
		}

		todo!()
	}
}

impl Iterator for SyntaxParser {
	type Item = syn::Result<ast::ExternalDeclaration>;
	fn next(&mut self) -> Option<Self::Item> {
		let _decl_specifier = self.declaration_specifiers();
		todo!()
	}
}
