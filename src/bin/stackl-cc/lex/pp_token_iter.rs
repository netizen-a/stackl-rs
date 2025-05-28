use crate::tok::PPToken;

use super::lexer::Lexer;
use crate::diag::lex;

enum Queue {
	Buffer(Vec<lex::ResultTriple<PPToken, usize>>),
	Lexer(Lexer),
}

pub struct PPTokenQueue {
	stack: Vec<Queue>,
	peeked: Option<Option<lex::ResultTriple<PPToken, usize>>>,
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
	pub fn push_token(&mut self, hi: usize, pp_token: PPToken, lo: usize) {
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
			Some(Queue::Buffer(buffer)) => buffer.push(Ok((hi, pp_token, lo))),
			_ => {
				let buffer = vec![Ok((hi, pp_token, lo))];
				self.stack.push(Queue::Buffer(buffer))
			}
		}
	}
	pub fn peek(&mut self) -> Option<&lex::ResultTriple<PPToken, usize>> {
		let iter = &mut self.stack;
		self.peeked.get_or_insert_with(|| next_token(iter)).as_ref()
	}
}

impl Iterator for PPTokenQueue {
	type Item = lex::ResultTriple<PPToken, usize>;
	fn next(&mut self) -> Option<Self::Item> {
		match self.peeked.take() {
			Some(v) => v,
			None => next_token(&mut self.stack),
		}
	}
}

fn next_token(iter: &mut Vec<Queue>) -> Option<lex::ResultTriple<PPToken, usize>> {
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
