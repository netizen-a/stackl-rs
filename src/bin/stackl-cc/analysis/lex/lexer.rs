// Copyright (c) 2024-2026 Jonathan A. Thomason

use std::fmt::Debug;
use std::iter::{
	Enumerate,
	Peekable,
};
use std::vec::IntoIter;

use crate::analysis::tok;
use crate::diagnostics::{
	self as diag,
	ToSpan,
	lex,
};

#[derive(Debug)]
pub struct Lexer {
	chars: Peekable<Enumerate<IntoIter<char>>>,
	span: diag::Span,
	leading_space: bool,
	include_state: u8,
	is_comment: bool,
}

impl ToSpan for Lexer {
	fn to_span(&self) -> diag::Span {
		self.span.clone()
	}
}

impl Lexer {
	pub fn new(text: String, file_id: usize) -> Self {
		let char_vec: Vec<char> = text.chars().collect();
		let char_iter = char_vec.into_iter();
		Self {
			chars: char_iter.enumerate().peekable(),
			leading_space: false,
			include_state: 1,
			span: diag::Span {
				file_id,
				line: 1,
				name_id: file_id,
				..Default::default()
			},
			is_comment: false,
		}
	}

	fn set_start(&mut self, start: usize) {
		self.span.loc.0 = start;
	}
	fn set_end(&mut self, end: usize) {
		self.span.loc.1 = end;
	}

	#[allow(dead_code)]
	fn header_name(&mut self, c: char) -> Result<tok::PPToken, diag::Diagnostic> {
		let mut name = String::new();
		// name.push(c);
		let is_builtin;
		let char_seq = match c {
			'<' => {
				is_builtin = true;
				let seq = self.h_char_sequence()?;
				if self.chars.next_if(|(_, c)| *c == '>').is_none() {
					return Err(diag::Diagnostic::error(
						diag::DiagKind::InvalidToken,
						self.to_span(),
					));
				}
				seq
			}
			'"' => {
				is_builtin = false;
				let seq = self.q_char_sequence()?;
				if self.chars.next_if(|(_, c)| *c == '"').is_none() {
					return Err(diag::Diagnostic::error(
						diag::DiagKind::InvalidToken,
						self.to_span(),
					));
				}
				seq
			}
			_ => unreachable!(),
		};
		name.push_str(&char_seq);
		self.include_state = 0;
		let head_name = tok::HeaderName { is_builtin, name };
		Ok(tok::PPToken {
			kind: tok::PPTokenKind::HeaderName(head_name),
			leading_space: self.leading_space,
			span: self.to_span(),
		})
	}

	fn identifier(&mut self, c: char) -> Result<tok::PPToken, diag::Diagnostic> {
		let mut name = String::new();
		name.push(c);
		while let Some((pos, next_c)) = self
			.chars
			.next_if(|&(_, c)| c.is_ascii_alphanumeric() || c == '_')
		{
			name.push(next_c);
			self.set_end(pos);
		}
		if self.include_state == 2 && name == "include" {
			self.include_state = 3;
		} else {
			self.include_state = 0;
		}
		let ident = tok::Ident {
			name,
			is_type: false,
			expandable: true,
		};
		Ok(tok::PPToken {
			kind: tok::PPTokenKind::Ident(ident),
			leading_space: self.leading_space,
			span: self.to_span(),
		})
	}

	#[allow(dead_code)]
	fn pp_number(&mut self) -> Result<tok::PPToken, diag::Diagnostic> {
		todo!("pp-number")
	}

	fn character_constant(&mut self, mut c: char) -> Result<tok::PPToken, diag::Diagnostic> {
		self.include_state = 0;
		let is_wide = c == 'L';
		if is_wide {
			if let Some((_, next_c)) = self.chars.next() {
				#[allow(unused_assignments)]
				{
					c = next_c;
				}
			} else {
				return Err(diag::Diagnostic::error(
					diag::DiagKind::UnexpectedEof,
					self.to_span(),
				));
			}
		}
		let seq = self.c_char_sequence()?;
		if let Some((curr_pos, _)) = self.chars.next_if(|&(_, c)| c == '\'') {
			// name.push('\'');
			self.set_end(curr_pos);
		} else {
			return Err(diag::Diagnostic::error(
				diag::DiagKind::InvalidToken,
				self.to_span(),
			));
		}

		let str_lit = tok::CharConst { seq, is_wide };
		Ok(tok::PPToken {
			kind: tok::PPTokenKind::CharConst(str_lit),
			leading_space: self.leading_space,
			span: self.to_span(),
		})
	}

	fn string_literal(&mut self, c: char) -> Result<tok::PPToken, diag::Diagnostic> {
		let is_wide = c == 'L';
		if is_wide {
			if let Some((pos, _)) = self.chars.next() {
				self.set_end(pos);
			} else {
				return Err(diag::Diagnostic::error(
					diag::DiagKind::UnexpectedEof,
					self.to_span(),
				));
			}
		}
		let seq = self.s_char_sequence()?;
		if let Some((pos, _)) = self.chars.next_if(|&(_, c)| c == '"') {
			self.set_end(pos);
		} else {
			return Err(diag::Diagnostic::error(
				diag::DiagKind::InvalidToken,
				self.to_span(),
			));
		}

		let str_lit = tok::StrLit {
			seq,
			is_wide,
			file_id: self.to_span().file_id,
		};
		Ok(tok::PPToken {
			kind: tok::PPTokenKind::StrLit(str_lit),
			leading_space: self.leading_space,
			span: self.to_span(),
		})
	}
	#[allow(dead_code)]
	fn punctuator(&mut self) -> Result<tok::PPToken, diag::Diagnostic> {
		todo!("punctuator")
	}

	fn escape_sequence(&mut self) -> Result<char, diag::Diagnostic> {
		let Some((curr_pos, term)) = self.chars.next() else {
			return Err(diag::Diagnostic::error(
				diag::DiagKind::UnexpectedEscape,
				self.to_span(),
			));
		};
		match term {
			// alert
			'a' => Ok('\x07'),
			// backspace
			'b' => Ok('\x08'),
			// form feed
			'f' => Ok('\x0C'),
			// new line
			'n' => Ok('\n'),
			'r' => Ok('\r'),
			't' => Ok('\t'),
			'v' => Ok('\x0B'),
			// [c89] simple-escape-sequence
			c @ ('\'' | '"' | '?' | '\\') => {
				self.set_end(curr_pos);
				Ok(c)
			}
			// [c89] octal-escape-sequence
			'0'..='7' => todo!("octal-escape-sequence"),
			// [c89] hexadecimal-escape-sequence
			'x' => todo!("hexadecimal-escape-sequence"),
			// [c99] universal-character-name
			// 'u' | 'U' => todo!("universal-character-name"),
			_ => Err(diag::Diagnostic::error(
				diag::DiagKind::UnexpectedEscape,
				self.to_span(),
			)),
		}
	}

	fn s_char_sequence(&mut self) -> Result<String, diag::Diagnostic> {
		let mut seq = String::new();
		while let Some((pos, c)) = self.chars.next_if(|&(_, c)| c != '"' && c != '\n') {
			self.set_end(pos);
			let s_char = if c == '\\' {
				self.escape_sequence()?
			} else {
				c
			};
			seq.push(s_char);
		}
		Ok(seq)
	}
	fn c_char_sequence(&mut self) -> Result<String, diag::Diagnostic> {
		let mut seq = String::new();
		while let Some((pos, c)) = self.chars.next_if(|&(_, c)| c != '\'' && c != '\n') {
			self.set_end(pos);
			let c_char = if c == '\\' {
				self.escape_sequence()?
			} else {
				c
			};
			seq.push(c_char);
		}
		Ok(seq)
	}
	fn h_char_sequence(&mut self) -> Result<String, diag::Diagnostic> {
		let mut seq = String::new();
		while let Some((_, h_char)) = self.chars.next_if(|&(_, c)| c != '>' && c != '\n') {
			seq.push(h_char);
		}
		Ok(seq)
	}
	fn q_char_sequence(&mut self) -> Result<String, diag::Diagnostic> {
		let mut seq = String::new();
		while let Some((_, q_char)) = self.chars.next_if(|&(_, c)| c != '"' && c != '\n') {
			seq.push(q_char);
		}
		Ok(seq)
	}
	fn get_newline(&mut self, new_line: char) -> tok::PPToken {
		let mut name = String::new();
		self.span.line += 1;
		name.push(new_line);
		if new_line == '\r' {
			if let Some((pos, _)) = self.chars.next_if(|&(_, c)| c == '\n') {
				self.set_end(pos);
				name.push('\n');
			}
		}
		self.include_state = 1;
		let new_line = tok::NewLine {
			name,
			is_deleted: false,
		};
		tok::PPToken {
			kind: tok::PPTokenKind::NewLine(new_line),
			leading_space: self.leading_space,
			span: self.to_span(),
		}
	}
	fn seek_end_comment(
		&mut self,
		found_end: &mut bool,
		last_c: &mut char,
	) -> Option<tok::PPToken> {
		for (_, c) in self.chars.by_ref() {
			match (*last_c, c) {
				('*', '/') => {
					*found_end = true;
					self.is_comment = false;
					break;
				}
				(
					new_line @ ('\r' | '\n' | '\u{000B}' | '\u{000C}' | '\u{0085}' | '\u{2028}'
					| '\u{2029}'),
					_,
				)
				| (
					_,
					new_line @ ('\r' | '\n' | '\u{000B}' | '\u{000C}' | '\u{0085}' | '\u{2028}'
					| '\u{2029}'),
				) => {
					return Some(self.get_newline(new_line));
				}
				_ => {
					// do nothing
				}
			}
			*last_c = c;
		}
		None
	}
}

impl Iterator for Lexer {
	type Item = Result<tok::PPToken, diag::Diagnostic>;
	fn next(&mut self) -> Option<Self::Item> {
		self.leading_space = false;
		// skip whitespace
		while let Some((pos, whitespace)) = self
			.chars
			.next_if(|&(_, c)| c != '\n' && c.is_ascii_whitespace())
		{
			match whitespace {
				' ' | '\t' => self.leading_space = true,
				_ => (),
			}
		}

		let (mut pos, mut c) = self.chars.next()?;
		self.set_start(pos);
		self.set_end(pos);
		let mut curr_pos = pos;

		if self.is_comment
			&& !matches!(
				c,
				'\r' | '\n' | '\u{000B}' | '\u{000C}' | '\u{0085}' | '\u{2028}' | '\u{2029}'
			) {
			let mut found_end = false;
			if let Some(token) = self.seek_end_comment(&mut found_end, &mut c) {
				return Some(Ok(token));
			}
			(pos, c) = self.chars.next()?;
		}

		if c == '"' {
			if self.include_state == 3 {
				return Some(self.header_name(c));
			} else {
				return Some(self.string_literal(c));
			}
		}
		if c == 'L' && self.chars.peek().is_some_and(|&(_, c)| c == '"') {
			return Some(self.string_literal(c));
		}
		if c == '\'' || c == 'L' && self.chars.peek().is_some_and(|&(_, c)| c == '\'') {
			return Some(self.character_constant(c));
		}

		let mut name = String::new();
		match c {
			new_line @ ('\r' | '\n' | '\u{000B}' | '\u{000C}' | '\u{0085}' | '\u{2028}'
			| '\u{2029}') => Some(Ok(self.get_newline(new_line))),
			// punctuators without trailing characters
			'[' | ']' | '(' | ')' | '{' | '}' | '?' | ',' | '~' | ';' => {
				self.include_state = 0;
				let punct = tok::Punct::try_from(c).unwrap();
				Some(Ok(tok::PPToken {
					kind: tok::PPTokenKind::Punct(punct),
					leading_space: self.leading_space,
					span: self.to_span(),
				}))
			}

			// identifier
			'a'..='z' | 'A'..='Z' | '_' => Some(self.identifier(c)),
			// pp-number
			'0'..='9' => {
				self.include_state = 0;
				name.push(c);
				while let Some(&(_, next_c)) = self.chars.peek() {
					if next_c.is_ascii_digit() || next_c == '.' {
						name.push(self.chars.next()?.1);
					} else if next_c.is_ascii_alphabetic() || next_c == '_' {
						let (pos, curr_c) = self.chars.next()?;
						self.set_end(pos);
						name.push(curr_c);
						if matches!(next_c, 'e' | 'E' | 'p' | 'P') {
							let Some((_, sign)) = self.chars.peek() else {
								return Some(Err(diag::Diagnostic::error(
									diag::DiagKind::UnexpectedEof,
									self.to_span(),
								)));
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
				let num = tok::PPNumber { name };
				Some(Ok(tok::PPToken {
					kind: tok::PPTokenKind::PPNumber(num),
					leading_space: self.leading_space,
					span: self.to_span(),
				}))
			}
			// `.` or `...` or pp-number
			'.' => {
				// case: `.`
				self.include_state = 0;
				if self.chars.next_if(|&(_, c)| c == '.').is_some() {
					// case: `..`
					if let Some((pos, _)) = self.chars.next_if(|&(_, c)| c == '.') {
						// case: `...`
						self.set_end(pos);
						let punct = tok::Punct::Ellipsis;
						Some(Ok(tok::PPToken {
							kind: tok::PPTokenKind::Punct(punct),
							leading_space: self.leading_space,
							span: self.to_span(),
						}))
					} else {
						Some(Err(diag::Diagnostic::error(
							diag::DiagKind::InvalidToken,
							self.to_span(),
						)))
					}
				} else if let Some((_, digit)) = self.chars.next_if(|&(_, c)| c.is_ascii_digit()) {
					// case: `.[0-9]`
					name.push(digit);
					while let Some(&(_, next_c)) = self.chars.peek() {
						if next_c.is_ascii_digit() || next_c == '.' {
							let (pos, c) = self.chars.next()?;
							self.set_end(pos);
							name.push(c);
						} else if next_c.is_ascii_alphabetic() || next_c == '_' {
							let (pos, c) = self.chars.next()?;
							self.set_end(pos);
							name.push(c);
							if matches!(next_c, 'e' | 'E' | 'p' | 'P') {
								let Some((_, sign)) = self.chars.peek() else {
									return Some(Err(diag::Diagnostic::error(
										diag::DiagKind::UnexpectedEof,
										self.to_span(),
									)));
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
					let num = tok::PPNumber { name };
					Some(Ok(tok::PPToken {
						kind: tok::PPTokenKind::PPNumber(num),
						leading_space: self.leading_space,
						span: self.to_span(),
					}))
				} else {
					Some(Ok(tok::PPToken {
						kind: tok::PPTokenKind::Punct(tok::Punct::Dot),
						leading_space: self.leading_space,
						span: self.to_span(),
					}))
				}
			}
			'#' => {
				if self.include_state == 1 {
					self.include_state = 2;
				}
				if let Some((pos, _)) = self.chars.next_if(|&(_, c)| c == '#') {
					self.include_state = 0;
					self.set_end(pos);
				}
				let punct = tok::Punct::Hash;
				Some(Ok(tok::PPToken {
					kind: tok::PPTokenKind::Punct(punct),
					leading_space: self.leading_space,
					span: self.to_span(),
				}))
			}
			'<' => {
				let term = if self.include_state == 3 {
					return Some(self.header_name(c));
				} else if self.chars.next_if(|&(_, c)| c == '<').is_some() {
					// case: `<<`
					self.set_end(pos);
					tok::Punct::LessLess
				} else if let Some((pos, _)) = self.chars.next_if(|&(_, c)| c == ':') {
					// case: `<:` => `[`
					self.set_end(pos);
					tok::Punct::LSquare
				} else if let Some((pos, _)) = self.chars.next_if(|&(_, c)| c == '%') {
					// case: `<%` => `{`
					self.set_end(pos);
					tok::Punct::LCurly
				} else {
					// case: `<`
					self.set_end(pos);
					tok::Punct::Less
				};
				Some(Ok(tok::PPToken {
					kind: tok::PPTokenKind::Punct(term),
					leading_space: self.leading_space,
					span: self.to_span(),
				}))
			}
			'/' => {
				self.include_state = 0;
				let term = if self.chars.next_if(|&(_, c)| c == '/').is_some() {
					// case: `//`
					while let Some((_, c)) = self.chars.next_if(|&(_, c)| c != '\n') {}
					return self.next();
				} else if let Some((pos, _)) = self.chars.next_if(|&(_, c)| c == '=') {
					// case: `/=`
					self.set_end(pos);
					tok::Punct::PlusEqual
				} else if self.chars.next_if(|&(_, c)| c == '*').is_some() {
					self.is_comment = true;
					let Some((pos, mut last_c)) = self.chars.next() else {
						return Some(Err(diag::Diagnostic::error(
							diag::DiagKind::UnexpectedEof,
							self.to_span(),
						)));
					};
					let mut found_end = false;
					// println!("multi comment");
					if let Some(token) = self.seek_end_comment(&mut found_end, &mut last_c) {
						return Some(Ok(token));
					}
					// println!("end multi comment");

					self.set_end(pos);
					if found_end {
						self.is_comment = false;
						return self.next();
					} else {
						return Some(Err(diag::Diagnostic::error(
							diag::DiagKind::UnexpectedEof,
							self.to_span(),
						)));
					}
				} else {
					tok::Punct::FSlash
				};
				Some(Ok(tok::PPToken {
					kind: tok::PPTokenKind::Punct(term),
					leading_space: self.leading_space,
					span: self.to_span(),
				}))
			}
			'\\' => match self.next() {
				Some(Ok(tok::PPToken {
					kind: tok::PPTokenKind::NewLine(mut new_line),
					..
				})) => {
					new_line.is_deleted = true;
					Some(Ok(tok::PPToken {
						kind: tok::PPTokenKind::NewLine(new_line),
						leading_space: self.leading_space,
						span: self.to_span(),
					}))
				}
				Some(Ok(_)) => Some(Err(diag::Diagnostic::error(
					diag::DiagKind::InvalidToken,
					self.to_span(),
				))),
				Some(Err(error)) => Some(Err(error)),
				None => Some(Err(diag::Diagnostic::error(
					diag::DiagKind::UnexpectedEof,
					self.to_span(),
				))),
			},
			'+' => {
				self.include_state = 0;
				let term = if let Some((pos, _)) = self.chars.next_if(|&(_, c)| c == '+') {
					// case: `++`
					self.set_end(pos);
					tok::Punct::PlusPlus
				} else if let Some((pos, _)) = self.chars.next_if(|&(_, c)| c == '=') {
					// case: `+=`
					self.set_end(pos);
					tok::Punct::PlusEqual
				} else {
					// case: `+`
					tok::Punct::Plus
				};
				Some(Ok(tok::PPToken {
					kind: tok::PPTokenKind::Punct(term),
					leading_space: self.leading_space,
					span: self.to_span(),
				}))
			}
			'-' => {
				self.include_state = 0;
				let term = if let Some((pos, _)) = self.chars.next_if(|&(_, c)| c == '-') {
					// case: `--`
					self.set_end(pos);
					tok::Punct::MinusMinus
				} else if let Some((pos, _)) = self.chars.next_if(|&(_, c)| c == '=') {
					// case: `-=`
					self.set_end(pos);
					tok::Punct::MinusEqual
				} else {
					// case: `-`
					tok::Punct::Minus
				};
				Some(Ok(tok::PPToken {
					kind: tok::PPTokenKind::Punct(term),
					leading_space: self.leading_space,
					span: self.to_span(),
				}))
			}
			'=' => {
				self.include_state = 0;
				let term = if let Some((pos, _)) = self.chars.next_if(|&(_, c)| c == '=') {
					// case: `==`
					self.set_end(pos);
					tok::Punct::EqualEqual
				} else {
					// case: `=`
					tok::Punct::Equal
				};
				Some(Ok(tok::PPToken {
					kind: tok::PPTokenKind::Punct(term),
					leading_space: self.leading_space,
					span: self.to_span(),
				}))
			}
			'*' => {
				self.include_state = 0;
				let term = if let Some((pos, _)) = self.chars.next_if(|&(_, c)| c == '=') {
					// case: `*=`
					self.set_end(pos);
					tok::Punct::StarEqual
				} else {
					// case: `*`
					tok::Punct::Star
				};
				Some(Ok(tok::PPToken {
					kind: tok::PPTokenKind::Punct(term),
					leading_space: self.leading_space,
					span: self.to_span(),
				}))
			}
			':' => {
				self.include_state = 0;
				let term = if let Some((pos, _)) = self.chars.next_if(|&(_, c)| c == '>') {
					// case: `:>`
					self.set_end(pos);
					tok::Punct::RSquare
				} else {
					// case: `:`
					tok::Punct::Colon
				};
				Some(Ok(tok::PPToken {
					kind: tok::PPTokenKind::Punct(term),
					leading_space: self.leading_space,
					span: self.to_span(),
				}))
			}
			'!' => {
				self.include_state = 0;
				let term = if let Some((pos, _)) = self.chars.next_if(|&(_, c)| c == '=') {
					// case: `!=`
					self.set_end(pos);
					tok::Punct::BangEqual
				} else {
					// case: `!`
					tok::Punct::Bang
				};
				Some(Ok(tok::PPToken {
					kind: tok::PPTokenKind::Punct(term),
					leading_space: self.leading_space,
					span: self.to_span(),
				}))
			}
			'&' => {
				self.include_state = 0;
				let term = if let Some((pos, _)) = self.chars.next_if(|&(_, c)| c == '&') {
					// case: `&&`
					self.set_end(pos);
					tok::Punct::AmpAmp
				} else {
					// case: `&`
					tok::Punct::Amp
				};
				Some(Ok(tok::PPToken {
					kind: tok::PPTokenKind::Punct(term),
					leading_space: self.leading_space,
					span: self.to_span(),
				}))
			}
			_ => todo!("{}", c as i32),
		}
	}
}
