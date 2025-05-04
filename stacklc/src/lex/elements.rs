//! Lexical Elements

#[derive(Debug)]
pub struct Span {
    pub location: (isize, isize),
    pub file_key: usize,
}

#[derive(Debug)]
#[non_exhaustive]
pub enum Keyword {
    Auto(Span),
    Break(Span),
    Case(Span),
    Char(Span),
    Const(Span),
    Continue(Span),
    Default(Span),
    Do(Span),
    Double(Span),
    Else(Span),
    Enum(Span),
    Extern(Span),
    Float(Span),
    For(Span),
    Goto(Span),
    If(Span),
    Int(Span),
    Long(Span),
    Register(Span),
    Return(Span),
    Short(Span),
    Signed(Span),
    SizeOf(Span),
    Static(Span),
    Struct(Span),
    Switch(Span),
    TypeDef(Span),
    Union(Span),
    Unsigned(Span),
    Void(Span),
    Volatile(Span),
    While(Span),
}

#[derive(Debug)]
pub struct Identifier {
    pub span: Span,
    pub name: String,
}

#[derive(Debug)]
pub struct Constant {
    pub span: Span,
    pub name: String,
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
}
