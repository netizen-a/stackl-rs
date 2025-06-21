use super::*;

pub struct Builder {
	module: ssa::Module,
	next_id: u32,
	selected_function: Option<usize>,
	selected_block: Option<usize>,
}

impl Builder {
	pub fn new() -> Self {
		Self {
			module: ssa::Module::default(),
			next_id: 0,
			selected_function: None,
			selected_block: None,
		}
	}
	#[inline]
	pub fn id(&mut self) -> u32 {
		let next_id = self.next_id;
		self.next_id += 1;
		next_id
	}
	pub fn i_add(&mut self, result_type: u32, lhs_id: u32, rhs_id: u32) -> u32 {
		let next_id = self.id();
		self.module.insts.push(ssa::Instruction {
			op: ssa::Opcode::Add,
			result_id: next_id,
			result_type,
			operands: vec![ssa::Operand::IdRef(lhs_id), ssa::Operand::IdRef(rhs_id)],
		});
		next_id
	}
	pub fn module(self) -> ssa::Module {
		todo!()
	}
}
