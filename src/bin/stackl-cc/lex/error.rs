use std::{error::Error, fmt};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct TryFromCharError(pub(crate) ());

impl fmt::Display for TryFromCharError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        "unicode code point out of range".fmt(f)
    }
}

impl Error for TryFromCharError {}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct TryFromIdentifierError(pub(crate) ());
impl fmt::Display for TryFromIdentifierError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        "identifier is not a keyword".fmt(f)
    }
}

impl Error for TryFromIdentifierError {}
