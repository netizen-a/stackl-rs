//! Lexical Elements

pub mod keyword;
pub mod punct;

use crate::diag::lex;
pub use keyword::*;
pub use punct::*;
use std::iter::Peekable;
use std::str::Chars;

#[derive(Debug, Clone)]
pub struct Ident(pub String);

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
	CharConst(CharConst),
}

#[derive(Debug, Clone)]
pub struct StringLiteral {
	pub name: String,
	pub is_wide: bool,
}

#[derive(Debug, Clone)]
pub struct HeaderName {
	pub is_builtin: bool,
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
			Ok(Token::Constant(Constant::Floating(floating)))
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
			Ok(Token::Constant(constant))
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
			Ok(Token::Constant(constant))
		} else {
			todo!("error decimal-constant")
		}
	}
}

#[derive(Debug, Clone)]
pub struct CharConst {
	pub name: String,
	pub is_wide: bool,
}

#[derive(Debug, Clone)]
pub struct NewLine {
	pub name: String,
	pub is_deleted: bool,
}

#[derive(Debug)]
pub enum Token {
	Keyword(Keyword),
	Ident(Ident),
	Constant(Constant),
	StringLiteral(StringLiteral),
	Punct(Punct),
}

impl Token {
	pub fn is_keyword(&self) -> bool {
		matches!(self, Self::Keyword(_))
	}
	pub fn is_ident(&self) -> bool {
		matches!(self, Self::Ident(_))
	}
	pub fn is_constant(&self) -> bool {
		matches!(self, Self::Constant(_))
	}
	pub fn is_string_literal(&self) -> bool {
		matches!(self, Self::StringLiteral(_))
	}
	pub fn is_punct(&self) -> bool {
		matches!(self, Self::Punct(_))
	}
	pub fn unwrap_keyword(self) -> Keyword {
		match self {
			Self::Keyword(token) => token,
			other => panic!("called `Token::unwrap_keyword` on an `{other:?}` value"),
		}
	}
	pub fn unwrap_ident(self) -> Ident {
		match self {
			Self::Ident(token) => token,
			other => panic!("called `Token::unwrap_ident` on an `{other:?}` value"),
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
	pub fn unwrap_punct(self) -> Punct {
		match self {
			Self::Punct(token) => token,
			other => panic!("called `Token::unwrap_punct` on an `{other:?}` value"),
		}
	}
}

impl From<Punct> for Token {
	fn from(value: Punct) -> Self {
		Self::Punct(value)
	}
}

impl From<Ident> for Token {
	fn from(value: Ident) -> Self {
		Self::Ident(value)
	}
}

impl From<StringLiteral> for Token {
	fn from(value: StringLiteral) -> Self {
		Self::StringLiteral(value)
	}
}

impl TryFrom<PPNumber> for Token {
	type Error = lex::Error;
	fn try_from(value: PPNumber) -> Result<Self, Self::Error> {
		if value.is_float() {
			value.floating_constant()
		} else {
			value.integer_constant()
		}
	}
}

#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum PPTokenKind {
	HeaderName(HeaderName),
	Ident(Ident),
	PPNumber(PPNumber),
	CharConst(CharConst),
	StringLiteral(StringLiteral),
	Punct(Punct),
	NewLine(NewLine),
}

impl PPTokenKind {
	pub fn as_token_name(&self) -> &str {
		match self {
			Self::HeaderName(_) => "header-name",
			Self::Ident(_) => "identifier",
			Self::PPNumber(_) => "pp-number",
			Self::CharConst(_) => "character-constant",
			Self::StringLiteral(_) => "string-literal",
			Self::Punct(_) => "punctuator",
			Self::NewLine(_) => "new-line",
		}
	}
	pub fn to_name(&self) -> String {
		match self {
			Self::HeaderName(value) => value.name.clone(),
			Self::Ident(value) => value.0.clone(),
			Self::PPNumber(value) => value.name.clone(),
			Self::CharConst(value) => value.name.clone(),
			Self::StringLiteral(value) => value.name.clone(),
			Self::Punct(value) => format!("{value}"),
			Self::NewLine(_) => String::from("\\n"),
		}
	}
}

impl From<Punct> for PPTokenKind {
	fn from(value: Punct) -> Self {
		Self::Punct(value)
	}
}

#[derive(Debug, Clone)]
pub struct PPToken {
	pub kind: PPTokenKind,
	pub leading_spaces: usize,
	pub leading_tabs: usize,
}
impl PPToken {
	pub fn unwrap_ident(self) -> Ident {
		match self.kind {
			PPTokenKind::Ident(token) => token,
			other => panic!("called `Token::unwrap_ident` on an `{other:?}` value"),
		}
	}
	pub fn unwrap_punct(self) -> Punct {
		match self.kind {
			PPTokenKind::Punct(token) => token,
			other => panic!("called `Token::unwrap_punctuator` on an `{other:?}` value"),
		}
	}
	pub fn unwrap_string_literal(self) -> StringLiteral {
		match self.kind {
			PPTokenKind::StringLiteral(token) => token,
			other => panic!("called `Token::unwrap_string_literal` on an `{other:?}` value"),
		}
	}
	pub fn unwrap_char_const(self) -> CharConst {
		match self.kind {
			PPTokenKind::CharConst(token) => token,
			other => panic!("called `Token::unwrap_char_const` on an `{other:?}` value"),
		}
	}
	pub fn unwrap_pp_number(self) -> PPNumber {
		match self.kind {
			PPTokenKind::PPNumber(token) => token,
			other => panic!("called `Token::unwrap_pp_number` on an `{other:?}` value"),
		}
	}
}
