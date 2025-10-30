pub enum Opcode {
	Nop,
}

pub struct Instruction {
	pub opcode: Opcode,
	pub operand: Vec<u32>,
}
pub struct Module {}
pub struct Function {}
