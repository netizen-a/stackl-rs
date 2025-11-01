pub enum Opcode {
	Nop,
	IAdd,
	ISub,
	IMul,
	SDiv,
	SMod,
	Switch,
	Return,
	ReturnValue,
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
	TypeVoid,
	TypeBool,
	TypeInt,
	TypeFloat,
	TypeArray,
}

pub struct Instruction {
	pub opcode: Opcode,
	pub result_type: Option<u32>,
	pub result_id: Option<u32>,
	pub operands: Box<[u32]>,
}
pub struct Module {}
pub struct Function {}
