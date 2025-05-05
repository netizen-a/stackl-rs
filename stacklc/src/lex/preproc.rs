use super::elements as tok;
use std::{io, path};

pub struct Preprocessor {
    files: bimap::BiHashMap<usize, path::PathBuf>,
}

impl Preprocessor {
    pub fn new<P>(file_path: P) -> io::Result<Self>
    where
        P: AsRef<path::Path>,
    {
        Ok(Self {
            files: bimap::BiHashMap::new(),
        })
    }
    pub fn parse(&self) -> Vec<tok::Token> {
        todo!()
    }
}
