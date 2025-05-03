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
pub enum Punctuator {
    /// `[`
    LSquare(Span),
    /// `]`
    RSquare(Span),
    /// `(`
    LParen(Span),
    /// `)`
    RParen(Span),
    /// `{`
    LCurly(Span),
    /// `}`
    RCurly(Span),
    /// `.`
    Dot(Span),
    /// `->
    Arrow(Span),
    /// `++`
    PlusPlus(Span),
    /// --
    MinusMinus(Span),
    /// &
    Amp(Span),
    /// `*`
    Star(Span),
    /// `+`
    Plus(Span),
    /// `-`
    Minus(Span),
    /// `~`
    Tilde(Span),
    /// `!`
    Bang(Span),
    /// `/`
    FSlash(Span),
    /// `%`
    Percent(Span),
    /// `<<`
    LessLess(Span),
    /// `>>`
    GreatGreat(Span),
    /// `<`
    Less(Span),
    /// `>`
    Great(Span),
    /// `<=`
    LessEqual(Span),
    /// `>=`
    GreatEqual(Span),
    /// `==`
    EqualEqual(Span),
    /// `!=`
    BangEqual(Span),
    /// `^`
    Caret(Span),
    /// `|`
    VBar(Span),
    /// `&&`
    AmpAmp(Span),
    /// `||`
    VBarVBar(Span),
    /// `?`
    QMark(Span),
    /// `:`
    Colon(Span),
    /// `;`
    SemiColon(Span),
    /// `...`
    Ellipsis(Span),
    /// `=`
    Equal(Span),
    /// `*=`
    StarEqual(Span),
    /// `/=`
    FSlashEqual(Span),
    /// `%=`
    PercentEqual(Span),
    /// `+=`
    PlusEqual(Span),
    /// `-=`
    MinusEqual(Span),
    /// `<<=`
    LessLessEqual(Span),
    /// `>>=`
    GreatGreatEqual(Span),
    /// `&=`
    AmpEqual(Span),
    /// `^=`
    CaretEqual(Span),
    /// `|=`
    VBarEqual(Span),
    /// `,`
    Comma(Span),
    /// `#`
    Hash(Span),
    /// `##`
    HashHash(Span),
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
