use super::error::*;
use std::fmt::Debug;

use crate::tok;

#[derive(Debug)]
pub struct Lexer {
	buf: Vec<char>,
	pos: usize,
	file_key: usize,
	include_state: u8,
}

impl Lexer {
	pub fn new(text: String, file_key: usize) -> Self {
		Self {
			buf: text.chars().collect(),
			pos: 0,
			file_key,
			include_state: 1,
		}
	}
	fn next_char(&mut self) -> Option<char> {
		let result = self.buf.get(self.pos);
		self.pos += 1;
		result.copied()
	}
	fn peek_char(&self) -> Option<char> {
		self.buf.get(self.pos).copied()
	}

	fn header_name(&mut self, c: char, span: tok::Span) -> Result<tok::PPToken, LexicalError> {
		let mut name = String::new();
		// name.push(c);
		let is_std;
		let char_seq = match c {
			'<' => {
				is_std = true;
				let seq = self.h_char_sequence()?;
				if let Some('>') = self.buf.get(self.pos) {
					return Err(LexicalError {
						kind: LexicalErrorKind::InvalidToken,
						span,
					});
				}
				seq
			}
			'"' => {
				is_std = false;
				let seq = self.q_char_sequence()?;
				if let Some('"') = self.buf.get(self.pos) {
					return Err(LexicalError {
						kind: LexicalErrorKind::InvalidToken,
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

	fn identifier(&mut self, c: char, mut span: tok::Span) -> Result<tok::PPToken, LexicalError> {
		let mut name = String::new();
		name.push(c);
		while let Some(next_c) = self
			.buf
			.get(self.pos)
			.filter(|&&c| c.is_ascii_alphanumeric() || c == '_')
		{
			name.push(*next_c);
			self.pos += 1;
		}
		span.location.1 = self.pos;
		if self.include_state == 2 && name == "include" {
			self.include_state = 3;
		} else {
			self.include_state = 0;
		}
		let ident = tok::Identifier { span, name };
		Ok(tok::PPToken::Identifier(ident))
	}

	// #[allow(dead_code)]
	// fn pp_number(&mut self) -> Result<tok::PPToken, LexicalError> {
	// 	todo!("pp-number")
	// }

	fn character_constant(
		&mut self,
		mut c: char,
		mut span: tok::Span,
	) -> Result<tok::PPToken, LexicalError> {
		let mut name = String::new();
		self.include_state = 0;
		let is_l = c == 'L';
		if is_l {
			name.push(c);
			if let Some(&next_c) = self.buf.get(self.pos) {
				self.pos += 1;
				c = next_c;
			} else {
				return Err(LexicalError {
					kind: LexicalErrorKind::UnexpectedEof,
					span,
				});
			}
			self.pos += 1;
		}
		name.push(c);
		name.push_str(&self.c_char_sequence(&mut span)?);
		if let Some('\'') = self.buf.get(self.pos) {
			self.pos += 1;
			name.push('\'');
		} else {
			return Err(LexicalError {
				kind: LexicalErrorKind::InvalidToken,
				span,
			});
		}

		span.location.1 = self.pos;
		let str_lit = tok::CharacterConstant { span, name };
		Ok(tok::PPToken::CharacterConstant(str_lit))
	}

	fn string_literal(
		&mut self,
		mut c: char,
		mut span: tok::Span,
	) -> Result<tok::PPToken, LexicalError> {
		let mut name = String::new();
		let is_l = c == 'L';
		if is_l {
			name.push(c);
			if let Some(&next_c) = self.buf.get(self.pos) {
				self.pos += 1;
				c = next_c;
			} else {
				return Err(LexicalError {
					kind: LexicalErrorKind::UnexpectedEof,
					span: span.clone(),
				});
			}
			self.pos += 1;
		}
		name.push(c);
		name.push_str(&self.s_char_sequence(&mut span)?);
		if let Some('"') = self.buf.get(self.pos) {
			self.pos += 1;
			name.push('"');
		} else {
			return Err(LexicalError {
				kind: LexicalErrorKind::InvalidToken,
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
	fn punctuator(&mut self) -> Result<tok::PPToken, LexicalError> {
		todo!("punctuator")
	}

	fn escape_sequence(&mut self, span: &mut tok::Span) -> Result<char, LexicalError> {
		let Some(term) = self.buf.get(self.pos + 1) else {
			return Err(LexicalError {
				kind: LexicalErrorKind::UnexpectedEscape,
				span: span.clone(),
			});
		};
		match term {
			// [c89] simple-escape-sequence
			'\'' | '"' | '?' | '\\' | 'a' | 'b' | 'f' | 'n' | 'r' | 't' | 'v' => {
				self.pos += 1;
				Ok(self.buf[self.pos])
			}
			// [c89] octal-escape-sequence
			'0'..='7' => todo!("octal-escape-sequence"),
			// [c89] hexadecimal-escape-sequence
			'x' => todo!("hexadecimal-escape-sequence"),
			// [c99] universal-character-name
			// 'u' | 'U' => todo!("universal-character-name"),
			_ => Err(LexicalError {
				kind: LexicalErrorKind::UnexpectedEscape,
				span: span.clone(),
			}),
		}
	}

	fn s_char_sequence(&mut self, span: &mut tok::Span) -> Result<String, LexicalError> {
		let mut seq = String::new();
		while let Some(&c) = self.buf.get(self.pos) {
			if c == '"' || c == '\n' {
				break;
			}
			let s_char = if c == '\\' {
				self.escape_sequence(span)?
			} else {
				c
			};
			self.pos += 1;
			seq.push(s_char);
		}
		Ok(seq)
	}
	fn c_char_sequence(&mut self, span: &mut tok::Span) -> Result<String, LexicalError> {
		let mut seq = String::new();
		while let Some(&c) = self.buf.get(self.pos) {
			if c == '\'' || c == '\n' {
				break;
			}
			let c_char = if c == '\\' {
				self.escape_sequence(span)?
			} else {
				c
			};
			self.pos += 1;
			seq.push(c_char);
		}
		Ok(seq)
	}
	fn h_char_sequence(&mut self) -> Result<String, LexicalError> {
		let mut seq = String::new();
		while let Some(&h_char) = self.buf.get(self.pos) {
			if h_char == '>' || h_char == '\n' {
				break;
			}
			self.pos += 1;
			seq.push(h_char);
		}
		Ok(seq)
	}
	fn q_char_sequence(&mut self) -> Result<String, LexicalError> {
		let mut seq = String::new();
		while let Some(&q_char) = self.buf.get(self.pos) {
			if q_char == '"' || q_char == '\n' {
				break;
			}
			self.pos += 1;
			seq.push(q_char);
		}
		Ok(seq)
	}
}

impl Iterator for Lexer {
	type Item = Result<tok::PPToken, LexicalError>;
	fn next(&mut self) -> Option<Self::Item> {
		let (mut leading_tabs, mut leading_spaces) = (0, 0);
		// skip whitespace
		while let Some(c) = self.peek_char() {
			if c == '\n' || !c.is_ascii_whitespace() {
				break;
			}
			leading_spaces += (c == ' ') as usize;
			leading_tabs += (c == '\t') as usize;
			self.pos += 1;
		}
		let start_pos = self.pos;
		let c = self.next_char()?;
		let mut span = tok::Span {
			location: (start_pos, self.pos),
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
		// if c == 'L' && self.chars.peek().is_some_and(|&val| val == '"') {
		// 	return Some(self.string_literal(c, span));
		// }
		if c == '\'' || c == 'L' && self.buf.get(self.pos + 1).is_some_and(|&val| val == '\'') {
			return Some(self.character_constant(c, span));
		}

		let mut name = String::new();
		match c {
			'\n' => {
				self.include_state = 1;
				span.location = (start_pos, self.pos);
				name.push(c);
				let new_line = tok::NewLine {
					span,
					is_deleted: false,
				};
				Some(Ok(tok::PPToken::NewLine(new_line)))
			}
			// punctuators without trailing characters
			'[' | ']' | '(' | ')' | '{' | '}' | '?' | ',' | '~' | ';' => {
				self.include_state = 0;
				span.location = (start_pos, self.pos);
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
				while let Some(next_c) = self.peek_char() {
					if next_c.is_ascii_digit() || next_c == '.' {
						name.push(self.next_char()?);
					} else if next_c.is_ascii_alphabetic() || next_c == '_' {
						name.push(self.next_char()?);
						if matches!(next_c, 'e' | 'E' | 'p' | 'P') {
							let Some(&sign) = self.buf.get(self.pos + 1) else {
								span.location = (start_pos, self.pos);
								return Some(Err(LexicalError {
									kind: LexicalErrorKind::UnexpectedEof,
									span,
								}));
							};
							if matches!(sign, '-' | '+' | '0'..='9') {
								name.push(*self.buf.get(self.pos)?);
								self.pos += 2;
								continue;
							}
						}
						self.pos += 1;
					} else {
						break;
					}
				}
				span.location = (start_pos, self.pos);
				let num = tok::PPNumber { span, name };
				Some(Ok(tok::PPToken::PPNumber(num)))
			}
			// `.` or `...` or pp-number
			'.' => {
				// case: `.`
				self.include_state = 0;
				name.push(c);
				if let Some('.') = self.buf.get(self.pos) {
					// case: `..`
					self.pos += 1;
					if let Some('.') = self.buf.get(self.pos) {
						// case: `...`
						self.pos += 1;
						span.location.1 = self.pos;
						let punct = tok::Punctuator {
							term: tok::PunctuatorTerminal::Ellipsis,
							span,
						};
						Some(Ok(tok::PPToken::Punctuator(punct)))
					} else {
						span.location = (start_pos, self.pos);
						Some(Err(LexicalError {
							kind: LexicalErrorKind::InvalidToken,
							span,
						}))
					}
				} else if let Some(&digit) = self.buf.get(self.pos).filter(|c| c.is_ascii_digit()) {
					// case: `.[0-9]`
					name.push(digit);
					while let Some(&next_c) = self.buf.get(self.pos + 1) {
						if next_c.is_ascii_digit() || next_c == '.' {
							name.push(*self.buf.get(self.pos)?);
							self.pos += 1;
						} else if next_c.is_ascii_alphabetic() || next_c == '_' {
							name.push(*self.buf.get(self.pos)?);
							if matches!(next_c, 'e' | 'E' | 'p' | 'P') {
								let Some(&sign) = self.buf.get(self.pos + 1) else {
									span.location = (start_pos, self.pos);
									return Some(Err(LexicalError {
										kind: LexicalErrorKind::UnexpectedEof,
										span,
									}));
								};
								if matches!(sign, '-' | '+' | '0'..='9') {
									name.push(*self.buf.get(self.pos)?);
									self.pos += 2;
									continue;
								}
							}
							self.pos += 1;
						} else {
							break;
						}
					}
					span.location = (start_pos, self.pos);
					let num = tok::PPNumber { span, name };
					Some(Ok(tok::PPToken::PPNumber(num)))
				} else {
					span.location = (start_pos, self.pos);
					Some(Err(LexicalError {
						kind: LexicalErrorKind::InvalidToken,
						span,
					}))
				}
			}
			'#' => {
				if self.include_state == 1 {
					self.include_state = 2;
				}
				let term = if let Some('#') = self.buf.get(self.pos) {
					self.include_state = 0;
					self.pos += 1;
					span.location = (start_pos, self.pos);
					tok::PunctuatorTerminal::HashHash
				} else {
					tok::PunctuatorTerminal::Hash
				};
				let punct = tok::Punctuator { span, term };
				Some(Ok(tok::PPToken::Punctuator(punct)))
			}
			'<' => {
				let term = if self.include_state == 3 {
					return Some(self.header_name(c, span));
				} else if let Some('<') = self.buf.get(self.pos) {
					// case: `<<`
					todo!("<<")
				} else if let Some(':') = self.buf.get(self.pos) {
					// case: `<:` => `[`
					self.pos += 1;
					span.location = (start_pos, self.pos);
					tok::PunctuatorTerminal::LSquare
				} else if let Some('%') = self.buf.get(self.pos) {
					// case: `<%` => `{`
					self.pos += 1;
					span.location = (start_pos, self.pos);
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
				let term = if let Some('/') = self.buf.get(self.pos) {
					// case: `//`
					self.pos += 1;
					name.push_str("//");
					while let Some(c) = self.buf.get(self.pos) {
						if *c == '\n' {
							break;
						}
						name.push(*c);
						self.pos += 1;
					}
					let comment = tok::Comment { span, name };
					return Some(Ok(tok::PPToken::Comment(comment)));
				} else if let Some('=') = self.buf.get(self.pos) {
					// case: `/=`
					self.pos += 1;
					span.location = (start_pos, self.pos);
					tok::PunctuatorTerminal::PlusEqual
				} else if let Some('*') = self.buf.get(self.pos) {
					self.pos += 1;
					name.push_str("/*");
					let Some(last_c) = self.buf.get(self.pos) else {
						return Some(Err(LexicalError {
							span,
							kind: LexicalErrorKind::UnexpectedEof,
						}));
					};
					let mut last_c = *last_c;
					while let Some(&c) = self.buf.get(self.pos) {
						self.pos += 1;
						name.push(c);
						if last_c == '*' && c == '/' {
							break;
						}
						last_c = c;
					}
					let comment = tok::Comment { span, name };
					return Some(Ok(tok::PPToken::Comment(comment)));
				} else {
					tok::PunctuatorTerminal::FSlash
				};
				let punct = tok::Punctuator { span, term };
				Some(Ok(tok::PPToken::Punctuator(punct)))
			}
			'\\' => {
				if let Some('\n') = self.buf.get(self.pos) {
					self.pos += 1;
					let new_line = tok::PPToken::NewLine(tok::NewLine {
						span,
						is_deleted: true,
					});
					Some(Ok(new_line))
				} else {
					Some(Err(LexicalError {
						span,
						kind: LexicalErrorKind::InvalidToken,
					}))
				}
			}
			'+' => {
				self.include_state = 0;
				let term = if let Some('+') = self.buf.get(self.pos) {
					// case: `++`
					self.pos += 1;
					span.location = (start_pos, self.pos);
					tok::PunctuatorTerminal::PlusPlus
				} else if let Some('=') = self.buf.get(self.pos) {
					// case: `+=`
					self.pos += 1;
					span.location = (start_pos, self.pos);
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
				let term = if let Some('-') = self.buf.get(self.pos) {
					// case: `--`
					self.pos += 1;
					span.location = (start_pos, self.pos);
					tok::PunctuatorTerminal::MinusMinus
				} else if let Some('=') = self.buf.get(self.pos) {
					// case: `+=`
					self.pos += 1;
					span.location = (start_pos, self.pos);
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
				let term = if let Some('=') = self.buf.get(self.pos) {
					// case: `==`
					self.pos += 1;
					span.location = (start_pos, self.pos);
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
				let term = if let Some('=') = self.buf.get(self.pos) {
					// case: `*=`
					self.pos += 1;
					span.location = (start_pos, self.pos);
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
				let term = if let Some('>') = self.buf.get(self.pos) {
					// case: `:>`
					self.pos += 1;
					span.location = (start_pos, self.pos);
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
				let term = if let Some('=') = self.buf.get(self.pos) {
					// case: `!=`
					self.pos += 1;
					span.location = (start_pos, self.pos);
					tok::PunctuatorTerminal::BangEqual
				} else {
					// case: `!`
					tok::PunctuatorTerminal::Bang
				};
				let punct = tok::Punctuator { span, term };
				Some(Ok(tok::PPToken::Punctuator(punct)))
			}
			_ => todo!("`{c}` : {}", c as i32),
		}
	}
}
