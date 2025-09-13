use std::fmt;

use super::*;

#[derive(Debug)]
pub enum DiagLevel {
	Warning,
	Error,
	Fatal,
	Internal,
}

#[derive(Debug)]
pub struct Diagnostic {
	pub level: DiagLevel,
	pub kind: kind::DiagKind,
	pub span: Span,
}

impl Diagnostic {
	#[inline]
	pub fn internal(msg: &str) -> Self {
		Self {
			level: DiagLevel::Internal,
			kind: DiagKind::Internal(msg.to_string()),
			span: Span::default(),
		}
	}
	#[inline]
	pub const fn error(kind: DiagKind, span: Span) -> Self {
		Self {
			level: DiagLevel::Error,
			kind,
			span,
		}
	}
	#[inline]
	pub const fn warn(kind: kind::DiagKind, span: Span) -> Self {
		Self {
			level: DiagLevel::Warning,
			kind,
			span,
		}
	}
}

impl fmt::Display for Diagnostic {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		todo!()
	}
}
