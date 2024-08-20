// Copyright (c) 2024-2026 Jonathan A. Thomason

use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagLevel {
	Info,
	Warning,
	Error,
	Fatal,
}

#[derive(Debug, Clone)]
pub struct Diagnostic {
	pub level: DiagLevel,
	pub kind: kind::DiagKind,
	pub(super) notes: Vec<String>,
	pub(super) span_list: Vec<(Span, String)>,
}

impl Diagnostic {
	#[inline]
	pub fn fatal(kind: DiagKind, span: Option<Span>) -> Self {
		Self {
			level: DiagLevel::Fatal,
			kind,
			notes: vec![],
			span_list: span
				.map(|span| vec![(span, String::new())])
				.unwrap_or_default(),
		}
	}
	#[inline]
	pub fn error(kind: DiagKind, span: Span) -> Self {
		Self {
			level: DiagLevel::Error,
			kind,
			notes: vec![],
			span_list: vec![(span, String::new())],
		}
	}
	#[inline]
	pub fn warn(kind: kind::DiagKind, span: Span) -> Self {
		Self {
			level: DiagLevel::Warning,
			kind,
			notes: vec![],
			span_list: vec![(span, String::new())],
		}
	}
	#[inline]
	pub fn info(kind: kind::DiagKind, span: Option<Span>) -> Self {
		Self {
			level: DiagLevel::Info,
			kind,
			notes: vec![],
			span_list: span
				.map(|span| vec![(span, String::new())])
				.unwrap_or_default(),
		}
	}
	#[inline]
	pub fn push_note(&mut self, hint: &str) {
		self.notes.push(hint.to_string())
	}
	#[inline]
	pub fn push_span(&mut self, span: Span, msg: &str) {
		self.span_list.push((span, msg.to_string()))
	}
	/// for internal diagnostic use only.
	pub(super) fn pop_first_msg(&mut self, new_msg: &str) {
		if let Some((_, msg)) = self.span_list.first_mut() {
			*msg = new_msg.to_string();
		}
	}
}
