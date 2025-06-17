use stackl::dr;

pub struct ModuleBuilder {
	// dependency graph
	dep: Vec<Vec<u32>>,
	builder: dr::Builder,
}

impl ModuleBuilder {
	pub fn new() -> Self {
		Self {
			dep: Vec::new(),
			builder: dr::Builder::new(),
		}
	}
	pub fn push(&mut self) -> u32 {
		let new_id = self.dep.len() as u32;
		self.dep.push(vec![]);
		new_id
	}
	pub fn i32_add(&mut self, id: u32) -> u32 {
		let new_id = self.dep.len() as u32;
		self.dep.push(vec![id]);
		new_id
	}
	pub fn i32_mul(&mut self, id: u32) -> u32 {
		let new_id = self.dep.len() as u32;
		self.dep.push(vec![id]);
		new_id
	}
	pub fn build(self) -> dr::Module {
		self.builder.build()
	}
}
