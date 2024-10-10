use std::path::Path;

use lalrpop_util::{ErrorRecovery, ParseError};
use stackl::tok::{LexicalError, Token};

pub fn print_errors(filename: &Path, error_list: Vec<ErrorRecovery<usize, Token, LexicalError>>) {
    let filename = filename.display();
    for error_record in error_list {
        let parse_error = error_record.error;
        match parse_error {
            ParseError::InvalidToken { location } => eprintln!("invalid token at {location}"),
            ParseError::UnrecognizedEof { location, expected } => {
                eprintln!(
                    "error: unexpected EOF\n\
                    --> {filename}:{location},\n\
                    expected {:?}",
                    expected
                );
            }
            ParseError::UnrecognizedToken { token, expected } => {
                let (start, token, _end) = token;
                eprintln!(
                    "error: unrecognized token: {}\n\
                    --> {filename}:{}\n\
                    expected: {:?}",
                    token, start, expected
                )
            }
            ParseError::ExtraToken { token } => eprintln!("unrecognized token: {:?}", token),
            ParseError::User { error } => eprintln!("lexical error: {:?}", error),
        }
    }
}
