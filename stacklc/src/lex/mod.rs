mod elements;
use std::path::PathBuf;

use elements as tok;

pub enum LexerError {}

pub struct Lexer {
    file_names: bimap::BiHashMap<usize, PathBuf>,
    source: String,
}

impl Lexer {
    pub fn new(src: &str) -> Self {
        Self {
            file_names: bimap::BiHashMap::new(),
            source: src.to_owned(),
        }
    }
    pub fn to_preprocessing_tokens(&mut self) -> Result<Vec<tok::PreprocessingToken>, LexerError> {
        todo!()
    }
    pub fn to_tokens(&mut self) -> Result<Vec<tok::Token>, LexerError> {
        todo!()
    }
}
