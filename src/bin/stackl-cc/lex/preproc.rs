use super::elements as tok;
use super::elements::Spanned;
use super::lexer as lex;
use std::io::BufReader;
use std::io::Read;
use std::iter::Peekable;
use std::str::Chars;
use std::{fs, io, path};

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
        let mut tokens = vec![];
        for result in lexer {
            match result {
                Ok(pp_token) => match self.tokenize(pp_token) {
                    Ok(mut processed_tokens) => tokens.append(&mut processed_tokens),
                    Err(processed_errors) => errors.push(processed_errors),
                },
                Err(lex_error) => errors.push(lex_error),
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
        pp_token: tok::PreprocessingToken,
    ) -> Result<Vec<tok::Token>, lex::LexicalError> {
        use tok::PreprocessingToken as PPToken;
        use tok::Token;
        match pp_token {
            PPToken::NewLine(token) => {
                if self.stdout > 0 {
                    token.span.print_whitespace();
                    println!();
                }
                self.is_preproc = false;
                self.is_newline = true;
                Ok(vec![])
            }
            PPToken::Comment(token) => {
                if self.stdout > 1 {
                    token.span.print_whitespace();
                    print!("{}", token.name);
                }
                Ok(vec![])
            }
            PPToken::Identifier(token) => {
                if self.stdout > 0 {
                    token.span.print_whitespace();
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
                    token.span.print_whitespace();
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
                    token.span.print_whitespace();
                    print!("{}", token.name);
                }
                Ok(vec![Token::StringLiteral(token)])
            }
            PPToken::CharacterConstant(token) => {
                self.is_newline = false;
                if self.stdout > 0 {
                    token.span.print_whitespace();
                    print!("{}", token.name);
                }
                Ok(vec![Token::Constant(tok::Constant::Character(token))])
            }
            PPToken::PPNumber(token) => {
                self.is_newline = false;
                if token.is_float() {
                    Ok(vec![floating_constant(token)?])
                } else {
                    let token = integer_constant(token)?;
                    if self.stdout > 0 {
                        token.span().print_whitespace();
                        print!("{token}");
                    }
                    Ok(vec![token])
                }
            }
            PPToken::HeaderName(token) => todo!("header-name = {token:?}"),
        }
    }
}

fn floating_constant(pp_number: tok::PPNumber) -> Result<tok::Token, lex::LexicalError> {
    let mut chars = pp_number.name.chars().peekable();
    let c = chars.next().expect("empty pp-number");
    if c == '0' && chars.next_if(|&c| c == 'x' || c == 'X').is_some() {
        todo!("hexadecimal-floating-constant")
    } else {
        todo!("decimal-floating-constant")
    }
}

fn integer_constant(pp_number: tok::PPNumber) -> Result<tok::Token, lex::LexicalError> {
    let mut chars = pp_number.name.chars().peekable();
    match chars.next().expect("empty pp-number") {
        '0' => {
            if chars.next_if(|&c| c == 'x' || c == 'X').is_some() {
                todo!("hexadecimal-constant")
            } else {
                todo!("octal-constant")
            }
        }
        c @ '1'..='9' => {
            let name = String::from(c);
            decimal_constant(&pp_number, name, chars)
        }
        _ => Err(lex::LexicalError {
            kind: lex::LexicalErrorKind::InvalidToken,
            span: pp_number.span,
        }),
    }
}

fn decimal_constant(
    pp_number: &tok::PPNumber,
    mut name: String,
    mut chars: Peekable<Chars>,
) -> Result<tok::Token, lex::LexicalError> {
    while let Some(digit) = chars.next_if(char::is_ascii_digit) {
        name.push(digit);
    }
    let data = name.parse::<i128>().or(Err(lex::LexicalError {
        kind: lex::LexicalErrorKind::InvalidToken,
        span: pp_number.span.clone(),
    }))?;
    if chars.next_if(|&c| c == 'u' || c == 'U').is_some() {
        todo!("unsigned-suffix")
    } else if chars.next_if(|&c| c == 'l' || c == 'L').is_some() {
        todo!("long-suffix")
    } else if chars.peek().is_none() {
        let integer = tok::IntegerConstant {
            span: pp_number.span.clone(),
            data,
            suff: tok::IntegerSuffix::None,
        };
        let constant = tok::Constant::Integer(integer);
        Ok(tok::Token::Constant(constant))
    } else {
        todo!("error")
    }
}
