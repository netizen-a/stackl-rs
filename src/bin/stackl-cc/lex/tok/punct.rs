use super::Span;
use crate::lex::error;
use std::fmt;

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
    type Error = error::TryFromCharError;
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
            _ => Err(error::TryFromCharError(())),
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
