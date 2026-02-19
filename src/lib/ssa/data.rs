// Copyright (c) 2024-2026 Jonathan A. Thomason

use std::collections::HashMap;

#[derive(Debug, Clone, Copy)]
pub enum Opcode {
	Nop,
	Undef,
	IAdd,
	FAdd,
	ISub,
	FSub,
	IMul,
	FMul,
	SDiv,
	UDiv,
	FDiv,
	SRem,
	URem,
	FRem,
	Ret,
	RetValue,
	Store,
	Load,
	LogicalEqual,
	LogicalNotEqual,
	LogicalOr,
	LogicalAnd,
	LogicalNot,
	LogicalShiftRight,
	LogicalShiftLeft,
	BitwiseNot,
	BitwiseOr,
	BitwiseXor,
	BitwiseAnd,
	ArithmeticShiftRight,
	ArithmeticShiftLeft,
	IEqual,
	INotEqual,
	UGreaterThan,
	SGreaterThan,
	PtrEqual,
	PtrNotEqual,
	TypeVoid,
	TypeBool,
	TypeInt,
	TypeFloat,
	TypeArray,
	TypeRuntimeArray,
	TypePointer,
	TypeFunction,
	TypeVariadicFunction,
	TypeStruct,
	Halt,
	LifetimeStart,
	LifetimeEnd,
	Function,
	FunctionParameter,
	FunctionEnd,
	FunctionCall,
	CopyMemory,
	CopyMemorySized,
	Phi,
	LoopMerge,
	Label,
	Switch,
	Branch,
	BranchConditional,
	Unreachable,
	Decorate,
	MemberDecorate,
	DecorateId,
	DecorateString,
	MemberDecorateString,
	Variable,
	Constant,
	Assembler,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum StorageClass {
	Automatic,
	Static,
}

#[derive(Debug, Clone)]
pub enum Operand {
	IdRef(u32),
	LiteralString,
	LiteralBit32(u32),
	LiteralBit64(u64),
	LiteralBit128(u128),
	StorageClass(StorageClass),
	FunctionControl(u32),
	Text(String),
}

#[derive(Debug, Clone)]
pub struct Instruction {
	pub opcode: Opcode,
	pub result_type: Option<u32>,
	pub result_id: Option<u32>,
	pub operands: Box<[Operand]>,
}

#[derive(Debug)]
pub struct Module {
	pub type_list: Box<[Instruction]>,
	pub sections: HashMap<String, Vec<DataKind>>,
}

#[derive(Debug)]
pub enum DataKind {
	Func(Function),
	Data(Instruction),
}

#[derive(Debug)]
pub struct Function {
	pub begin: Instruction,
	pub params: Vec<Instruction>,
	pub body: Vec<Instruction>,
	pub end: Option<Instruction>,
}

impl Function {
	pub const fn new(instruction: Instruction) -> Self {
		Self {
			begin: instruction,
			params: vec![],
			body: vec![],
			end: None,
		}
	}
}
