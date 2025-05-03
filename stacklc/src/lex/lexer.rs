use std::iter;
use std::str::Chars;

use super::elements as tok;

macro_rules! punctuator_token {
    ($this:ident, $punct:ident, $span:ident, $start_pos:ident) => {{
        $this.include_state = 0;
        $span.location = ($start_pos, $this.pos);
        let punct = tok::Punctuator::$punct($span);
        Some(Ok(tok::PreprocessingToken::Punctuator(punct)))
    }};
}

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
    #[allow(dead_code)]
    pub fn new(text: &'a str, file_key: usize) -> Self {
        Self {
            chars: text.chars().peekable(),
            pos: 0,
            file_key,
            include_state: 1,
        }
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
        let mut name = String::new();
        let mut c = self.chars.next()?;
        let start_pos = self.pos;
        self.pos += 1;
        let mut span = tok::Span {
            location: (start_pos, self.pos),
            file_key: self.file_key,
        };

        if c == '"' || c == 'L' && self.chars.peek().is_some_and(|&val| val == '"') {
            let is_l = c == 'L';
            if is_l {
                name.push(c);
                c = self.chars.next()?;
                self.pos += 1;
            }
            name.push(c);
            while let Some(next_c) = self.chars.next() {
                name.push(next_c);
                self.pos += 1;
                if next_c == '\\' {
                    span.location = (start_pos, self.pos);
                    let Some(&next_c) = self.chars.peek() else {
                        return Some(Err(LexicalError::UnexpectedEof(span)));
                    };
                    match next_c {
                        '\'' | '"' | '?' | '\\' | 'a' | 'b' | 'f' | 'n' | 'r' | 't' | 'v' => {
                            name.push(next_c);
                            self.pos += 1;
                        }
                        _ => {
                            return Some(Err(LexicalError::UnexpectedEscape(span)));
                        }
                    }
                } else if next_c == '"' {
                    break;
                }
            }
            span.location = (start_pos, self.pos);
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
            '\n' => {
                self.include_state = 1;
                span.location = (start_pos, self.pos);
                name.push(c);
                let new_line = tok::NewLine { span };
                Some(Ok(tok::PreprocessingToken::NewLine(new_line)))
            }
            // punctuators without trailing characters
            '[' => punctuator_token!(self, LSquare, span, start_pos),
            ']' => punctuator_token!(self, RSquare, span, start_pos),
            '(' => punctuator_token!(self, LParen, span, start_pos),
            ')' => punctuator_token!(self, RParen, span, start_pos),
            '{' => punctuator_token!(self, LCurly, span, start_pos),
            '}' => punctuator_token!(self, RCurly, span, start_pos),
            '?' => punctuator_token!(self, QMark, span, start_pos),
            ',' => punctuator_token!(self, QMark, span, start_pos),
            '~' => punctuator_token!(self, QMark, span, start_pos),
            ';' => punctuator_token!(self, QMark, span, start_pos),

            // identifier
            'a'..='z' | 'A'..='Z' | '_' => {
                name.push(c);
                while let Some(&next_c) = self.chars.peek() {
                    if next_c.is_ascii_alphanumeric() || next_c == '_' {
                        name.push(self.chars.next()?);
                        self.pos += 1;
                    } else {
                        break;
                    }
                }
                span.location = (start_pos, self.pos);
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
                while let Some(&next_c) = self.chars.peek() {
                    if next_c.is_ascii_digit() || next_c == '.' {
                        name.push(self.chars.next()?);
                        self.pos += 1;
                    } else if next_c.is_ascii_alphabetic() || next_c == '_' {
                        name.push(self.chars.next()?);
                        if matches!(next_c, 'e' | 'E' | 'p' | 'P') {
                            let Some(&sign) = self.chars.peek() else {
                                span.location = (start_pos, self.pos);
                                return Some(Err(LexicalError::UnexpectedEof(span)));
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
            '.' => {
                // case: `.`
                self.include_state = 0;
                name.push(c);
                let Some(&next_c) = self.chars.peek() else {
                    span.location = (self.pos, self.pos);
                    let punct = tok::Punctuator::Dot(span);
                    return Some(Ok(tok::PreprocessingToken::Punctuator(punct)));
                };
                if next_c == '.' {
                    // case: `..`
                    name.push(self.chars.next()?);
                    self.pos += 1;
                    if let Some('.') = self.chars.peek() {
                        // case: `...`
                        name.push(self.chars.next()?);
                        self.pos += 1;
                        span.location = (start_pos, self.pos);
                        let num = tok::PPNumber { span, name };
                        Some(Ok(tok::PreprocessingToken::PPNumber(num)))
                    } else {
                        span.location = (start_pos, self.pos);
                        Some(Err(LexicalError::InvalidToken(span)))
                    }
                } else if next_c.is_ascii_digit() {
                    // case: `.[0-9]`
                    while let Some(&next_c) = self.chars.peek() {
                        if next_c.is_ascii_digit() || next_c == '.' {
                            name.push(self.chars.next()?);
                            self.pos += 1;
                        } else if next_c.is_ascii_alphabetic() || next_c == '_' {
                            name.push(self.chars.next()?);
                            if matches!(next_c, 'e' | 'E' | 'p' | 'P') {
                                let Some(&sign) = self.chars.peek() else {
                                    span.location = (start_pos, self.pos);
                                    return Some(Err(LexicalError::UnexpectedEof(span)));
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
                    Some(Err(LexicalError::InvalidToken(span)))
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
                    let punct = tok::Punctuator::HashHash(span);
                    Some(Ok(tok::PreprocessingToken::Punctuator(punct)))
                } else {
                    let punct = tok::Punctuator::Hash(span);
                    Some(Ok(tok::PreprocessingToken::Punctuator(punct)))
                }
            }
            '<' => {
                if self.include_state == 3 {
                    while let Some(next_c) = self.chars.next_if(|&c| c != '>' && c != '\n') {
                        name.push(next_c);
                        self.pos += 1;
                    }
                    span.location = (start_pos, self.pos);
                    if let Some(c) = self.chars.next_if_eq(&'>') {
                        name.push(c);
                        let hname = tok::HeaderName { span, name };
                        Some(Ok(tok::PreprocessingToken::HeaderName(hname)))
                    } else if let Some(&'\n') = self.chars.peek() {
                        Some(Err(LexicalError::InvalidToken(span)))
                    } else {
                        Some(Err(LexicalError::UnexpectedEof(span)))
                    }
                } else if self.chars.next_if_eq(&'<').is_some() {
                    // case: `<<`
                    todo!()
                } else if self.chars.next_if_eq(&':').is_some() {
                    // case: `<:`
                    name.clear();
                    self.pos += 1;
                    todo!()
                } else if self.chars.next_if_eq(&'%').is_some() {
                    // case: `<%`
                    name.clear();
                    name.push('{');
                    self.pos += 1;
                    todo!();
                } else {
                    // case: `<`
                    todo!();
                }
            }
            '/' => {
                if self.chars.next_if_eq(&'/').is_some() {
                    self.pos += 1;
                    while self.chars.next_if(|c| *c != '\n').is_some() {
                        self.pos += 1;
                    }
                    //todo jump to handle newline
                    if self.chars.next_if_eq(&'\n').is_some() {
                        self.pos += 1;
                        span.location = (self.pos - 1, self.pos);
                        let new_line = tok::NewLine { span };
                        Some(Ok(tok::PreprocessingToken::NewLine(new_line)))
                    } else {
                        None
                    }
                } else if self.chars.next_if_eq(&'=').is_some() {
                    todo!("'/='")
                } else {
                    todo!()
                }
            }
            // '+' => {
            //     if self.chars.next_if_eq(&'+') {
            //         self.pos += 1;
            //     }
            // }
            _ => todo!("{}", c as i32),
        }
    }
}
