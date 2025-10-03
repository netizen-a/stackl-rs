use super::*;

#[derive(Debug, Clone, Copy)]
pub enum DiagLevel {
	Warning,
	Error,
	Fatal,
}

#[derive(Debug, Clone)]
pub struct Diagnostic {
	pub level: DiagLevel,
	pub kind: kind::DiagKind,
	pub span: Option<Span>,
	pub(super) notes: Vec<String>,
}

impl Diagnostic {
	#[inline]
	pub const fn fatal(kind: DiagKind) -> Self {
		Self {
			level: DiagLevel::Fatal,
			kind,
			span: None,
			notes: vec![],
		}
	}
	#[inline]
	pub const fn error(kind: DiagKind, span: Span) -> Self {
		Self {
			level: DiagLevel::Error,
			kind,
			span: Some(span),
			notes: vec![],
		}
	}
	#[inline]
	pub const fn warn(kind: kind::DiagKind, span: Span) -> Self {
		Self {
			level: DiagLevel::Warning,
			kind,
			span: Some(span),
			notes: vec![],
		}
	}
	pub fn push_note(&mut self, hint: &str) {
		self.notes.push(hint.to_string())
	}
}
