use std::fmt::Debug;
use std::iter::{Enumerate, Peekable};
use std::vec::IntoIter;

use crate::diag::lex;
use crate::tok;

#[derive(Debug)]
pub struct Lexer {
	chars: Peekable<Enumerate<IntoIter<char>>>,
	file_key: usize,
	include_state: u8,
}

impl Lexer {
	pub fn new(text: String, file_key: usize) -> Self {
		let char_vec: Vec<char> = text.chars().collect();
		let char_iter = char_vec.into_iter();
		Self {
			chars: char_iter.enumerate().peekable(),
			file_key,
			include_state: 1,
		}
	}

	#[allow(dead_code)]
	fn header_name(&mut self, c: char, span: tok::Span) -> lex::Result<tok::PPToken> {
		let mut name = String::new();
		// name.push(c);
		let is_std;
		let char_seq = match c {
			'<' => {
				is_std = true;
				let seq = self.h_char_sequence()?;
				if self.chars.next_if(|(_, c)| *c == '>').is_none() {
					return Err(lex::Error {
						kind: lex::ErrorKind::InvalidToken,
						span,
					});
				}
				seq
			}
			'"' => {
				is_std = false;
				let seq = self.q_char_sequence()?;
				if self.chars.next_if(|(_, c)| *c == '"').is_none() {
					return Err(lex::Error {
						kind: lex::ErrorKind::InvalidToken,
						span,
					});
				}
				seq
			}
			_ => unreachable!(),
		};
		name.push_str(&char_seq);
		self.include_state = 0;
		let head_name = tok::HeaderName { span, is_std, name };
		Ok(tok::PPToken::HeaderName(head_name))
	}

	fn identifier(&mut self, c: char, mut span: tok::Span) -> lex::Result<tok::PPToken> {
		let mut name = String::new();
		name.push(c);
		let mut curr_pos = 0;
		while let Some((pos, next_c)) = self
			.chars
			.next_if(|&(_, c)| c.is_ascii_alphanumeric() || c == '_')
		{
			name.push(next_c);
			curr_pos = pos
		}
		span.location.1 = curr_pos;
		if self.include_state == 2 && name == "include" {
			self.include_state = 3;
		} else {
			self.include_state = 0;
		}
		let ident = tok::Identifier { span, name };
		Ok(tok::PPToken::Identifier(ident))
	}

	#[allow(dead_code)]
	fn pp_number(&mut self) -> lex::Result<tok::PPToken> {
		todo!("pp-number")
	}

	fn character_constant(
		&mut self,
		mut c: char,
		mut span: tok::Span,
	) -> lex::Result<tok::PPToken> {
		let mut name = String::new();
		self.include_state = 0;
		let is_l = c == 'L';
		if is_l {
			name.push(c);
			if let Some((_, next_c)) = self.chars.next() {
				c = next_c;
			} else {
				return Err(lex::Error {
					kind: lex::ErrorKind::UnexpectedEof,
					span,
				});
			}
		}
		name.push(c);
		name.push_str(&self.c_char_sequence(&mut span)?);
		let curr_pos;
		if let Some((pos, _)) = self.chars.next_if(|&(_, c)| c == '\'') {
			name.push('\'');
			curr_pos = pos;
		} else {
			return Err(lex::Error {
				kind: lex::ErrorKind::InvalidToken,
				span,
			});
		}

		span.location.1 = curr_pos;
		let str_lit = tok::CharacterConstant { span, name };
		Ok(tok::PPToken::CharacterConstant(str_lit))
	}

	fn string_literal(&mut self, mut c: char, mut span: tok::Span) -> lex::Result<tok::PPToken> {
		let mut name = String::new();
		let is_l = c == 'L';
		if is_l {
			name.push(c);
			if let Some((_, next_c)) = self.chars.next() {
				c = next_c;
			} else {
				return Err(lex::Error {
					kind: lex::ErrorKind::UnexpectedEof,
					span: span.clone(),
				});
			}
		}
		name.push(c);
		name.push_str(&self.s_char_sequence(&mut span)?);
		if self.chars.next_if(|&(_, c)| c == '"').is_some() {
			name.push('"');
		} else {
			return Err(lex::Error {
				kind: lex::ErrorKind::InvalidToken,
				span: span.clone(),
			});
		}

		let str_lit = tok::StringLiteral {
			span: span.clone(),
			name,
		};
		Ok(tok::PPToken::StringLiteral(str_lit))
	}
	#[allow(dead_code)]
	fn punctuator(&mut self) -> lex::Result<tok::PPToken> {
		todo!("punctuator")
	}

	fn escape_sequence(&mut self, span: &mut tok::Span) -> lex::Result<char> {
		let Some((_, term)) = self.chars.peek() else {
			return Err(lex::Error {
				kind: lex::ErrorKind::UnexpectedEscape,
				span: span.clone(),
			});
		};
		match term {
			// [c89] simple-escape-sequence
			'\'' | '"' | '?' | '\\' | 'a' | 'b' | 'f' | 'n' | 'r' | 't' | 'v' => {
				Ok(self.chars.next().unwrap().1)
			}
			// [c89] octal-escape-sequence
			'0'..='7' => todo!("octal-escape-sequence"),
			// [c89] hexadecimal-escape-sequence
			'x' => todo!("hexadecimal-escape-sequence"),
			// [c99] universal-character-name
			// 'u' | 'U' => todo!("universal-character-name"),
			_ => Err(lex::Error {
				kind: lex::ErrorKind::UnexpectedEscape,
				span: span.clone(),
			}),
		}
	}

	fn s_char_sequence(&mut self, span: &mut tok::Span) -> lex::Result<String> {
		let mut seq = String::new();
		while let Some((_, c)) = self.chars.next_if(|&(_, c)| c != '"' && c != '\n') {
			let s_char = if c == '\\' {
				self.escape_sequence(span)?
			} else {
				c
			};
			seq.push(s_char);
		}
		Ok(seq)
	}
	fn c_char_sequence(&mut self, span: &mut tok::Span) -> lex::Result<String> {
		let mut seq = String::new();
		while let Some((_, c)) = self.chars.next_if(|&(_, c)| c != '\'' && c != '\n') {
			let c_char = if c == '\\' {
				self.escape_sequence(span)?
			} else {
				c
			};
			seq.push(c_char);
		}
		Ok(seq)
	}
	fn h_char_sequence(&mut self) -> lex::Result<String> {
		let mut seq = String::new();
		while let Some((_, h_char)) = self.chars.next_if(|&(_, c)| c != '>' && c != '\n') {
			seq.push(h_char);
		}
		Ok(seq)
	}
	fn q_char_sequence(&mut self) -> lex::Result<String> {
		let mut seq = String::new();
		while let Some((_, q_char)) = self.chars.next_if(|&(_, c)| c != '"' && c != '\n') {
			seq.push(q_char);
		}
		Ok(seq)
	}
}

impl Iterator for Lexer {
	type Item = lex::Result<tok::PPToken>;
	fn next(&mut self) -> Option<Self::Item> {
		let (mut leading_tabs, mut leading_spaces) = (0, 0);
		let mut curr_pos = 0;
		// skip whitespace
		while let Some((pos, whitespace)) = self
			.chars
			.next_if(|&(_, c)| c != '\n' && c.is_ascii_whitespace())
		{
			curr_pos = pos;
			match whitespace {
				' ' => leading_spaces += 1,
				'\t' => leading_tabs += 1,
				_ => (),
			}
		}
		let start_pos = curr_pos;
		let (_, c) = self.chars.next()?;
		let mut span = tok::Span {
			location: (start_pos, curr_pos),
			file_key: self.file_key,
			leading_spaces,
			leading_tabs,
		};

		if c == '"' {
			if self.include_state == 3 {
				return Some(self.header_name(c, span));
			} else {
				return Some(self.string_literal(c, span));
			}
		}
		if c == 'L' && self.chars.peek().is_some_and(|&(_, c)| c == '"') {
			return Some(self.string_literal(c, span));
		}
		if c == '\'' || c == 'L' && self.chars.peek().is_some_and(|&(_, c)| c == '\'') {
			return Some(self.character_constant(c, span));
		}

		let mut name = String::new();
		match c {
			new_line @ ('\r' | '\n' | '\u{000B}' | '\u{000C}' | '\u{0085}' | '\u{2028}'
			| '\u{2029}') => {
				name.push(new_line);
				let mut curr_pos = 0;
				if c == '\r' {
					if let Some((pos, _)) = self.chars.next_if(|&(_, c)| c == '\n') {
						curr_pos = pos;
						name.push('\n');
					}
				}
				self.include_state = 1;
				span.location = (start_pos, curr_pos);
				let new_line = tok::NewLine {
					span,
					name,
					is_deleted: false,
				};
				Some(Ok(tok::PPToken::NewLine(new_line)))
			}
			// punctuators without trailing characters
			'[' | ']' | '(' | ')' | '{' | '}' | '?' | ',' | '~' | ';' => {
				self.include_state = 0;
				let punct = tok::Punctuator {
					span,
					term: tok::PunctuatorTerminal::try_from(c).unwrap(),
				};
				Some(Ok(tok::PPToken::Punctuator(punct)))
			}

			// identifier
			'a'..='z' | 'A'..='Z' | '_' => Some(self.identifier(c, span)),
			// pp-number
			'0'..='9' => {
				self.include_state = 0;
				name.push(c);
				let mut curr_pos = 0;
				while let Some(&(pos, next_c)) = self.chars.peek() {
					if next_c.is_ascii_digit() || next_c == '.' {
						name.push(self.chars.next()?.1);
						curr_pos = pos;
					} else if next_c.is_ascii_alphabetic() || next_c == '_' {
						let (pos, curr_c) = self.chars.next()?;
						curr_pos = pos;
						name.push(curr_c);
						if matches!(next_c, 'e' | 'E' | 'p' | 'P') {
							let Some((_, sign)) = self.chars.peek() else {
								span.location = (start_pos, curr_pos);
								return Some(Err(lex::Error {
									kind: lex::ErrorKind::UnexpectedEof,
									span,
								}));
							};
							if matches!(sign, '-' | '+' | '0'..='9') {
								name.push(self.chars.next()?.1);
								continue;
							}
						}
					} else {
						break;
					}
				}
				span.location = (start_pos, curr_pos);
				let num = tok::PPNumber { span, name };
				Some(Ok(tok::PPToken::PPNumber(num)))
			}
			// `.` or `...` or pp-number
			'.' => {
				// case: `.`
				self.include_state = 0;
				name.push(c);
				if self.chars.next_if(|&(_, c)| c == '.').is_some() {
					// case: `..`
					if let Some((pos, _)) = self.chars.next_if(|&(_, c)| c == '.') {
						// case: `...`
						span.location.1 = pos;
						let punct = tok::Punctuator {
							term: tok::PunctuatorTerminal::Ellipsis,
							span,
						};
						Some(Ok(tok::PPToken::Punctuator(punct)))
					} else {
						Some(Err(lex::Error {
							kind: lex::ErrorKind::InvalidToken,
							span,
						}))
					}
				} else if let Some((_, digit)) = self.chars.next_if(|&(_, c)| c.is_ascii_digit()) {
					// case: `.[0-9]`
					name.push(digit);
					let mut curr_pos = 0;
					while let Some(&(_, next_c)) = self.chars.peek() {
						if next_c.is_ascii_digit() || next_c == '.' {
							let (pos, c) = self.chars.next()?;
							curr_pos = pos;
							name.push(c);
						} else if next_c.is_ascii_alphabetic() || next_c == '_' {
							let (pos, c) = self.chars.next()?;
							curr_pos = pos;
							name.push(c);
							if matches!(next_c, 'e' | 'E' | 'p' | 'P') {
								let Some((_, sign)) = self.chars.peek() else {
									span.location = (start_pos, pos);
									return Some(Err(lex::Error {
										kind: lex::ErrorKind::UnexpectedEof,
										span,
									}));
								};
								if matches!(sign, '-' | '+' | '0'..='9') {
									name.push(self.chars.next()?.1);
									continue;
								}
							}
						} else {
							break;
						}
					}
					span.location = (start_pos, curr_pos);
					let num = tok::PPNumber { span, name };
					Some(Ok(tok::PPToken::PPNumber(num)))
				} else {
					Some(Err(lex::Error {
						kind: lex::ErrorKind::InvalidToken,
						span,
					}))
				}
			}
			'#' => {
				if self.include_state == 1 {
					self.include_state = 2;
				}
				name.push(c);
				if let Some((pos, _)) = self.chars.next_if(|&(_, c)| c == '#') {
					self.include_state = 0;
					span.location = (start_pos, pos);
				}
				let punct = tok::Punctuator {
					span,
					term: tok::PunctuatorTerminal::Hash,
				};
				Some(Ok(tok::PPToken::Punctuator(punct)))
			}
			'<' => {
				let term = if self.include_state == 3 {
					return Some(self.header_name(c, span));
				} else if self.chars.next_if(|&(_, c)| c == '<').is_some() {
					// case: `<<`
					todo!("<<")
				} else if let Some((pos, _)) = self.chars.next_if(|&(_, c)| c == ':') {
					// case: `<:` => `[`
					span.location = (start_pos, pos);
					tok::PunctuatorTerminal::LSquare
				} else if let Some((pos, _)) = self.chars.next_if(|&(_, c)| c == '%') {
					// case: `<%` => `{`
					span.location = (start_pos, pos);
					tok::PunctuatorTerminal::LCurly
				} else {
					// case: `<`
					tok::PunctuatorTerminal::Less
				};
				let punct = tok::Punctuator { span, term };
				Some(Ok(tok::PPToken::Punctuator(punct)))
			}
			'/' => {
				self.include_state = 0;
				let term = if self.chars.next_if(|&(_, c)| c == '/').is_some() {
					// case: `//`
					name.push_str("//");
					while let Some((_, c)) = self.chars.next_if(|&(_, c)| c != '\n') {
						name.push(c);
					}
					let comment = tok::Comment { span, name };
					return Some(Ok(tok::PPToken::Comment(comment)));
				} else if let Some((pos, _)) = self.chars.next_if(|&(_, c)| c == '=') {
					// case: `/=`
					span.location = (start_pos, pos);
					tok::PunctuatorTerminal::PlusEqual
				} else if self.chars.next_if(|&(_, c)| c == '*').is_some() {
					name.push_str("/*");
					let Some((pos, mut last_c)) = self.chars.next() else {
						return Some(Err(lex::Error {
							span,
							kind: lex::ErrorKind::UnexpectedEof,
						}));
					};
					name.push(last_c);
					let mut found_end = false;
					for (_, c) in self.chars.by_ref() {
						name.push(c);
						if last_c == '*' && c == '/' {
							found_end = true;
							break;
						}
						last_c = c;
					}
					span.location.1 = pos;
					if found_end {
						return Some(Ok(tok::PPToken::Comment(tok::Comment { span, name })));
					} else {
						return Some(Err(lex::Error {
							kind: lex::ErrorKind::UnexpectedEof,
							span,
						}));
					}
				} else {
					tok::PunctuatorTerminal::FSlash
				};
				let punct = tok::Punctuator { span, term };
				Some(Ok(tok::PPToken::Punctuator(punct)))
			}
			'\\' => match self.next() {
				Some(Ok(tok::PPToken::NewLine(mut new_line))) => {
					new_line.is_deleted = true;
					Some(Ok(tok::PPToken::NewLine(new_line)))
				}
				Some(Ok(_)) => Some(Err(lex::Error {
					span,
					kind: lex::ErrorKind::InvalidToken,
				})),
				Some(Err(error)) => Some(Err(error)),
				None => Some(Err(lex::Error {
					span,
					kind: lex::ErrorKind::UnexpectedEof,
				})),
			},
			'+' => {
				self.include_state = 0;
				let term = if let Some((pos, _)) = self.chars.next_if(|&(_, c)| c == '+') {
					// case: `++`
					span.location = (start_pos, pos);
					tok::PunctuatorTerminal::PlusPlus
				} else if let Some((pos, _)) = self.chars.next_if(|&(_, c)| c == '=') {
					// case: `+=`
					span.location = (start_pos, pos);
					tok::PunctuatorTerminal::PlusEqual
				} else {
					// case: `+`
					tok::PunctuatorTerminal::Plus
				};
				let punct = tok::Punctuator { span, term };
				Some(Ok(tok::PPToken::Punctuator(punct)))
			}
			'-' => {
				self.include_state = 0;
				let term = if let Some((pos, _)) = self.chars.next_if(|&(_, c)| c == '-') {
					// case: `--`
					span.location = (start_pos, pos);
					tok::PunctuatorTerminal::MinusMinus
				} else if let Some((pos, _)) = self.chars.next_if(|&(_, c)| c == '=') {
					// case: `+=`
					span.location = (start_pos, pos);
					tok::PunctuatorTerminal::MinusEqual
				} else {
					// case: `+`
					tok::PunctuatorTerminal::Minus
				};
				let punct = tok::Punctuator { span, term };
				Some(Ok(tok::PPToken::Punctuator(punct)))
			}
			'=' => {
				self.include_state = 0;
				let term = if let Some((pos, _)) = self.chars.next_if(|&(_, c)| c == '=') {
					// case: `==`
					span.location = (start_pos, pos);
					tok::PunctuatorTerminal::EqualEqual
				} else {
					// case: `=`
					tok::PunctuatorTerminal::Equal
				};
				let punct = tok::Punctuator { span, term };
				Some(Ok(tok::PPToken::Punctuator(punct)))
			}
			'*' => {
				self.include_state = 0;
				let term = if let Some((pos, _)) = self.chars.next_if(|&(_, c)| c == '=') {
					// case: `*=`
					span.location = (start_pos, pos);
					tok::PunctuatorTerminal::StarEqual
				} else {
					// case: `*`
					tok::PunctuatorTerminal::Star
				};
				let punct = tok::Punctuator { span, term };
				Some(Ok(tok::PPToken::Punctuator(punct)))
			}
			':' => {
				self.include_state = 0;
				let term = if let Some((pos, _)) = self.chars.next_if(|&(_, c)| c == '>') {
					// case: `:>`
					span.location = (start_pos, pos);
					tok::PunctuatorTerminal::RSquare
				} else {
					// case: `:`
					tok::PunctuatorTerminal::Colon
				};
				let punct = tok::Punctuator { span, term };
				Some(Ok(tok::PPToken::Punctuator(punct)))
			}
			'!' => {
				self.include_state = 0;
				let term = if let Some((pos, _)) = self.chars.next_if(|&(_, c)| c == '=') {
					// case: `!=`
					span.location = (start_pos, pos);
					tok::PunctuatorTerminal::BangEqual
				} else {
					// case: `!`
					tok::PunctuatorTerminal::Bang
				};
				let punct = tok::Punctuator { span, term };
				Some(Ok(tok::PPToken::Punctuator(punct)))
			}
			_ => todo!("{}", c as i32),
		}
	}
}
