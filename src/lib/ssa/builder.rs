// Copyright (c) 2024-2026 Jonathan A. Thomason

use crate::ssa::data::Operand;
use crate::ssa::data::StorageClass;

use super::Error;
use super::data;

macro_rules! return_if_detached {
	($in_func:expr, $instruction:ident) => {
		if !$in_func {
			// type check macro here.
			let instruction: data::Instruction = $instruction;
			return Err(Error::DetachedInstruction(Some(instruction)));
		}
	};
}

#[derive(Debug)]
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
		return_if_detached!(self.in_func, instruction);
		self.func_list.push(instruction);
		Ok(())
	}
	pub fn i_add(&mut self, result_type: u32, lhs: u32, rhs: u32) -> Result<u32, Error> {
		let id = self.id();
		let instruction = data::Instruction {
			opcode: data::Opcode::IAdd,
			result_id: Some(id),
			result_type: Some(result_type),
			operands: [Operand::IdRef(lhs), Operand::IdRef(rhs)].into(),
		};
		return_if_detached!(self.in_func, instruction);
		self.func_list.push(instruction);
		Ok(id)
	}
	pub fn i_sub(&mut self, result_type: u32, lhs: u32, rhs: u32) -> Result<u32, Error> {
		let id = self.id();
		let instruction = data::Instruction {
			opcode: data::Opcode::ISub,
			result_id: Some(id),
			result_type: Some(result_type),
			operands: [Operand::IdRef(lhs), Operand::IdRef(rhs)].into(),
		};
		return_if_detached!(self.in_func, instruction);
		self.decl_list.push(instruction);
		Ok(id)
	}
	pub fn i_mul(&mut self, result_type: u32, lhs: u32, rhs: u32) -> Result<u32, Error> {
		let id = self.id();
		let instruction = data::Instruction {
			opcode: data::Opcode::IMul,
			result_id: Some(id),
			result_type: Some(result_type),
			operands: [Operand::IdRef(lhs), Operand::IdRef(rhs)].into(),
		};
		return_if_detached!(self.in_func, instruction);
		self.func_list.push(instruction);
		Ok(id)
	}
	pub fn s_div(&mut self, result_type: u32, lhs: u32, rhs: u32) -> Result<u32, Error> {
		let id = self.id();
		let instruction = data::Instruction {
			opcode: data::Opcode::SDiv,
			result_id: Some(id),
			result_type: Some(result_type),
			operands: [Operand::IdRef(lhs), Operand::IdRef(rhs)].into(),
		};
		return_if_detached!(self.in_func, instruction);
		self.func_list.push(instruction);
		Ok(id)
	}
	pub fn s_mod(&mut self, result_type: u32, lhs: u32, rhs: u32) -> Result<u32, Error> {
		let id = self.id();
		let instruction = data::Instruction {
			opcode: data::Opcode::SMod,
			result_id: Some(id),
			result_type: Some(result_type),
			operands: [Operand::IdRef(lhs), Operand::IdRef(rhs)].into(),
		};
		return_if_detached!(self.in_func, instruction);
		self.func_list.push(instruction);
		Ok(id)
	}
	pub fn load(&mut self, result_type: u32, pointer: u32) -> Result<u32, Error> {
		let id = self.id();
		let instruction = data::Instruction {
			opcode: data::Opcode::Load,
			result_id: Some(id),
			result_type: Some(result_type),
			operands: [Operand::IdRef(pointer)].into(),
		};
		return_if_detached!(self.in_func, instruction);
		self.func_list.push(instruction);
		Ok(id)
	}
	pub fn store(&mut self, pointer: u32, object: u32) -> Result<(), Error> {
		let instruction = data::Instruction {
			opcode: data::Opcode::Store,
			result_id: None,
			result_type: None,
			operands: [Operand::IdRef(pointer), Operand::IdRef(object)].into(),
		};
		return_if_detached!(self.in_func, instruction);
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
		return_if_detached!(self.in_func, instruction);
		self.func_list.push(instruction);
		Ok(())
	}
	pub fn build(self) -> data::Module {
		data::Module {}
	}
	pub fn type_bool(&mut self) -> u32 {
		let id = self.id();
		self.type_list.push(data::Instruction {
			opcode: data::Opcode::TypeBool,
			result_id: Some(id),
			result_type: None,
			operands: [].into(),
		});
		id
	}
	pub fn type_void(&mut self) -> u32 {
		let id = self.id();
		self.type_list.push(data::Instruction {
			opcode: data::Opcode::TypeVoid,
			result_id: Some(id),
			result_type: None,
			operands: [].into(),
		});
		id
	}
	pub fn type_int(&mut self, width: u32, is_signed: bool) -> u32 {
		let id = self.id();
		self.type_list.push(data::Instruction {
			opcode: data::Opcode::TypeInt,
			result_id: Some(id),
			result_type: None,
			operands: [
				Operand::IdRef(width),
				Operand::LiteralBit32(is_signed as u32),
			]
			.into(),
		});
		id
	}
	pub fn type_float(&mut self, width: u32) -> Result<u32, Error> {
		let id = self.id();
		self.type_list.push(data::Instruction {
			opcode: data::Opcode::TypeFloat,
			result_id: Some(id),
			result_type: None,
			operands: [Operand::IdRef(width)].into(),
		});
		Ok(id)
	}
	pub fn type_array(&mut self, element_type: u32, length: u32) -> u32 {
		let id = self.id();
		self.type_list.push(data::Instruction {
			opcode: data::Opcode::TypeArray,
			result_id: Some(id),
			result_type: None,
			operands: [Operand::IdRef(element_type), Operand::LiteralBit32(length)].into(),
		});
		id
	}
	pub fn type_pointer(&mut self, type_id: u32) -> u32 {
		let id = self.id();
		self.type_list.push(data::Instruction {
			opcode: data::Opcode::TypePointer,
			result_id: Some(id),
			result_type: None,
			operands: [Operand::IdRef(type_id)].into(),
		});
		id
	}
	pub fn type_struct(&mut self, member_types: &[u32]) -> u32 {
		let id = self.id();
		let member_ids: Box<[Operand]> = member_types
			.iter()
			.map(|param| Operand::IdRef(*param))
			.collect();
		self.type_list.push(data::Instruction {
			opcode: data::Opcode::TypeStruct,
			result_id: Some(id),
			result_type: None,
			operands: member_ids.into(),
		});
		id
	}
	pub fn type_function(
		&mut self,
		return_type: u32,
		parameter_types: &[u32],
	) -> Result<u32, Error> {
		let id = self.id();
		let mut operands = vec![Operand::IdRef(return_type)];
		let param_ids: Box<[Operand]> = parameter_types
			.iter()
			.map(|param| Operand::IdRef(*param))
			.collect();
		operands.extend_from_slice(&param_ids);
		self.type_list.push(data::Instruction {
			opcode: data::Opcode::TypeFunction,
			result_id: Some(id),
			result_type: None,
			operands: operands.into(),
		});
		Ok(id)
	}
	pub fn type_variadic_function(
		&mut self,
		return_type: u32,
		parameter_types: &[u32],
	) -> Result<u32, Error> {
		let id = self.id();
		let mut operands = vec![Operand::IdRef(return_type)];
		let param_ids: Box<[Operand]> = parameter_types
			.iter()
			.map(|param| Operand::IdRef(*param))
			.collect();
		operands.extend_from_slice(&param_ids);
		self.type_list.push(data::Instruction {
			opcode: data::Opcode::TypeVariadicFunction,
			result_id: Some(id),
			result_type: None,
			operands: operands.into(),
		});
		Ok(id)
	}
	pub fn variable(
		&mut self,
		result_type: u32,
		storage_class: StorageClass,
		initializer: Option<u32>,
	) -> u32 {
		let id = self.id();
		let mut operands = vec![Operand::StorageClass(storage_class)];
		if let Some(initializer) = initializer {
			operands.push(Operand::IdRef(initializer));
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
		id
	}
	/// function_control:
	/// None = 0
	/// Inline = 1
	/// DontInline = 2
	/// Pure = 4
	/// Const = 8
	pub fn function_begin(
		&mut self,
		result_type: u32,
		function_control: u32,
	) -> Result<u32, Error> {
		let id = self.id();
		let instruction = data::Instruction {
			opcode: data::Opcode::Function,
			result_id: Some(id),
			result_type: Some(result_type),
			operands: [Operand::FunctionControl(function_control)].into(),
		};
		if self.in_func {
			return Err(Error::NestedFunction);
		}
		self.func_list.push(instruction);
		self.in_func = true;
		Ok(id)
	}
	pub fn function_parameter(&mut self, result_type: u32) -> Result<u32, Error> {
		let id = self.id();
		let instruction = data::Instruction {
			opcode: data::Opcode::FunctionParameter,
			result_id: Some(id),
			result_type: Some(result_type),
			operands: [].into(),
		};
		return_if_detached!(self.in_func, instruction);
		self.func_list.push(instruction);
		Ok(id)
	}
	pub fn function_end(&mut self) -> Result<(), Error> {
		let instruction = data::Instruction {
			opcode: data::Opcode::FunctionEnd,
			result_id: None,
			result_type: None,
			operands: [].into(),
		};
		return_if_detached!(self.in_func, instruction);
		self.func_list.push(instruction);
		self.in_func = false;
		Ok(())
	}
	pub fn constant_bit32(&mut self, result_type: u32, value: u32) -> u32 {
		let id = self.id();
		self.type_list.push(data::Instruction {
			opcode: data::Opcode::Constant,
			result_id: Some(id),
			result_type: Some(result_type),
			operands: [Operand::LiteralBit32(value)].into(),
		});
		id
	}
	pub fn assembler(&mut self, text: String) -> u32 {
		let id = self.id();
		self.type_list.push(data::Instruction {
			opcode: data::Opcode::Assembler,
			result_id: Some(id),
			result_type: None,
			operands: [Operand::Text(text)].into(),
		});
		id
	}
}
