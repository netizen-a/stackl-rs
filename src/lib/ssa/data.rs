// Copyright (c) 2024-2026 Jonathan A. Thomason

#[derive(Debug)]
pub enum Opcode {
	Nop,
	Undef,
	IAdd,
	ISub,
	IMul,
	SDiv,
	SMod,
	Ret,
	RetValue,
	Store,
	Load,
	LogicalEqual,
	LogicalNotEqual,
	LogicalOr,
	LogicalAnd,
	LogicalNot,
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
	StorageClass(StorageClass),
	FunctionControl(u32),
	Text(String),
}

#[derive(Debug)]
pub struct Instruction {
	pub opcode: Opcode,
	pub result_type: Option<u32>,
	pub result_id: Option<u32>,
	pub operands: Box<[Operand]>,
}
pub struct Module {}
pub struct Function {}
