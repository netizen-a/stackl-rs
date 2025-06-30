use super::ssa;

impl super::Builder {
	pub fn variable(&mut self, result_type: u32, init_val: Option<u32>) -> u32 {
		let next_id = self.id();
		let mut operands = vec![];
		if let Some(val) = init_val {
			operands.push(ssa::Operand::IdRef(val))
		}
		self.insts.push(ssa::Instruction {
			op: ssa::Opcode::Variable,
			result_id: next_id,
			result_type,
			operands,
		});
		next_id
	}
}
