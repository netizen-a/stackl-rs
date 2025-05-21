use crate::tok::PPToken;

use super::lexer::Lexer;
use crate::diag::lex;

enum Queue {
	Buffer(Vec<lex::Result<PPToken>>),
	Lexer(Lexer),
}

pub struct PPTokenQueue {
	stack: Vec<Queue>,
	peeked: Option<Option<lex::Result<PPToken>>>,
}

impl PPTokenQueue {
	pub fn new() -> Self {
		Self {
			stack: Vec::new(),
			peeked: None,
		}
	}
	pub fn push_lexer(&mut self, lexer: Lexer) {
		if let Some(Some(pp_token)) = self.peeked.take() {
			match self.stack.last_mut() {
				Some(Queue::Buffer(buffer)) => buffer.push(pp_token),
				_ => {
					let buffer = vec![pp_token];
					self.stack.push(Queue::Buffer(buffer))
				}
			}
		}
		self.stack.push(Queue::Lexer(lexer));
	}
	pub fn push_token(&mut self, pp_token: PPToken) {
		if let Some(Some(pp_token)) = self.peeked.take() {
			match self.stack.last_mut() {
				Some(Queue::Buffer(buffer)) => buffer.push(pp_token),
				_ => {
					let buffer = vec![pp_token];
					self.stack.push(Queue::Buffer(buffer))
				}
			}
		}
		match self.stack.last_mut() {
			Some(Queue::Buffer(buffer)) => buffer.push(Ok(pp_token)),
			_ => {
				let buffer = vec![Ok(pp_token)];
				self.stack.push(Queue::Buffer(buffer))
			}
		}
	}
	pub fn peek(&mut self) -> Option<&lex::Result<PPToken>> {
		let iter = &mut self.stack;
		self.peeked.get_or_insert_with(|| next_token(iter)).as_ref()
	}
}

impl Iterator for PPTokenQueue {
	type Item = lex::Result<PPToken>;
	fn next(&mut self) -> Option<Self::Item> {
		match self.peeked.take() {
			Some(v) => v,
			None => next_token(&mut self.stack),
		}
	}
}

fn next_token(iter: &mut Vec<Queue>) -> Option<lex::Result<PPToken>> {
	while let Some(queue) = iter.last_mut() {
		if let Queue::Buffer(buffer) = queue {
			if let Some(result) = buffer.pop() {
				return Some(result);
			}
		} else if let Queue::Lexer(lexer) = queue {
			if let Some(result) = lexer.next() {
				return Some(result);
			}
		}
		iter.pop();
	}
	None
}
