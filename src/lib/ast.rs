use lalrpop_util::ErrorRecovery;

use crate::{
    grammar::ProgramParser,
    lex,
    tok::{LexicalError, Token},
};

#[derive(Debug, PartialEq, Clone)]
pub struct Stmt {
    pub labels: Vec<String>,
    pub inst: Inst,
}

impl Stmt {
    pub fn new(inst: Inst) -> Self {
        Self {
            labels: Vec::new(),
            inst,
        }
    }
    pub fn with_labels(labels: Vec<String>, inst: Inst) -> Self {
        Self { labels, inst }
    }
}

// Instructions
#[derive(Debug, PartialEq, Clone)]
#[non_exhaustive]
pub enum Inst {
    Mnemonic(Opcode),
    Directive(Directive, Vec<String>),
    DataDecl8(Vec<Value>),
    DataDecl32(Vec<Value>),
}

#[derive(Debug, PartialEq, Clone)]
pub enum Value {
    String(String),
    Int(i32),
    Label(String),
}

// Primitive Directives
#[derive(Debug, PartialEq, Clone, Copy)]
#[non_exhaustive]
pub enum Directive {
    Segment,
    Extern,
    Global,
    Interrupt,
    Systrap,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Operand {
    Int(i32),
    Label(String),
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Reg {
    BP = 0,
    LP = 1,
    IP = 2,
    SP = 3,
    FP = 4,
    Flag = 5,
    IVec = 6,
}

#[derive(Debug, PartialEq, Clone)]
#[non_exhaustive]
pub enum Opcode {
    Nop,
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Eq,
    Ne,
    Gt,
    Lt,
    Ge,
    Le,
    And,
    Or,
    Not,
    Swap,
    Dup,
    Halt,
    Pop,
    Ret,
    Retv,
    Neg,
    PushCVarInd,
    Outs,
    Inp,
    PushFP,
    JmpUser(Operand),
    Trap,
    Rti,
    Calli,
    PushReg(Reg),
    PopReg(Reg),
    BAnd,
    BOr,
    BXOr,
    ShiftLeft,
    ShiftRight,
    PushVarInd,
    PopCVarInd,
    PopVarInd,
    Comp,
    Push(Operand),
    Jmp(Operand),
    Jz(Operand),
    PushVar(Operand),
    PopVar(Operand),
    AdjSP(Operand),
    PopArgs(Operand),
    Call(Operand),
    PushCVar(Operand),
    PopCVar(Operand),
    SetTrace,
    ClrTrace,
    ClrIntDis,
    SetIntDis,
    RotateLeft,
    RotateRight,
    Illegal,
}

pub fn parse_grammar(
    input: &str,
) -> Result<Vec<Stmt>, Vec<ErrorRecovery<usize, Token, LexicalError>>> {
    let tokens = lex::Lexer::new(input);
    let mut errors = Vec::new();
    let mut ast = match ProgramParser::new().parse(&mut errors, tokens) {
        Ok(v) => v,
        Err(_) => return Err(errors),
    };
    // prepend .text directive in case fixup rotates vector
    ast.insert(
        0,
        Stmt::new(Inst::Directive(
            Directive::Segment,
            vec![".text".to_string()],
        )),
    );
    if errors.is_empty() {
        Ok(ast)
    } else {
        Err(errors)
    }
}

// move labels to opcodes.
// must be done before fixup_start
pub fn fixup_labels(ast: &mut Vec<Stmt>) {
    let mut labels = Vec::<String>::new();
    for stmt in ast {
        match stmt.inst {
            Inst::Directive(_, _) => labels.append(&mut stmt.labels),
            _ => stmt.labels.append(&mut labels),
        }
    }
}

pub fn fixup_start(ast: &mut [Stmt]) {
    let start = "_start".to_string();
    let mid = ast
        .iter()
        .position(|stmt| stmt.labels.contains(&start))
        .unwrap();
    ast.rotate_left(mid);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn directives() {
        let source = "[section abc]\n
            [segment a,b]\n
            [extern foo]\n
            [global bar]";
        let ast = parse_grammar(source).unwrap();
        assert_eq!(
            ast,
            vec![
                Stmt::new(Inst::Directive(Directive::Segment, vec!["abc".to_string()])),
                Stmt::new(Inst::Directive(
                    Directive::Segment,
                    vec!["a".to_string(), "b".to_string()]
                )),
                Stmt::new(Inst::Directive(Directive::Extern, vec!["foo".to_string()])),
                Stmt::new(Inst::Directive(Directive::Global, vec!["bar".to_string()])),
            ]
        );
    }
    #[test]
    fn opcodes() {
        let source = "nop\n
            pushreg sp\n
            popreg 4\n
            push 34\n
            JMPUSER 8\n";
        let ast = parse_grammar(source).unwrap();
        assert_eq!(
            ast,
            vec![
                Stmt::new(Inst::Mnemonic(Opcode::Nop)),
                Stmt::new(Inst::Mnemonic(Opcode::PushReg(Reg::SP))),
                Stmt::new(Inst::Mnemonic(Opcode::PopReg(Reg::FP))),
                Stmt::new(Inst::Mnemonic(Opcode::Push(Operand::Int(34)))),
                Stmt::new(Inst::Mnemonic(Opcode::JmpUser(Operand::Int(8)))),
            ]
        );
    }

    #[test]
    fn labels() {
        let source = "label1 [section abc]\n
            label2\nlabel3: nop\n
            label4 nop\n
            label5\nnop";
        let ast = parse_grammar(source).unwrap();
        assert_eq!(
            ast,
            vec![
                Stmt::with_labels(
                    vec!["label1".to_string()],
                    Inst::Directive(Directive::Segment, vec!["abc".to_string()])
                ),
                Stmt::with_labels(
                    vec!["label2".to_string(), "label3".to_string()],
                    Inst::Mnemonic(Opcode::Nop)
                ),
                Stmt::with_labels(vec!["label4".to_string()], Inst::Mnemonic(Opcode::Nop)),
                Stmt::with_labels(vec!["label5".to_string()], Inst::Mnemonic(Opcode::Nop)),
            ]
        );
    }

    #[test]
    fn datadecls() {
        let source = "label db 'this is a string'\n
            dd `another string`\n
            db \"\\tstring with unicode\\n\"\n
            db 'a','b','c'";
        let ast = parse_grammar(source).unwrap();
        assert_eq!(
            ast,
            vec![
                Stmt::with_labels(
                    vec!["label".to_string()],
                    Inst::DataDecl8(vec![Value::String("this is a string".to_string())])
                ),
                Stmt::new(Inst::DataDecl32(vec![Value::String(
                    "another string".to_string()
                )])),
                Stmt::new(Inst::DataDecl8(vec![Value::String(
                    "\tstring with unicode\n".to_string()
                )])),
                Stmt::new(Inst::DataDecl8(vec![
                    Value::String("a".to_string()),
                    Value::String("b".to_string()),
                    Value::String("c".to_string())
                ])),
            ]
        );
    }
}
