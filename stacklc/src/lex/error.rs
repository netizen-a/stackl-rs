use std::{error::Error, fmt};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct TryFromCharError(pub(crate) ());

impl fmt::Display for TryFromCharError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        "unicode code point out of range".fmt(f)
    }
}

impl Error for TryFromCharError {}
