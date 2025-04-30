use std::iter;
use std::str::Chars;

use super::elements as tok;

#[derive(Debug)]
pub enum LexerError {
    Unknown,
}

#[derive(Debug)]
pub struct Lexer<'a> {
    chars: iter::Peekable<Chars<'a>>,
    pos: usize,
    file_key: usize,
    line_start: bool,
    last_token: Option<tok::PreprocessingToken>,
}
impl<'a> Lexer<'a> {
    pub fn new(text: &'a str, file_key: usize) -> Self {
        Self {
            chars: text.chars().peekable(),
            pos: 0,
            file_key,
            line_start: true,
            last_token: None,
        }
    }
}

impl Iterator for Lexer<'_> {
    type Item = Result<tok::PreprocessingToken, LexerError>;
    fn next(&mut self) -> Option<Self::Item> {
        let mut value = String::new();
        let mut c = self.chars.next()?;
        self.pos += 1;

        // skip whitespace
        while c.is_ascii_whitespace() {
            // If line_start then whitespace should not overwrite this value.
            if !self.line_start {
                self.line_start = c == '\n';
            }
            c = self.chars.next()?;
            self.pos += 1;
        }

        if c.is_ascii_alphabetic() || c == '_' {
            value.push(c);
            let mut last_pos = self.pos;
            while let Some(next_c) = self.chars.peek() {
                if next_c.is_ascii_alphanumeric() || *next_c == '_' {
                    value.push(self.chars.next()?);
                    last_pos += 1;
                } else {
                    break;
                }
            }
            let span = tok::Span {
                location: (self.pos, last_pos),
                file_key: self.file_key,
            };
            let ident = tok::Identifier { span, name: value };
            self.pos = last_pos;
            self.line_start = false;
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
                self.line_start = false;
                return Some(Ok(tok::PreprocessingToken::Punctuator(punct)));
            }
            _ => todo!("{}", c as i32),
        }
    }
}
