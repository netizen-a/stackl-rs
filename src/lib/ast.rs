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
        Err(parse_error) => {
            errors.push(ErrorRecovery{
                error: parse_error,
                dropped_tokens: vec![]
            });
            return Err(errors)
        },
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
