pub trait ToSpan {
	fn to_span(&self) -> Span;
}

#[derive(Debug, Clone, Default)]
pub struct Span {
	pub loc: (usize, usize),
	/// actual file id
	pub file_id: usize,
	pub line: usize,
	/// reported name of file
	pub name_id: usize,
}

impl ToSpan for Span {
	fn to_span(&self) -> Span {
		self.clone()
	}
}

impl Span {
	pub fn column(&self, source: &str) -> Option<usize> {
		let mut column = 0;
		let mut last_byte = 1;
		for byte in source.as_bytes().get(0..=self.loc.0)? {
			if last_byte == b'\n' {
				column = 0;
			}
			column += 1;
			last_byte = *byte;
		}
		Some(column)
	}
	pub fn to_vec(&self, source: &str) -> Vec<(usize, String, usize)> {
		let column = self.column(source).unwrap();
		let mut length = self.loc.1 - self.loc.0;
		let line_min = source[..self.loc.0].chars().filter(|x| *x == '\n').count();
		let line_max = source[..self.loc.1].chars().filter(|x| *x == '\n').count();
		let mut line_num = 0;
		let mut result = vec![];

		let mut min_column = column - 1;

		let mut is_first = true;
		for line in source.lines() {
			if line_min <= line_num && line_num <= line_max {
				if is_first {
					is_first = false;
				} else {
					min_column = 0;
					for b in line.as_bytes() {
						if !b.is_ascii_whitespace() {
							break;
						}
						min_column += 1;
						length -= 1;
					}
				}
				let mut line_left = line.len() - min_column;
				let max_column = if length <= line_left {
					min_column + length
				} else {
					length -= line_left;
					for b in line.as_bytes().into_iter().rev() {
						if !b.is_ascii_whitespace() {
							break;
						}
						if line_left == 0 {
							break;
						}
						line_left -= 1;
					}
					min_column + line_left - 1
				};
				result.push((min_column, line.to_string(), max_column));
			}
			line_num += 1;
		}
		result
	}
}
