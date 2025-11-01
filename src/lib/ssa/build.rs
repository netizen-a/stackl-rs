use super::data;
use super::Error;

pub struct Builder {
	inst_list: Vec<data::Instruction>,
	next_id: u32,
}

impl Builder {
	pub fn new() -> Self {
		Self {
			inst_list: vec![],
			next_id: 0,
		}
	}
	/// Returns the next unused id
	pub fn id(&mut self) -> u32 {
		let result = self.next_id;
		self.next_id += 1;
		result
	}
	pub fn nop(&mut self) -> Result<(), Error> {
		self.inst_list.push(data::Instruction {
			opcode: data::Opcode::Nop,
			result_id: None,
			result_type: None,
			operands: [].into(),
		});
		Ok(())
	}
	pub fn i_add(&mut self, result_type: u32, operands: [u32; 2]) -> Result<u32, Error> {
		let id = self.id();
		self.inst_list.push(data::Instruction {
			opcode: data::Opcode::IAdd,
			result_id: Some(id),
			result_type: Some(result_type),
			operands: operands.into(),
		});
		Ok(id)
	}
	pub fn i_sub(&mut self, result_type: u32, operands: [u32; 2]) -> Result<u32, Error> {
		let id = self.id();
		self.inst_list.push(data::Instruction {
			opcode: data::Opcode::ISub,
			result_id: Some(id),
			result_type: Some(result_type),
			operands: operands.into(),
		});
		Ok(id)
	}
	pub fn i_mul(&mut self, result_type: u32, operands: [u32; 2]) -> Result<u32, Error> {
		let id = self.id();
		self.inst_list.push(data::Instruction {
			opcode: data::Opcode::IMul,
			result_id: Some(id),
			result_type: Some(result_type),
			operands: operands.into(),
		});
		Ok(id)
	}
	pub fn s_div(&mut self, result_type: u32, operands: [u32; 2]) -> Result<u32, Error> {
		let id = self.id();
		self.inst_list.push(data::Instruction {
			opcode: data::Opcode::SDiv,
			result_id: Some(id),
			result_type: Some(result_type),
			operands: operands.into(),
		});
		Ok(id)
	}
	pub fn s_mod(&mut self, result_type: u32, operands: [u32; 2]) -> Result<u32, Error> {
		let id = self.id();
		self.inst_list.push(data::Instruction {
			opcode: data::Opcode::SMod,
			result_id: Some(id),
			result_type: Some(result_type),
			operands: operands.into(),
		});
		Ok(id)
	}
	pub fn load(&mut self, result_type: u32, pointer: u32) -> Result<u32, Error> {
		let id = self.id();
        self.inst_list.push(data::Instruction {
            opcode: data::Opcode::Load,
			result_id: Some(id),
			result_type: Some(result_type),
            operands: [pointer].into(),
        });
        Ok(id)
    }
	pub fn store(&mut self, pointer: u32, object: u32) -> Result<(), Error> {
        self.inst_list.push(data::Instruction {
            opcode: data::Opcode::Store,
			result_id: None,
			result_type: None,
            operands: [pointer, object].into(),
        });
        Ok(())
    }
	pub fn build(self) -> data::Module {
		todo!()
	}
}
