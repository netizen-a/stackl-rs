//! Lexical Elements

pub mod keyword;
pub mod punct;
pub mod span;

use crate::lex::error::*;
pub use keyword::*;
pub use punct::*;
pub use span::*;
use std::iter::Peekable;
use std::str::Chars;
use std::{fmt, result};

pub type Result<T> = result::Result<T, LexicalError>;

#[derive(Debug, Clone)]
pub struct Identifier {
	pub span: Span,
	pub name: String,
}

impl span::Spanned for Identifier {
	fn span(&self) -> Span {
		self.span.clone()
	}
	fn set_span(&mut self, span: Span) {
		self.span = span;
	}
}

impl fmt::Display for Identifier {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{}{}", self.span, self.name)
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

impl fmt::Display for IntegerSuffix {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let name = match self {
			Self::None => "",
			Self::U => "U",
			Self::L => "L",
			Self::UL => "UL",
			Self::LL => "LL",
			Self::ULL => "ULL",
		};
		write!(f, "{name}")
	}
}

#[derive(Debug)]
pub enum IntegerConstant {
	U32 { span: Span, data: u32 },
	I32 { span: Span, data: i32 },
	U64 { span: Span, data: u64 },
	I64 { span: Span, data: i64 },
	U128 { span: Span, data: u128 },
	I128 { span: Span, data: i128 },
}

impl Spanned for IntegerConstant {
	fn span(&self) -> Span {
		match self {
			Self::U32 { span, .. } => span.clone(),
			Self::I32 { span, .. } => span.clone(),
			Self::U64 { span, .. } => span.clone(),
			Self::I64 { span, .. } => span.clone(),
			Self::U128 { span, .. } => span.clone(),
			Self::I128 { span, .. } => span.clone(),
		}
	}
	fn set_span(&mut self, value: Span) {
		match self {
			Self::U32 { span, .. } => *span = value,
			Self::I32 { span, .. } => *span = value,
			Self::U64 { span, .. } => *span = value,
			Self::I64 { span, .. } => *span = value,
			Self::U128 { span, .. } => *span = value,
			Self::I128 { span, .. } => *span = value,
		}
	}
}

impl fmt::Display for IntegerConstant {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let (span, name) = match self {
			Self::U32 { span, data } => (span, format!("{data}U")),
			Self::I32 { span, data } => (span, format!("{data}")),
			Self::U64 { span, data } => (span, format!("{data}UL")),
			Self::I64 { span, data } => (span, format!("{data}L")),
			Self::U128 { span, data } => (span, format!("{data}ULL")),
			Self::I128 { span, data } => (span, format!("{data}LL")),
		};
		write!(f, "{span}{name}")
	}
}

#[derive(Debug)]
pub enum FloatingConstant {
	F32 { span: Span, data: f32 },
	F64 { span: Span, data: f64 },
	// same as F64, but higher rank
	Long { span: Span, data: f64 },
}

impl fmt::Display for FloatingConstant {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let (span, name) = match self {
			Self::F32 { span, data } => (span, format!("{data}F")),
			Self::F64 { span, data } => (span, format!("{data}")),
			Self::Long { span, data } => (span, format!("{data}L")),
		};
		write!(f, "{span}{name}")
	}
}

impl Spanned for FloatingConstant {
	fn span(&self) -> Span {
		match self {
			Self::F32 { span, .. } => span.clone(),
			Self::F64 { span, .. } => span.clone(),
			Self::Long { span, .. } => span.clone(),
		}
	}
	fn set_span(&mut self, value: Span) {
		match self {
			Self::F32 { span, .. } => *span = value,
			Self::F64 { span, .. } => *span = value,
			Self::Long { span, .. } => *span = value,
		}
	}
}

#[derive(Debug)]
pub enum Constant {
	Integer(IntegerConstant),
	Floating(FloatingConstant),
	Enumeration,
	Character(CharacterConstant),
}

impl fmt::Display for Constant {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::Integer(token) => write!(f, "{token}"),
			Self::Floating(token) => write!(f, "{token}"),
			Self::Enumeration => todo!("enumeration"),
			Self::Character(token) => write!(f, "{token}"),
		}
	}
}

impl Spanned for Constant {
	fn span(&self) -> Span {
		match self {
			Self::Integer(token) => token.span(),
			Self::Floating(token) => token.span(),
			Self::Enumeration => todo!("enumeration span"),
			Self::Character(token) => token.span.clone(),
		}
	}
	fn set_span(&mut self, span: Span) {
		match self {
			Self::Integer(token) => token.set_span(span),
			Self::Floating(token) => token.set_span(span),
			Self::Enumeration => todo!("enumeration span"),
			Self::Character(token) => token.span = span,
		}
	}
}

#[derive(Debug, Clone)]
pub struct StringLiteral {
	pub span: Span,
	pub name: String,
}

impl span::Spanned for StringLiteral {
	fn span(&self) -> Span {
		self.span.clone()
	}
	fn set_span(&mut self, span: Span) {
		self.span = span;
	}
}

impl fmt::Display for StringLiteral {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{}{}", self.span, self.name)
	}
}

#[derive(Debug, Clone)]
pub struct HeaderName {
	pub span: Span,
	pub is_std: bool,
	pub name: String,
}

impl fmt::Display for HeaderName {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		if self.is_std {
			write!(f, "<{}>", self.name)
		} else {
			write!(f, "\"{}\"", self.name)
		}
	}
}

impl span::Spanned for HeaderName {
	fn span(&self) -> Span {
		self.span.clone()
	}
	fn set_span(&mut self, span: Span) {
		self.span = span;
	}
}

#[derive(Debug, Clone)]
pub struct PPNumber {
	pub span: Span,
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
	fn floating_constant(&self) -> Result<Token> {
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
				exponent = digit_seq.parse().or(Err(LexicalError {
					kind: LexicalErrorKind::InvalidToken,
					span: self.span(),
				}))?;
				if let Some('-') = sign {
					exponent *= -1;
				}
			}
			let floating = match chars.next_if(|&c| c == 'f' || c == 'F' || c == 'l' || c == 'L') {
				Some('f' | 'F') => {
					let data: f32 = decimal.parse().or(Err(LexicalError {
						kind: LexicalErrorKind::InvalidToken,
						span: self.span(),
					}))?;
					let data = data.powi(exponent);
					FloatingConstant::F32 {
						span: self.span(),
						data,
					}
				}
				Some('l' | 'L') => {
					let data: f64 = decimal.parse().or(Err(LexicalError {
						kind: LexicalErrorKind::InvalidToken,
						span: self.span(),
					}))?;
					let data = data.powi(exponent);
					FloatingConstant::Long {
						span: self.span(),
						data,
					}
				}
				None => {
					let data: f64 = decimal.parse().or(Err(LexicalError {
						kind: LexicalErrorKind::InvalidToken,
						span: self.span(),
					}))?;
					let data = data.powi(exponent);
					FloatingConstant::F64 {
						span: self.span(),
						data,
					}
				}
				_ => unreachable!(),
			};
			Ok(Token::Constant(Constant::Floating(floating)))
		}
	}

	fn integer_constant(&self) -> Result<Token> {
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
			_ => Err(LexicalError {
				kind: LexicalErrorKind::InvalidToken,
				span: self.span(),
			}),
		}
	}
	fn octal_constant(&self, mut chars: Peekable<Chars>) -> Result<Token> {
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
				IntegerConstant::I32 {
					span: self.span(),
					data,
				}
			} else if let Ok(data) = i64::from_str_radix(&name, 8) {
				IntegerConstant::I64 {
					span: self.span(),
					data,
				}
			} else if let Ok(data) = i128::from_str_radix(&name, 8) {
				IntegerConstant::I128 {
					span: self.span(),
					data,
				}
			} else {
				return Err(LexicalError {
					kind: LexicalErrorKind::InvalidToken,
					span: self.span(),
				});
			};
			let constant = Constant::Integer(integer);
			Ok(Token::Constant(constant))
		} else {
			todo!("error octal-constant")
		}
	}

	fn decimal_constant(&self, mut name: String, mut chars: Peekable<Chars>) -> Result<Token> {
		while let Some(digit) = chars.next_if(char::is_ascii_digit) {
			name.push(digit);
		}
		if chars.next_if(|&c| c == 'u' || c == 'U').is_some() {
			todo!("unsigned-suffix")
		} else if chars.next_if(|&c| c == 'l' || c == 'L').is_some() {
			todo!("long-suffix")
		} else if chars.peek().is_none() {
			let integer = if let Ok(data) = name.parse::<i32>() {
				IntegerConstant::I32 {
					span: self.span(),
					data,
				}
			} else if let Ok(data) = name.parse::<i64>() {
				IntegerConstant::I64 {
					span: self.span(),
					data,
				}
			} else if let Ok(data) = name.parse::<i128>() {
				IntegerConstant::I128 {
					span: self.span(),
					data,
				}
			} else {
				return Err(LexicalError {
					kind: LexicalErrorKind::InvalidToken,
					span: self.span(),
				});
			};
			let constant = Constant::Integer(integer);
			Ok(Token::Constant(constant))
		} else {
			todo!("error decimal-constant")
		}
	}
}

impl span::Spanned for PPNumber {
	fn span(&self) -> Span {
		self.span.clone()
	}
	fn set_span(&mut self, span: Span) {
		self.span = span;
	}
}

#[derive(Debug, Clone)]
pub struct CharacterConstant {
	pub span: Span,
	pub name: String,
}

impl span::Spanned for CharacterConstant {
	fn span(&self) -> Span {
		self.span.clone()
	}
	fn set_span(&mut self, span: Span) {
		self.span = span;
	}
}

impl fmt::Display for CharacterConstant {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{}{}", self.span, self.name)
	}
}

#[derive(Debug, Clone)]
pub struct NewLine {
	pub span: span::Span,
	pub name: String,
	pub is_deleted: bool,
}

impl fmt::Display for NewLine {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let name = if self.is_deleted { "" } else { "\n" };
		write!(f, "{}{name}", self.span)
	}
}

impl span::Spanned for NewLine {
	fn span(&self) -> Span {
		self.span.clone()
	}
	fn set_span(&mut self, span: Span) {
		self.span = span;
	}
}

#[derive(Debug, Clone)]
pub struct Comment {
	pub span: Span,
	pub name: String,
}

impl span::Spanned for Comment {
	fn span(&self) -> Span {
		self.span.clone()
	}
	fn set_span(&mut self, span: Span) {
		self.span = span;
	}
}

impl fmt::Display for Comment {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{}{}", self.span, self.name)
	}
}

#[derive(Debug)]
pub enum Token {
	Keyword(Keyword),
	Identifier(Identifier),
	Constant(Constant),
	StringLiteral(StringLiteral),
	Punctuator(Punctuator),
}

impl TryFrom<PPNumber> for Token {
	type Error = LexicalError;
	fn try_from(value: PPNumber) -> Result<Self> {
		if value.is_float() {
			value.floating_constant()
		} else {
			value.integer_constant()
		}
	}
}

impl fmt::Display for Token {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::Keyword(token) => write!(f, "{token}"),
			Self::Identifier(token) => write!(f, "{token}"),
			Self::Constant(token) => write!(f, "{token}"),
			Self::StringLiteral(token) => write!(f, "{token}"),
			Self::Punctuator(token) => write!(f, "{token}"),
		}
	}
}

impl Spanned for Token {
	fn span(&self) -> Span {
		match self {
			Self::Keyword(value) => value.span(),
			Self::Identifier(value) => value.span(),
			Self::Constant(value) => value.span(),
			Self::StringLiteral(value) => value.span(),
			Self::Punctuator(value) => value.span(),
		}
	}
	fn set_span(&mut self, span: Span) {
		match self {
			Self::Keyword(value) => value.set_span(span),
			Self::Identifier(value) => value.set_span(span),
			Self::Constant(value) => value.set_span(span),
			Self::StringLiteral(value) => value.set_span(span),
			Self::Punctuator(value) => value.set_span(span),
		}
	}
}

#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum PPToken {
	HeaderName(HeaderName),
	Identifier(Identifier),
	PPNumber(PPNumber),
	CharacterConstant(CharacterConstant),
	StringLiteral(StringLiteral),
	Punctuator(Punctuator),
	NewLine(NewLine),
	Comment(Comment),
}

impl PPToken {
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
			Self::Identifier(value) => value.name.clone(),
			Self::PPNumber(value) => value.name.clone(),
			Self::CharacterConstant(value) => value.name.clone(),
			Self::StringLiteral(value) => value.name.clone(),
			Self::Punctuator(value) => format!("{value}"),
			Self::NewLine(_) => String::from("\\n"),
			Self::Comment(value) => value.name.clone(),
		}
	}
}

impl Spanned for PPToken {
	fn span(&self) -> Span {
		match self {
			Self::HeaderName(value) => value.span(),
			Self::Identifier(value) => value.span(),
			Self::PPNumber(value) => value.span(),
			Self::CharacterConstant(value) => value.span(),
			Self::StringLiteral(value) => value.span(),
			Self::Punctuator(value) => value.span(),
			Self::NewLine(value) => value.span(),
			Self::Comment(value) => value.span(),
		}
	}
	fn set_span(&mut self, span: Span) {
		match self {
			Self::HeaderName(value) => value.set_span(span),
			Self::Identifier(value) => value.set_span(span),
			Self::PPNumber(value) => value.set_span(span),
			Self::CharacterConstant(value) => value.set_span(span),
			Self::StringLiteral(value) => value.set_span(span),
			Self::Punctuator(value) => value.set_span(span),
			Self::NewLine(value) => value.set_span(span),
			Self::Comment(value) => value.set_span(span),
		}
	}
}
