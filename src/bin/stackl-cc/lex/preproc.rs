use super::lexer as lex;
use super::tok::{self, Spanned};
use std::io::BufReader;
use std::io::Read;
use std::iter::Peekable;
use std::slice::Iter;
use std::str::Chars;
use std::{fs, io, path};
use tok::PPToken;
use tok::Token;

#[derive(Debug)]
pub enum ParseError {
    LexicalErrors(Vec<lex::LexicalError>),
    IOError(io::Error),
}

pub struct Preprocessor {
    file_map: bimap::BiHashMap<usize, path::PathBuf>,
    stdout: i32,
    is_newline: bool,
    is_preproc: bool,
}

impl Preprocessor {
    pub fn new<P>(file_path: P, stdout: i32) -> Self
    where
        P: AsRef<path::Path>,
    {
        let mut file_map = bimap::BiHashMap::new();
        file_map.insert(0, file_path.as_ref().to_owned());
        Self {
            file_map,
            stdout,
            is_newline: true,
            is_preproc: false,
        }
    }
    pub fn parse(&mut self) -> Result<Vec<tok::Token>, ParseError> {
        let file_path = self.file_map.get_by_left(&0).unwrap();
        let file = fs::File::open(file_path).map_err(ParseError::IOError)?;
        let mut reader = BufReader::new(file);
        let mut buf = String::new();
        reader
            .read_to_string(&mut buf)
            .map_err(ParseError::IOError)?;
        drop(reader);
        let lexer = lex::Lexer::new(&buf, 0);

        let mut errors = vec![];
        let mut pp_tokens = vec![];
        for result in lexer {
            match result {
                Ok(pp_token) => pp_tokens.push(pp_token),
                Err(lex_error) => errors.push(lex_error),
            }
        }

        let mut pp_iter = pp_tokens.iter();
        let mut tokens = vec![];
        while let Some(pp_token) = pp_iter.next() {
            match self.tokenize(pp_token.clone(), &pp_iter) {
                Ok(mut processed_tokens) => tokens.append(&mut processed_tokens),
                Err(processed_errors) => errors.push(processed_errors),
            }
        }

        if !errors.is_empty() {
            Err(ParseError::LexicalErrors(errors))
        } else {
            Ok(tokens)
        }
    }
    fn tokenize(
        &mut self,
        pp_token: PPToken,
        pp_iter: &Iter<PPToken>,
    ) -> Result<Vec<tok::Token>, lex::LexicalError> {
        match pp_token {
            PPToken::NewLine(token) => {
                if self.stdout > 0 {
                    token.span().print_whitespace();
                    println!();
                }
                self.is_preproc = false;
                self.is_newline = true;
                Ok(vec![])
            }
            PPToken::Comment(token) => {
                if self.stdout > 1 {
                    token.span().print_whitespace();
                    print!("{}", token.name);
                }
                Ok(vec![])
            }
            PPToken::Identifier(token) => {
                if self.stdout > 0 {
                    token.span().print_whitespace();
                    print!("{}", token.name);
                }
                if self.is_preproc {
                    todo!("preproc")
                } else if let Ok(kw) = tok::Keyword::try_from(token.clone()) {
                    Ok(vec![Token::Keyword(kw)])
                } else {
                    Ok(vec![Token::Identifier(token)])
                }
            }
            PPToken::Punctuator(token) => {
                if self.stdout > 0 {
                    token.span().print_whitespace();
                    print!("{}", token.term);
                }
                if let tok::PunctuatorTerminal::Hash = token.term {
                    self.is_preproc = self.is_newline;
                    self.is_newline = false;
                    Ok(vec![])
                } else {
                    Ok(vec![Token::Punctuator(token)])
                }
            }
            PPToken::StringLiteral(token) => {
                self.is_newline = false;
                if self.stdout > 0 {
                    token.span().print_whitespace();
                    print!("{}", token.name);
                }
                Ok(vec![Token::StringLiteral(token)])
            }
            PPToken::CharacterConstant(token) => {
                self.is_newline = false;
                if self.stdout > 0 {
                    token.span().print_whitespace();
                    print!("{}", token.name);
                }
                Ok(vec![Token::Constant(tok::Constant::Character(token))])
            }
            PPToken::PPNumber(token) => {
                self.is_newline = false;
                let token = Token::try_from(token)?;
                if self.stdout > 0 {
                    token.span().print_whitespace();
                    print!("{token}");
                }
                Ok(vec![token])
            }
            PPToken::HeaderName(token) => todo!("header-name = {token:?}"),
        }
    }
}
