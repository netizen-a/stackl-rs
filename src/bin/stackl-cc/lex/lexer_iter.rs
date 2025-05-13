use std::collections::VecDeque;

use crate::tok::PPToken;

use super::{error::LexicalError, lexer::Lexer};

pub struct LexerQueue {
	queue: VecDeque<Lexer>,
}

impl LexerQueue {
	pub fn new() -> Self {
		Self {
			queue: VecDeque::new(),
		}
	}
	pub fn push_front(&mut self, lexer: Lexer) {
		self.queue.push_front(lexer);
	}
}

impl Iterator for LexerQueue {
	type Item = Result<PPToken, LexicalError>;
	fn next(&mut self) -> Option<Self::Item> {
		while let Some(lexer) = self.queue.front_mut() {
			if let Some(result) = lexer.next() {
				return Some(result);
			}
			self.queue.pop_front();
		}
		None
	}
}
