use std::{error, fmt, result};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct TryFromCharError;

impl fmt::Display for TryFromCharError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		"unicode code point out of range".fmt(f)
	}
}

impl error::Error for TryFromCharError {}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct TryFromIdentifierError;
impl fmt::Display for TryFromIdentifierError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		"identifier is not a keyword".fmt(f)
	}
}

impl error::Error for TryFromIdentifierError {}

#[derive(Debug)]
pub enum ErrorKind {
	UnexpectedEof,
	UnexpectedEscape,
	InvalidToken,
	HeaderNameError,
}

#[derive(Debug)]
pub struct Error {
	pub kind: ErrorKind,
	pub loc: (usize, usize),
}

pub type ResultTriple<Tok, Loc> = super::ResultTriple<Tok, Loc, Error>;
pub type Result<T> = result::Result<T, Error>;
