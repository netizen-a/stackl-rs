use std::collections::{HashMap, HashSet};

use crate::ast::{self, Inst, Opcode, Operand, Value};

#[derive(Debug)]
pub struct SymTabError {
    _mis_labels: HashSet<String>,
    _dup_labels: HashSet<String>,
}

/// On success returns symbol table with corresponding offsets.
/// On failure returns `SymTabError`.
pub(crate) fn build_symtab(ast: &[ast::Stmt]) -> Result<HashMap<String, usize>, SymTabError> {
    // Associate symbol names and offsets
    let mut symtab: HashMap<String, usize> = HashMap::new();
    // set for missing labels
    let mut mis_labels = HashSet::<String>::new();
    // set for duplicate labels
    let mut dup_labels = HashSet::<String>::new();
    // offset in bytes
    let mut pos = 0;

    for stmt in ast {
        for label in &stmt.labels {
            if mis_labels.contains(label) {
                mis_labels.remove(label);
            }
            if symtab.insert(label.clone(), pos).is_some() {
                dup_labels.insert(label.to_string());
            }
        }

        if let Inst::Mnemonic(op) = &stmt.inst {
            let some_label = match op {
                Opcode::JmpUser(Operand::Label(label))
                | Opcode::Push(Operand::Label(label))
                | Opcode::Jmp(Operand::Label(label))
                | Opcode::Jz(Operand::Label(label))
                | Opcode::PushVar(Operand::Label(label))
                | Opcode::PopVar(Operand::Label(label))
                | Opcode::AdjSP(Operand::Label(label))
                | Opcode::PopArgs(Operand::Label(label))
                | Opcode::Call(Operand::Label(label))
                | Opcode::PushCVar(Operand::Label(label))
                | Opcode::PopCVar(Operand::Label(label)) => {
                    (!symtab.contains_key(label)).then_some(label.to_string())
                }
                _ => None,
            };
            if let Some(label) = some_label {
                mis_labels.insert(label);
            }
        };

        pos += get_inst_size(&stmt.inst);
    }

    if !mis_labels.is_empty() {
        return Err(SymTabError {
            _mis_labels: mis_labels,
            _dup_labels: dup_labels,
        });
    }
    Ok(symtab)
}

fn get_inst_size(inst: &Inst) -> usize {
    match inst {
        Inst::Directive(_, _) => 0,
        Inst::Mnemonic(op) => match op {
            Opcode::JmpUser(_)
            | Opcode::PushReg(_)
            | Opcode::PopReg(_)
            | Opcode::Push(_)
            | Opcode::Jmp(_)
            | Opcode::Jz(_)
            | Opcode::PushVar(_)
            | Opcode::PopVar(_)
            | Opcode::AdjSP(_)
            | Opcode::PopArgs(_)
            | Opcode::Call(_)
            | Opcode::PushCVar(_)
            | Opcode::PopCVar(_) => 8,
            _ => 4,
        },
        Inst::DataDecl8(list) => {
            let mut total = 0;
            for data in list {
                total += match data {
                    Value::Int(_) => 1,
                    Value::String(s) => s.as_bytes().len(),
                    _ => panic!("label cannot fit in target declaration"),
                }
            }
            total
        }
        Inst::DataDecl32(list) => {
            let mut total = 0;
            for data in list {
                total += match data {
                    Value::Int(_) => 4,
                    Value::String(s) => {
                        let len = s.as_bytes().len();
                        if len % 4 == 0 {
                            len
                        } else {
                            len + 4 - (len % 4)
                        }
                    }
                    Value::Label(_) => 4,
                }
            }
            total
        }
    }
}
