// Copyright (c) 2024-2026 Jonathan A. Thomason

//! Lexical Elements

pub mod keyword;
pub mod punct;

use crate::diagnostics as diag;
pub use keyword::*;
pub use punct::*;
use std::fmt;
use std::iter::Peekable;
use std::str::Chars;

#[derive(Debug, Clone)]
pub struct Ident {
	pub name: String,
	/// is the identifier previously declared in a typedef?
	pub is_type: bool,
	pub expandable: bool,
}

impl fmt::Display for Ident {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{}", self.name)
	}
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
	Float(f32),
	Double(f64),
	Long(f64),
}

#[derive(Debug, Clone)]
pub enum Const {
	Integer(IntegerConstant),
	Floating(FloatingConstant),
	CharConst(CharConst),
}

#[derive(Debug, Clone, Default)]
pub struct StrLit {
	pub seq: String,
	pub is_wide: bool,
	pub file_id: usize,
}

impl fmt::Display for StrLit {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let prefix = if self.is_wide { "L" } else { "" };
		let seq = self.seq.as_str();
		write!(f, "{prefix}\"{seq}\"")
	}
}

#[derive(Debug, Clone)]
pub struct HeaderName {
	pub is_builtin: bool,
	pub name: String,
}

impl fmt::Display for HeaderName {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self.is_builtin {
			true => write!(f, "<{}>", self.name),
			false => write!(f, "\"{}\"", self.name),
		}
	}
}

#[derive(Debug, Clone)]
pub struct PPNumber {
	pub name: String,
}

impl fmt::Display for PPNumber {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{}", self.name)
	}
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
	fn floating_constant(&self) -> Result<TokenKind, diag::DiagKind> {
		let mut chars = self.name.chars().peekable();
		let c = chars.next().expect("empty pp-number");
		if c == '0' && chars.next_if(|&c| c == 'x' || c == 'X').is_some() {
			todo!("hexadecimal-floating-constant")
		} else {
			let mut decimal = String::from(c);
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
				exponent = digit_seq.parse().or(Err(diag::DiagKind::InvalidToken))?;
				if let Some('-') = sign {
					exponent *= -1;
				}
			}
			let floating = match chars.next_if(|&c| c == 'f' || c == 'F' || c == 'l' || c == 'L') {
				Some('f' | 'F') => {
					let data: f32 = decimal.parse().or(Err(diag::DiagKind::InvalidToken))?;
					let data = data * 10f32.powi(exponent);
					FloatingConstant::Float(data)
				}
				Some('l' | 'L') => {
					let data: f64 = decimal.parse().or(Err(diag::DiagKind::InvalidToken))?;
					let data = data * 10f64.powi(exponent);
					FloatingConstant::Long(data)
				}
				None => {
					let data: f64 = decimal.parse().or(Err(diag::DiagKind::InvalidToken))?;
					let data = data * 10f64.powi(exponent);
					FloatingConstant::Double(data)
				}
				_ => unreachable!(),
			};
			Ok(TokenKind::Const(Const::Floating(floating)))
		}
	}

	fn integer_constant(&self) -> Result<TokenKind, diag::DiagKind> {
		let mut chars = self.name.chars().peekable();
		match chars.peek().expect("empty pp-number") {
			'0' => {
				if chars.next_if(|&c| c == 'x' || c == 'X').is_some() {
					todo!("hexadecimal-constant")
				} else {
					self.octal_constant(chars)
				}
			}
			'1'..='9' => {
				let name = String::from(chars.next().unwrap());
				self.decimal_constant(name, chars)
			}
			e => {
				eprintln!("err: '{e}'");
				Err(diag::DiagKind::InvalidToken)
			}
		}
	}
	fn octal_constant(&self, mut chars: Peekable<Chars>) -> Result<TokenKind, diag::DiagKind> {
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
				return Err(diag::DiagKind::InvalidToken);
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
	) -> Result<TokenKind, diag::DiagKind> {
		while let Some(digit) = chars.next_if(char::is_ascii_digit) {
			name.push(digit);
		}
		if let Some(c) = chars.next_if(|&c| c == 'l' || c == 'L' || c == 'u' || c == 'U') {
			let mut l_count = (c == 'l' || c == 'L') as u32;
			let mut u_count = (c == 'u' || c == 'U') as u32;
			while let Some(suffix) =
				chars.next_if(|&c| c == 'l' || c == 'L' || c == 'u' || c == 'U')
			{
				match suffix {
					'l' | 'L' => l_count += 1,
					'u' | 'U' => u_count += 1,
					_ => unreachable!(),
				}
			}
			let integer = match (l_count, u_count) {
				(0, 1) => {
					if let Ok(data) = name.parse::<u32>() {
						Ok(IntegerConstant::U32(data))
					} else {
						Err(diag::DiagKind::InvalidToken)
					}
				}
				(1, 0) => {
					if let Ok(data) = name.parse::<i64>() {
						Ok(IntegerConstant::I64(data))
					} else {
						Err(diag::DiagKind::InvalidToken)
					}
				}
				(1, 1) => {
					if let Ok(data) = name.parse::<u64>() {
						Ok(IntegerConstant::U64(data))
					} else {
						Err(diag::DiagKind::InvalidToken)
					}
				}
				(2, 0) => {
					if let Ok(data) = name.parse::<i128>() {
						Ok(IntegerConstant::I128(data))
					} else {
						Err(diag::DiagKind::InvalidToken)
					}
				}
				(2, 1) => {
					if let Ok(data) = name.parse::<u128>() {
						Ok(IntegerConstant::U128(data))
					} else {
						Err(diag::DiagKind::InvalidToken)
					}
				}
				_ => Err(diag::DiagKind::InvalidToken),
			};
			let constant = integer.and_then(|inner| Ok(Const::Integer(inner)));
			constant.and_then(|inner| Ok(TokenKind::Const(inner)))
		} else if chars.peek().is_none() && !name.is_empty() {
			let integer = if let Ok(data) = name.parse::<i32>() {
				IntegerConstant::I32(data)
			} else if let Ok(data) = name.parse::<i64>() {
				IntegerConstant::I64(data)
			} else if let Ok(data) = name.parse::<i128>() {
				IntegerConstant::I128(data)
			} else {
				return Err(diag::DiagKind::InvalidToken);
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

impl fmt::Display for CharConst {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let prefix = if self.is_wide { "L" } else { "" };
		let seq = self.seq.as_str();
		write!(f, "{prefix}'{seq}'")
	}
}

#[derive(Debug, Clone)]
pub struct NewLine {
	pub name: String,
	pub is_deleted: bool,
}

impl fmt::Display for NewLine {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let new_line = if self.is_deleted {
			""
		} else {
			self.name.as_str()
		};
		write!(f, "{new_line}")
	}
}

#[derive(Debug, Clone)]
pub enum Pragma {
	StdcFpContract(bool),
	StdcFenvAccess(bool),
	StdcCxLimitedRange(bool),
	StacklStackSize(u32),
	StacklFeature(String, bool),
	StacklSection(String),
	StacklTrace(bool),
	StacklVersion(u32),
}

#[derive(Debug, Clone)]
pub enum TokenKind {
	Keyword(Keyword),
	Ident(Ident),
	Const(Const),
	StrLit(StrLit),
	Punct(Punct),
	Pragma(Pragma),
}

impl TokenKind {
	pub fn unwrap_str_lit(self) -> StrLit {
		match self {
			Self::StrLit(token) => token,
			other => panic!("called `Token::unwrap_string_literal` on an `{other:?}` value"),
		}
	}
	pub fn unwrap_pragma(self) -> Pragma {
		match self {
			Self::Pragma(token) => token,
			other => panic!("called `Token::unwrap_string_literal` on an `{other:?}` value"),
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
	type Error = diag::DiagKind;
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
			_ => Err(diag::DiagKind::InvalidToken),
		}
	}
}

#[derive(Debug, Clone)]
pub struct Token {
	pub kind: TokenKind,
	pub span: diag::Span,
}

impl diag::ToSpan for Token {
	fn to_span(&self) -> diag::Span {
		self.span.clone()
	}
}

#[derive(Debug, Clone, Copy)]
pub enum Directive {
	Include,
	If,
	Ifdef,
	Ifndef,
	Elif,
	Else,
	Endif,
	Define,
	Undef,
	Line,
	Error,
	Pragma,
}

impl fmt::Display for Directive {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let dir_str = match self {
			Self::Include => "#include",
			Self::If => "#if",
			Self::Ifdef => "#ifdef",
			Self::Ifndef => "#ifndef",
			Self::Elif => "#elif",
			Self::Else => "#else",
			Self::Endif => "#endif",
			Self::Define => "#define",
			Self::Undef => "#undef",
			Self::Line => "#line",
			Self::Error => "#error",
			Self::Pragma => "#pragma",
		};
		write!(f, "{dir_str}")
	}
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
	Directive(Directive),
}

impl fmt::Display for PPTokenKind {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let tok_str = match &self {
			PPTokenKind::Punct(punct) => punct.to_string(),
			PPTokenKind::Ident(ident) => ident.to_string(),
			PPTokenKind::CharConst(char_const) => char_const.to_string(),
			PPTokenKind::NewLine(new_line) => new_line.to_string(),
			PPTokenKind::StrLit(literal) => literal.to_string(),
			PPTokenKind::HeaderName(header) => header.to_string(),
			PPTokenKind::PPNumber(number) => number.to_string(),
			PPTokenKind::Directive(directive) => directive.to_string(),
		};
		write!(f, "{tok_str}")
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
	pub leading_space: bool,
	pub span: diag::Span,
}

impl diag::ToSpan for PPToken {
	fn to_span(&self) -> diag::Span {
		self.span.clone()
	}
}

impl fmt::Display for PPToken {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let tok_str = match &self.kind {
			PPTokenKind::Punct(punct) => punct.to_string(),
			PPTokenKind::Ident(ident) => ident.to_string(),
			PPTokenKind::CharConst(char_const) => char_const.to_string(),
			PPTokenKind::NewLine(new_line) => new_line.to_string(),
			PPTokenKind::StrLit(literal) => literal.to_string(),
			PPTokenKind::HeaderName(header) => header.to_string(),
			PPTokenKind::PPNumber(number) => number.to_string(),
			PPTokenKind::Directive(directive) => directive.to_string(),
		};
		let space = if self.leading_space { " " } else { "" };
		write!(f, "{space}{tok_str}")
	}
}

pub type TokenTriple = (usize, Token, usize);
// pub type PPTokenTriple = (usize, PPToken, usize);
