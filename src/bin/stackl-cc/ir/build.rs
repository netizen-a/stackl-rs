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
	pub fn alloc(&mut self, _result_type: u32, _init_val: Option<u32>) -> u32 {
		todo!()
	}
	pub fn i_add(&mut self, result_type: u32, lhs_id: u32, rhs_id: u32) -> u32 {
		let next_id = self.id();
		self.insts.push(ssa::Instruction {
			op: ssa::Opcode::IAdd,
			result_id: next_id,
			result_type,
			operands: vec![ssa::Operand::IdRef(lhs_id), ssa::Operand::IdRef(rhs_id)],
		});
		next_id
	}
	pub fn f_add(&mut self, result_type: u32, lhs_id: u32, rhs_id: u32) -> u32 {
		let next_id = self.id();
		self.insts.push(ssa::Instruction {
			op: ssa::Opcode::FAdd,
			result_id: next_id,
			result_type,
			operands: vec![ssa::Operand::IdRef(lhs_id), ssa::Operand::IdRef(rhs_id)],
		});
		next_id
	}
}
