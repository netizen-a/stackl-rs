use std::fs;
use std::io;
use std::io::Read;
use std::iter::Peekable;
use std::path::PathBuf;

use super::PPTokenIter;
use crate::analysis::lex::lexer::Lexer;
use crate::analysis::tok::Directive;
use crate::analysis::tok::PPToken;
use crate::analysis::tok::PPTokenKind;
use crate::analysis::tok::Token;
use crate::analysis::tok::TokenTriple;
use crate::diagnostics as diag;
use crate::diagnostics::ToSpan;
use crate::tok;

pub struct TokensParser<'a> {
	diag_engine: &'a mut diag::DiagnosticEngine,
	iter: PPTokenIter,
}

impl<'a> TokensParser<'a> {
	pub fn new(diag_engine: &'a mut diag::DiagnosticEngine, iter: PPTokenIter) -> Self {
		Self {
			diag_engine,
			iter,
		}
	}
	pub fn parse(&mut self) -> Vec<tok::TokenTriple> {
		let mut triple_list = vec![];
		while let Some(result) = self.iter.next() {
			match result {
				Ok(pp_token) => match pp_token.kind {
					PPTokenKind::Directive(directive) => self.exec_directive(directive, pp_token.to_span()),
					PPTokenKind::NewLine(_) | PPTokenKind::Punct(tok::Punct::Hash) => {
						// this branch is handled in the iterator
					}
					_ => {
						if let Some(triple) = self.convert_token(pp_token) {
							triple_list.push(triple);
						}
					}
				},
				Err(error) => self.diag_engine.push(error),
			}
		}
		triple_list
	}
	fn convert_token(&mut self, pp_token: PPToken) -> Option<TokenTriple> {
		let span = pp_token.to_span();
		match pp_token.kind.try_into() {
			Ok(kind) => Some((
				span.loc.0,
				Token {
					kind,
					span: span.clone(),
				},
				span.loc.1,
			)),
			Err(kind) => {
				let diag = diag::Diagnostic::error(kind, span);
				self.diag_engine.push(diag);
				None
			}
		}
	}
	fn exec_directive(&mut self, directive: Directive, span: diag::Span) {
        let mut error_found = false;
        let mut dir_args = vec![];
        while let Some(peeked_result) = self.iter.next() {
            match peeked_result {
                Err(error) => {
                    self.diag_engine.push(error.clone());
                    error_found = true;
                },
                Ok(pp_token) => match pp_token.kind {
                    PPTokenKind::NewLine(_) => {
                        break;
                    }
                    _ => dir_args.push(pp_token)
                }
            }
        }

        if error_found {
            return;
        }

		let maybe_diag = match directive {
            Directive::Line => self.iter.stack_ref.directive_line(dir_args),
			Directive::Include => self.directive_include(dir_args),
			Directive::Define => self.directive_define(dir_args),
			Directive::Undef => self.directive_undef(dir_args),
			Directive::Error => self.directive_error(dir_args, span),
            _ => todo!()
        };
        if let Some(diagnostic) = maybe_diag {
            self.diag_engine.push(diagnostic);
        }
	}
	fn directive_error(&mut self, tokens: Vec<PPToken>, span: diag::Span) -> Option<diag::Diagnostic> {
		let mut error_str = String::new();
		for (index, pp_token) in tokens.iter().enumerate() {
			if index == 0 {
				// omit leading space
				error_str.push_str(&pp_token.kind.to_string());
			} else {
				error_str.push_str(&format!("{pp_token}"));
			}
		}
		let kind = diag::DiagKind::ErrorDirective(error_str);
		Some(diag::Diagnostic::error(kind, span))
	}
	fn directive_define(&mut self, tokens: Vec<PPToken>) -> Option<diag::Diagnostic> {
		let mut tok_iter = tokens.into_iter();
		let Some(PPToken{kind: PPTokenKind::Ident(identifier), span, ..}) = tok_iter.next() else {
			panic!()
		};

		if let Some(PPToken{kind: PPTokenKind::Punct(tok::Punct::LParen), ..}) = tok_iter.next() {
			panic!()
		} else {
			let mut replacement_list: Vec<PPToken> = vec![];
			for pp_tok in tok_iter {
				replacement_list.push(pp_tok);
			}
			self.iter.stack_ref.define_obj_macro(identifier.name, replacement_list, span.clone());
		}

		None
	}
	fn directive_undef(&mut self, tokens: Vec<PPToken>) -> Option<diag::Diagnostic> {
		for (index, token) in tokens.iter().enumerate() {
			if index == 0 {
				if let PPTokenKind::Ident(ident) = &token.kind {
					let name = ident.name.clone();
					let mut stack = &mut self.iter.stack_ref;
					if let Some(error) = stack.undef_macro(name, token.to_span()) {
						self.diag_engine.push(error);
					}
				}
			} else {
				// error
				todo!("undef error")
			}
		}
		None
	}
	fn directive_include(&mut self, tokens: Vec<PPToken>) -> Option<diag::Diagnostic> {
		for (index, token) in tokens.iter().enumerate() {
			let span = token.to_span();
			if index == 0 {
				if let PPTokenKind::HeaderName(header) = &token.kind {
					let origin_path = self.diag_engine.get_file_path(span.file_id).unwrap();
					let header_name = PathBuf::from(token.kind.to_string());
					let full_path = origin_path.parent().unwrap().join(header_name);
					let mut stack = &mut self.iter.stack_ref;
					let Ok(file) = fs::File::open(&full_path) else {
						panic!("fatal error")
					};

					let mut reader = io::BufReader::new(file);
					let mut buf = String::new();
					reader.read_to_string(&mut buf).unwrap();

					let file_id = if let Some(file_id) = self.diag_engine.get_file_id(&full_path) {
						file_id
					} else {
						let file_id = self.diag_engine.id();
						self.diag_engine.insert_file_info(file_id, full_path);
						file_id
					};
					stack.push_lexer(Lexer::new(buf, file_id));
				} else {
					let error = diag::Diagnostic::error(diag::DiagKind::InvalidToken, span);
					return Some(error)
				}
			} else {
				let error = diag::Diagnostic::error(diag::DiagKind::DirectiveIncludeExtraTokens, span);
				return Some(error)
			}
		}
		None
	}
}
