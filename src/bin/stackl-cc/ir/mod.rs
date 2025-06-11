use std::collections::HashMap;

use stackl::dr::Module;
use stackl::sr;

pub struct ModuleBuilder {
	// dependency graph
	dep: HashMap<u32, Vec<u32>>,
}

impl ModuleBuilder {
	pub fn new() -> Self {
		Self {
			dep: HashMap::new(),
		}
	}
	pub fn push(&mut self) -> u32 {
		let new_id = self.dep.len() as u32;
		self.dep.insert(new_id, vec![]);
		new_id
	}
	pub fn i32_add(&mut self, id: u32) -> u32 {
		let new_id = self.dep.len() as u32;
		self.dep.insert(new_id, vec![id]);
		new_id
	}
	pub fn i32_mul(&mut self, id: u32) -> u32 {
		let new_id = self.dep.len() as u32;
		self.dep.insert(new_id, vec![id]);
		new_id
	}
	pub fn build(self) -> Module {
		Module {}
	}
}
