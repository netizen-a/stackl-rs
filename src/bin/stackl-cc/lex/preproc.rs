use super::error::*;
use super::lexer as lex;
use super::pp_token_iter::PPTokenQueue;
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

pub struct Preprocessor {
	file_map: bimap::BiHashMap<usize, path::PathBuf>,
	macros: HashMap<String, MacroDef>,
	pp_tokens: PPTokenQueue,
	stdout: i32,
	is_newline: bool,
	is_preproc: bool,
	line: usize,
}

impl Iterator for Preprocessor {
	type Item = Result<tok::Token, LexicalError>;
	fn next(&mut self) -> Option<Self::Item> {
		while let Some(result) = self.pp_tokens.next() {
			match result {
				Ok(pp_token) => match self.tokenize(pp_token) {
					Ok(Some(value)) => return Some(Ok(value)),
					Err(error) => return Some(Err(error)),
					Ok(None) => { /*continue*/ }
				},
				Err(tok_err) => return Some(Err(tok_err)),
			}
		}
		None
	}
}

impl Preprocessor {
	pub fn new<P>(file_path: P, stdout: i32) -> io::Result<Self>
	where
		P: AsRef<path::Path>,
	{
		let mut file_map = bimap::BiHashMap::new();
		file_map.insert(0, file_path.as_ref().to_owned());
		let file = fs::File::open(file_path)?;
		let mut reader = BufReader::new(file);
		let mut buf = String::new();
		reader.read_to_string(&mut buf)?;
		let main_lexer = lex::Lexer::new(buf, 0);
		let mut pp_token_queue = PPTokenQueue::new();
		pp_token_queue.push_lexer_front(main_lexer);
		Ok(Self {
			file_map,
			macros: HashMap::new(),
			pp_tokens: pp_token_queue,
			stdout,
			is_newline: true,
			is_preproc: false,
			line: 1,
		})
	}
	fn tokenize(&mut self, pp_token: PPToken) -> Result<Option<tok::Token>, LexicalError> {
		match pp_token {
			PPToken::NewLine(token) => {
				self.pp_newline(token);
				Ok(None)
			}
			PPToken::Comment(token) => {
				if self.stdout > 1 {
					print!("{token}");
				}
				Ok(None)
			}
			PPToken::Identifier(token) => {
				if self.is_preproc {
					self.directive(token)?;
					return Ok(None);
				}
				match self.expand_macro(token.clone()) {
					// not a macro
					Ok(false) => {
						if self.stdout > 0 {
							print!("{token}");
						}
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
					if self.stdout > 0 {
						print!("{token}");
					}
					Ok(Some(Token::Punctuator(token)))
				}
			}
			PPToken::StringLiteral(token) => {
				self.is_newline = false;
				if self.stdout > 0 {
					print!("{token}");
				}
				Ok(Some(Token::StringLiteral(token)))
			}
			PPToken::CharacterConstant(token) => {
				self.is_newline = false;
				if self.stdout > 0 {
					print!("{token}");
				}
				Ok(Some(Token::Constant(tok::Constant::Character(token))))
			}
			PPToken::PPNumber(token) => {
				self.is_newline = false;
				let token = Token::try_from(token)?;
				if self.stdout > 0 {
					print!("{token}");
				}
				Ok(Some(token))
			}
			PPToken::HeaderName(token) => todo!("header-name = {token:?}"),
		}
	}
	fn pp_newline(&mut self, token: tok::NewLine) {
		if self.stdout > 0 {
			print!("{token}");
		}
		self.is_preproc = false;
		self.is_newline = true;
		self.line += 1;
	}
	fn expand_macro(&mut self, macro_name: tok::Identifier) -> Result<bool, LexicalError> {
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
					self.pp_tokens.push_token_front(pp_token)
				}
				Ok(true)
			}
			// function-like macro
			Some(MacroDef::Function {
				args,
				replacement_list,
			}) => {
				// if let Some(PPToken::Punctuator(tok::Punctuator {
				// 	term: tok::PunctuatorTerminal::LParen,
				// 	..
				// })) = self.pp_tokens.front()
				// {
				// 	// consume `(`
				// 	self.pp_tokens.pop_front();
				// } else {
				// 	return Ok(false);
				// }
				match self.pp_tokens.peek() {
					Some(Ok(PPToken::Punctuator(tok::Punctuator {
						term: tok::PunctuatorTerminal::LParen,
						..
					}))) => {
						self.pp_tokens.next();
					}
					Some(Err(_)) => {
						self.pp_tokens.next().unwrap()?;
					}
					_ => return Ok(false),
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
					return Err(LexicalError {
						kind: LexicalErrorKind::InvalidToken,
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
					self.pp_tokens.push_token_front(pp_token)
				}
				Ok(true)
			}
		}
	}
	fn directive(&mut self, ident: tok::Identifier) -> Result<(), LexicalError> {
		match ident.name.as_str() {
			"define" => self.pp_define(ident.span),
			"undef" => self.pp_undef(ident.span),
			"include" => self.pp_include(ident.span),
			_ => todo!("{} | undefined directive: `{}`", self.line, ident.name),
		}
	}
	fn pp_define(&mut self, last_span: tok::Span) -> Result<(), LexicalError> {
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
				return Err(LexicalError {
					kind: LexicalErrorKind::UnexpectedEof,
					span,
				});
			}
		};
		let tok::PPToken::Identifier(ident) = pp_token else {
			return Err(LexicalError {
				kind: LexicalErrorKind::InvalidToken,
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
								return Err(LexicalError {
									kind: LexicalErrorKind::InvalidToken,
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
								return Err(LexicalError {
									kind: LexicalErrorKind::InvalidToken,
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
								return Err(LexicalError {
									kind: LexicalErrorKind::InvalidToken,
									span,
								});
							}
							expected_rparen = true;
						}

						Ok(other) => {
							return Err(LexicalError {
								kind: LexicalErrorKind::InvalidToken,
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
	fn pp_undef(&mut self, last_span: tok::Span) -> Result<(), LexicalError> {
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
				return Err(LexicalError {
					kind: LexicalErrorKind::UnexpectedEof,
					span,
				});
			}
		};
		let tok::PPToken::Identifier(ident) = pp_token else {
			return Err(LexicalError {
				kind: LexicalErrorKind::InvalidToken,
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
	fn pp_include(&mut self, last_span: tok::Span) -> Result<(), LexicalError> {
		self.is_preproc = false;
		self.is_newline = true;
		let header = match self.pp_tokens.next() {
			Some(Ok(PPToken::HeaderName(header))) => header,
			Some(Ok(token)) => {
				let lex_err = LexicalError {
					kind: LexicalErrorKind::InvalidToken,
					span: token.span(),
				};
				return Err(lex_err);
			}
			Some(Err(error)) => return Err(error),
			None => {
				let lex_err = LexicalError {
					kind: LexicalErrorKind::UnexpectedEof,
					span: last_span,
				};
				return Err(lex_err);
			}
		};
		if header.is_std {
			todo!()
		} else {
			let header_span = header.span();
			let file_path: path::PathBuf = path::PathBuf::from(header.name);
			let file = fs::File::open(&file_path).map_err(|_| LexicalError {
				span: header_span.clone(),
				kind: LexicalErrorKind::HeaderNameError,
			})?;
			let mut reader = BufReader::new(file);
			let mut buf = String::new();
			reader.read_to_string(&mut buf).map_err(|_| LexicalError {
				span: header_span,
				kind: LexicalErrorKind::HeaderNameError,
			})?;
			drop(reader);

			let file_key = if let Some(file_key) = self.file_map.get_by_right(&file_path) {
				*file_key
			} else {
				let file_key = self.file_map.len();
				self.file_map.insert(file_key, file_path);
				file_key
			};

			let header_lexer = lex::Lexer::new(buf, file_key);
			self.pp_tokens.push_lexer_front(header_lexer);
			Ok(())
		}
	}
}
