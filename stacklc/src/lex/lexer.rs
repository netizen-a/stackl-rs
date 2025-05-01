use std::iter;
use std::str::Chars;

use super::elements as tok;

#[derive(Debug)]
pub enum LexicalError {
    UnexpectedEof(tok::Span),
    UnexpectedEscape(tok::Span),
    InvalidToken(tok::Span),
}

#[derive(Debug)]
pub struct Lexer<'a> {
    chars: iter::Peekable<Chars<'a>>,
    pos: isize,
    file_key: usize,
    inc_stage: u8,
}
impl<'a> Lexer<'a> {
    pub fn new(text: &'a str, file_key: usize) -> Self {
        Self {
            chars: text.chars().peekable(),
            pos: -1,
            file_key,
            inc_stage: 0,
        }
    }
}

impl Iterator for Lexer<'_> {
    type Item = Result<tok::PreprocessingToken, LexicalError>;
    fn next(&mut self) -> Option<Self::Item> {
        let mut name = String::new();
        let mut c = self.chars.next()?;
        self.pos += 1;
        let mut span = tok::Span {
            location: (self.pos, self.pos),
            file_key: self.file_key,
        };

        // skip whitespace
        while c.is_ascii_whitespace() {
            if c == '\n' {
                self.inc_stage = 1;
                span.location = (self.pos, self.pos);
                name.push(c);
                let new_line = tok::NewLine { span, name };
                return Some(Ok(tok::PreprocessingToken::NewLine(new_line)));
            }
            c = self.chars.next()?;
            self.pos += 1;
        }

        if c == '"' || c == 'L' && self.chars.peek().is_some_and(|&val| val == '"') {
            let mut last_pos = self.pos;
            let is_l = c == 'L';
            if is_l {
                name.push(c);
                c = self.chars.next()?;
                last_pos += 1;
            }
            name.push(c);
            while let Some(&next_c) = self.chars.peek() {
                if next_c != '"' && next_c != '\\' {
                    name.push(self.chars.next()?);
                    last_pos += 1;
                } else if next_c == '\\' {
                    name.push(self.chars.next()?);
                    last_pos += 1;
                    span.location = (self.pos, last_pos);
                    let Some(&next_c) = self.chars.peek() else {
                        return Some(Err(LexicalError::UnexpectedEof(span)));
                    };
                    match next_c {
                        '\'' | '"' | '?' | '\\' | 'a' | 'b' | 'f' | 'n' | 'r' | 't' | 'v' => {
                            name.push(next_c);
                            last_pos += 1;
                        }
                        _ => {
                            return Some(Err(LexicalError::UnexpectedEscape(span)));
                        }
                    }
                } else {
                    break;
                }
            }
            span.location = (self.pos, last_pos);
            self.pos = last_pos;
            if is_l && self.inc_stage == 3 {
                let head_name = tok::HeaderName { span, name };
                return Some(Ok(tok::PreprocessingToken::HeaderName(head_name)));
            } else {
                let str_lit = tok::StringLiteral { span, name };
                return Some(Ok(tok::PreprocessingToken::StringLiteral(str_lit)));
            }
        }

        if c.is_ascii_alphabetic() || c == '_' {
            name.push(c);
            let mut last_pos = self.pos;
            while let Some(&next_c) = self.chars.peek() {
                if next_c.is_ascii_alphanumeric() || next_c == '_' {
                    name.push(self.chars.next()?);
                    last_pos += 1;
                } else {
                    break;
                }
            }
            span.location = (self.pos, last_pos);
            let ident = tok::Identifier { span, name };
            self.pos = last_pos;
            if self.inc_stage == 2 {
                self.inc_stage = 3;
            }
            return Some(Ok(tok::PreprocessingToken::Identifier(ident)));
        }

        match c {
            // punctuator without trailing characters
            '[' | ']' | '(' | ')' | '{' | '}' | '!' | '?' | ',' | '~' | ':' | ';' => {
                let span = tok::Span {
                    location: (self.pos, self.pos),
                    file_key: self.file_key,
                };
                let punct = tok::Punctuator {
                    span,
                    name: String::from(c),
                };
                Some(Ok(tok::PreprocessingToken::Punctuator(punct)))
            }
            '0'..='9' => {
                name.push(c);
                let mut last_pos = self.pos;
                while let Some(&next_c) = self.chars.peek() {
                    if next_c.is_ascii_digit() || next_c == '.' {
                        name.push(self.chars.next()?);
                        last_pos += 1;
                    } else if next_c.is_ascii_alphabetic() {
                        name.push(self.chars.next()?);
                        if matches!(next_c, 'e' | 'E' | 'p' | 'P') {
                            let Some(&sign) = self.chars.peek() else {
                                span.location = (self.pos, last_pos);
                                return Some(Err(LexicalError::UnexpectedEof(span)));
                            };
                            if matches!(sign, '-' | '+' | '0'..='9') {
                                name.push(self.chars.next()?);
                                last_pos += 2;
                                continue;
                            }
                        }
                        last_pos += 1;
                    } else {
                        break;
                    }
                }
                span.location = (self.pos, last_pos);
                self.pos = last_pos;
                let num = tok::PPNumber { span, name };
                Some(Ok(tok::PreprocessingToken::PPNumber(num)))
            }
            '.' => {
                // case: `.`
                name.push(c);
                let mut last_pos = self.pos;
                let Some(&next_c) = self.chars.peek() else {
                    span.location = (self.pos, last_pos);
                    let punct = tok::Punctuator { span, name };
                    return Some(Ok(tok::PreprocessingToken::Punctuator(punct)));
                };
                if next_c == '.' {
                    // case: `..`
                    name.push(self.chars.next()?);
                    last_pos += 1;
                    if let Some('.') = self.chars.peek() {
                        // case: `...`
                        name.push(self.chars.next()?);
                        last_pos += 1;
                        span.location = (self.pos, last_pos);
                        self.pos = last_pos;
                        let num = tok::PPNumber { span, name };
                        Some(Ok(tok::PreprocessingToken::PPNumber(num)))
                    } else {
                        span.location = (self.pos, last_pos);
                        self.pos = last_pos;
                        Some(Err(LexicalError::InvalidToken(span)))
                    }
                } else if next_c.is_ascii_digit() {
                    while let Some(&next_c) = self.chars.peek() {
                        if next_c.is_ascii_digit() || next_c == '.' {
                            name.push(self.chars.next()?);
                            last_pos += 1;
                        } else if next_c.is_ascii_alphabetic() {
                            name.push(self.chars.next()?);
                            if matches!(next_c, 'e' | 'E' | 'p' | 'P') {
                                let Some(&sign) = self.chars.peek() else {
                                    span.location = (self.pos, last_pos);
                                    return Some(Err(LexicalError::UnexpectedEof(span)));
                                };
                                if matches!(sign, '-' | '+' | '0'..='9') {
                                    name.push(self.chars.next()?);
                                    last_pos += 2;
                                    continue;
                                }
                            }
                            last_pos += 1;
                        } else {
                            break;
                        }
                    }
                    span.location = (last_pos, self.pos);
                    let num = tok::PPNumber { span, name };
                    Some(Ok(tok::PreprocessingToken::PPNumber(num)))
                } else {
                    todo!();
                }
            }
            _ => todo!("{}", c as i32),
        }
    }
}
