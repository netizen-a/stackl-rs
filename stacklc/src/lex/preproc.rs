use super::elements as tok;
use super::lexer as lex;
use std::io::BufReader;
use std::io::Read;
use std::{fs, io, path};

#[derive(Debug)]
pub enum ParseError {
    LexicalErrors(Vec<lex::LexicalError>),
    IOError(io::Error),
}

pub struct Preprocessor {
    file_map: bimap::BiHashMap<usize, path::PathBuf>,
    stdout: i32,
}

impl Preprocessor {
    pub fn new<P>(file_path: P, stdout: i32) -> Self
    where
        P: AsRef<path::Path>,
    {
        let mut file_map = bimap::BiHashMap::new();
        file_map.insert(0, file_path.as_ref().to_owned());
        Self { file_map, stdout }
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
                    print_whitespace(&token.span);
                    println!();
                }
                Ok(vec![])
            }
            PPToken::Comment(token) => {
                if self.stdout > 1 {
                    print_whitespace(&token.span);
                    print!("{}", token.name);
                }
                Ok(vec![])
            }
            PPToken::Identifier(token) => {
                if self.stdout > 0 {
                    print_whitespace(&token.span);
                    print!("{}", token.name);
                }
                Ok(vec![Token::Identifier(token)])
            }
            PPToken::Punctuator(token) => {
                if self.stdout > 0 {
                    print_whitespace(&token.span);
                    print!("{}", token.term);
                }
                Ok(vec![Token::Punctuator(token)])
            }
            PPToken::StringLiteral(token) => {
                if self.stdout > 0 {
                    print_whitespace(&token.span);
                    print!("{}", token.name);
                }
                Ok(vec![Token::StringLiteral(token)])
            }
            PPToken::CharacterConstant(token) => {
                if self.stdout > 0 {
                    print_whitespace(&token.span);
                    print!("{}", token.name);
                }
                Ok(vec![Token::Constant(tok::Constant::Character(token))])
            }
            PPToken::PPNumber(token) => {
                if is_floating_constant(&token)? {
                    Ok(vec![floating_constant(token)?])
                } else {
                    Ok(vec![integer_constant(token)?])
                }
            }
            PPToken::HeaderName(token) => todo!("header-name = {token:?}"),
        }
    }
}

fn print_whitespace(span: &tok::Span) {
    print!("{}", "\t".repeat(span.leading_tabs));
    print!("{}", " ".repeat(span.leading_spaces));
}

fn is_floating_constant(pp_number: &tok::PPNumber) -> Result<bool, lex::LexicalError> {
    if pp_number.name.contains('.') {
        // fractional-constant | hexadecimal-fractional-constant
        return Ok(true);
    }
    let mut chars = pp_number.name.chars().peekable();
    let c = chars.next().ok_or(lex::LexicalError {
        kind: lex::LexicalErrorKind::InvalidToken,
        span: pp_number.span.clone(),
    })?;
    if c == '0' && chars.next_if_eq(&'x').or(chars.next_if_eq(&'X')).is_some() {
        // binary-exponent-part
        Ok(chars.any(|c| c == 'p' || c == 'P'))
    } else {
        // exponent-part
        Ok(chars.any(|c| c == 'e' || c == 'E'))
    }
}

fn floating_constant(pp_number: tok::PPNumber) -> Result<tok::Token, lex::LexicalError> {
    todo!("floating-constant")
}

fn integer_constant(pp_number: tok::PPNumber) -> Result<tok::Token, lex::LexicalError> {
    todo!("integer-constant")
}
