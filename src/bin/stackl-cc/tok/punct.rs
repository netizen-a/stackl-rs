use super::span;
use crate::lex::error;
use std::fmt;

#[non_exhaustive]
#[derive(Debug, Clone, Copy)]
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
    type Error = error::TryFromCharError;
    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            '[' => Ok(Self::LSquare),
            ']' => Ok(Self::RSquare),
            '(' => Ok(Self::LParen),
            ')' => Ok(Self::RParen),
            '{' => Ok(Self::LCurly),
            '}' => Ok(Self::RCurly),
            '?' => Ok(Self::QMark),
            ',' => Ok(Self::Comma),
            '~' => Ok(Self::Tilde),
            ';' => Ok(Self::SemiColon),
            _ => Err(error::TryFromCharError(())),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Punctuator {
    pub span: span::Span,
    pub term: PunctuatorTerminal,
}

impl span::Spanned for Punctuator {
    fn span(&self) -> span::Span {
        self.span.clone()
    }
    fn set_span(&mut self, span: span::Span) {
        self.span = span;
    }
}

impl fmt::Display for Punctuator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.term)
    }
}
