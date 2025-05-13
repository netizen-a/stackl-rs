use std::fmt;

#[derive(Debug, Clone)]
pub struct Span {
	pub location: (usize, usize),
	pub file_key: usize,
	pub leading_tabs: usize,
	pub leading_spaces: usize,
}
impl fmt::Display for Span {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(
			f,
			"{}{}",
			"\t".repeat(self.leading_tabs),
			" ".repeat(self.leading_spaces)
		)
	}
}

pub trait Spanned {
	fn span(&self) -> Span;
	fn set_span(&mut self, span: Span);
}
