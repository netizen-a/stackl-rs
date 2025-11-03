use super::data;
use super::Error;

pub struct Builder {
	type_list: Vec<data::Instruction>,
	decl_list: Vec<data::Instruction>,
	func_list: Vec<data::Instruction>,
	next_id: u32,
	in_func: bool,
}

impl Builder {
	pub fn new() -> Self {
		Self {
			type_list: vec![],
			decl_list: vec![],
			func_list: vec![],
			next_id: 0,
			in_func: false,
		}
	}
	/// Returns the next unused id
	pub fn id(&mut self) -> u32 {
		let result = self.next_id;
		self.next_id += 1;
		result
	}
	pub fn nop(&mut self) -> Result<(), Error> {
		let instruction = data::Instruction {
			opcode: data::Opcode::Nop,
			result_id: None,
			result_type: None,
			operands: [].into(),
		};
		if !self.in_func {
			return Err(Error::DetachedInstruction(Some(instruction)))
		}
		self.func_list.push(instruction);
		Ok(())
	}
	pub fn i_add(&mut self, result_type: u32, operands: [u32; 2]) -> Result<u32, Error> {
		let id = self.id();
		let instruction = data::Instruction {
			opcode: data::Opcode::IAdd,
			result_id: Some(id),
			result_type: Some(result_type),
			operands: operands.into(),
		};
		if !self.in_func {
			return Err(Error::DetachedInstruction(Some(instruction)))
		}
		self.func_list.push(instruction);
		Ok(id)
	}
	pub fn i_sub(&mut self, result_type: u32, operands: [u32; 2]) -> Result<u32, Error> {
		let id = self.id();
		let instruction = data::Instruction {
			opcode: data::Opcode::ISub,
			result_id: Some(id),
			result_type: Some(result_type),
			operands: operands.into(),
		};
		if !self.in_func {
			return Err(Error::DetachedInstruction(Some(instruction)))
		}
		self.decl_list.push(instruction);
		Ok(id)
	}
	pub fn i_mul(&mut self, result_type: u32, operands: [u32; 2]) -> Result<u32, Error> {
		let id = self.id();
		let instruction = data::Instruction {
			opcode: data::Opcode::IMul,
			result_id: Some(id),
			result_type: Some(result_type),
			operands: operands.into(),
		};
		if !self.in_func {
			return Err(Error::DetachedInstruction(Some(instruction)))
		}
		self.func_list.push(instruction);
		Ok(id)
	}
	pub fn s_div(&mut self, result_type: u32, operands: [u32; 2]) -> Result<u32, Error> {
		let id = self.id();
		let instruction = data::Instruction {
			opcode: data::Opcode::SDiv,
			result_id: Some(id),
			result_type: Some(result_type),
			operands: operands.into(),
		};
		if !self.in_func {
			return Err(Error::DetachedInstruction(Some(instruction)))
		}
		self.func_list.push(instruction);
		Ok(id)
	}
	pub fn s_mod(&mut self, result_type: u32, operands: [u32; 2]) -> Result<u32, Error> {
		let id = self.id();
		let instruction = data::Instruction {
			opcode: data::Opcode::SMod,
			result_id: Some(id),
			result_type: Some(result_type),
			operands: operands.into(),
		};
		if !self.in_func {
			return Err(Error::DetachedInstruction(Some(instruction)))
		}
		self.func_list.push(instruction);
		Ok(id)
	}
	pub fn load(&mut self, result_type: u32, pointer: u32) -> Result<u32, Error> {
		let id = self.id();
		let instruction = data::Instruction {
			opcode: data::Opcode::Load,
			result_id: Some(id),
			result_type: Some(result_type),
			operands: [pointer].into(),
		};
		if !self.in_func {
			return Err(Error::DetachedInstruction(Some(instruction)))
		}
		self.func_list.push(instruction);
		Ok(id)
	}
	pub fn store(&mut self, pointer: u32, object: u32) -> Result<(), Error> {
		let instruction = data::Instruction {
			opcode: data::Opcode::Store,
			result_id: None,
			result_type: None,
			operands: [pointer, object].into(),
		};
		if !self.in_func {
			return Err(Error::DetachedInstruction(Some(instruction)))
		}
		self.func_list.push(instruction);
		Ok(())
	}
	pub fn ret(&mut self) -> Result<(), Error> {
		let instruction = data::Instruction {
			opcode: data::Opcode::Ret,
			result_id: None,
			result_type: None,
			operands: [].into(),
		};
		if !self.in_func {
			return Err(Error::DetachedInstruction(Some(instruction)))
		}
		self.func_list.push(instruction);
		Ok(())
	}
	pub fn ret_value(&mut self, operand: u32) -> Result<(), Error> {
		let instruction = data::Instruction {
			opcode: data::Opcode::RetValue,
			result_id: None,
			result_type: None,
			operands: [operand].into(),
		};
		if !self.in_func {
			return Err(Error::DetachedInstruction(Some(instruction)))
		}
		self.func_list.push(instruction);
		Ok(())
	}
	pub fn build(self) -> data::Module {
		todo!()
	}
	pub fn type_int(&mut self, width: u32, is_signed: bool) -> Result<u32, Error> {
		let id = self.id();
		self.type_list.push(data::Instruction {
			opcode: data::Opcode::TypeInt,
			result_id: Some(id),
			result_type: None,
			operands: [width, is_signed as u32].into(),
		});
		Ok(id)
	}
	pub fn type_float(&mut self, width: u32) -> Result<u32, Error> {
		let id = self.id();
		self.type_list.push(data::Instruction {
			opcode: data::Opcode::TypeFloat,
			result_id: Some(id),
			result_type: None,
			operands: [width].into(),
		});
		Ok(id)
	}
	pub fn type_array(&mut self, element_type: u32, length: u32) -> Result<u32, Error> {
		let id = self.id();
		self.type_list.push(data::Instruction {
			opcode: data::Opcode::TypeArray,
			result_id: Some(id),
			result_type: None,
			operands: [element_type, length].into(),
		});
		Ok(id)
	}
	pub fn type_pointer(&mut self, storage_class: u32, type_id: u32) -> Result<u32, Error> {
		let id = self.id();
		self.type_list.push(data::Instruction {
			opcode: data::Opcode::TypePointer,
			result_id: Some(id),
			result_type: None,
			operands: [storage_class, type_id].into(),
		});
		Ok(id)
	}
	pub fn variable(&mut self, result_type: u32, storage_class: u32, initializer: Option<u32>) -> Result<u32, Error> {
		let id = self.id();
		let mut operands = vec![storage_class];
		if let Some(initializer) = initializer {
			operands.push(initializer);
		}
		let instruction = data::Instruction {
			opcode: data::Opcode::Variable,
			result_id: Some(id),
			result_type: Some(result_type),
			operands: operands.into_boxed_slice(),
		};
		match self.in_func {
			true => self.func_list.push(instruction),
			false => self.decl_list.push(instruction),
		}
		Ok(id)
	}
}
