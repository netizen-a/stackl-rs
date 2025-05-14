use std::collections::VecDeque;

use crate::tok::PPToken;

use super::{error::LexicalError, lexer::Lexer};

enum Queue {
	Buffer(VecDeque<Result<PPToken, LexicalError>>),
	Lexer(Lexer),
}

pub struct PPTokenQueue {
	queue: VecDeque<Queue>,
	peeked: Option<Option<Result<PPToken, LexicalError>>>,
}

impl PPTokenQueue {
	pub fn new() -> Self {
		Self {
			queue: VecDeque::new(),
			peeked: None,
		}
	}
	pub fn push_lexer_front(&mut self, lexer: Lexer) {
		if let Some(Some(pp_token)) = self.peeked.take() {
			match self.queue.front_mut() {
				Some(Queue::Buffer(buffer)) => buffer.push_front(pp_token),
				_ => {
					let mut buffer = VecDeque::new();
					buffer.push_front(pp_token);
					self.queue.push_front(Queue::Buffer(buffer))
				}
			}
		}
		self.queue.push_front(Queue::Lexer(lexer));
	}
	pub fn push_token_front(&mut self, pp_token: PPToken) {
		if let Some(Some(pp_token)) = self.peeked.take() {
			match self.queue.front_mut() {
				Some(Queue::Buffer(buffer)) => buffer.push_front(pp_token),
				_ => {
					let mut buffer = VecDeque::new();
					buffer.push_front(pp_token);
					self.queue.push_front(Queue::Buffer(buffer))
				}
			}
		}
		match self.queue.front_mut() {
			Some(Queue::Buffer(buffer)) => buffer.push_front(Ok(pp_token)),
			_ => {
				let mut buffer = VecDeque::new();
				buffer.push_front(Ok(pp_token));
				self.queue.push_front(Queue::Buffer(buffer))
			}
		}
	}
	pub fn peek(&mut self) -> Option<&Result<PPToken, LexicalError>> {
		let iter = &mut self.queue;
		self.peeked.get_or_insert_with(|| next_token(iter)).as_ref()
	}
}

impl Iterator for PPTokenQueue {
	type Item = Result<PPToken, LexicalError>;
	fn next(&mut self) -> Option<Self::Item> {
		match self.peeked.take() {
			Some(v) => v,
			None => next_token(&mut self.queue),
		}
	}
}

fn next_token(iter: &mut VecDeque<Queue>) -> Option<Result<PPToken, LexicalError>> {
	while let Some(queue) = iter.front_mut() {
		if let Queue::Buffer(buffer) = queue {
			if let Some(result) = buffer.pop_front() {
				return Some(result);
			}
		} else if let Queue::Lexer(lexer) = queue {
			if let Some(result) = lexer.next() {
				return Some(result);
			}
		}
		iter.pop_front();
	}
	None
}
