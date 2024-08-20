// Copyright (c) 2024-2026 Jonathan A. Thomason

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
		calculate_location(source, self.loc.0)
	}
	pub fn to_vec(&self, source: &str) -> Vec<(usize, String, usize)> {
		let (actual_line_min, column) = self.get_location(source).unwrap();
		let mut length = self.loc.1 - self.loc.0;
		let line_min = actual_line_min - 1;
		let (actual_line_max, _) = calculate_location(source, self.loc.1).unwrap();
		let line_max = actual_line_max - 1;

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
				let mut line_left = line_count.checked_sub(min_column).expect(&format!(
					"{actual_line_min}:( {line_count} <= {min_column} ):{line}"
				));
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

pub fn calculate_location(source: &str, loc: usize) -> Option<(usize, usize)> {
	let (mut line, mut column) = (1, 1);
	let mut last_char = None;
	for character in source.chars().take(loc) {
		if let Some('\r' | '\n' | '\u{000B}' | '\u{000C}' | '\u{0085}' | '\u{2028}' | '\u{2029}') =
			last_char
		{
			column = 1;
			line += 1;
		}
		column += 1;
		last_char = Some(character);
	}
	Some((line, column))
}
