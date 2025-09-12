use std::fmt;

#[derive(Debug, Hash, Clone, PartialEq, Eq)]
pub enum ScalarType {
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
	Float,
	Double,
	LongDouble,
}

#[derive(Debug, Hash, Clone, PartialEq, Eq)]
pub struct Array {
	pub component: Box<DataType>,
	pub length: u32,
}

#[derive(Debug, Hash, Clone, PartialEq, Eq)]
pub struct Pointer {
	pub is_const: bool,
	pub is_volatile: bool,
	pub is_restrict: bool,
	inner: Box<DataType>,
}

#[derive(Debug, Hash, Clone, PartialEq, Eq)]
pub struct FuncType {
	pub params: Vec<DataType>,
	pub ret: Box<DataType>,
	pub is_variadic: bool,
}

#[derive(Debug, Hash, Clone, PartialEq, Eq)]
pub struct MemberType {
	pub ident: String,
	pub dtype: Box<DataType>,
}

impl fmt::Display for MemberType {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{} {};", self.dtype, self.ident)
	}
}

#[derive(Debug, Hash, Clone, PartialEq, Eq)]
pub enum DataType {
	Void,
	Scalar(ScalarType),
	Struct(Vec<MemberType>),
	Union(Vec<MemberType>),
	Enum,
	Function(FuncType),
	Pointer(Pointer),
	Array(Array),
}

impl fmt::Display for DataType {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::Void => write!(f, "()"),
			Self::Scalar(ScalarType::Bool) => write!(f, "_Bool"),
			Self::Scalar(ScalarType::U8) => write!(f, "unsigned char"),
			Self::Scalar(ScalarType::I8) => write!(f, "char"),
			Self::Scalar(ScalarType::U16) => write!(f, "unsigned short"),
			Self::Scalar(ScalarType::I16) => write!(f, "short"),
			Self::Scalar(ScalarType::U32) => write!(f, "unsigned int"),
			Self::Scalar(ScalarType::I32) => write!(f, "int"),
			Self::Scalar(ScalarType::U64) => write!(f, "unsigned long int"),
			Self::Scalar(ScalarType::I64) => write!(f, "long int"),
			Self::Scalar(ScalarType::U128) => write!(f, "unsigned long long int"),
			Self::Scalar(ScalarType::I128) => write!(f, "long long int"),
			Self::Scalar(ScalarType::Float) => write!(f, "float"),
			Self::Scalar(ScalarType::Double) => write!(f, "double"),
			Self::Scalar(ScalarType::LongDouble) => write!(f, "long double"),
			Self::Struct(fields) => {
				write!(f, "struct {{")?;
				for field in fields.iter() {
					write!(f, "{field}")?;
				}
				write!(f, "}}")
			}
			_ => todo!("{:?}", self),
		}
	}
}
