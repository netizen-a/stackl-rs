use std::fmt;

#[derive(Debug, Hash, Clone, PartialEq, Eq)]
enum Scalar {
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
	params: Vec<DataType>,
	ret: Box<DataType>,
	is_variadic: bool,
}

#[derive(Debug, Hash, Clone, PartialEq, Eq)]
pub enum DataType {
	Void,
	Scalar(Scalar),
	Struct(Vec<DataType>),
	Union(Vec<DataType>),
	Enum,
	Function(FuncType),
	Pointer(Pointer),
	Array(Array),
}

impl fmt::Display for DataType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Void => write!(f, "()"),
            Self::Scalar(Scalar::Bool) => write!(f, "bool"),
            Self::Scalar(Scalar::U8) => write!(f, "unsigned char"),
            Self::Scalar(Scalar::I8) => write!(f, "char"),
            Self::Scalar(Scalar::U16) => write!(f, "unsigned short"),
            Self::Scalar(Scalar::I16) => write!(f, "short"),
            Self::Scalar(Scalar::U32) => write!(f, "unsigned int"),
            Self::Scalar(Scalar::I32) => write!(f, "int"),
            Self::Scalar(Scalar::U64) => write!(f, "unsigned long int"),
            Self::Scalar(Scalar::I64) => write!(f, "long int"),
            Self::Scalar(Scalar::U128) => write!(f, "unsigned long long int"),
            Self::Scalar(Scalar::I128) => write!(f, "long long int"),
            Self::Scalar(Scalar::Float) => write!(f, "float"),
            Self::Scalar(Scalar::Double) => write!(f, "double"),
            Self::Scalar(Scalar::LongDouble) => write!(f, "long double"),
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
