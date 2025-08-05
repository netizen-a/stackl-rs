//! Lexical Elements

pub mod keyword;
pub mod punct;

use crate::analysis::prt::lex;
pub use keyword::*;
pub use punct::*;
use std::iter::Peekable;
use std::str::Chars;

#[derive(Debug, Clone)]
pub struct Ident {
	pub name: String,
	/// is the identifier previously declared in a typedef?
	pub is_type: bool,
}

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

#[derive(Debug, Clone)]
pub enum IntegerConstant {
	U32(u32),
	I32(i32),
	U64(u64),
	I64(i64),
	U128(u128),
	I128(i128),
}

#[derive(Debug, Clone)]
pub enum FloatingConstant {
	F32(f32),
	F64(f64),
	// same as F64, but higher rank
	Long(f64),
}

#[derive(Debug, Clone)]
pub enum Const {
	Integer(IntegerConstant),
	Floating(FloatingConstant),
	Enumeration,
	CharConst(CharConst),
}

#[derive(Debug, Clone)]
pub struct StrLit {
	pub seq: String,
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
	fn floating_constant(&self) -> Result<TokenKind, lex::ErrorKind> {
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
				exponent = digit_seq.parse().or(Err(lex::ErrorKind::InvalidToken))?;
				if let Some('-') = sign {
					exponent *= -1;
				}
			}
			let floating = match chars.next_if(|&c| c == 'f' || c == 'F' || c == 'l' || c == 'L') {
				Some('f' | 'F') => {
					let data: f32 = decimal.parse().or(Err(lex::ErrorKind::InvalidToken))?;
					let data = data.powi(exponent);
					FloatingConstant::F32(data)
				}
				Some('l' | 'L') => {
					let data: f64 = decimal.parse().or(Err(lex::ErrorKind::InvalidToken))?;
					let data = data.powi(exponent);
					FloatingConstant::Long(data)
				}
				None => {
					let data: f64 = decimal.parse().or(Err(lex::ErrorKind::InvalidToken))?;
					let data = data.powi(exponent);
					FloatingConstant::F64(data)
				}
				_ => unreachable!(),
			};
			Ok(TokenKind::Const(Const::Floating(floating)))
		}
	}

	fn integer_constant(&self) -> Result<TokenKind, lex::ErrorKind> {
		let mut chars = self.name.chars().peekable();
		match chars.peek().expect("empty pp-number") {
			'0' => {
				if chars.next_if(|&c| c == 'x' || c == 'X').is_some() {
					todo!("hexadecimal-constant")
				} else {
					self.octal_constant(chars)
				}
			}
			z @ '1'..='9' => {
				eprintln!("'{z}'");
				let name = String::from(*z);
				self.decimal_constant(name, chars)
			}
			e => {
				eprintln!("err: '{e}'");
				Err(lex::ErrorKind::InvalidToken)
			},
		}
	}
	fn octal_constant(&self, mut chars: Peekable<Chars>) -> Result<TokenKind, lex::ErrorKind> {
		let mut name = String::new();
		while let Some(digit) = chars.next_if(char::is_ascii_digit) {
			name.push(digit);
		}
		if chars.next_if(|&c| c == 'u' || c == 'U').is_some() {
			todo!("unsigned-suffix")
		} else if chars.next_if(|&c| c == 'l' || c == 'L').is_some() {
			todo!("long-suffix")
		} else if chars.peek().is_none() && !name.is_empty() {
			let integer = if let Ok(data) = i32::from_str_radix(&name, 8) {
				IntegerConstant::I32(data)
			} else if let Ok(data) = i64::from_str_radix(&name, 8) {
				IntegerConstant::I64(data)
			} else if let Ok(data) = i128::from_str_radix(&name, 8) {
				IntegerConstant::I128(data)
			} else {
				eprintln!("octal invalid token: `{name}`");
				return Err(lex::ErrorKind::InvalidToken);
			};
			let constant = Const::Integer(integer);
			Ok(TokenKind::Const(constant))
		} else {
			todo!("error octal-constant")
		}
	}

	fn decimal_constant(
		&self,
		mut name: String,
		mut chars: Peekable<Chars>,
	) -> Result<TokenKind, lex::ErrorKind> {
		while let Some(digit) = chars.next_if(char::is_ascii_digit) {
			name.push(digit);
		}
		if chars.next_if(|&c| c == 'u' || c == 'U').is_some() {
			todo!("unsigned-suffix")
		} else if chars.next_if(|&c| c == 'l' || c == 'L').is_some() {
			todo!("long-suffix")
		} else if chars.peek().is_none() && !name.is_empty() {
			let integer = if let Ok(data) = name.parse::<i32>() {
				IntegerConstant::I32(data)
			} else if let Ok(data) = name.parse::<i64>() {
				IntegerConstant::I64(data)
			} else if let Ok(data) = name.parse::<i128>() {
				IntegerConstant::I128(data)
			} else {
				return Err(lex::ErrorKind::InvalidToken);
			};
			let constant = Const::Integer(integer);
			Ok(TokenKind::Const(constant))
		} else {
			todo!("error decimal-constant")
		}
	}
}

#[derive(Debug, Clone)]
pub struct CharConst {
	pub seq: String,
	pub is_wide: bool,
}

#[derive(Debug, Clone)]
pub struct NewLine {
	pub name: String,
	pub is_deleted: bool,
}

#[derive(Debug, Clone)]
pub enum TokenKind {
	Keyword(Keyword),
	Ident(Ident),
	Const(Const),
	StrLit(StrLit),
	Punct(Punct),
}

impl TokenKind {
	pub fn is_keyword(&self) -> bool {
		matches!(self, Self::Keyword(_))
	}
	pub fn is_ident(&self) -> bool {
		matches!(self, Self::Ident(_))
	}
	pub fn is_constant(&self) -> bool {
		matches!(self, Self::Const(_))
	}
	pub fn is_str_lit(&self) -> bool {
		matches!(self, Self::StrLit(_))
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
	pub fn unwrap_const(self) -> Const {
		match self {
			Self::Const(token) => token,
			other => panic!("called `Token::unwrap_constant` on an `{other:?}` value"),
		}
	}
	pub fn unwrap_str_lit(self) -> StrLit {
		match self {
			Self::StrLit(token) => token,
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

impl From<Punct> for TokenKind {
	fn from(value: Punct) -> Self {
		Self::Punct(value)
	}
}

impl From<Ident> for TokenKind {
	fn from(value: Ident) -> Self {
		Self::Ident(value)
	}
}

impl From<StrLit> for TokenKind {
	fn from(value: StrLit) -> Self {
		Self::StrLit(value)
	}
}

impl TryFrom<PPTokenKind> for TokenKind {
	type Error = lex::ErrorKind;
	fn try_from(value: PPTokenKind) -> Result<Self, Self::Error> {
		match value {
			PPTokenKind::Ident(inner) => Keyword::try_from(inner.name.as_str())
				.map(TokenKind::Keyword)
				.or(Ok(TokenKind::Ident(inner))),
			PPTokenKind::PPNumber(inner) => {
				if inner.is_float() {
					inner.floating_constant()
				} else {
					inner.integer_constant()
				}
			}
			PPTokenKind::CharConst(inner) => Ok(Self::Const(Const::CharConst(inner))),
			PPTokenKind::StrLit(inner) => Ok(Self::StrLit(inner)),
			PPTokenKind::Punct(inner) => Ok(Self::Punct(inner)),
			PPTokenKind::NewLine(_) | PPTokenKind::HeaderName(_) => {
				Err(lex::ErrorKind::InvalidToken)
			}
		}
	}
}

#[derive(Debug, Clone)]
pub struct Token {
	pub kind: TokenKind,
	pub file_key: usize,
}

#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum PPTokenKind {
	HeaderName(HeaderName),
	Ident(Ident),
	PPNumber(PPNumber),
	CharConst(CharConst),
	StrLit(StrLit),
	Punct(Punct),
	NewLine(NewLine),
}

impl PPTokenKind {
	pub fn to_name(&self) -> String {
		match self {
			Self::HeaderName(value) => value.name.clone(),
			Self::Ident(value) => value.name.clone(),
			Self::PPNumber(value) => value.name.clone(),
			Self::CharConst(value) => value.seq.clone(),
			Self::StrLit(value) => value.seq.clone(),
			Self::Punct(value) => format!("{value}"),
			Self::NewLine(_) => String::from("\\n"),
		}
	}
	pub fn unwrap_str_lit(self) -> StrLit {
		match self {
			PPTokenKind::StrLit(token) => token,
			other => panic!("called `Token::unwrap_string_literal` on an `{other:?}` value"),
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
	pub file_key: usize,
	pub leading_space: bool,
}
