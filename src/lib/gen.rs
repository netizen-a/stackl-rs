use std::collections::HashMap;

use crate::ast::{Addr, Data, Inst, Opcode, Stmt};

pub struct StacklFormat {
    pub magic: [u8; 4],
    pub version: u32,
    /// Must be set to zero.
    pub reserved: u32,
    pub text: Vec<u8>,
}

pub fn code_gen(ast: Vec<Stmt>, symtab: HashMap<String, usize>) -> StacklFormat {
    let mut text = Vec::<u8>::new();
    for stmt in ast {
        let data: Vec<u8> = match stmt.inst {
            Inst::Mnemonic(op) => convert_op(&op, &symtab),
            Inst::DataDecl8(list) => {
                let mut data_list = Vec::<u8>::new();
                for data in list {
                    let vec: Vec<u8> = match data {
                        // convert i32 to u8
                        Data::Int(value) => vec![value.try_into().unwrap()],
                        // convert String to [u8]
                        Data::String(s) => s.as_bytes().to_vec(),
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
                        Data::Int(value) => Vec::from(value.to_le_bytes()),
                        // convert String to [u8]
                        Data::String(s) => {
                            let mut bytes = s.as_bytes().to_vec();
                            if bytes.len() % 4 == 0 {
                                bytes
                            } else {
                                let len = 4 - (bytes.len() % 4);
                                bytes.extend(vec![0;len]);
                                bytes
                            }
                        },
                    };
                    data_list.extend(vec);
                }
                data_list
            }
            _ => vec![],
        };
        text.extend(data);
    }

    StacklFormat {
        magic: [b's', b'l', 0, 0],
        version: 0,
        reserved: 0,
        text,
    }
}

fn convert_op(op: &Opcode, symtab: &HashMap<String, usize>) -> Vec<u8> {
    let code: Vec<u32> = match op {
        Opcode::Nop => vec![0],
        Opcode::Plus => vec![1],
        Opcode::Minus => vec![2],
        Opcode::Times => vec![3],
        Opcode::Divide => vec![4],
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
        Opcode::JmpUser(addr) => match addr {
            Addr::Offset(offset) => vec![26, *offset as _],
            Addr::Label(label) => vec![26, symtab[label].try_into().unwrap()],
        }
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
        Opcode::Push(value) => vec![41, *value as _],
        Opcode::Jump(addr) => match addr {
            Addr::Offset(offset) => vec![42, *offset as _],
            Addr::Label(label) => vec![42, symtab[label].try_into().unwrap()],
        }
        Opcode::Jumpe(addr) => match addr {
            Addr::Offset(offset) => vec![43, *offset as _],
            Addr::Label(label) => vec![43, symtab[label].try_into().unwrap()],
        },
        Opcode::PushVar(value) => vec![44, *value as _],
        Opcode::PopVar(value) => vec![45, *value as _],
        Opcode::AdjSP(value) => vec![46, *value as _],
        Opcode::PopArgs(value) => vec![47, *value as _],
        Opcode::Call(addr) => match addr {
            Addr::Offset(offset) => vec![48, *offset as _],
            Addr::Label(label) => vec![48, symtab[label].try_into().unwrap()],
        }
        Opcode::PushCVar(value) => vec![49, *value as _],
        Opcode::PopCVar(value) => vec![50, *value as _],
        Opcode::TraceOn => vec![51],
        Opcode::TraceOff => vec![52],
        Opcode::ClearIntDis => vec![53],
        Opcode::SetIntDis => vec![54],
        Opcode::Illegal => vec![55],
    };
    let mut ret = Vec::new();
    for dword in code {
        ret.extend_from_slice(&dword.to_le_bytes());
    }
    ret
}
