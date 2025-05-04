use std::iter;
use std::str::Chars;

use super::elements as tok;

#[derive(Debug)]
pub enum LexicalErrorKind {
    UnexpectedEof,
    UnexpectedEscape,
    InvalidToken,
}

#[derive(Debug)]
pub struct LexicalError {
    pub kind: LexicalErrorKind,
    pub span: tok::Span,
}

#[derive(Debug)]
pub struct Lexer<'a> {
    chars: iter::Peekable<Chars<'a>>,
    pos: isize,
    file_key: usize,
    include_state: u8,
}

impl<'a> Lexer<'a> {
    #[allow(dead_code)]
    pub fn new(text: &'a str, file_key: usize) -> Self {
        Self {
            chars: text.chars().peekable(),
            pos: 0,
            file_key,
            include_state: 1,
        }
    }

    #[allow(dead_code)]
    fn header_name(&mut self) -> Result<tok::PreprocessingToken, LexicalError> {
        todo!("header-name")
    }

    fn identifier(
        &mut self,
        c: char,
        start_pos: isize,
    ) -> Result<tok::PreprocessingToken, LexicalError> {
        let mut span = tok::Span {
            location: (start_pos, self.pos),
            file_key: self.file_key,
        };
        let mut name = String::new();
        name.push(c);
        while let Some(next_c) = self
            .chars
            .next_if(|&c| c.is_ascii_alphanumeric() || c == '_')
        {
            name.push(next_c);
            self.pos += 1;
        }
        span.location = (start_pos, self.pos);
        if self.include_state == 2 && name == "include" {
            self.include_state = 3;
        } else {
            self.include_state = 0;
        }
        let ident = tok::Identifier { span, name };
        Ok(tok::PreprocessingToken::Identifier(ident))
    }

    #[allow(dead_code)]
    fn pp_number(&mut self) -> Result<tok::PreprocessingToken, LexicalError> {
        todo!("pp-number")
    }

    fn character_constant(
        &mut self,
        mut c: char,
        start_pos: isize,
    ) -> Result<tok::PreprocessingToken, LexicalError> {
        let mut span = tok::Span {
            location: (start_pos, self.pos),
            file_key: self.file_key,
        };
        let mut name = String::new();
        self.include_state = 0;
        let is_l = c == 'L';
        if is_l {
            name.push(c);
            if let Some(next_c) = self.chars.next() {
                c = next_c;
            } else {
                return Err(LexicalError {
                    kind: LexicalErrorKind::UnexpectedEof,
                    span,
                });
            }
            self.pos += 1;
        }
        name.push(c);
        while let Some(next_c) = self.chars.next() {
            name.push(next_c);
            self.pos += 1;
            if next_c == '\\' {
                span.location.1 = self.pos;
                let Some(&next_c) = self.chars.peek() else {
                    return Err(LexicalError {
                        kind: LexicalErrorKind::UnexpectedEof,
                        span,
                    });
                };
                match next_c {
                    '\'' | '"' | '?' | '\\' | 'a' | 'b' | 'f' | 'n' | 'r' | 't' | 'v' => {
                        name.push(next_c);
                        self.pos += 1;
                    }
                    _ => {
                        return Err(LexicalError {
                            kind: LexicalErrorKind::UnexpectedEscape,
                            span,
                        });
                    }
                }
            } else if next_c == '\'' {
                break;
            }
        }
        span.location.1 = self.pos;
        let str_lit = tok::CharacterConstant { span, name };
        Ok(tok::PreprocessingToken::CharacterConstant(str_lit))
    }

    fn string_literal(
        &mut self,
        mut c: char,
        start_pos: isize,
    ) -> Result<tok::PreprocessingToken, LexicalError> {
        let mut span = tok::Span {
            location: (start_pos, self.pos),
            file_key: self.file_key,
        };
        let mut name = String::new();
        let is_l = c == 'L';
        if is_l {
            name.push(c);
            if let Some(next_c) = self.chars.next() {
                c = next_c;
            } else {
                return Err(LexicalError {
                    kind: LexicalErrorKind::UnexpectedEof,
                    span,
                });
            }
            self.pos += 1;
        }
        name.push(c);
        name.push_str(&self.s_char_sequence(start_pos)?);
        span.location = (start_pos, self.pos);
        let is_header = self.include_state == 3;
        self.include_state = 0;
        if !is_l && is_header {
            let head_name = tok::HeaderName { span, name };
            Ok(tok::PreprocessingToken::HeaderName(head_name))
        } else {
            let str_lit = tok::StringLiteral { span, name };
            Ok(tok::PreprocessingToken::StringLiteral(str_lit))
        }
    }
    #[allow(dead_code)]
    fn punctuator(&mut self) -> Result<tok::PreprocessingToken, LexicalError> {
        todo!("punctuator")
    }

    fn escape_sequence(&mut self, start_pos: isize) -> Result<char, LexicalError> {
        let span = tok::Span {
            location: (start_pos, self.pos),
            file_key: self.file_key,
        };
        let Some(term) = self.chars.peek() else {
            return Err(LexicalError {
                kind: LexicalErrorKind::UnexpectedEscape,
                span,
            });
        };
        match term {
            // [c89] simple-escape-sequence
            '\'' | '"' | '?' | '\\' | 'a' | 'b' | 'f' | 'n' | 'r' | 't' | 'v' => {
                Ok(self.chars.next().unwrap())
            }
            // [c89] octal-escape-sequence
            '0'..='7' => todo!("octal-escape-sequence"),
            // [c89] hexadecimal-escape-sequence
            'x' => todo!("hexadecimal-escape-sequence"),
            _ => Err(LexicalError {
                kind: LexicalErrorKind::UnexpectedEscape,
                span,
            }),
        }
    }

    fn s_char_sequence(&mut self, start_pos: isize) -> Result<String, LexicalError> {
        let mut seq = String::new();
        while let Some(c) = self.chars.next_if(|&c| c != '\"' && c != '\n') {
            let s_char = if c == '\\' {
                self.escape_sequence(start_pos)?
            } else {
                c
            };
            seq.push(s_char)
        }
        Ok(seq)
    }
}

impl Iterator for Lexer<'_> {
    type Item = Result<tok::PreprocessingToken, LexicalError>;
    fn next(&mut self) -> Option<Self::Item> {
        // skip whitespace
        while self
            .chars
            .next_if(|&c| c != '\n' && c.is_ascii_whitespace())
            .is_some()
        {
            self.pos += 1;
        }
        let c = self.chars.next()?;
        let start_pos = self.pos;
        self.pos += 1;
        let mut span = tok::Span {
            location: (start_pos, self.pos),
            file_key: self.file_key,
        };

        if c == '"' || c == 'L' && self.chars.peek().is_some_and(|&val| val == '"') {
            return Some(self.string_literal(c, start_pos));
        }
        if c == '\'' || c == 'L' && self.chars.peek().is_some_and(|&val| val == '\'') {
            return Some(self.character_constant(c, start_pos));
        }

        let mut name = String::new();
        match c {
            '\n' => {
                self.include_state = 1;
                span.location = (start_pos, self.pos);
                name.push(c);
                let new_line = tok::NewLine { span };
                Some(Ok(tok::PreprocessingToken::NewLine(new_line)))
            }
            // punctuators without trailing characters
            '[' | ']' | '(' | ')' | '{' | '}' | '?' | ',' | '~' | ';' => {
                self.include_state = 0;
                span.location = (start_pos, self.pos);
                let punct = tok::Punctuator {
                    span,
                    term: tok::PunctuatorTerminal::try_from(c).unwrap(),
                };
                Some(Ok(tok::PreprocessingToken::Punctuator(punct)))
            }

            // identifier
            'a'..='z' | 'A'..='Z' | '_' => Some(self.identifier(c, start_pos)),
            // pp-number
            '0'..='9' => {
                self.include_state = 0;
                name.push(c);
                while let Some(&next_c) = self.chars.peek() {
                    if next_c.is_ascii_digit() || next_c == '.' {
                        name.push(self.chars.next()?);
                        self.pos += 1;
                    } else if next_c.is_ascii_alphabetic() || next_c == '_' {
                        name.push(self.chars.next()?);
                        if matches!(next_c, 'e' | 'E' | 'p' | 'P') {
                            let Some(&sign) = self.chars.peek() else {
                                span.location = (start_pos, self.pos);
                                return Some(Err(LexicalError {
                                    kind: LexicalErrorKind::UnexpectedEof,
                                    span,
                                }));
                            };
                            if matches!(sign, '-' | '+' | '0'..='9') {
                                name.push(self.chars.next()?);
                                self.pos += 2;
                                continue;
                            }
                        }
                        self.pos += 1;
                    } else {
                        break;
                    }
                }
                span.location = (start_pos, self.pos);
                let num = tok::PPNumber { span, name };
                Some(Ok(tok::PreprocessingToken::PPNumber(num)))
            }
            // `.` or `...` or pp-number
            '.' => {
                // case: `.`
                self.include_state = 0;
                name.push(c);
                if self.chars.next_if_eq(&'.').is_some() {
                    // case: `..`
                    self.pos += 1;
                    if self.chars.next_if_eq(&'.').is_some() {
                        // case: `...`
                        self.pos += 1;
                        span.location.1 = self.pos;
                        let punct = tok::Punctuator {
                            term: tok::PunctuatorTerminal::Ellipsis,
                            span,
                        };
                        Some(Ok(tok::PreprocessingToken::Punctuator(punct)))
                    } else {
                        span.location = (start_pos, self.pos);
                        Some(Err(LexicalError {
                            kind: LexicalErrorKind::InvalidToken,
                            span,
                        }))
                    }
                } else if let Some(digit) = self.chars.next_if(|c| c.is_ascii_digit()) {
                    // case: `.[0-9]`
                    name.push(digit);
                    while let Some(&next_c) = self.chars.peek() {
                        if next_c.is_ascii_digit() || next_c == '.' {
                            name.push(self.chars.next()?);
                            self.pos += 1;
                        } else if next_c.is_ascii_alphabetic() || next_c == '_' {
                            name.push(self.chars.next()?);
                            if matches!(next_c, 'e' | 'E' | 'p' | 'P') {
                                let Some(&sign) = self.chars.peek() else {
                                    span.location = (start_pos, self.pos);
                                    return Some(Err(LexicalError {
                                        kind: LexicalErrorKind::UnexpectedEof,
                                        span,
                                    }));
                                };
                                if matches!(sign, '-' | '+' | '0'..='9') {
                                    name.push(self.chars.next()?);
                                    self.pos += 2;
                                    continue;
                                }
                            }
                            self.pos += 1;
                        } else {
                            break;
                        }
                    }
                    span.location = (start_pos, self.pos);
                    let num = tok::PPNumber { span, name };
                    Some(Ok(tok::PreprocessingToken::PPNumber(num)))
                } else {
                    span.location = (start_pos, self.pos);
                    Some(Err(LexicalError {
                        kind: LexicalErrorKind::InvalidToken,
                        span,
                    }))
                }
            }
            '#' => {
                if self.include_state == 1 {
                    self.include_state = 2;
                }
                name.push(c);
                if self.chars.next_if_eq(&'#').is_some() {
                    self.include_state = 0;
                    self.pos += 1;
                    span.location = (start_pos, self.pos);
                }
                let punct = tok::Punctuator {
                    span,
                    term: tok::PunctuatorTerminal::Hash,
                };
                Some(Ok(tok::PreprocessingToken::Punctuator(punct)))
            }
            '<' => {
                let term = if self.include_state == 3 {
                    while let Some(next_c) = self.chars.next_if(|&c| c != '>' && c != '\n') {
                        name.push(next_c);
                        self.pos += 1;
                    }
                    span.location = (start_pos, self.pos);
                    if let Some(c) = self.chars.next_if_eq(&'>') {
                        name.push(c);
                        let hname = tok::HeaderName { span, name };
                        return Some(Ok(tok::PreprocessingToken::HeaderName(hname)));
                    } else if let Some(&'\n') = self.chars.peek() {
                        return Some(Err(LexicalError {
                            kind: LexicalErrorKind::InvalidToken,
                            span,
                        }));
                    } else {
                        return Some(Err(LexicalError {
                            kind: LexicalErrorKind::UnexpectedEof,
                            span,
                        }));
                    }
                } else if self.chars.next_if_eq(&'<').is_some() {
                    // case: `<<`
                    todo!("<<")
                } else if self.chars.next_if_eq(&':').is_some() {
                    // case: `<:` => `[`
                    self.pos += 1;
                    span.location = (start_pos, self.pos);
                    tok::PunctuatorTerminal::LSquare
                } else if self.chars.next_if_eq(&'%').is_some() {
                    // case: `<%` => `{`
                    self.pos += 1;
                    span.location = (start_pos, self.pos);
                    tok::PunctuatorTerminal::LCurly
                } else {
                    // case: `<`
                    tok::PunctuatorTerminal::Less
                };
                let punct = tok::Punctuator { span, term };
                Some(Ok(tok::PreprocessingToken::Punctuator(punct)))
            }
            '/' => {
                self.include_state = 0;
                let term = if self.chars.next_if_eq(&'/').is_some() {
                    // case: `//`
                    self.pos += 1;
                    while self.chars.next_if(|c| *c != '\n').is_some() {
                        self.pos += 1;
                    }
                    //todo jump to handle newline
                    if self.chars.next_if_eq(&'\n').is_some() {
                        self.pos += 1;
                        span.location = (self.pos - 1, self.pos);
                        let new_line = tok::NewLine { span };
                        return Some(Ok(tok::PreprocessingToken::NewLine(new_line)));
                    } else {
                        return None;
                    }
                } else if self.chars.next_if_eq(&'=').is_some() {
                    // case: `/=`
                    self.pos += 1;
                    span.location = (start_pos, self.pos);
                    tok::PunctuatorTerminal::PlusEqual
                } else {
                    tok::PunctuatorTerminal::Plus
                };
                let punct = tok::Punctuator { span, term };
                Some(Ok(tok::PreprocessingToken::Punctuator(punct)))
            }
            '+' => {
                self.include_state = 0;
                let term = if self.chars.next_if_eq(&'+').is_some() {
                    // case: `++`
                    self.pos += 1;
                    span.location = (start_pos, self.pos);
                    tok::PunctuatorTerminal::PlusPlus
                } else if self.chars.next_if_eq(&'=').is_some() {
                    // case: `+=`
                    self.pos += 1;
                    span.location = (start_pos, self.pos);
                    tok::PunctuatorTerminal::PlusEqual
                } else {
                    // case: `+`
                    tok::PunctuatorTerminal::Plus
                };
                let punct = tok::Punctuator { span, term };
                Some(Ok(tok::PreprocessingToken::Punctuator(punct)))
            }
            '-' => {
                self.include_state = 0;
                let term = if self.chars.next_if_eq(&'-').is_some() {
                    // case: `--`
                    self.pos += 1;
                    span.location = (start_pos, self.pos);
                    tok::PunctuatorTerminal::MinusMinus
                } else if self.chars.next_if_eq(&'=').is_some() {
                    // case: `+=`
                    self.pos += 1;
                    span.location = (start_pos, self.pos);
                    tok::PunctuatorTerminal::MinusEqual
                } else {
                    // case: `+`
                    tok::PunctuatorTerminal::Minus
                };
                let punct = tok::Punctuator { span, term };
                Some(Ok(tok::PreprocessingToken::Punctuator(punct)))
            }
            '=' => {
                self.include_state = 0;
                let term = if self.chars.next_if_eq(&'=').is_some() {
                    // case: `==`
                    self.pos += 1;
                    span.location = (start_pos, self.pos);
                    tok::PunctuatorTerminal::EqualEqual
                } else {
                    // case: `=`
                    tok::PunctuatorTerminal::Equal
                };
                let punct = tok::Punctuator { span, term };
                Some(Ok(tok::PreprocessingToken::Punctuator(punct)))
            }
            '*' => {
                self.include_state = 0;
                let term = if self.chars.next_if_eq(&'=').is_some() {
                    // case: `*=`
                    self.pos += 1;
                    span.location = (start_pos, self.pos);
                    tok::PunctuatorTerminal::StarEqual
                } else {
                    // case: `*`
                    tok::PunctuatorTerminal::Star
                };
                let punct = tok::Punctuator { span, term };
                Some(Ok(tok::PreprocessingToken::Punctuator(punct)))
            }
            ':' => {
                self.include_state = 0;
                let term = if self.chars.next_if_eq(&'>').is_some() {
                    // case: `:>`
                    self.pos += 1;
                    span.location = (start_pos, self.pos);
                    tok::PunctuatorTerminal::RSquare
                } else {
                    // case: `:`
                    tok::PunctuatorTerminal::Colon
                };
                let punct = tok::Punctuator { span, term };
                Some(Ok(tok::PreprocessingToken::Punctuator(punct)))
            }
            '!' => {
                self.include_state = 0;
                let term = if self.chars.next_if_eq(&'=').is_some() {
                    // case: `!=`
                    self.pos += 1;
                    span.location = (start_pos, self.pos);
                    tok::PunctuatorTerminal::BangEqual
                } else {
                    // case: `!`
                    tok::PunctuatorTerminal::Bang
                };
                let punct = tok::Punctuator { span, term };
                Some(Ok(tok::PreprocessingToken::Punctuator(punct)))
            }
            _ => todo!("{}", c as i32),
        }
    }
}
