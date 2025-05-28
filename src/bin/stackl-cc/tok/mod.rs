//! Lexical Elements

pub mod keyword;
pub mod punct;

use crate::diag::lex;
pub use keyword::*;
pub use punct::*;
use std::iter::Peekable;
use std::str::Chars;

#[derive(Debug, Clone)]
pub struct Identifier(pub String);

#[derive(Debug)]
pub enum IntegerSuffix {
	None,
	U,
	L,
	UL,
	LL,
	#[allow(clippy::upper_case_acronyms)]
	ULL,
}

#[derive(Debug)]
pub enum IntegerConstant {
	U32(u32),
	I32(i32),
	U64(u64),
	I64(i64),
	U128(u128),
	I128(i128),
}

#[derive(Debug)]
pub enum FloatingConstant {
	F32(f32),
	F64(f64),
	// same as F64, but higher rank
	Long(f64),
}

#[derive(Debug)]
pub enum Constant {
	Integer(IntegerConstant),
	Floating(FloatingConstant),
	Enumeration,
	Character(CharacterConstant),
}

#[derive(Debug, Clone)]
pub struct StringLiteral {
	pub name: String,
}

#[derive(Debug, Clone)]
pub struct HeaderName {
	pub is_std: bool,
	pub name: String,
}

#[derive(Debug, Clone)]
pub struct PPNumber {
	pub name: String,
}

impl PPNumber {
	pub fn is_float(&self) -> bool {
		if self.name.contains('.') {
			// fractional-constant | hexadecimal-fractional-constant
			return true;
		}
		let mut chars = self.name.chars().peekable();
		let Some(c) = chars.next() else {
			return false;
		};
		if c == '0' && chars.next_if(|&c| c == 'x' || c == 'X').is_some() {
			// binary-exponent-part
			chars.any(|c| c == 'p' || c == 'P')
		} else {
			// exponent-part
			chars.any(|c| c == 'e' || c == 'E')
		}
	}
	fn floating_constant(&self) -> lex::Result<Token> {
		let mut chars = self.name.chars().peekable();
		let c = chars.next().expect("empty pp-number");
		if c == '0' && chars.next_if(|&c| c == 'x' || c == 'X').is_some() {
			todo!("hexadecimal-floating-constant")
		} else {
			let mut decimal = String::new();
			while let Some(c) = chars.next_if(|&c| c.is_ascii_digit() || c == '.') {
				decimal.push(c);
			}
			let mut exponent: i32 = 1;
			if chars.next_if(|&c| c == 'e' || c == 'E').is_some() {
				let sign = chars.next_if(|&c| c == '+' || c == '-');
				let mut digit_seq = String::new();
				while let Some(c) = chars.next_if(|&c| c.is_ascii_digit()) {
					digit_seq.push(c);
				}
				exponent = digit_seq.parse().or(Err(lex::Error {
					kind: lex::ErrorKind::InvalidToken,
					loc: (0, 0),
				}))?;
				if let Some('-') = sign {
					exponent *= -1;
				}
			}
			let floating = match chars.next_if(|&c| c == 'f' || c == 'F' || c == 'l' || c == 'L') {
				Some('f' | 'F') => {
					let data: f32 = decimal.parse().or(Err(lex::Error {
						kind: lex::ErrorKind::InvalidToken,
						loc: (0, 0),
					}))?;
					let data = data.powi(exponent);
					FloatingConstant::F32(data)
				}
				Some('l' | 'L') => {
					let data: f64 = decimal.parse().or(Err(lex::Error {
						kind: lex::ErrorKind::InvalidToken,
						loc: (0, 0),
					}))?;
					let data = data.powi(exponent);
					FloatingConstant::Long(data)
				}
				None => {
					let data: f64 = decimal.parse().or(Err(lex::Error {
						kind: lex::ErrorKind::InvalidToken,
						loc: (0, 0),
					}))?;
					let data = data.powi(exponent);
					FloatingConstant::F64(data)
				}
				_ => unreachable!(),
			};
			Ok(Token {
				kind: TokenKind::Constant(Constant::Floating(floating)),
				leading_spaces: 0,
				leading_tabs: 0,
			})
		}
	}

	fn integer_constant(&self) -> lex::Result<Token> {
		let mut chars = self.name.chars().peekable();
		match chars.next().expect("empty pp-number") {
			'0' => {
				if chars.next_if(|&c| c == 'x' || c == 'X').is_some() {
					todo!("hexadecimal-constant")
				} else {
					self.octal_constant(chars)
				}
			}
			c @ '1'..='9' => {
				let name = String::from(c);
				self.decimal_constant(name, chars)
			}
			_ => Err(lex::Error {
				kind: lex::ErrorKind::InvalidToken,
				loc: (0, 0),
			}),
		}
	}
	fn octal_constant(&self, mut chars: Peekable<Chars>) -> lex::Result<Token> {
		let mut name = String::new();
		while let Some(digit) = chars.next_if(char::is_ascii_digit) {
			name.push(digit);
		}
		if chars.next_if(|&c| c == 'u' || c == 'U').is_some() {
			todo!("unsigned-suffix")
		} else if chars.next_if(|&c| c == 'l' || c == 'L').is_some() {
			todo!("long-suffix")
		} else if chars.peek().is_none() {
			let integer = if let Ok(data) = i32::from_str_radix(&name, 8) {
				IntegerConstant::I32(data)
			} else if let Ok(data) = i64::from_str_radix(&name, 8) {
				IntegerConstant::I64(data)
			} else if let Ok(data) = i128::from_str_radix(&name, 8) {
				IntegerConstant::I128(data)
			} else {
				return Err(lex::Error {
					kind: lex::ErrorKind::InvalidToken,
					loc: (0, 0),
				});
			};
			let constant = Constant::Integer(integer);
			Ok(Token {
				kind: TokenKind::Constant(constant),
				leading_spaces: 0,
				leading_tabs: 0,
			})
		} else {
			todo!("error octal-constant")
		}
	}

	fn decimal_constant(&self, mut name: String, mut chars: Peekable<Chars>) -> lex::Result<Token> {
		while let Some(digit) = chars.next_if(char::is_ascii_digit) {
			name.push(digit);
		}
		if chars.next_if(|&c| c == 'u' || c == 'U').is_some() {
			todo!("unsigned-suffix")
		} else if chars.next_if(|&c| c == 'l' || c == 'L').is_some() {
			todo!("long-suffix")
		} else if chars.peek().is_none() {
			let integer = if let Ok(data) = name.parse::<i32>() {
				IntegerConstant::I32(data)
			} else if let Ok(data) = name.parse::<i64>() {
				IntegerConstant::I64(data)
			} else if let Ok(data) = name.parse::<i128>() {
				IntegerConstant::I128(data)
			} else {
				return Err(lex::Error {
					kind: lex::ErrorKind::InvalidToken,
					loc: (0, 0),
				});
			};
			let constant = Constant::Integer(integer);
			Ok(Token {
				kind: TokenKind::Constant(constant),
				leading_spaces: 0,
				leading_tabs: 0,
			})
		} else {
			todo!("error decimal-constant")
		}
	}
}

#[derive(Debug, Clone)]
pub struct CharacterConstant(pub String);

#[derive(Debug, Clone)]
pub struct NewLine {
	pub name: String,
	pub is_deleted: bool,
}

#[derive(Debug, Clone)]
pub struct Comment(pub String);

#[derive(Debug)]
pub enum TokenKind {
	Keyword(Keyword),
	Identifier(Identifier),
	Constant(Constant),
	StringLiteral(StringLiteral),
	Punctuator(Punctuator),
}

impl TokenKind {
	pub fn is_keyword(&self) -> bool {
		matches!(self, Self::Keyword(_))
	}
	pub fn is_identifier(&self) -> bool {
		matches!(self, Self::Identifier(_))
	}
	pub fn is_constant(&self) -> bool {
		matches!(self, Self::Constant(_))
	}
	pub fn is_string_literal(&self) -> bool {
		matches!(self, Self::StringLiteral(_))
	}
	pub fn is_punctuator(&self) -> bool {
		matches!(self, Self::Punctuator(_))
	}
	pub fn unwrap_keyword(self) -> Keyword {
		match self {
			Self::Keyword(token) => token,
			other => panic!("called `Token::unwrap_keyword` on an `{other:?}` value"),
		}
	}
	pub fn unwrap_identifier(self) -> Identifier {
		match self {
			Self::Identifier(token) => token,
			other => panic!("called `Token::unwrap_identifier` on an `{other:?}` value"),
		}
	}
	pub fn unwrap_constant(self) -> Constant {
		match self {
			Self::Constant(token) => token,
			other => panic!("called `Token::unwrap_constant` on an `{other:?}` value"),
		}
	}
	pub fn unwrap_string_literal(self) -> StringLiteral {
		match self {
			Self::StringLiteral(token) => token,
			other => panic!("called `Token::unwrap_string_literal` on an `{other:?}` value"),
		}
	}
	pub fn unwrap_punctuator(self) -> Punctuator {
		match self {
			Self::Punctuator(token) => token,
			other => panic!("called `Token::unwrap_punctuator` on an `{other:?}` value"),
		}
	}
}

pub struct Token {
	kind: TokenKind,
	leading_spaces: usize,
	leading_tabs: usize,
}

#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum PPTokenKind {
	HeaderName(HeaderName),
	Identifier(Identifier),
	PPNumber(PPNumber),
	CharacterConstant(CharacterConstant),
	StringLiteral(StringLiteral),
	Punctuator(Punctuator),
	NewLine(NewLine),
	Comment(Comment),
}

impl PPTokenKind {
	pub fn as_token_name(&self) -> &str {
		match self {
			Self::HeaderName(_) => "header-name",
			Self::Identifier(_) => "identifier",
			Self::PPNumber(_) => "pp-number",
			Self::CharacterConstant(_) => "character-constant",
			Self::StringLiteral(_) => "string-literal",
			Self::Punctuator(_) => "punctuator",
			Self::NewLine(_) => "new-line",
			Self::Comment(_) => "comment",
		}
	}
	pub fn to_name(&self) -> String {
		match self {
			Self::HeaderName(value) => value.name.clone(),
			Self::Identifier(value) => value.0.clone(),
			Self::PPNumber(value) => value.name.clone(),
			Self::CharacterConstant(value) => value.0.clone(),
			Self::StringLiteral(value) => value.name.clone(),
			Self::Punctuator(value) => format!("{value}"),
			Self::NewLine(_) => String::from("\\n"),
			Self::Comment(value) => value.0.clone(),
		}
	}
}

#[derive(Debug, Clone)]
pub struct PPToken {
	pub kind: PPTokenKind,
	pub leading_spaces: usize,
	pub leading_tabs: usize,
}
