//! Lexical Elements

use std::fmt;

#[derive(Debug, Clone)]
pub struct Span {
    pub location: (isize, isize),
    pub file_key: usize,
    pub leading_tabs: usize,
    pub leading_spaces: usize,
}

pub trait Spanned {
    fn span(&self) -> Span;
    fn set_span(&mut self, span: Span);
}

#[derive(Debug)]
#[non_exhaustive]
pub enum KeywordTerminal {
    Auto,
    Break,
    Case,
    Char,
    Const,
    Continue,
    Default,
    Do,
    Double,
    Else,
    Enum,
    Extern,
    Float,
    For,
    Goto,
    If,
    Int,
    Long,
    Register,
    Return,
    Short,
    Signed,
    SizeOf,
    Static,
    Struct,
    Switch,
    TypeDef,
    Union,
    Unsigned,
    Void,
    Volatile,
    While,
}

pub struct TryFromIdentifierError(());
impl fmt::Display for TryFromIdentifierError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        "identifier is not a keyword".fmt(f)
    }
}

impl TryFrom<Identifier> for Keyword {
    type Error = TryFromIdentifierError;
    fn try_from(value: Identifier) -> Result<Self, Self::Error> {
        use KeywordTerminal as Term;
        let terminal = match value.name.as_str() {
            "auto" => Term::Auto,
            "break" => Term::Break,
            "case" => Term::Case,
            "char" => Term::Char,
            "const" => Term::Const,
            "continue" => Term::Continue,
            "default" => Term::Default,
            "do" => Term::Do,
            "double" => Term::Double,
            "else" => Term::Else,
            "enum" => Term::Enum,
            "extern" => Term::Extern,
            "float" => Term::Float,
            "for" => Term::For,
            "goto" => Term::Goto,
            "if" => Term::If,
            "int" => Term::Int,
            "long" => Term::Long,
            "register" => Term::Register,
            "return" => Term::Return,
            "short" => Term::Short,
            "signed" => Term::Signed,
            "sizeof" => Term::SizeOf,
            "static" => Term::Static,
            "struct" => Term::Struct,
            "switch" => Term::Switch,
            "typedef" => Term::TypeDef,
            "union" => Term::Union,
            "unsigned" => Term::Unsigned,
            "void" => Term::Void,
            "volatile" => Term::Volatile,
            "while" => Term::While,
            _ => return Err(TryFromIdentifierError(())),
        };
        Ok(Keyword {
            span: value.span,
            term: terminal,
        })
    }
}

impl fmt::Display for KeywordTerminal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Self::Auto => "auto",
            Self::Break => "break",
            Self::Case => "case",
            Self::Char => "char",
            Self::Const => "const",
            Self::Continue => "continue",
            Self::Default => "default",
            Self::Do => "do",
            Self::Double => "double",
            Self::Else => "else",
            Self::Enum => "enum",
            Self::Extern => "extern",
            Self::Float => "float",
            Self::For => "for",
            Self::Goto => "goto",
            Self::If => "if",
            Self::Int => "int",
            Self::Long => "long",
            Self::Register => "register",
            Self::Return => "return",
            Self::Short => "short",
            Self::Signed => "signed",
            Self::SizeOf => "sizeof",
            Self::Static => "static",
            Self::Struct => "struct",
            Self::Switch => "switch",
            Self::TypeDef => "typedef",
            Self::Union => "union",
            Self::Unsigned => "unsigned",
            Self::Void => "void",
            Self::Volatile => "volatile",
            Self::While => "while",
        };
        write!(f, "{name}")
    }
}

#[derive(Debug)]
pub struct Keyword {
    pub span: Span,
    pub term: KeywordTerminal,
}

impl fmt::Display for Keyword {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.term)
    }
}

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

#[non_exhaustive]
#[derive(Debug)]
pub enum PunctuatorTerminal {
    /// `[`
    LSquare,
    /// `]`
    RSquare,
    /// `(`
    LParen,
    /// `)`
    RParen,
    /// `{`
    LCurly,
    /// `}`
    RCurly,
    /// `.`
    Dot,
    /// `->
    Arrow,
    /// `++`
    PlusPlus,
    /// --
    MinusMinus,
    /// &
    Amp,
    /// `*`
    Star,
    /// `+`
    Plus,
    /// `-`
    Minus,
    /// `~`
    Tilde,
    /// `!`
    Bang,
    /// `/`
    FSlash,
    /// `%`
    Percent,
    /// `<<`
    LessLess,
    /// `>>`
    GreatGreat,
    /// `<`
    Less,
    /// `>`
    Great,
    /// `<=`
    LessEqual,
    /// `>=`
    GreatEqual,
    /// `==`
    EqualEqual,
    /// `!=`
    BangEqual,
    /// `^`
    Caret,
    /// `|`
    VBar,
    /// `&&`
    AmpAmp,
    /// `||`
    VBarVBar,
    /// `?`
    QMark,
    /// `:`
    Colon,
    /// `;`
    SemiColon,
    /// `...`
    Ellipsis,
    /// `=`
    Equal,
    /// `*=`
    StarEqual,
    /// `/=`
    FSlashEqual,
    /// `%=`
    PercentEqual,
    /// `+=`
    PlusEqual,
    /// `-=`
    MinusEqual,
    /// `<<=`
    LessLessEqual,
    /// `>>=`
    GreatGreatEqual,
    /// `&=`
    AmpEqual,
    /// `^=`
    CaretEqual,
    /// `|=`
    VBarEqual,
    /// `,`
    Comma,
    /// `#`
    Hash,
    /// `##`
    HashHash,
}

impl fmt::Display for PunctuatorTerminal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use PunctuatorTerminal::*;
        let symbol = match self {
            LSquare => "[",
            RSquare => "]",
            LParen => "(",
            RParen => ")",
            LCurly => "{",
            RCurly => "}",
            Dot => ".",
            Arrow => "->",
            PlusPlus => "++",
            MinusMinus => "--",
            Amp => "&",
            Star => "*",
            Plus => "+",
            Minus => "-",
            Tilde => "~",
            Bang => "!",
            FSlash => "/",
            Percent => "%",
            LessLess => "<<",
            GreatGreat => ">>",
            Less => "<",
            Great => ">",
            LessEqual => "<=",
            GreatEqual => ">=",
            EqualEqual => "==",
            BangEqual => "!=",
            Caret => "^",
            VBar => "|",
            AmpAmp => "&&",
            VBarVBar => "||",
            QMark => "?",
            Colon => ":",
            SemiColon => ";",
            Ellipsis => "...",
            Equal => "=",
            StarEqual => "*=",
            FSlashEqual => "/=",
            PercentEqual => "%=",
            PlusEqual => "+=",
            MinusEqual => "-=",
            LessLessEqual => "<<=",
            GreatGreatEqual => ">>=",
            AmpEqual => "&=",
            CaretEqual => "^=",
            VBarEqual => "|=",
            Comma => ",",
            Hash => "#",
            HashHash => "##",
        };
        write!(f, "{}", symbol)
    }
}

impl TryFrom<char> for PunctuatorTerminal {
    type Error = super::error::TryFromCharError;
    fn try_from(value: char) -> Result<Self, Self::Error> {
        use PunctuatorTerminal::*;
        match value {
            '[' => Ok(LSquare),
            ']' => Ok(RSquare),
            '(' => Ok(LParen),
            ')' => Ok(RParen),
            '{' => Ok(LCurly),
            '}' => Ok(RCurly),
            '?' => Ok(QMark),
            ',' => Ok(Comma),
            '~' => Ok(Tilde),
            ';' => Ok(SemiColon),
            _ => Err(super::error::TryFromCharError(())),
        }
    }
}

#[derive(Debug)]
pub struct Punctuator {
    pub span: Span,
    pub term: PunctuatorTerminal,
}

impl fmt::Display for Punctuator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.term)
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
pub enum PreprocessingToken {
    HeaderName(HeaderName),
    Identifier(Identifier),
    PPNumber(PPNumber),
    CharacterConstant(CharacterConstant),
    StringLiteral(StringLiteral),
    Punctuator(Punctuator),
    NewLine(NewLine),
    Comment(Comment),
}

impl Spanned for PreprocessingToken {
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
