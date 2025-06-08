use stackl::dr::Module;
use stackl::sr::DataType;

pub struct ModuleBuilder {}
impl ModuleBuilder {
	pub fn new() -> Self {
		Self {}
	}
	pub fn alloc_var(&mut self, data: DataType) -> u32 {
		todo!()
	}
	pub fn build(self) -> Module {
		Module {}
	}
}
