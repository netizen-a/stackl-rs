pub enum Token {
    Keyword,
    Identifier,
    Constant,
    StringLiteral,
    Punctuator,
}

pub enum PreprocessingToken {
    HeaderName,
    Identifier,
    PreprocessingNumber,
    CharacterConstant,
    StringLiteral,
    Punctuator,
    Other,
}
