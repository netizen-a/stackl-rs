use super::ssa;
impl super::Builder {
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
