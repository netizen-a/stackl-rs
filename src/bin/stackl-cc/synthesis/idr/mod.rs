pub struct IntDataRepr {
	code: Vec<SSA>,
	next_id: u32,
}

impl IntDataRepr {
	pub fn new() -> Self {
		Self {
			code: vec![],
			next_id: 0,
		}
	}
	fn id(&mut self) -> u32 {
		let id = self.next_id;
		self.next_id += 1;
		id
	}
	pub fn add(&mut self, src1: u32, src2: u32) -> u32 {
		let dest = self.id();
		self.code.push(SSA::Add([dest, src1, src2]));
		dest
	}
}

pub enum SSA {
	Add([u32; 3]),
	Sub([u32; 3]),
	BeginFn,
}
