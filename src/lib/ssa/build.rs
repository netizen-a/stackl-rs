use super::data;
use super::Error;

pub struct Builder {
	inst_list: Vec<data::Instruction>,
	next_id: u32,
}

impl Builder {
	pub fn new() -> Self {
		Self {
			inst_list: vec![],
			next_id: 0,
		}
	}
	/// Returns the next unused id
	pub fn id(&mut self) -> u32 {
		let result = self.next_id;
		self.next_id += 1;
		result
	}
	pub fn nop(&mut self) -> Result<(), Error> {
		self.inst_list.push(data::Instruction {
			opcode: data::Opcode::Nop,
			operand: vec![],
		});
		Ok(())
	}
	pub fn build(self) -> data::Module {
		todo!()
	}
}
