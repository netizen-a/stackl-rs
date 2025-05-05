use super::elements as tok;
use super::lexer as lex;
use std::io::Read;
use std::{fs, io, path};

pub enum ParseError {
    LexicalErrors(Vec<lex::LexicalError>),
    IOError(io::Error),
}

pub struct Preprocessor {
    file_map: bimap::BiHashMap<usize, path::PathBuf>,
}

impl Preprocessor {
    pub fn new<P>(file_path: P) -> Self
    where
        P: AsRef<path::Path>,
    {
        let mut file_map = bimap::BiHashMap::new();
        file_map.insert(0, file_path.as_ref().to_owned());
        Self { file_map }
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
        todo!("{pp_token:?}")
    }
}
