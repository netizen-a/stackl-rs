#[derive(Debug)]
pub struct CharIter {
	buf: Vec<char>,
	pub pos: usize,
}
impl CharIter {
	pub fn new(text: String) -> Self {
		Self {
			buf: text.chars().collect(),
			pos: 0,
		}
	}
	pub fn peek(&self) -> Option<char> {
		self.buf.get(self.pos).copied()
	}
	pub fn next_if(&mut self, func: impl FnOnce(char) -> bool) -> Option<char> {
		match self.peek() {
			Some(matched) if func(matched) => {
				self.pos += 1;
				Some(matched)
			}
			_ => None,
		}
	}
	pub fn next_if_eq(&mut self, expected: char) -> Option<char> {
		self.next_if(|next| next == expected)
	}
}

impl Iterator for CharIter {
	type Item = char;
	fn next(&mut self) -> Option<Self::Item> {
		let result = self.buf.get(self.pos);
		self.pos += 1;
		result.copied()
	}
}
