pub struct Module {}

pub struct ModuleBuilder {
	module: Module,
}
impl ModuleBuilder {
	pub fn new() -> Self {
		Self { module: Module {} }
	}
	pub fn build(self) -> Module {
		self.module
	}
}
