#[derive(Debug, Clone)]
pub struct IntegerLayout {
	pub width: u32,
	pub is_signed: bool,
}

#[derive(Debug, Clone)]
pub struct FloatLayout {
	pub width: u32,
}

#[derive(Debug, Clone)]
pub struct ArrayLayout {
	pub component: Box<Layout>,
	pub length: u32,
}

#[derive(Debug, Clone)]
pub struct RuntimeArrayLayout(Box<Layout>);

#[derive(Debug, Clone)]
pub struct FunctionLayout {
	pub params: Vec<Layout>,
	pub ret: Box<Layout>,
}

#[derive(Debug, Clone)]
pub struct StructLayout(Box<[Layout]>);

#[derive(Debug, Clone)]
pub enum Layout {
	Void,
	Bool,
	Pointer(Box<Layout>),
	Integer(IntegerLayout),
	Array(ArrayLayout),
	RuntimeArray(RuntimeArrayLayout),
	Function(FunctionLayout),
	Struct(StructLayout),
}
