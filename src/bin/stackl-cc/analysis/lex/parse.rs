// Copyright (c) 2024-2026 Jonathan A. Thomason

use std::fs;
use std::io;
use std::io::Read;
use std::iter::Peekable;
use std::path::PathBuf;
use std::vec;

use super::PPTokenIter;
use crate::analysis::lex::lexer::Lexer;
use crate::analysis::tok::Directive;
use crate::analysis::tok::PPToken;
use crate::analysis::tok::PPTokenKind;
use crate::analysis::tok::Token;
use crate::analysis::tok::TokenTriple;
use crate::cli;
use crate::diagnostics as diag;
use crate::diagnostics::ToSpan;
use crate::tok;

pub struct TokensParser<'a> {
	diag_engine: &'a mut diag::DiagnosticEngine,
	iter: PPTokenIter,
	stdout_preproc: bool,
	triple_list: Vec<TokenTriple>,
	warn_lvl: cli::WarnLevel,
}

impl<'a> TokensParser<'a> {
	pub fn new(
		diag_engine: &'a mut diag::DiagnosticEngine,
		iter: PPTokenIter,
		stdout_preproc: bool,
		warn_lvl: cli::WarnLevel,
	) -> Self {
		Self {
			diag_engine,
			iter,
			stdout_preproc,
			triple_list: vec![],
			warn_lvl,
		}
	}
	pub fn parse(&mut self) -> Vec<tok::TokenTriple> {
		while let Some(result) = self.iter.next() {
			match result {
				Ok(pp_token) => match pp_token.kind {
					PPTokenKind::Directive(directive) => {
						if let Some(diag::DiagLevel::Fatal) =
							self.exec_directive(directive, pp_token.to_span())
						{
							// fatal error was encountered.
							break;
						}
					}
					PPTokenKind::NewLine(_) => {
						if self.stdout_preproc {
							println!();
						}
					}
					PPTokenKind::Punct(tok::Punct::Hash) => {
						// this branch is handled in the iterator
					}
					_ => {
						if let Some(triple) = self.convert_token(pp_token) {
							self.triple_list.push(triple);
						}
					}
				},
				Err(error) => self.diag_engine.push(error),
			}
		}
		self.triple_list.drain(..).collect()
	}
	fn convert_token(&mut self, pp_token: PPToken) -> Option<TokenTriple> {
		if self.stdout_preproc {
			print!("{pp_token}");
		}
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
	fn exec_directive(
		&mut self,
		directive: Directive,
		span: diag::Span,
	) -> Option<diag::DiagLevel> {
		let mut error_found = None;
		let mut dir_args = vec![];
		while let Some(peeked_result) = self.iter.next() {
			match peeked_result {
				Err(error) => {
					error_found = Some(error.level);
					self.diag_engine.push(error.clone());
				}
				Ok(pp_token) => match pp_token.kind {
					PPTokenKind::NewLine(_) => {
						break;
					}
					_ => dir_args.push(pp_token),
				},
			}
		}

		if error_found.is_some() {
			return error_found;
		}

		let maybe_diag = match directive {
			Directive::Line => self.iter.stack_ref.directive_line(dir_args),
			Directive::Include => self.directive_include(dir_args),
			Directive::Define => self.directive_define(dir_args),
			Directive::Undef => self.directive_undef(dir_args),
			Directive::Error => self.directive_error(dir_args, span),
			Directive::Pragma => self.directive_pragma(dir_args, span),
			_ => todo!(),
		};
		if let Some(diagnostic) = maybe_diag {
			error_found = Some(diagnostic.level);
			self.diag_engine.push(diagnostic);
		}
		error_found
	}
	fn directive_pragma(
		&mut self,
		tokens: Vec<PPToken>,
		span: diag::Span,
	) -> Option<diag::Diagnostic> {
		let mut iter = tokens.into_iter();
		let Some(pragma_namespace_token) = iter.next() else {
			if let cli::WarnLevel::All = self.warn_lvl {
				let warning = diag::Diagnostic::warn(diag::DiagKind::PragmaIgnored, span);
				self.diag_engine.push(warning);
			}
			return None;
		};
		let span = pragma_namespace_token.to_span();
		let PPTokenKind::Ident(tok::Ident {
			name: pragma_namespace,
			..
		}) = pragma_namespace_token.kind
		else {
			if let cli::WarnLevel::All = self.warn_lvl {
				let warning = diag::Diagnostic::warn(diag::DiagKind::PragmaIgnored, span.to_span());
				self.diag_engine.push(warning);
			}
			return None;
		};
		match pragma_namespace.as_str() {
			"STDC" => self.pragma_stdc(iter, span),
			"STACKL" => self.pragma_stackl(iter),
			_ => {
				if let cli::WarnLevel::All = self.warn_lvl {
					let warning = diag::Diagnostic::warn(diag::DiagKind::PragmaIgnored, span);
					self.diag_engine.push(warning);
				}
				None
			}
		}
	}
	fn pragma_stdc(
		&mut self,
		mut iter: vec::IntoIter<PPToken>,
		span: diag::Span,
	) -> Option<diag::Diagnostic> {
		// TODO:
		// #pragma STDC FP_CONTRACT on-off-switch
		// #pragma STDC FENV_ACCESS on-off-switch
		// #pragma STDC CX_LIMITED_RANGE on-off-switch
		let Some(pragma_kind_token) = iter.next() else {
			if let cli::WarnLevel::All = self.warn_lvl {
				let warning = diag::Diagnostic::warn(diag::DiagKind::PragmaIgnored, span);
				self.diag_engine.push(warning);
			}
			return None;
		};
		let span = pragma_kind_token.to_span();
		let PPTokenKind::Ident(tok::Ident {
			name: pragma_kind, ..
		}) = pragma_kind_token.kind
		else {
			if let cli::WarnLevel::All = self.warn_lvl {
				let warning = diag::Diagnostic::warn(diag::DiagKind::PragmaIgnored, span);
				self.diag_engine.push(warning);
			}
			return None;
		};
		match pragma_kind.as_str() {
			"FP_CONTRACT" => {
				todo!("FP_CONTRACT")
			}
			"FENV_ACCESS" => {
				todo!("FENV_ACCESS")
			}
			"CX_LIMITED_RANGE" => {
				if let cli::WarnLevel::All = self.warn_lvl {
					let warning =
						diag::Diagnostic::warn(diag::DiagKind::PragmaCxLimitedRange, span);
					self.diag_engine.push(warning);
				}
				None
			}
			// unrecognized pragmas are ignored
			_ => {
				if let cli::WarnLevel::All = self.warn_lvl {
					let warning = diag::Diagnostic::warn(diag::DiagKind::PragmaIgnored, span);
					self.diag_engine.push(warning);
				}
				None
			}
		}
	}
	fn pragma_stackl(&mut self, mut iter: vec::IntoIter<PPToken>) -> Option<diag::Diagnostic> {
		// TODO:
		// #pragma STACKL STACK_SIZE integer-constant
		// #pragma STACKL FEATURE identifier on-off-switch
		// #pragma STACKL SECTION string-literal
		// #pragma STACKL TRACE on-off-switch
		// #pragma STACKL VERSION integer-constant
		let Some(pragma_kind_token) = iter.next() else {
			// unrecognized pragmas are ignored
			return None;
		};
		let PPTokenKind::Ident(tok::Ident {
			name: pragma_kind, ..
		}) = pragma_kind_token.kind
		else {
			// unrecognized pragmas are ignored
			return None;
		};
		match pragma_kind.as_str() {
			"STACK_SIZE" => {
				todo!("STACK_SIZE")
			}
			"FEATURE" => {
				todo!("FEATURE")
			}
			"SECTION" => {
				todo!("SECTION")
			}
			"TRACE" => {
				todo!("TRACE")
			}
			"VERSION" => {
				todo!("VERSION")
			}
			_ => None,
		}
	}
	fn directive_error(
		&mut self,
		tokens: Vec<PPToken>,
		span: diag::Span,
	) -> Option<diag::Diagnostic> {
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
		let Some(PPToken {
			kind: PPTokenKind::Ident(identifier),
			span,
			..
		}) = tok_iter.next()
		else {
			panic!()
		};

		if let Some(PPToken {
			kind: PPTokenKind::Punct(tok::Punct::LParen),
			..
		}) = tok_iter.next()
		{
			panic!()
		} else {
			let mut replacement_list: Vec<PPToken> = vec![];
			for pp_tok in tok_iter {
				replacement_list.push(pp_tok);
			}
			self.iter
				.stack_ref
				.define_obj_macro(identifier.name, replacement_list, span.clone());
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
					let full_path = origin_path.parent().unwrap().join(&header_name);
					let mut stack = &mut self.iter.stack_ref;

					let (file_id, file_data) =
						if let Some(file_id) = self.diag_engine.get_file_id(&full_path) {
							(file_id, self.diag_engine.get_file_data(file_id))
						} else {
							let file_id = self.diag_engine.id();
							(
								file_id,
								self.diag_engine.insert_file_info(file_id, &full_path).ok(),
							)
						};
					if let Some(buf) = file_data {
						stack.push_lexer(Lexer::new(buf.to_string(), file_id));
					} else {
						let error = diag::Diagnostic::fatal(
							diag::DiagKind::FileNotFound(header_name),
							Some(token.to_span()),
						);
						return Some(error);
					}
				} else {
					let error = diag::Diagnostic::error(diag::DiagKind::InvalidToken, span);
					return Some(error);
				}
			} else {
				let error =
					diag::Diagnostic::error(diag::DiagKind::DirectiveIncludeExtraTokens, span);
				return Some(error);
			}
		}
		None
	}
}
