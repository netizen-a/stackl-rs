//! Lexical Elements

pub mod keyword;
pub mod punct;
pub mod span;

pub use keyword::*;
pub use punct::*;
pub use span::*;
use std::fmt;

#[derive(Debug, Clone)]
pub struct Identifier {
    pub span: Span,
    pub name: String,
}

impl fmt::Display for Identifier {
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
pub struct IntegerConstant {
    pub span: Span,
    pub data: i128,
    pub suff: IntegerSuffix,
}

impl fmt::Display for IntegerConstant {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.data, self.suff)
    }
}

#[derive(Debug)]
pub enum Constant {
    Integer(IntegerConstant),
    Floating,
    Enumeration,
    Character(CharacterConstant),
}

impl fmt::Display for Constant {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Integer(token) => write!(f, "{token}"),
            Self::Floating => todo!("floating"),
            Self::Enumeration => todo!("enumeration"),
            Self::Character(token) => write!(f, "{token}"),
        }
    }
}

impl Spanned for Constant {
    fn span(&self) -> Span {
        match self {
            Self::Integer(token) => token.span.clone(),
            Self::Floating => todo!("floating span"),
            Self::Enumeration => todo!("enumeration span"),
            Self::Character(token) => token.span.clone(),
        }
    }
    fn set_span(&mut self, span: Span) {
        match self {
            Self::Integer(token) => token.span = span,
            Self::Floating => todo!("floating span"),
            Self::Enumeration => todo!("enumeration span"),
            Self::Character(token) => token.span = span,
        }
    }
}

#[derive(Debug)]
pub struct StringLiteral {
    pub span: Span,
    pub name: String,
}

impl fmt::Display for StringLiteral {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

#[derive(Debug)]
pub struct HeaderName {
    pub span: Span,
    pub name: String,
}

#[derive(Debug)]
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
}

#[derive(Debug)]
pub struct CharacterConstant {
    pub span: Span,
    pub name: String,
}

impl fmt::Display for CharacterConstant {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

#[derive(Debug)]
pub struct NewLine {
    pub span: Span,
}

#[derive(Debug)]
pub struct Comment {
    pub span: Span,
    pub name: String,
}

#[derive(Debug)]
pub enum Token {
    Keyword(Keyword),
    Identifier(Identifier),
    Constant(Constant),
    StringLiteral(StringLiteral),
    Punctuator(Punctuator),
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
            Self::Keyword(value) => value.span.clone(),
            Self::Identifier(value) => value.span.clone(),
            Self::Constant(value) => value.span(),
            Self::StringLiteral(value) => value.span.clone(),
            Self::Punctuator(value) => value.span.clone(),
        }
    }
    fn set_span(&mut self, span: Span) {
        match self {
            Self::Keyword(value) => value.span = span,
            Self::Identifier(value) => value.span = span,
            Self::Constant(value) => value.set_span(span),
            Self::StringLiteral(value) => value.span = span,
            Self::Punctuator(value) => value.span = span,
        }
    }
}

#[derive(Debug)]
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

impl Spanned for PPToken {
    fn span(&self) -> Span {
        match self {
            Self::HeaderName(value) => value.span.clone(),
            Self::Identifier(value) => value.span.clone(),
            Self::PPNumber(value) => value.span.clone(),
            Self::CharacterConstant(value) => value.span.clone(),
            Self::StringLiteral(value) => value.span.clone(),
            Self::Punctuator(value) => value.span.clone(),
            Self::NewLine(value) => value.span.clone(),
            Self::Comment(value) => value.span.clone(),
        }
    }
    fn set_span(&mut self, span: Span) {
        match self {
            Self::HeaderName(value) => value.span = span,
            Self::Identifier(value) => value.span = span,
            Self::PPNumber(value) => value.span = span,
            Self::CharacterConstant(value) => value.span = span,
            Self::StringLiteral(value) => value.span = span,
            Self::Punctuator(value) => value.span = span,
            Self::NewLine(value) => value.span = span,
            Self::Comment(value) => value.span = span,
        }
    }
}
