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
    include_state: u8,
}
impl<'a> Lexer<'a> {
    pub fn new(text: &'a str, file_key: usize) -> Self {
        Self {
            chars: text.chars().peekable(),
            pos: -1,
            file_key,
            include_state: 1,
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
                self.include_state = 1;
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
            while let Some(next_c) = self.chars.next() {
                name.push(next_c);
                last_pos += 1;
                if next_c == '\\' {
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
                } else if next_c == '"' {
                    break;
                }
            }
            span.location = (self.pos, last_pos);
            self.pos = last_pos;
            let is_header = self.include_state == 3;
            self.include_state = 0;
            if !is_l && is_header {
                let head_name = tok::HeaderName { span, name };
                return Some(Ok(tok::PreprocessingToken::HeaderName(head_name)));
            } else {
                println!("\nis not header: {}, {}\n", is_l, is_header);
                let str_lit = tok::StringLiteral { span, name };
                return Some(Ok(tok::PreprocessingToken::StringLiteral(str_lit)));
            }
        }

        match c {
            // punctuator without trailing characters
            '[' | ']' | '(' | ')' | '{' | '}' | '!' | '?' | ',' | '~' | ':' | ';' => {
                self.include_state = 0;
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
            // identifier
            'a'..='z' | 'A'..='Z' | '_' => {
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
                self.pos = last_pos;
                if self.include_state == 2 && name == "include" {
                    self.include_state = 3;
                } else {
                    self.include_state = 0;
                }
                let ident = tok::Identifier { span, name };
                Some(Ok(tok::PreprocessingToken::Identifier(ident)))
            }
            // pp-number
            '0'..='9' => {
                self.include_state = 0;
                name.push(c);
                let mut last_pos = self.pos;
                while let Some(&next_c) = self.chars.peek() {
                    if next_c.is_ascii_digit() || next_c == '.' {
                        name.push(self.chars.next()?);
                        last_pos += 1;
                    } else if next_c.is_ascii_alphabetic() || next_c == '_' {
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
                self.include_state = 0;
                name.push(c);
                let Some(&next_c) = self.chars.peek() else {
                    span.location = (self.pos, self.pos);
                    let punct = tok::Punctuator { span, name };
                    return Some(Ok(tok::PreprocessingToken::Punctuator(punct)));
                };
                let mut last_pos = self.pos;
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
                    // case: `.[0-9]`
                    while let Some(&next_c) = self.chars.peek() {
                        if next_c.is_ascii_digit() || next_c == '.' {
                            name.push(self.chars.next()?);
                            last_pos += 1;
                        } else if next_c.is_ascii_alphabetic() || next_c == '_' {
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
                    span.location = (self.pos, last_pos);
                    self.pos = last_pos;
                    Some(Err(LexicalError::InvalidToken(span)))
                }
            }
            '#' => {
                if self.include_state == 1 {
                    self.include_state = 2;
                }
                name.push(c);
                let last_pos;
                if let Some('#') = self.chars.peek() {
                    name.push(self.chars.next()?);
                    last_pos = self.pos + 1;
                    self.include_state = 0;
                } else {
                    last_pos = self.pos;
                }
                span.location = (self.pos, last_pos);
                let punct = tok::Punctuator { span, name };
                Some(Ok(tok::PreprocessingToken::Punctuator(punct)))
            }
            '<' => {
                let mut last_pos = self.pos;
                if self.include_state == 3 {
                    let mut is_valid_seq = None;
                    for next_c in self.chars.by_ref() {
                        name.push(next_c);
                        last_pos += 1;
                        if next_c == '>' {
                            is_valid_seq = Some(true);
                            break;
                        }
                        if next_c == '\n' {
                            is_valid_seq = Some(false);
                            break;
                        }
                    }
                    span.location = (self.pos, last_pos);
                    match is_valid_seq {
                        Some(true) => {
                            let hname = tok::HeaderName { span, name };
                            Some(Ok(tok::PreprocessingToken::HeaderName(hname)))
                        }
                        Some(false) => Some(Err(LexicalError::InvalidToken(span))),
                        None => Some(Err(LexicalError::UnexpectedEof(span))),
                    }
                } else {
                    let Some(&next_c) = self.chars.peek() else {
                        span.location = (self.pos, self.pos);
                        let punct = tok::Punctuator { span, name };
                        return Some(Ok(tok::PreprocessingToken::Punctuator(punct)));
                    };
                    if next_c == '<' {
                        // case: `<<`
                        todo!();
                    } else if next_c == ':' {
                        // case: `<:`
                        name.clear();
                        name.push('[');
                        last_pos += 1;
                        todo!()
                    } else if next_c == '%' {
                        // case: `<%`
                        name.clear();
                        name.push('{');
                        last_pos += 1;
                        todo!();
                    } else {
                        todo!();
                    }
                }
            }
            _ => todo!("{}", c as i32),
        }
    }
}
