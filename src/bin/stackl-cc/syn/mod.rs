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
	fn struct_declaration_list(&mut self) -> syn::Result<Option<ast::StructDeclarationList>> {
		todo!()
	}
	fn type_specifier(&mut self, kw: tok::Keyword) -> syn::Result<ast::TypeSpecifier> {
		use tok::KeywordTerminal as Term;
		match kw.term {
			Term::Struct | Term::Union => {
				let identifier = self
					.iter
					.next_if(tok::Token::is_identifier)
					.map(|token| token.unwrap_identifier());
				let struct_declaration_list = self.struct_declaration_list()?;
				let specifier = ast::StructOrUnionSpecifier {
					struct_or_union: kw,
					identifier,
					struct_declaration_list,
				};
				Ok(ast::TypeSpecifier::StructOrUnionSpecifier(specifier))
			}
			Term::Enum => {
				//
				todo!()
			}
			_ => todo!("error"),
		}
	}
	fn declaration_specifier(&mut self) -> syn::Result<Option<ast::DeclarationSpecifier>> {
		if let Some(tok::Token::Keyword(keyword)) = self.iter.next_if(tok::Token::is_keyword) {
			use tok::KeywordTerminal as Term;
			match keyword.term {
				// storage-class-specifiers
				Term::Typedef | Term::Extern | Term::Static | Term::Auto | Term::Register => Ok(
					Some(ast::DeclarationSpecifier::StorageClassSpecifier(keyword)),
				),
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
					Ok(Some(ast::DeclarationSpecifier::TypeSpecifier(
						type_specifier,
					)))
				}
				// type-specifier
				Term::Struct | Term::Union | Term::Enum => {
					let type_specifier = self.type_specifier(keyword)?;
					Ok(Some(ast::DeclarationSpecifier::TypeSpecifier(
						type_specifier,
					)))
				}
				// type-qualifier
				Term::Const | Term::Restrict | Term::Volatile => {
					Ok(Some(ast::DeclarationSpecifier::TypeQualifier(keyword)))
				}
				Term::Inline => Ok(Some(ast::DeclarationSpecifier::FunctionSpecifier(keyword))),
				_ => Ok(None),
			}
		} else if let Some(tok::Token::Identifier(identifier)) =
			self.iter.next_if(tok::Token::is_identifier)
		{
			let typedef_name = ast::TypeSpecifier::TypedefName(identifier);
			Ok(Some(ast::DeclarationSpecifier::TypeSpecifier(typedef_name)))
		} else {
			Ok(None)
		}
	}
}

impl Iterator for SyntaxParser {
	type Item = syn::Result<ast::ExternalDeclaration>;
	fn next(&mut self) -> Option<Self::Item> {
		let mut declaration_specifiers = vec![];
		loop {
			match self.declaration_specifier() {
				Ok(Some(decl_spec)) => declaration_specifiers.push(decl_spec),
				Err(error) => return Some(Err(error)),
				Ok(None) => break,
			}
		}
		todo!()
	}
}
