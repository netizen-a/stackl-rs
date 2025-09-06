use std::fmt::Debug;
use std::iter::{Enumerate, Peekable};
use std::vec::IntoIter;

use crate::analysis::tok;
use crate::diagnostics::{self as diag, lex};

#[derive(Debug)]
pub struct Lexer {
	chars: Peekable<Enumerate<IntoIter<char>>>,
	file_id: usize,
	leading_space: bool,
	location: (usize, usize),
	include_state: u8,
}

impl Lexer {
	pub fn new(text: String, file_id: usize) -> Self {
		let char_vec: Vec<char> = text.chars().collect();
		let char_iter = char_vec.into_iter();
		Self {
			chars: char_iter.enumerate().peekable(),
			file_id,
			leading_space: false,
			location: (0, 0),
			include_state: 1,
		}
	}

	fn set_start(&mut self, start: usize) {
		self.location.0 = start;
	}
	fn set_end(&mut self, end: usize) {
		self.location.1 = end;
	}
	fn pop_location(&mut self) -> (usize, usize) {
		let result = self.location;
		self.location = (0, 0);
		result
	}

	#[allow(dead_code)]
	fn header_name(&mut self, c: char) -> diag::ResultTriple<tok::PPToken, usize> {
		let mut name = String::new();
		// name.push(c);
		let is_builtin;
		let char_seq = match c {
			'<' => {
				is_builtin = true;
				let seq = self.h_char_sequence()?;
				if self.chars.next_if(|(_, c)| *c == '>').is_none() {
					return Err(diag::Diagnostic {
						level: diag::DiagLevel::Error,
						kind: diag::DiagKind::InvalidToken,
						span: diag::Span {
							loc: self.pop_location(),
							file_id: self.file_id,
						},
					});
				}
				seq
			}
			'"' => {
				is_builtin = false;
				let seq = self.q_char_sequence()?;
				if self.chars.next_if(|(_, c)| *c == '"').is_none() {
					return Err(diag::Diagnostic {
						level: diag::DiagLevel::Error,
						kind: diag::DiagKind::InvalidToken,
						span: diag::Span {
							loc: self.pop_location(),
							file_id: self.file_id,
						},
					});
				}
				seq
			}
			_ => unreachable!(),
		};
		name.push_str(&char_seq.1);
		self.include_state = 0;
		let head_name = tok::HeaderName { is_builtin, name };
		let (lo, hi) = self.pop_location();
		Ok((
			lo,
			tok::PPToken {
				kind: tok::PPTokenKind::HeaderName(head_name),
				file_id: self.file_id,
				leading_space: self.leading_space,
			},
			hi,
		))
	}

	fn identifier(&mut self, c: char) -> diag::ResultTriple<tok::PPToken, usize> {
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
		};
		let (lo, hi) = self.pop_location();
		Ok((
			lo,
			tok::PPToken {
				kind: tok::PPTokenKind::Ident(ident),
				file_id: self.file_id,
				leading_space: self.leading_space,
			},
			hi,
		))
	}

	#[allow(dead_code)]
	fn pp_number(&mut self) -> diag::ResultTriple<tok::PPToken, usize> {
		todo!("pp-number")
	}

	fn character_constant(&mut self, mut c: char) -> diag::ResultTriple<tok::PPToken, usize> {
		self.include_state = 0;
		let is_wide = c == 'L';
		if is_wide {
			if let Some((_, next_c)) = self.chars.next() {
				#[allow(unused_assignments)]
				{
					c = next_c;
				}
			} else {
				return Err(diag::Diagnostic {
					level: diag::DiagLevel::Error,
					kind: diag::DiagKind::UnexpectedEof,
					span: diag::Span {
						loc: self.pop_location(),
						file_id: self.file_id,
					},
				});
			}
		}
		let seq = self.c_char_sequence()?;
		if let Some((curr_pos, _)) = self.chars.next_if(|&(_, c)| c == '\'') {
			// name.push('\'');
			self.set_end(curr_pos);
		} else {
			return Err(diag::Diagnostic {
				level: diag::DiagLevel::Error,
				kind: diag::DiagKind::InvalidToken,
				span: diag::Span {
					loc: self.pop_location(),
					file_id: self.file_id,
				},
			});
		}

		let str_lit = tok::CharConst { seq, is_wide };
		let (lo, hi) = self.pop_location();
		Ok((
			lo,
			tok::PPToken {
				kind: tok::PPTokenKind::CharConst(str_lit),
				file_id: self.file_id,
				leading_space: self.leading_space,
			},
			hi,
		))
	}

	fn string_literal(&mut self, c: char) -> diag::ResultTriple<tok::PPToken, usize> {
		let is_wide = c == 'L';
		if is_wide {
			if let Some((pos, _)) = self.chars.next() {
				self.set_end(pos);
			} else {
				return Err(diag::Diagnostic {
					level: diag::DiagLevel::Error,
					kind: diag::DiagKind::UnexpectedEof,
					span: diag::Span {
						loc: self.pop_location(),
						file_id: self.file_id,
					},
				});
			}
		}
		let seq = self.s_char_sequence()?;
		if let Some((pos, _)) = self.chars.next_if(|&(_, c)| c == '"') {
			self.set_end(pos);
		} else {
			return Err(diag::Diagnostic {
				level: diag::DiagLevel::Error,
				kind: diag::DiagKind::InvalidToken,
				span: diag::Span {
					loc: self.pop_location(),
					file_id: self.file_id,
				},
			});
		}

		let str_lit = tok::StrLit { seq, is_wide };
		let (lo, hi) = self.pop_location();
		Ok((
			lo,
			tok::PPToken {
				kind: tok::PPTokenKind::StrLit(str_lit),
				file_id: self.file_id,
				leading_space: self.leading_space,
			},
			hi,
		))
	}
	#[allow(dead_code)]
	fn punctuator(&mut self) -> diag::ResultTriple<tok::PPToken, usize> {
		todo!("punctuator")
	}

	fn escape_sequence(&mut self) -> diag::Result<char> {
		let Some((curr_pos, term)) = self.chars.next() else {
			return Err(diag::Diagnostic {
				level: diag::DiagLevel::Error,
				kind: diag::DiagKind::UnexpectedEscape,
				span: diag::Span {
					loc: self.pop_location(),
					file_id: self.file_id,
				},
			});
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
			_ => Err(diag::Diagnostic {
				level: diag::DiagLevel::Error,
				kind: diag::DiagKind::UnexpectedEscape,
				span: diag::Span {
					loc: self.pop_location(),
					file_id: self.file_id,
				},
			}),
		}
	}

	fn s_char_sequence(&mut self) -> diag::Result<String> {
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
	fn c_char_sequence(&mut self) -> diag::Result<String> {
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
	fn h_char_sequence(&mut self) -> diag::ResultTriple<String, usize> {
		let mut seq = String::new();
		while let Some((_, h_char)) = self.chars.next_if(|&(_, c)| c != '>' && c != '\n') {
			seq.push(h_char);
		}
		Ok((0, seq, 0))
	}
	fn q_char_sequence(&mut self) -> diag::ResultTriple<String, usize> {
		let mut seq = String::new();
		while let Some((_, q_char)) = self.chars.next_if(|&(_, c)| c != '"' && c != '\n') {
			seq.push(q_char);
		}
		Ok((0, seq, 0))
	}
}

impl Iterator for Lexer {
	type Item = diag::ResultTriple<tok::PPToken, usize>;
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

		let (pos, c) = self.chars.next()?;
		self.set_start(pos);
		self.set_end(pos);
		let mut curr_pos = pos;

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
			| '\u{2029}') => {
				name.push(new_line);
				if c == '\r' {
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
				let (lo, hi) = self.pop_location();
				Some(Ok((
					lo,
					tok::PPToken {
						kind: tok::PPTokenKind::NewLine(new_line),
						file_id: self.file_id,
						leading_space: self.leading_space,
					},
					hi,
				)))
			}
			// punctuators without trailing characters
			'[' | ']' | '(' | ')' | '{' | '}' | '?' | ',' | '~' | ';' => {
				self.include_state = 0;
				let (lo, hi) = self.pop_location();
				let punct = tok::Punct::try_from(c).unwrap();
				Some(Ok((
					lo,
					tok::PPToken {
						kind: tok::PPTokenKind::Punct(punct),
						file_id: self.file_id,
						leading_space: self.leading_space,
					},
					hi,
				)))
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
								return Some(Err(diag::Diagnostic {
									level: diag::DiagLevel::Error,
									kind: diag::DiagKind::UnexpectedEof,
									span: diag::Span {
										loc: self.pop_location(),
										file_id: self.file_id,
									},
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
				let num = tok::PPNumber { name };
				let (lo, hi) = self.pop_location();
				Some(Ok((
					lo,
					tok::PPToken {
						kind: tok::PPTokenKind::PPNumber(num),
						file_id: self.file_id,
						leading_space: self.leading_space,
					},
					hi,
				)))
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
						self.set_end(pos);
						let punct = tok::Punct::Ellipsis;
						let (lo, hi) = self.pop_location();
						Some(Ok((
							lo,
							tok::PPToken {
								kind: tok::PPTokenKind::Punct(punct),
								file_id: self.file_id,
								leading_space: self.leading_space,
							},
							hi,
						)))
					} else {
						Some(Err(diag::Diagnostic {
							level: diag::DiagLevel::Error,
							kind: diag::DiagKind::InvalidToken,
							span: diag::Span {
								loc: self.pop_location(),
								file_id: self.file_id,
							},
						}))
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
									return Some(Err(diag::Diagnostic {
										level: diag::DiagLevel::Error,
										kind: diag::DiagKind::UnexpectedEof,
										span: diag::Span {
											loc: self.pop_location(),
											file_id: self.file_id,
										},
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
					let num = tok::PPNumber { name };
					let (lo, hi) = self.pop_location();
					Some(Ok((
						lo,
						tok::PPToken {
							kind: tok::PPTokenKind::PPNumber(num),
							file_id: self.file_id,
							leading_space: self.leading_space,
						},
						hi,
					)))
				} else {
					Some(Err(diag::Diagnostic {
						level: diag::DiagLevel::Error,
						kind: diag::DiagKind::InvalidToken,
						span: diag::Span {
							loc: self.pop_location(),
							file_id: self.file_id,
						},
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
					self.set_end(pos);
				}
				let punct = tok::Punct::Hash;
				let (lo, hi) = self.pop_location();
				Some(Ok((
					lo,
					tok::PPToken {
						kind: tok::PPTokenKind::Punct(punct),
						file_id: self.file_id,
						leading_space: self.leading_space,
					},
					hi,
				)))
			}
			'<' => {
				let term = if self.include_state == 3 {
					return Some(self.header_name(c));
				} else if self.chars.next_if(|&(_, c)| c == '<').is_some() {
					// case: `<<`
					todo!("<<")
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
					tok::Punct::Less
				};
				let (lo, hi) = self.pop_location();
				Some(Ok((
					lo,
					tok::PPToken {
						kind: tok::PPTokenKind::Punct(term),
						file_id: self.file_id,
						leading_space: self.leading_space,
					},
					hi,
				)))
			}
			'/' => {
				self.include_state = 0;
				let term = if self.chars.next_if(|&(_, c)| c == '/').is_some() {
					// case: `//`
					name.push_str("//");
					while let Some((_, c)) = self.chars.next_if(|&(_, c)| c != '\n') {
						name.push(c);
					}
					return self.next();
				} else if let Some((pos, _)) = self.chars.next_if(|&(_, c)| c == '=') {
					// case: `/=`
					self.set_end(pos);
					tok::Punct::PlusEqual
				} else if self.chars.next_if(|&(_, c)| c == '*').is_some() {
					name.push_str("/*");
					let Some((pos, mut last_c)) = self.chars.next() else {
						return Some(Err(diag::Diagnostic {
							level: diag::DiagLevel::Error,
							kind: diag::DiagKind::UnexpectedEof,
							span: diag::Span {
								loc: self.pop_location(),
								file_id: self.file_id,
							},
						}));
					};
					self.set_end(pos);
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
					if found_end {
						return self.next();
					} else {
						return Some(Err(diag::Diagnostic {
							level: diag::DiagLevel::Error,
							kind: diag::DiagKind::UnexpectedEof,
							span: diag::Span {
								loc: self.pop_location(),
								file_id: self.file_id,
							},
						}));
					}
				} else {
					tok::Punct::FSlash
				};
				let (lo, hi) = self.pop_location();
				Some(Ok((
					lo,
					tok::PPToken {
						kind: tok::PPTokenKind::Punct(term),
						file_id: self.file_id,
						leading_space: self.leading_space,
					},
					hi,
				)))
			}
			'\\' => match self.next() {
				Some(Ok((
					lo,
					tok::PPToken {
						kind: tok::PPTokenKind::NewLine(mut new_line),
						..
					},
					hi,
				))) => {
					new_line.is_deleted = true;
					Some(Ok((
						lo,
						tok::PPToken {
							kind: tok::PPTokenKind::NewLine(new_line),
							file_id: self.file_id,
							leading_space: self.leading_space,
						},
						hi,
					)))
				}
				Some(Ok(_)) => Some(Err(diag::Diagnostic {
					level: diag::DiagLevel::Error,
					kind: diag::DiagKind::InvalidToken,
					span: diag::Span {
						loc: self.pop_location(),
						file_id: self.file_id,
					},
				})),
				Some(Err(error)) => Some(Err(error)),
				None => Some(Err(diag::Diagnostic {
					level: diag::DiagLevel::Error,
					kind: diag::DiagKind::UnexpectedEof,
					span: diag::Span {
						loc: self.pop_location(),
						file_id: self.file_id,
					},
				})),
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
				let (lo, hi) = self.pop_location();
				Some(Ok((
					lo,
					tok::PPToken {
						kind: tok::PPTokenKind::Punct(term),
						file_id: self.file_id,
						leading_space: self.leading_space,
					},
					hi,
				)))
			}
			'-' => {
				self.include_state = 0;
				let term = if let Some((pos, _)) = self.chars.next_if(|&(_, c)| c == '-') {
					// case: `--`
					self.set_end(pos);
					tok::Punct::MinusMinus
				} else if let Some((pos, _)) = self.chars.next_if(|&(_, c)| c == '=') {
					// case: `+=`
					self.set_end(pos);
					tok::Punct::MinusEqual
				} else {
					// case: `+`
					tok::Punct::Minus
				};
				let (lo, hi) = self.pop_location();
				Some(Ok((
					lo,
					tok::PPToken {
						kind: tok::PPTokenKind::Punct(term),
						file_id: self.file_id,
						leading_space: self.leading_space,
					},
					hi,
				)))
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
				let (lo, hi) = self.pop_location();
				Some(Ok((
					lo,
					tok::PPToken {
						kind: tok::PPTokenKind::Punct(term),
						file_id: self.file_id,
						leading_space: self.leading_space,
					},
					hi,
				)))
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
				let (lo, hi) = self.pop_location();
				Some(Ok((
					lo,
					tok::PPToken {
						kind: tok::PPTokenKind::Punct(term),
						file_id: self.file_id,
						leading_space: self.leading_space,
					},
					hi,
				)))
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
				let (lo, hi) = self.pop_location();
				Some(Ok((
					lo,
					tok::PPToken {
						kind: tok::PPTokenKind::Punct(term),
						file_id: self.file_id,
						leading_space: self.leading_space,
					},
					hi,
				)))
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
				let (lo, hi) = self.pop_location();
				Some(Ok((
					lo,
					tok::PPToken {
						kind: tok::PPTokenKind::Punct(term),
						file_id: self.file_id,
						leading_space: self.leading_space,
					},
					hi,
				)))
			}
			_ => todo!("{}", c as i32),
		}
	}
}
