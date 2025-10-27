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
	pub fn get_location(&self, source: &str) -> Option<(usize, usize)> {
		let (mut line, mut column) = (1, 1);
		let mut last_char = None;
		for character in source.chars().take(self.loc.0) {
			if let Some(
				'\r' | '\n' | '\u{000B}' | '\u{000C}' | '\u{0085}' | '\u{2028}' | '\u{2029}',
			) = last_char
			{
				column = 1;
				line += 1;
			}
			column += 1;
			last_char = Some(character);
		}
		Some((line, column))
	}
	pub fn to_vec(&self, source: &str) -> Vec<(usize, String, usize)> {
		let (_, column) = self.get_location(source).unwrap();
		let mut length = self.loc.1 - self.loc.0;
		let line_min = source[..self.loc.0]
			.chars()
			.filter(|x| {
				matches!(
					*x,
					'\r' | '\n' | '\u{000B}' | '\u{000C}' | '\u{0085}' | '\u{2028}' | '\u{2029}'
				)
			})
			.count();
		let line_max = source[..self.loc.1]
			.chars()
			.filter(|x| {
				matches!(
					*x,
					'\r' | '\n' | '\u{000B}' | '\u{000C}' | '\u{0085}' | '\u{2028}' | '\u{2029}'
				)
			})
			.count();
		let mut line_num = 0;
		let mut result = vec![];

		let mut min_column = column - 1;

		let mut is_first = true;
		for line in source.lines() {
			let line_count = line.chars().count();
			if line_min <= line_num && line_num <= line_max {
				if is_first {
					is_first = false;
				} else {
					min_column = 0;
					for b in line.chars() {
						if !b.is_ascii_whitespace() {
							break;
						}
						min_column += 1;
						length -= 1;
					}
				}
				let mut line_left = line_count.saturating_sub(min_column);
				let max_column = if length <= line_left {
					min_column + length
				} else {
					length -= line_left;
					for b in line.chars().rev() {
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
