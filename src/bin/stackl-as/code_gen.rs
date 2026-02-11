// Copyright (c) 2024-2026 Jonathan A. Thomason

use std::{
	collections::HashMap,
	error::Error,
};

use crate::sym;
use stackl::{
	StacklFlags,
	StacklFormatV2,
	asm::ast::*,
	asm::op,
};

pub fn ast_to_fmt2(ast: Vec<Stmt>) -> Result<StacklFormatV2, Box<dyn Error>> {
	let symtab: HashMap<String, usize> = sym::build_symtab(&ast).unwrap();
	let mut text = vec![0u8; 8];
	let mut is_start_global = false;
	let mut int_vec: i32 = -1;
	let mut trap_vec: i32 = -1;
	let mut flags = StacklFlags::empty();
	for stmt in ast {
		let data: Vec<u8> = match stmt.inst {
			Inst::Mnemonic(op) => convert_op(&op, &symtab),
			Inst::DataDecl8(list) => {
				let mut data_list = Vec::<u8>::new();
				for data in list {
					let vec: Vec<u8> = match data {
						// convert i32 to u8
						Atom::Int(value) => vec![value.try_into().unwrap()],
						// convert String to [u8]
						Atom::String(s) => s.as_bytes().to_vec(),
						_ => unimplemented!(),
					};
					data_list.extend(vec);
				}
				data_list
			}
			Inst::DataDecl32(list) => {
				let mut data_list = Vec::<u8>::new();
				for data in list {
					let vec: Vec<u8> = match data {
						// convert i32 to [u8]
						Atom::Int(value) => value.to_le_bytes().to_vec(),
						// convert String to [u8]
						Atom::String(s) => {
							let mut bytes = s.as_bytes().to_vec();
							if bytes.len().is_multiple_of(4) {
								bytes
							} else {
								let len = 4 - (bytes.len() % 4);
								bytes.extend(vec![0; len]);
								bytes
							}
						}
						Atom::Label(label) => (symtab[&label] as u32).to_le_bytes().to_vec(),
					};
					data_list.extend(vec);
				}
				data_list
			}
			Inst::Directive(Directive::Extern, _) => {
				panic!("binary does not support extern directive")
			}
			Inst::Directive(Directive::Segment, _) => {
				if !text.len().is_multiple_of(4) {
					vec![0; 4 - (text.len() % 4)]
				} else {
					vec![]
				}
			}
			Inst::Directive(Directive::Global, sym) => {
				if !is_start_global {
					is_start_global = sym.contains(&"_start".to_string());
				}
				vec![]
			}
			Inst::Directive(Directive::Interrupt, sym) => {
				if sym.len() != 1 {
					panic!("invalid directive args");
				}
				int_vec = symtab[&sym[0]].try_into().unwrap();
				vec![]
			}
			Inst::Directive(Directive::Systrap, sym) => {
				if sym.len() != 1 {
					panic!("invalid directive args");
				}
				trap_vec = symtab[&sym[0]].try_into().unwrap();
				vec![]
			}
			Inst::Directive(Directive::Feature, symbols) => {
				for sym in symbols {
					match sym.to_ascii_lowercase().as_str() {
						"pio_term" => flags.set(StacklFlags::FEATURE_PIO_TERM, true),
						"dma_term" => flags.set(StacklFlags::FEATURE_DMA_TERM, true),
						"disk" => flags.set(StacklFlags::FEATURE_DISK, true),
						"inp" => flags.set(StacklFlags::FEATURE_INP, true),
						_ => panic!("invalid feature argument"),
					}
				}
				vec![]
			}
			_ => unimplemented!(),
		};
		if !data.is_empty() {
			text.extend(data);
		}
	}

	if !is_start_global {
		panic!("Symbol _start not global");
	}

	text[0..4].copy_from_slice(&int_vec.to_le_bytes());
	text[4..8].copy_from_slice(&trap_vec.to_le_bytes());

	Ok(StacklFormatV2 {
		magic: [b's', b'l', 0, 0],
		version: stackl::Version::new(1, 1, 0, 0),
		flags,
		stack_size: 1000,
		text,
	})
}

fn convert_op(op: &Opcode, symtab: &HashMap<String, usize>) -> Vec<u8> {
	let text: Vec<i32> = match op {
		Opcode::Nop => vec![op::NOP],
		Opcode::Add => vec![op::ADD],
		Opcode::Sub => vec![op::SUB],
		Opcode::Mul => vec![op::MUL],
		Opcode::Div => vec![op::DIV],
		Opcode::Mod => vec![op::MOD],
		Opcode::Eq => vec![op::EQ],
		Opcode::Ne => vec![op::NE],
		Opcode::Gt => vec![op::GT],
		Opcode::Lt => vec![op::LT],
		Opcode::Ge => vec![op::GE],
		Opcode::Le => vec![op::LE],
		Opcode::And => vec![op::AND],
		Opcode::Or => vec![op::OR],
		Opcode::Not => vec![op::NOT],
		Opcode::Swap => vec![op::SWAP],
		Opcode::Dup => vec![op::DUP],
		Opcode::Halt => vec![op::HALT],
		Opcode::Pop => vec![op::POP],
		Opcode::Ret => vec![op::RET],
		Opcode::Retv => vec![op::RETV],
		Opcode::Neg => vec![op::NEG],
		Opcode::PushCVarInd => vec![op::PUSHCVARIND],
		Opcode::Outs => vec![op::OUTS],
		Opcode::Inp => vec![op::INP],
		Opcode::PushFP => vec![op::PUSHFP],
		Opcode::JmpUser(operand) => match operand {
			&Operand::Int(offset) => vec![op::JMPUSER, offset as _],
			Operand::Label(label) => vec![op::JMPUSER, symtab[label].try_into().unwrap()],
		},
		Opcode::Trap => vec![op::TRAP],
		Opcode::Rti => vec![op::RTI],
		Opcode::Calli => vec![op::CALLI],
		Opcode::PushReg(reg) => vec![op::PUSHREG, *reg as _],
		Opcode::PopReg(reg) => vec![op::POPREG, *reg as _],
		Opcode::BAnd => vec![op::BAND],
		Opcode::BOr => vec![op::BOR],
		Opcode::BXOr => vec![op::BXOR],
		Opcode::ShiftLeft => vec![op::SHIFT_LEFT],
		Opcode::ShiftRight => vec![op::SHIFT_RIGHT],
		Opcode::PushVarInd => vec![op::PUSHVARIND],
		Opcode::PopCVarInd => vec![op::POPCVARIND],
		Opcode::PopVarInd => vec![op::POPVARIND],
		Opcode::Comp => vec![op::COMP],
		Opcode::Push(operand) => match operand {
			&Operand::Int(value) => vec![op::PUSH, value as _],
			Operand::Label(label) => vec![op::PUSH, symtab[label].try_into().unwrap()],
		},
		Opcode::Jmp(operand) => match operand {
			&Operand::Int(value) => vec![op::JMP, value as _],
			Operand::Label(label) => vec![op::JMP, symtab[label].try_into().unwrap()],
		},
		Opcode::Jz(operand) => match operand {
			&Operand::Int(value) => vec![op::JZ, value as _],
			Operand::Label(label) => vec![op::JZ, symtab[label].try_into().unwrap()],
		},
		Opcode::PushVar(operand) => match operand {
			&Operand::Int(value) => vec![op::PUSHVAR, value as _],
			Operand::Label(label) => vec![op::PUSHVAR, symtab[label].try_into().unwrap()],
		},
		Opcode::PopVar(operand) => match operand {
			&Operand::Int(value) => vec![op::POPVAR, value as _],
			Operand::Label(label) => vec![op::POPVAR, symtab[label].try_into().unwrap()],
		},
		Opcode::AdjSP(operand) => match operand {
			&Operand::Int(value) => vec![op::ADJSP, value as _],
			Operand::Label(label) => vec![op::ADJSP, symtab[label].try_into().unwrap()],
		},
		Opcode::PopArgs(operand) => match operand {
			&Operand::Int(value) => vec![op::POPARGS, value as _],
			Operand::Label(label) => vec![op::POPARGS, symtab[label].try_into().unwrap()],
		},
		Opcode::Call(addr) => match addr {
			&Operand::Int(value) => vec![op::CALL, value as _],
			Operand::Label(label) => vec![op::CALL, symtab[label].try_into().unwrap()],
		},
		Opcode::PushCVar(operand) => match operand {
			&Operand::Int(value) => vec![op::PUSHCVAR, value as _],
			Operand::Label(label) => vec![op::PUSHCVAR, symtab[label].try_into().unwrap()],
		},
		Opcode::PopCVar(operand) => match operand {
			&Operand::Int(value) => vec![op::POPCVAR, value as _],
			Operand::Label(label) => vec![op::POPCVAR, symtab[label].try_into().unwrap()],
		},
		Opcode::SetTrace => vec![op::SET_TRACE],
		Opcode::ClrTrace => vec![op::CLR_TRACE],
		Opcode::ClrIntDis => vec![op::CLR_INT_DIS],
		Opcode::SetIntDis => vec![op::SET_INT_DIS],
		Opcode::RotateLeft => vec![op::ROTATE_LEFT],
		Opcode::RotateRight => vec![op::ROTATE_RIGHT],
		Opcode::Illegal => vec![op::ILLEGAL],
		_ => unimplemented!(),
	};

	let mut ret = Vec::new();
	for word in text {
		ret.extend_from_slice(&word.to_le_bytes());
	}
	ret
}
