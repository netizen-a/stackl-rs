use crate::analysis::tok::file_id::FileId;

#[derive(Debug, Clone, Default)]
pub struct Span {
	pub loc: (usize, usize),
	pub file_id: usize,
}

impl FileId for Span {
	fn file_id(&self) -> usize {
		self.file_id
	}
}

impl Span {
	/// returns (line, column)
	pub fn location(&self, source: &str) -> Option<(usize, usize)> {
		let mut line = 1;
		let mut column = 0;
		let mut last_byte = 1;
		for byte in source.as_bytes().get(0..=self.loc.0)? {
			if last_byte == b'\n' {
				column = 0;
			}
			column += 1;
			if *byte == b'\n' {
				line += 1;
			}
			last_byte = *byte;
		}
		Some((line, column))
	}
	pub fn to_string_vec(&self, source: &str) -> Vec<String> {
		let line_min = source[..self.loc.0].chars().filter(|x| *x == '\n').count();
		let line_max = source[..self.loc.1].chars().filter(|x| *x == '\n').count();
		let mut line_num = 0;
		let mut result = vec![];
		for line in source.lines() {
			if line_min <= line_num && line_num <= line_max {
				result.push(line.to_string());
			}
			line_num += 1;
		}
		result
	}
}
