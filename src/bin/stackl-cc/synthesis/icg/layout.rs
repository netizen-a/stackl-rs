#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ScalarLayout {
	Bool,
	I8,
	U8,
	I16,
	U16,
	I32,
	U32,
	I64,
	U64,
	I128,
	U128,
	F32,
	F64,
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
pub enum Layout {
	Scalar(ScalarLayout),
	Array(ArrayLayout),
	RuntimeArray(RuntimeArrayLayout),
	Function(FunctionLayout),
}
