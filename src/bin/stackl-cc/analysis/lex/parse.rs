use std::iter::Peekable;
use std::path::PathBuf;

use super::PPTokenIter;
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
					PPTokenKind::Directive(directive) => self.exec_directive(directive),
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
	fn exec_directive(&mut self, directive: Directive) {
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
            _ => todo!()
        };
        if let Some(diagnostic) = maybe_diag {
            self.diag_engine.push(diagnostic);
        }
	}
}
