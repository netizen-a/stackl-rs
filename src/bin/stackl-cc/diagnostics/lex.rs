// Copyright (c) 2024-2026 Jonathan A. Thomason

use std::{
	error,
	fmt,
};

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
