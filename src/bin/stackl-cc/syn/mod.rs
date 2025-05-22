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
	fn type_specifier(&mut self, _kw: tok::Keyword) -> Option<ast::TypeSpecifier> {
		todo!()
	}
	fn declaration_specifier(&mut self) -> Option<ast::DeclarationSpecifier> {
		if let Some(tok::Token::Keyword(keyword)) = self.iter.next_if(tok::Token::is_keyword) {
			use tok::KeywordTerminal as Term;
			match keyword.term {
				// storage-class-specifiers
				Term::Typedef | Term::Extern | Term::Static | Term::Auto | Term::Register => {
					Some(ast::DeclarationSpecifier::StorageClassSpecifier(keyword))
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
				| Term::Bool => {
					let type_specifier = ast::TypeSpecifier::Keyword(keyword);
					Some(ast::DeclarationSpecifier::TypeSpecifier(type_specifier))
				}
				// type-specifier
				Term::Struct | Term::Union | Term::Enum => {
					let type_specifier = self.type_specifier(keyword)?;
					Some(ast::DeclarationSpecifier::TypeSpecifier(type_specifier))
				}
				// type-qualifier
				Term::Const | Term::Restrict | Term::Volatile => {
					Some(ast::DeclarationSpecifier::TypeQualifier(keyword))
				}
				Term::Inline => Some(ast::DeclarationSpecifier::FunctionSpecifier(keyword)),
				_ => None,
			}
		} else if let Some(tok::Token::Identifier(identifier)) =
			self.iter.next_if(tok::Token::is_identifier)
		{
			let typedef_name = ast::TypeSpecifier::TypedefName(identifier);
			Some(ast::DeclarationSpecifier::TypeSpecifier(typedef_name))
		} else {
			None
		}
	}
}

impl Iterator for SyntaxParser {
	type Item = syn::Result<ast::ExternalDeclaration>;
	fn next(&mut self) -> Option<Self::Item> {
		let mut declaration_specifiers = vec![];
		while let Some(decl_spec) = self.declaration_specifier() {
			declaration_specifiers.push(decl_spec);
		}
		todo!()
	}
}
