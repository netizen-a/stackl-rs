use super::elements as tok;
use super::lexer as lex;
use std::io::Read;
use std::{fs, io, path};

#[derive(Debug)]
pub enum ParseError {
    LexicalErrors(Vec<lex::LexicalError>),
    IOError(io::Error),
}

pub struct Preprocessor {
    file_map: bimap::BiHashMap<usize, path::PathBuf>,
    stdout: bool,
}

impl Preprocessor {
    pub fn new<P>(file_path: P, stdout: bool) -> Self
    where
        P: AsRef<path::Path>,
    {
        let mut file_map = bimap::BiHashMap::new();
        file_map.insert(0, file_path.as_ref().to_owned());
        Self { file_map, stdout }
    }
    pub fn parse(&mut self) -> Result<Vec<tok::Token>, ParseError> {
        let file_path = self.file_map.get_by_left(&0).unwrap();
        let mut file = fs::File::open(file_path).map_err(ParseError::IOError)?;
        let mut buf = String::new();
        file.read_to_string(&mut buf).map_err(ParseError::IOError)?;
        let lexer = lex::Lexer::new(&buf, 0);

        let mut errors = vec![];
        let mut tokens = vec![];
        for result in lexer {
            match result {
                Ok(pp_token) => tokens.append(&mut self.tokenize(pp_token)),
                Err(lex_error) => errors.push(lex_error),
            }
        }
        if !errors.is_empty() {
            Err(ParseError::LexicalErrors(errors))
        } else {
            Ok(tokens)
        }
    }
    fn tokenize(&mut self, pp_token: tok::PreprocessingToken) -> Vec<tok::Token> {
        use tok::PreprocessingToken as PPToken;
        use tok::Token;
        match pp_token {
            PPToken::NewLine(token) => {
                if self.stdout {
                    print_whitespace(&token.span);
                    println!();
                }
                vec![]
            }
            PPToken::Comment(_) => vec![],
            PPToken::Identifier(token) => {
                if self.stdout {
                    print_whitespace(&token.span);
                    print!("{}", token.name);
                }
                vec![Token::Identifier(token)]
            }
            PPToken::Punctuator(token) => {
                if self.stdout {
                    print_whitespace(&token.span);
                    print!("{}", token.term);
                }
                vec![Token::Punctuator(token)]
            }
            PPToken::StringLiteral(token) => {
                if self.stdout {
                    print_whitespace(&token.span);
                    print!("{}", token.name);
                }
                vec![Token::StringLiteral(token)]
            }
            PPToken::CharacterConstant(token) => {
                if self.stdout {
                    print_whitespace(&token.span);
                    print!("{}", token.name);
                }
                vec![Token::Constant(tok::Constant::Character(token))]
            }
            _ => todo!("{pp_token:?}"),
        }
    }
}
fn print_whitespace(span: &tok::Span) {
    print!("{}", "\t".repeat(span.leading_tabs));
    print!("{}", " ".repeat(span.leading_spaces));
}
