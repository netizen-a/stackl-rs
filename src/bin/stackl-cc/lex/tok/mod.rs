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
        write!(f, "{}", self.name)
    }
}

#[derive(Debug, Clone)]
pub struct HeaderName {
    pub span: Span,
    pub name: String,
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
        write!(f, "{}", self.name)
    }
}

#[derive(Debug, Clone)]
#[repr(transparent)]
pub struct NewLine {
    pub span: span::Span,
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
