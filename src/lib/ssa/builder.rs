// Copyright (c) 2024-2026 Jonathan A. Thomason

use std::collections::HashMap;

use crate::ssa::data::Operand;
use crate::ssa::data::StorageClass;

use super::Error;
use super::data;

macro_rules! return_if_detached {
	($in_func:expr, $instruction:ident) => {
		if !$in_func {
			// type check macro here.
			let instruction: data::Instruction = $instruction;
			return Err(Error::DetachedInstruction(instruction));
		}
	};
}

#[derive(Debug)]
pub struct Builder {
	type_list: Vec<data::Instruction>,
	sections: HashMap<String, Vec<data::DataKind>>,
	next_id: u32,
	in_func: bool,
	// default to .code and .data until explicitly mentioned
	curr_section: Option<String>,
}

impl Builder {
	pub fn new() -> Self {
		let mut sections = HashMap::new();
		sections.insert(".code".to_owned(), vec![]);
		sections.insert(".data".to_owned(), vec![]);
		Self {
			type_list: vec![],
			sections,
			next_id: 0,
			in_func: false,
			curr_section: None,
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
		match self
			.curr_section
			.as_ref()
			.and_then(|section| self.sections.get_mut(section))
		{
			Some(section) => {
				let data::DataKind::Func(func) = section.last_mut().unwrap() else {
					return Err(Error::DetachedInstruction(instruction));
				};
				func.body.push(instruction);
			}
			None => {
				let section = self.sections.get_mut(".code").unwrap();
				let data::DataKind::Func(func) = section.last_mut().unwrap() else {
					return Err(Error::DetachedInstruction(instruction));
				};
				func.body.push(instruction);
			}
		}
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
		match self
			.curr_section
			.as_ref()
			.and_then(|section| self.sections.get_mut(section))
		{
			Some(section) => {
				let data::DataKind::Func(func) = section.last_mut().unwrap() else {
					return Err(Error::DetachedInstruction(instruction));
				};
				func.body.push(instruction);
			}
			None => {
				let section = self.sections.get_mut(".code").unwrap();
				let data::DataKind::Func(func) = section.last_mut().unwrap() else {
					return Err(Error::DetachedInstruction(instruction));
				};
				func.body.push(instruction);
			}
		}
		Ok(id)
	}
	pub fn f_add(&mut self, result_type: u32, lhs: u32, rhs: u32) -> Result<u32, Error> {
		let id = self.id();
		let instruction = data::Instruction {
			opcode: data::Opcode::FAdd,
			result_id: Some(id),
			result_type: Some(result_type),
			operands: [Operand::IdRef(lhs), Operand::IdRef(rhs)].into(),
		};
		return_if_detached!(self.in_func, instruction);
		match self
			.curr_section
			.as_ref()
			.and_then(|section| self.sections.get_mut(section))
		{
			Some(section) => {
				let data::DataKind::Func(func) = section.last_mut().unwrap() else {
					return Err(Error::DetachedInstruction(instruction));
				};
				func.body.push(instruction);
			}
			None => {
				let section = self.sections.get_mut(".code").unwrap();
				let data::DataKind::Func(func) = section.last_mut().unwrap() else {
					return Err(Error::DetachedInstruction(instruction));
				};
				func.body.push(instruction);
			}
		}
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
		match self
			.curr_section
			.as_ref()
			.and_then(|section| self.sections.get_mut(section))
		{
			Some(section) => {
				let data::DataKind::Func(func) = section.last_mut().unwrap() else {
					return Err(Error::DetachedInstruction(instruction));
				};
				func.body.push(instruction);
			}
			None => {
				let section = self.sections.get_mut(".code").unwrap();
				let data::DataKind::Func(func) = section.last_mut().unwrap() else {
					return Err(Error::DetachedInstruction(instruction));
				};
				func.body.push(instruction);
			}
		}
		Ok(id)
	}
	pub fn f_sub(&mut self, result_type: u32, lhs: u32, rhs: u32) -> Result<u32, Error> {
		let id = self.id();
		let instruction = data::Instruction {
			opcode: data::Opcode::FSub,
			result_id: Some(id),
			result_type: Some(result_type),
			operands: [Operand::IdRef(lhs), Operand::IdRef(rhs)].into(),
		};
		return_if_detached!(self.in_func, instruction);
		match self
			.curr_section
			.as_ref()
			.and_then(|section| self.sections.get_mut(section))
		{
			Some(section) => {
				let data::DataKind::Func(func) = section.last_mut().unwrap() else {
					return Err(Error::DetachedInstruction(instruction));
				};
				func.body.push(instruction);
			}
			None => {
				let section = self.sections.get_mut(".code").unwrap();
				let data::DataKind::Func(func) = section.last_mut().unwrap() else {
					return Err(Error::DetachedInstruction(instruction));
				};
				func.body.push(instruction);
			}
		}
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
		match self
			.curr_section
			.as_ref()
			.and_then(|section| self.sections.get_mut(section))
		{
			Some(section) => {
				let data::DataKind::Func(func) = section.last_mut().unwrap() else {
					return Err(Error::DetachedInstruction(instruction));
				};
				func.body.push(instruction);
			}
			None => {
				let section = self.sections.get_mut(".code").unwrap();
				let data::DataKind::Func(func) = section.last_mut().unwrap() else {
					return Err(Error::DetachedInstruction(instruction));
				};
				func.body.push(instruction);
			}
		}
		Ok(id)
	}
	pub fn f_mul(&mut self, result_type: u32, lhs: u32, rhs: u32) -> Result<u32, Error> {
		let id = self.id();
		let instruction = data::Instruction {
			opcode: data::Opcode::FMul,
			result_id: Some(id),
			result_type: Some(result_type),
			operands: [Operand::IdRef(lhs), Operand::IdRef(rhs)].into(),
		};
		return_if_detached!(self.in_func, instruction);
		match self
			.curr_section
			.as_ref()
			.and_then(|section| self.sections.get_mut(section))
		{
			Some(section) => {
				let data::DataKind::Func(func) = section.last_mut().unwrap() else {
					return Err(Error::DetachedInstruction(instruction));
				};
				func.body.push(instruction);
			}
			None => {
				let section = self.sections.get_mut(".code").unwrap();
				let data::DataKind::Func(func) = section.last_mut().unwrap() else {
					return Err(Error::DetachedInstruction(instruction));
				};
				func.body.push(instruction);
			}
		}
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
		match self
			.curr_section
			.as_ref()
			.and_then(|section| self.sections.get_mut(section))
		{
			Some(section) => {
				let data::DataKind::Func(func) = section.last_mut().unwrap() else {
					return Err(Error::DetachedInstruction(instruction));
				};
				func.body.push(instruction);
			}
			None => {
				let section = self.sections.get_mut(".code").unwrap();
				let data::DataKind::Func(func) = section.last_mut().unwrap() else {
					return Err(Error::DetachedInstruction(instruction));
				};
				func.body.push(instruction);
			}
		}
		Ok(id)
	}
	pub fn u_div(&mut self, result_type: u32, lhs: u32, rhs: u32) -> Result<u32, Error> {
		let id = self.id();
		let instruction = data::Instruction {
			opcode: data::Opcode::UDiv,
			result_id: Some(id),
			result_type: Some(result_type),
			operands: [Operand::IdRef(lhs), Operand::IdRef(rhs)].into(),
		};
		return_if_detached!(self.in_func, instruction);
		match self
			.curr_section
			.as_ref()
			.and_then(|section| self.sections.get_mut(section))
		{
			Some(section) => {
				let data::DataKind::Func(func) = section.last_mut().unwrap() else {
					return Err(Error::DetachedInstruction(instruction));
				};
				func.body.push(instruction);
			}
			None => {
				let section = self.sections.get_mut(".code").unwrap();
				let data::DataKind::Func(func) = section.last_mut().unwrap() else {
					return Err(Error::DetachedInstruction(instruction));
				};
				func.body.push(instruction);
			}
		}
		Ok(id)
	}
	pub fn f_div(&mut self, result_type: u32, lhs: u32, rhs: u32) -> Result<u32, Error> {
		let id = self.id();
		let instruction = data::Instruction {
			opcode: data::Opcode::FDiv,
			result_id: Some(id),
			result_type: Some(result_type),
			operands: [Operand::IdRef(lhs), Operand::IdRef(rhs)].into(),
		};
		return_if_detached!(self.in_func, instruction);
		match self
			.curr_section
			.as_ref()
			.and_then(|section| self.sections.get_mut(section))
		{
			Some(section) => {
				let data::DataKind::Func(func) = section.last_mut().unwrap() else {
					return Err(Error::DetachedInstruction(instruction));
				};
				func.body.push(instruction);
			}
			None => {
				let section = self.sections.get_mut(".code").unwrap();
				let data::DataKind::Func(func) = section.last_mut().unwrap() else {
					return Err(Error::DetachedInstruction(instruction));
				};
				func.body.push(instruction);
			}
		}
		Ok(id)
	}
	pub fn s_rem(&mut self, result_type: u32, lhs: u32, rhs: u32) -> Result<u32, Error> {
		let id = self.id();
		let instruction = data::Instruction {
			opcode: data::Opcode::SRem,
			result_id: Some(id),
			result_type: Some(result_type),
			operands: [Operand::IdRef(lhs), Operand::IdRef(rhs)].into(),
		};
		return_if_detached!(self.in_func, instruction);
		match self
			.curr_section
			.as_ref()
			.and_then(|section| self.sections.get_mut(section))
		{
			Some(section) => {
				let data::DataKind::Func(func) = section.last_mut().unwrap() else {
					return Err(Error::DetachedInstruction(instruction));
				};
				func.body.push(instruction);
			}
			None => {
				let section = self.sections.get_mut(".code").unwrap();
				let data::DataKind::Func(func) = section.last_mut().unwrap() else {
					return Err(Error::DetachedInstruction(instruction));
				};
				func.body.push(instruction);
			}
		}
		Ok(id)
	}
	pub fn u_rem(&mut self, result_type: u32, lhs: u32, rhs: u32) -> Result<u32, Error> {
		let id = self.id();
		let instruction = data::Instruction {
			opcode: data::Opcode::URem,
			result_id: Some(id),
			result_type: Some(result_type),
			operands: [Operand::IdRef(lhs), Operand::IdRef(rhs)].into(),
		};
		return_if_detached!(self.in_func, instruction);
		match self
			.curr_section
			.as_ref()
			.and_then(|section| self.sections.get_mut(section))
		{
			Some(section) => {
				let data::DataKind::Func(func) = section.last_mut().unwrap() else {
					return Err(Error::DetachedInstruction(instruction));
				};
				func.body.push(instruction);
			}
			None => {
				let section = self.sections.get_mut(".code").unwrap();
				let data::DataKind::Func(func) = section.last_mut().unwrap() else {
					return Err(Error::DetachedInstruction(instruction));
				};
				func.body.push(instruction);
			}
		}
		Ok(id)
	}
	pub fn f_rem(&mut self, result_type: u32, lhs: u32, rhs: u32) -> Result<u32, Error> {
		let id = self.id();
		let instruction = data::Instruction {
			opcode: data::Opcode::FRem,
			result_id: Some(id),
			result_type: Some(result_type),
			operands: [Operand::IdRef(lhs), Operand::IdRef(rhs)].into(),
		};
		return_if_detached!(self.in_func, instruction);
		match self
			.curr_section
			.as_ref()
			.and_then(|section| self.sections.get_mut(section))
		{
			Some(section) => {
				let data::DataKind::Func(func) = section.last_mut().unwrap() else {
					return Err(Error::DetachedInstruction(instruction));
				};
				func.body.push(instruction);
			}
			None => {
				let section = self.sections.get_mut(".code").unwrap();
				let data::DataKind::Func(func) = section.last_mut().unwrap() else {
					return Err(Error::DetachedInstruction(instruction));
				};
				func.body.push(instruction);
			}
		}
		Ok(id)
	}
	pub fn label(&mut self) -> Result<u32, Error> {
		let id = self.id();
		let instruction = data::Instruction {
			opcode: data::Opcode::Label,
			result_id: Some(id),
			result_type: None,
			operands: [].into(),
		};
		return_if_detached!(self.in_func, instruction);
		match self
			.curr_section
			.as_ref()
			.and_then(|section| self.sections.get_mut(section))
		{
			Some(section) => {
				let data::DataKind::Func(func) = section.last_mut().unwrap() else {
					return Err(Error::DetachedInstruction(instruction));
				};
				func.body.push(instruction);
			}
			None => {
				let section = self.sections.get_mut(".code").unwrap();
				let data::DataKind::Func(func) = section.last_mut().unwrap() else {
					return Err(Error::DetachedInstruction(instruction));
				};
				func.body.push(instruction);
			}
		}
		Ok(id)
	}
	pub fn branch(&mut self, target_label: u32) -> Result<u32, Error> {
		let id = self.id();
		let instruction = data::Instruction {
			opcode: data::Opcode::Branch,
			result_id: Some(id),
			result_type: None,
			operands: [Operand::IdRef(target_label)].into(),
		};
		return_if_detached!(self.in_func, instruction);
		match self
			.curr_section
			.as_ref()
			.and_then(|section| self.sections.get_mut(section))
		{
			Some(section) => {
				let data::DataKind::Func(func) = section.last_mut().unwrap() else {
					return Err(Error::DetachedInstruction(instruction));
				};
				func.body.push(instruction);
			}
			None => {
				let section = self.sections.get_mut(".code").unwrap();
				let data::DataKind::Func(func) = section.last_mut().unwrap() else {
					return Err(Error::DetachedInstruction(instruction));
				};
				func.body.push(instruction);
			}
		}
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
		match self
			.curr_section
			.as_ref()
			.and_then(|section| self.sections.get_mut(section))
		{
			Some(section) => {
				let data::DataKind::Func(func) = section.last_mut().unwrap() else {
					return Err(Error::DetachedInstruction(instruction));
				};
				func.body.push(instruction);
			}
			None => {
				let section = self.sections.get_mut(".code").unwrap();
				let data::DataKind::Func(func) = section.last_mut().unwrap() else {
					return Err(Error::DetachedInstruction(instruction));
				};
				func.body.push(instruction);
			}
		}
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
		match self
			.curr_section
			.as_ref()
			.and_then(|section| self.sections.get_mut(section))
		{
			Some(section) => {
				let data::DataKind::Func(func) = section.last_mut().unwrap() else {
					return Err(Error::DetachedInstruction(instruction));
				};
				func.body.push(instruction);
			}
			None => {
				let section = self.sections.get_mut(".code").unwrap();
				let data::DataKind::Func(func) = section.last_mut().unwrap() else {
					return Err(Error::DetachedInstruction(instruction));
				};
				func.body.push(instruction);
			}
		}
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
		match self
			.curr_section
			.as_ref()
			.and_then(|section| self.sections.get_mut(section))
		{
			Some(section) => {
				let data::DataKind::Func(func) = section.last_mut().unwrap() else {
					return Err(Error::DetachedInstruction(instruction));
				};
				func.body.push(instruction);
			}
			None => {
				let section = self.sections.get_mut(".code").unwrap();
				let data::DataKind::Func(func) = section.last_mut().unwrap() else {
					return Err(Error::DetachedInstruction(instruction));
				};
				func.body.push(instruction);
			}
		}
		Ok(())
	}
	pub fn build(self) -> data::Module {
		data::Module {
			type_list: self.type_list.into_boxed_slice(),
			sections: self.sections,
		}
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
	) -> Result<u32, Error> {
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
		match self
			.curr_section
			.as_ref()
			.and_then(|section| self.sections.get_mut(section))
		{
			Some(section) => match section.last_mut().unwrap() {
				data::DataKind::Func(func) => {
					func.body.push(instruction);
				}
				_ => section.push(data::DataKind::Data(instruction)),
			},
			None => {
				let section = self.sections.get_mut(".data").unwrap();
				match section.last_mut() {
					Some(data::DataKind::Func(func)) => {
						func.body.push(instruction);
					}
					_ => section.push(data::DataKind::Data(instruction)),
				}
			}
		}
		Ok(id)
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
		match self
			.curr_section
			.as_ref()
			.and_then(|section| self.sections.get_mut(section))
		{
			Some(section) => {
				section.push(data::DataKind::Func(data::Function::new(instruction)));
			}
			None => {
				let section = self.sections.get_mut(".code").unwrap();
				section.push(data::DataKind::Func(data::Function::new(instruction)));
			}
		}
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
		match self
			.curr_section
			.as_ref()
			.and_then(|section| self.sections.get_mut(section))
		{
			Some(section) => {
				let data::DataKind::Func(func) = section.last_mut().unwrap() else {
					return Err(Error::DetachedInstruction(instruction));
				};
				func.body.push(instruction);
			}
			None => {
				let section = self.sections.get_mut(".code").unwrap();
				let data::DataKind::Func(func) = section.last_mut().unwrap() else {
					return Err(Error::DetachedInstruction(instruction));
				};
				func.body.push(instruction);
			}
		}
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
		match self
			.curr_section
			.as_ref()
			.and_then(|section| self.sections.get_mut(section))
		{
			Some(section) => {
				let data::DataKind::Func(func) = section.last_mut().unwrap() else {
					return Err(Error::DetachedInstruction(instruction));
				};
				func.end = Some(instruction);
			}
			None => {
				let section = self.sections.get_mut(".code").unwrap();
				let data::DataKind::Func(func) = section.last_mut().unwrap() else {
					return Err(Error::DetachedInstruction(instruction));
				};
				func.end = Some(instruction);
			}
		}
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
