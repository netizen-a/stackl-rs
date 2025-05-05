//! Lexical Elements

use std::fmt;

#[derive(Debug, Clone)]
pub struct Span {
    pub location: (isize, isize),
    pub file_key: usize,
    pub leading_tabs: usize,
    pub leading_spaces: usize,
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

#[derive(Debug)]
pub struct Keyword {
    pub span: Span,
    pub term: KeywordTerminal,
}

#[derive(Debug)]
pub struct Identifier {
    pub span: Span,
    pub name: String,
}

#[derive(Debug)]
pub enum Constant {
    Integer,
    Floating,
    Enumeration,
    Character(CharacterConstant),
}

#[derive(Debug)]
pub struct StringLiteral {
    pub span: Span,
    pub name: String,
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
