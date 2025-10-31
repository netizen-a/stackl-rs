pub enum Opcode {
	Nop,
    Add,
	Sub,
	Mul,
	Div,
	Mod,
}

pub struct Instruction {
	pub opcode: Opcode,
	pub result_type: Option<u32>,
	pub result_id: Option<u32>,
	pub operand: Box<[u32]>,
}
pub struct Module {}
pub struct Function {}
