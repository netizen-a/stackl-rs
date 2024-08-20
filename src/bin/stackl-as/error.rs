// Copyright (c) 2024-2026 Jonathan A. Thomason

use std::path::Path;

use crate::tok::{
	LexicalError,
	Token,
};
use lalrpop_util::{
	ErrorRecovery,
	ParseError,
};

pub fn print_errors(
	filename: &Path,
	error_list: Vec<ErrorRecovery<usize, Token, LexicalError>>,
	source: &str,
) {
	let filename = filename.display();
	for error_record in error_list {
		let parse_error = error_record.error;
		match parse_error {
			ParseError::InvalidToken { location } => {
				let line = loc2line(source, location).unwrap();
				eprintln!("invalid token at {line}")
			}
			ParseError::UnrecognizedEof { location, expected } => {
				let line = loc2line(source, location).unwrap();
				eprintln!(
					"error: unexpected EOF\n\
                    --> {filename}:{line},\n\
                    expected {:?}",
					expected
				);
			}
			ParseError::UnrecognizedToken { token, expected } => {
				let (start, token, _) = token;
				let line = loc2line(source, start).unwrap();
				eprintln!(
					"error: unrecognized token: {}\n\
                    --> {filename}:{}\n\
                    expected: {:?}",
					token, line, expected
				)
			}
			ParseError::ExtraToken { token } => eprintln!("extra token: {:?}", token),
			ParseError::User { error } => eprintln!("lexical error: {:?}", error),
		}
	}
}

fn loc2line(source: &str, loc: usize) -> Option<usize> {
	let mut line = 1;
	for byte in source.as_bytes().get(0..loc)? {
		if *byte == b'\n' {
			line += 1;
		}
	}
	Some(line)
}
