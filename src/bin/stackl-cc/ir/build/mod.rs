mod decl;
mod expr;
use std::collections::HashMap;

use super::*;

pub struct Builder {
	types: HashMap<u32, ssa::DataType>,
	insts: Vec<ssa::Instruction>,
	next_id: u32,
	/// stack frame position
	sf_pos: u32,
}

impl Builder {
	pub fn new() -> Self {
		Self {
			types: HashMap::new(),
			insts: vec![],
			next_id: 0,
			sf_pos: 0,
		}
	}
	#[inline]
	pub fn id(&mut self) -> u32 {
		let next_id = self.next_id;
		self.next_id += 1;
		next_id
	}
}
