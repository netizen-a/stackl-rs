// Copyright (c) 2024-2026 Jonathan A. Thomason

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
	DataDecl8(Vec<Atom>),
	DataDecl32(Vec<Atom>),
}

#[derive(Debug, PartialEq, Clone)]
pub enum Atom {
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
	Feature,
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
