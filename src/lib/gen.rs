use std::{collections::HashMap, error::Error};

use crate::{ast::*, sym, StacklFormat};

impl TryFrom<Vec<Stmt>> for StacklFormat {
    type Error = Box<dyn Error>;
    fn try_from(ast: Vec<Stmt>) -> Result<crate::StacklFormat, Self::Error> {
        let symtab: HashMap<String, usize> = sym::build_symtab(&ast).unwrap();
        let mut text = Vec::<u8>::new();
        let mut is_start_global = false;
        let mut int_vec = -1;
        let mut trap_vec = -1;
        for stmt in ast {
            let data: Vec<u8> = match stmt.inst {
                Inst::Mnemonic(op) => convert_op(&op, &symtab),
                Inst::DataDecl8(list) => {
                    let mut data_list = Vec::<u8>::new();
                    for data in list {
                        let vec: Vec<u8> = match data {
                            // convert i32 to u8
                            Value::Int(value) => vec![value.try_into().unwrap()],
                            // convert String to [u8]
                            Value::String(s) => s.as_bytes().to_vec(),
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
                            Value::Int(value) => Vec::from(value.to_le_bytes()),
                            // convert String to [u8]
                            Value::String(s) => {
                                let mut bytes = s.as_bytes().to_vec();
                                if bytes.len() % 4 == 0 {
                                    bytes
                                } else {
                                    let len = 4 - (bytes.len() % 4);
                                    bytes.extend(vec![0; len]);
                                    bytes
                                }
                            }
                            _ => unimplemented!(),
                        };
                        data_list.extend(vec);
                    }
                    data_list
                }
                Inst::Directive(Directive::Extern, _) => {
                    panic!("binary does not support extern directive")
                }
                Inst::Directive(Directive::Segment, _) => {
                    if text.len() % 4 != 0 {
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
            };
            if !data.is_empty() {
                text.extend(data);
            }
        }

        if !is_start_global {
            panic!("Symbol _start not global");
        }

        Ok(crate::StacklFormat {
            magic: [b's', b'l', 0, 0],
            version: 0,
            flags: 0,
            int_vec,
            trap_vec,
            text,
        })
    }
}

fn convert_op(op: &Opcode, symtab: &HashMap<String, usize>) -> Vec<u8> {
    let text: Vec<u32> = match op {
        Opcode::Nop => vec![0],
        Opcode::Add => vec![1],
        Opcode::Sub => vec![2],
        Opcode::Mul => vec![3],
        Opcode::Div => vec![4],
        Opcode::Mod => vec![5],
        Opcode::Eq => vec![6],
        Opcode::Ne => vec![7],
        Opcode::Gt => vec![8],
        Opcode::Lt => vec![9],
        Opcode::Ge => vec![10],
        Opcode::Le => vec![11],
        Opcode::And => vec![12],
        Opcode::Or => vec![13],
        Opcode::Not => vec![14],
        Opcode::Swap => vec![15],
        Opcode::Dup => vec![16],
        Opcode::Halt => vec![17],
        Opcode::Pop => vec![18],
        Opcode::Return => vec![19],
        Opcode::Returnv => vec![20],
        Opcode::Neg => vec![21],
        Opcode::PushCVarInd => vec![22],
        Opcode::Outs => vec![23],
        Opcode::Inp => vec![24],
        Opcode::PushFP => vec![25],
        Opcode::JmpUser(operand) => match operand {
            &Operand::Int(offset) => vec![26, offset as _],
            Operand::Label(label) => vec![26, symtab[label].try_into().unwrap()],
        },
        Opcode::Trap => vec![27],
        Opcode::Rti => vec![28],
        Opcode::Calli => vec![29],
        Opcode::PushReg(reg) => vec![30, *reg as _],
        Opcode::PopReg(reg) => vec![31, *reg as _],
        Opcode::BAnd => vec![32],
        Opcode::BOr => vec![33],
        Opcode::BXOr => vec![34],
        Opcode::Shiftl => vec![35],
        Opcode::Shiftr => vec![36],
        Opcode::PushVarInd => vec![37],
        Opcode::PopCVarInd => vec![38],
        Opcode::PopVarInd => vec![39],
        Opcode::Comp => vec![40],
        Opcode::Push(operand) => match operand {
            &Operand::Int(value) => vec![41, value as _],
            Operand::Label(label) => vec![41, symtab[label].try_into().unwrap()],
        },
        Opcode::Jmp(operand) => match operand {
            &Operand::Int(value) => vec![42, value as _],
            Operand::Label(label) => vec![42, symtab[label].try_into().unwrap()],
        },
        Opcode::Jz(operand) => match operand {
            &Operand::Int(value) => vec![43, value as _],
            Operand::Label(label) => vec![43, symtab[label].try_into().unwrap()],
        },
        Opcode::PushVar(operand) => match operand {
            &Operand::Int(value) => vec![44, value as _],
            Operand::Label(label) => vec![44, symtab[label].try_into().unwrap()],
        },
        Opcode::PopVar(operand) => match operand {
            &Operand::Int(value) => vec![45, value as _],
            Operand::Label(label) => vec![45, symtab[label].try_into().unwrap()],
        },
        Opcode::AdjSP(operand) => match operand {
            &Operand::Int(value) => vec![46, value as _],
            Operand::Label(label) => vec![46, symtab[label].try_into().unwrap()],
        },
        Opcode::PopArgs(operand) => match operand {
            &Operand::Int(value) => vec![47, value as _],
            Operand::Label(label) => vec![47, symtab[label].try_into().unwrap()],
        },
        Opcode::Call(addr) => match addr {
            &Operand::Int(value) => vec![48, value as _],
            Operand::Label(label) => vec![48, symtab[label].try_into().unwrap()],
        },
        Opcode::PushCVar(operand) => match operand {
            &Operand::Int(value) => vec![49, value as _],
            Operand::Label(label) => vec![49, symtab[label].try_into().unwrap()],
        },
        Opcode::PopCVar(operand) => match operand {
            &Operand::Int(value) => vec![50, value as _],
            Operand::Label(label) => vec![50, symtab[label].try_into().unwrap()],
        },
        Opcode::TraceOn => vec![51],
        Opcode::TraceOff => vec![52],
        Opcode::ClearIntDis => vec![53],
        Opcode::SetIntDis => vec![54],
        Opcode::Illegal => vec![55],
    };

    let mut ret = Vec::new();
    for word in text {
        ret.extend_from_slice(&word.to_le_bytes());
    }
    ret
}
