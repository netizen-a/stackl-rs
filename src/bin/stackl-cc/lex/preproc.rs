use super::lexer::Lexer;
use super::pp_token_iter::PPTokenQueue;
use crate::cli::PreprocStdout;
use crate::diag::{self, lex};
use crate::tok::{self, Spanned};
use std::collections::HashMap;
use std::io::BufReader;
use std::io::Read;
use std::{fs, io, path};
use tok::PPToken;
use tok::Token;

#[derive(Debug)]
pub struct MacroArgs {
	params: Vec<String>,
	ellipsis: bool,
}

#[derive(Debug)]
pub enum MacroDef {
	Object(Vec<PPToken>),
	Function {
		args: MacroArgs,
		replacement_list: Vec<PPToken>,
	},
}

pub struct Preprocessor<'a> {
	diagnostics: &'a diag::DiagnosticEngine,
	file_map: bimap::BiHashMap<usize, path::PathBuf>,
	macros: HashMap<String, MacroDef>,
	pp_tokens: PPTokenQueue,
	is_newline: bool,
	is_preproc: bool,
	line: usize,
}

impl Iterator for Preprocessor<'_> {
	type Item = tok::Token;
	fn next(&mut self) -> Option<Self::Item> {
		while let Some(result) = self.pp_tokens.next() {
			if let Ok(pp_token) = result {
				match self.tokenize(pp_token) {
					Ok(Some(value)) => return Some(value),
					Err(error) => {
						self.diagnostics.push_lex(error);
					}
					Ok(None) => { /*continue*/ }
				}
			} else if let Err(tok_err) = result {
				self.diagnostics.push_lex(tok_err);
			}
		}
		None
	}
}

impl<'a> Preprocessor<'a> {
	pub fn new<P>(file_path: P, diagnostics: &'a diag::DiagnosticEngine) -> io::Result<Self>
	where
		P: AsRef<path::Path>,
	{
		let mut file_map = bimap::BiHashMap::new();
		file_map.insert(0, file_path.as_ref().to_owned());
		let file = fs::File::open(file_path)?;
		let mut reader = BufReader::new(file);
		let mut buf = String::new();
		reader.read_to_string(&mut buf)?;
		let main_lexer = Lexer::new(buf, 0);
		let mut pp_token_queue = PPTokenQueue::new();
		pp_token_queue.push_lexer(main_lexer);
		Ok(Self {
			diagnostics,
			file_map,
			macros: HashMap::new(),
			pp_tokens: pp_token_queue,
			is_newline: true,
			is_preproc: false,
			line: 1,
		})
	}
	pub fn to_string(&mut self, pp_stdout: PreprocStdout) -> String {
		let mut string_result = String::new();
		while let Some(result) = self.pp_tokens.next() {
			if let Ok(pp_token) = result {
				let is_comment = matches!(pp_token, PPToken::Comment(_));
				if (pp_stdout == PreprocStdout::Token && !is_comment)
					|| pp_stdout == PreprocStdout::TokenComments
				{
					if let PPToken::NewLine(tok::NewLine {
						is_deleted: false, ..
					}) = pp_token
					{
						string_result.push_str(&format!(
							"{} `{}` ",
							pp_token.as_token_name(),
							pp_token.to_name()
						));
					} else if let PPToken::NewLine(tok::NewLine {
						is_deleted: true, ..
					}) = pp_token
					{
						string_result.push_str(&format!(
							"\x1b[9m{} `{}`\x1b[0m ",
							pp_token.as_token_name(),
							pp_token.to_name()
						));
					} else {
						print!("{} `{}` ", pp_token.as_token_name(), pp_token.to_name());
					}
				} else if let tok::PPToken::NewLine(new_line) = &pp_token {
					string_result.push_str(&format!("{new_line}"));
				} else if let (PreprocStdout::PrintComments, tok::PPToken::Comment(comment)) =
					(pp_stdout, &pp_token)
				{
					string_result.push_str(&format!("{comment}"));
				}

				match self.tokenize(pp_token) {
					Ok(Some(value)) => {
						if pp_stdout >= PreprocStdout::Print {
							string_result.push_str(&format!("{value}"))
						}
					}
					Err(error) => {
						self.diagnostics.push_lex(error);
					}
					Ok(None) => { /*continue*/ }
				}
			} else if let Err(tok_err) = result {
				self.diagnostics.push_lex(tok_err);
			}
		}
		string_result
	}
	fn tokenize(&mut self, pp_token: PPToken) -> lex::Result<Option<tok::Token>> {
		match pp_token {
			PPToken::NewLine(token) => {
				self.pp_newline(token);
				Ok(None)
			}
			PPToken::Comment(_token) => Ok(None),
			PPToken::Identifier(token) => {
				if self.is_preproc {
					self.directive(token)?;
					return Ok(None);
				}
				match self.expand_macro(token.clone()) {
					// not a macro
					Ok(false) => {
						if let Ok(kw) = tok::Keyword::try_from(token.clone()) {
							Ok(Some(Token::Keyword(kw)))
						} else {
							Ok(Some(Token::Identifier(token)))
						}
					}
					// is a macro
					Ok(true) => Ok(None),
					// macro had errors
					Err(error) => Err(error),
				}
			}
			PPToken::Punctuator(token) => {
				if let tok::PunctuatorTerminal::Hash = token.term {
					self.is_preproc = self.is_newline;
					self.is_newline = false;
					Ok(None)
				} else {
					Ok(Some(Token::Punctuator(token)))
				}
			}
			PPToken::StringLiteral(token) => {
				self.is_newline = false;
				Ok(Some(Token::StringLiteral(token)))
			}
			PPToken::CharacterConstant(token) => {
				self.is_newline = false;
				Ok(Some(Token::Constant(tok::Constant::Character(token))))
			}
			PPToken::PPNumber(token) => {
				self.is_newline = false;
				let token = Token::try_from(token)?;
				Ok(Some(token))
			}
			PPToken::HeaderName(token) => todo!("header-name = {token:?}"),
		}
	}
	fn pp_newline(&mut self, _token: tok::NewLine) {
		self.is_preproc = false;
		self.is_newline = true;
		self.line += 1;
	}
	fn expand_macro(&mut self, macro_name: tok::Identifier) -> lex::Result<bool> {
		match self.macros.get(&macro_name.name) {
			// not a macro
			None => Ok(false),
			// object-like macro
			Some(MacroDef::Object(replacement_list)) => {
				let mut replacer = replacement_list.clone();
				if let Some(pp_token) = replacer.first_mut() {
					let mut inner_span = pp_token.span();
					let outer_span = macro_name.span();
					inner_span.leading_spaces = outer_span.leading_spaces;
					inner_span.leading_tabs = outer_span.leading_tabs;
					pp_token.set_span(inner_span);
				}

				for pp_token in replacer.into_iter().rev() {
					self.pp_tokens.push_token(pp_token)
				}
				Ok(true)
			}
			// function-like macro
			Some(MacroDef::Function {
				args,
				replacement_list,
			}) => {
				if let Some(Ok(PPToken::Punctuator(tok::Punctuator {
					term: tok::PunctuatorTerminal::LParen,
					..
				}))) = self.pp_tokens.peek()
				{
					self.pp_tokens.next();
				} else if let Some(Err(_)) = self.pp_tokens.peek() {
					self.pp_tokens.next().unwrap()?;
				} else {
					return Ok(false);
				}
				let mut paren_level = 1;
				let mut last_span = macro_name.span();
				let mut arg = vec![];
				let mut param_list = vec![];
				for pp_token in self.pp_tokens.by_ref() {
					match pp_token {
						Ok(PPToken::Punctuator(tok::Punctuator {
							term: tok::PunctuatorTerminal::RParen,
							span,
						})) => {
							if !args.params.is_empty() {
								param_list.push(arg);
								arg = vec![];
							}
							paren_level -= 1;
							last_span = span;
						}
						Ok(PPToken::Punctuator(tok::Punctuator {
							term: tok::PunctuatorTerminal::LParen,
							span,
						})) => {
							paren_level += 1;
							last_span = span;
						}
						Ok(PPToken::Punctuator(tok::Punctuator {
							term: tok::PunctuatorTerminal::Comma,
							span,
						})) => {
							if paren_level == 1 {
								param_list.push(arg);
								arg = vec![];
								last_span = span;
							}
						}
						Ok(pp_token) => {
							last_span = pp_token.span();
							arg.push(pp_token);
						}
						Err(error) => return Err(error),
					}
					if paren_level == 0 {
						break;
					}
				}
				// TODO: handle ellipsis
				if args.params.len() != param_list.len() {
					return Err(lex::Error {
						kind: lex::ErrorKind::InvalidToken,
						span: last_span,
					});
				}
				let mut replacer = replacement_list.clone();
				let mut index = 0;
				while index < replacer.len() {
					if let PPToken::Identifier(ident) = replacer[index].clone() {
						for (param_name, param_arg) in args.params.iter().zip(&mut param_list) {
							if ident.name == *param_name {
								if let Some(pp_token) = param_arg.first_mut() {
									let inner_span = ident.span();
									let mut outer_span = pp_token.span();
									outer_span.leading_spaces = inner_span.leading_spaces;
									outer_span.leading_tabs = inner_span.leading_tabs;
									pp_token.set_span(outer_span);
								}
								if param_arg.is_empty() {
									replacer.remove(index);
								} else {
									replacer.splice(index..=index, param_arg.clone());
									index += param_arg.len();
								}
							}
						}
					}
					index += 1;
				}
				//fixup span
				if let Some(pp_token) = replacer.first_mut() {
					let mut inner_span = pp_token.span();
					let outer_span = macro_name.span();
					inner_span.leading_spaces = outer_span.leading_spaces;
					inner_span.leading_tabs = outer_span.leading_tabs;
					pp_token.set_span(inner_span);
				}

				for pp_token in replacer.into_iter().rev() {
					self.pp_tokens.push_token(pp_token)
				}
				Ok(true)
			}
		}
	}
	fn directive(&mut self, ident: tok::Identifier) -> lex::Result<()> {
		match ident.name.as_str() {
			"define" => self.pp_define(ident.span),
			"undef" => self.pp_undef(ident.span),
			"include" => self.pp_include(ident.span),
			_ => todo!("{} | undefined directive: `{}`", self.line, ident.name),
		}
	}
	fn pp_define(&mut self, last_span: tok::Span) -> lex::Result<()> {
		let pp_token = match self.pp_tokens.next() {
			Some(Ok(token)) => token,
			Some(Err(error)) => return Err(error),
			None => {
				let (_, hi) = last_span.location;
				let span = tok::Span {
					location: (hi + 1, hi + 1),
					file_key: last_span.file_key,
					leading_tabs: 0,
					leading_spaces: 0,
				};
				return Err(lex::Error {
					kind: lex::ErrorKind::UnexpectedEof,
					span,
				});
			}
		};
		let tok::PPToken::Identifier(ident) = pp_token else {
			return Err(lex::Error {
				kind: lex::ErrorKind::InvalidToken,
				span: pp_token.span(),
			});
		};

		let mut args = MacroArgs {
			params: vec![],
			ellipsis: false,
		};
		let mut is_obj = true;

		match self.pp_tokens.peek() {
			Some(Ok(PPToken::Punctuator(tok::Punctuator {
				term: tok::PunctuatorTerminal::LParen,
				..
			}))) => {
				is_obj = false;
				// consume `(`
				self.pp_tokens.next();
				let mut expected_ident = true;
				let mut expected_rparen = false;
				for pp_token in self.pp_tokens.by_ref() {
					match pp_token {
						Ok(PPToken::Identifier(ident)) => {
							if !expected_ident
								|| expected_rparen || args.params.contains(&ident.name)
							{
								return Err(lex::Error {
									kind: lex::ErrorKind::InvalidToken,
									span: ident.span,
								});
							} else {
								args.params.push(ident.name);
								expected_ident = false;
							}
						}
						Ok(PPToken::Punctuator(tok::Punctuator {
							term: tok::PunctuatorTerminal::RParen,
							..
						})) => {
							break;
						}
						Ok(PPToken::Punctuator(tok::Punctuator {
							term: tok::PunctuatorTerminal::Comma,
							span,
						})) => {
							if expected_ident || expected_rparen {
								return Err(lex::Error {
									kind: lex::ErrorKind::InvalidToken,
									span,
								});
							}
							expected_ident = true;
						}
						Ok(PPToken::Punctuator(tok::Punctuator {
							term: tok::PunctuatorTerminal::Ellipsis,
							span,
						})) => {
							if expected_ident || expected_rparen {
								return Err(lex::Error {
									kind: lex::ErrorKind::InvalidToken,
									span,
								});
							}
							expected_rparen = true;
						}

						Ok(other) => {
							return Err(lex::Error {
								kind: lex::ErrorKind::InvalidToken,
								span: other.span(),
							});
						}
						Err(error) => return Err(error),
					}
				}
			}
			Some(Ok(_)) => {}
			Some(Err(_)) => {
				return Err(self.pp_tokens.next().unwrap().unwrap_err());
			}
			None => {
				todo!("eof error");
			}
		}

		let mut replacement_list = vec![];
		while let Some(pp_token) = self.pp_tokens.next() {
			match pp_token {
				Ok(PPToken::NewLine(token)) => {
					self.pp_newline(token);
					break;
				}
				Err(error) => {
					return Err(error);
				}
				Ok(replacement_token) => {
					replacement_list.push(replacement_token);
				}
			}
		}
		let macro_def = if is_obj {
			MacroDef::Object(replacement_list)
		} else {
			MacroDef::Function {
				args,
				replacement_list,
			}
		};
		self.macros.insert(ident.name, macro_def);
		Ok(())
	}
	fn pp_undef(&mut self, last_span: tok::Span) -> lex::Result<()> {
		let pp_token = match self.pp_tokens.next() {
			Some(Ok(pp_token)) => pp_token,
			Some(Err(error)) => return Err(error),
			None => {
				let (_, hi) = last_span.location;
				let span = tok::Span {
					location: (hi + 1, hi + 1),
					file_key: last_span.file_key,
					leading_tabs: 0,
					leading_spaces: 0,
				};
				return Err(lex::Error {
					kind: lex::ErrorKind::UnexpectedEof,
					span,
				});
			}
		};
		let tok::PPToken::Identifier(ident) = pp_token else {
			return Err(lex::Error {
				kind: lex::ErrorKind::InvalidToken,
				span: pp_token.span(),
			});
		};
		let _ = self.macros.remove(ident.name.as_str());
		while let Some(pp_token) = self.pp_tokens.next() {
			match pp_token {
				Ok(PPToken::NewLine(newline)) => {
					self.pp_newline(newline);
					break;
				}
				Err(error) => return Err(error),
				_ => {}
			}
		}

		Ok(())
	}
	fn pp_include(&mut self, last_span: tok::Span) -> lex::Result<()> {
		self.is_preproc = false;
		self.is_newline = true;
		let header = match self.pp_tokens.next() {
			Some(Ok(PPToken::HeaderName(header))) => header,
			Some(Ok(token)) => {
				let lex_err = lex::Error {
					kind: lex::ErrorKind::InvalidToken,
					span: token.span(),
				};
				return Err(lex_err);
			}
			Some(Err(error)) => return Err(error),
			None => {
				let lex_err = lex::Error {
					kind: lex::ErrorKind::UnexpectedEof,
					span: last_span,
				};
				return Err(lex_err);
			}
		};
		if header.is_std {
			todo!()
		} else {
			let header_span = header.span();
			let origin_path = self.file_map.get_by_left(&0).unwrap();
			let header_path: path::PathBuf = path::PathBuf::from(header.name);
			let full_path = origin_path.parent().unwrap().join(header_path);
			let file = fs::File::open(&full_path).map_err(|_| lex::Error {
				span: header_span.clone(),
				kind: lex::ErrorKind::HeaderNameError,
			})?;
			let mut reader = BufReader::new(file);
			let mut buf = String::new();
			reader.read_to_string(&mut buf).map_err(|_| lex::Error {
				span: header_span,
				kind: lex::ErrorKind::HeaderNameError,
			})?;
			drop(reader);

			let file_key = if let Some(file_key) = self.file_map.get_by_right(&full_path) {
				*file_key
			} else {
				let file_key = self.file_map.len();
				self.file_map.insert(file_key, full_path);
				file_key
			};

			let header_lexer = Lexer::new(buf, file_key);
			self.pp_tokens.push_lexer(header_lexer);
			Ok(())
		}
	}
}
