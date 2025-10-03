use std::fmt;

use crate::analysis::syn;

#[derive(Debug, Clone, PartialEq, Eq)]
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

impl ScalarType {
	pub const fn is_integral(&self) -> bool {
		matches!(
			self,
			Self::Bool
				| Self::I8 | Self::U8
				| Self::I16 | Self::U16
				| Self::I32 | Self::U32
				| Self::I64 | Self::U64
				| Self::I128 | Self::U128
		)
	}
	pub const fn bits(&self) -> u32 {
		match self {
			Self::Bool => 1,
			Self::I8 => 8,
			Self::U8 => 8,
			Self::I16 => 16,
			Self::U16 => 16,
			Self::I32 => 32,
			Self::U32 => 32,
			Self::I64 => 64,
			Self::U64 => 64,
			Self::I128 => 128,
			Self::U128 => 128,
			Self::Float => 32,
			Self::Double => 64,
			Self::LongDouble => 64,
		}
	}
	pub fn set_signedness(&mut self, is_signed: bool) {
		if is_signed {
			match self {
				ScalarType::U8 => *self = ScalarType::I8,
				ScalarType::U16 => *self = ScalarType::I16,
				ScalarType::U32 => *self = ScalarType::I32,
				ScalarType::U64 => *self = ScalarType::I64,
				ScalarType::U128 => *self = ScalarType::I128,
				_ => {
					// do nothing
				}
			}
		} else {
			match self {
				ScalarType::I8 => *self = ScalarType::U8,
				ScalarType::I16 => *self = ScalarType::U16,
				ScalarType::I32 => *self = ScalarType::U32,
				ScalarType::I64 => *self = ScalarType::U64,
				ScalarType::I128 => *self = ScalarType::U128,
				_ => {
					// do nothing
				}
			}
		}
	}
}

#[derive(Debug, Clone)]
pub enum VlaLength {
	Star,
	Expr(syn::Expr),
}

#[derive(Debug, Clone)]
pub enum ArrayLength {
	Incomplete,
	Fixed(u32),
	VLA(VlaLength),
}

#[derive(Debug, Clone)]
pub struct ArrayType {
	pub component: Box<DataType>,
	pub length: ArrayLength,
	pub is_decayed: bool,
	pub has_static: bool,
}

#[derive(Debug, Clone)]
pub struct PtrType(pub Box<DataType>);

#[derive(Debug, Clone, Default)]
pub struct TypeQual {
	pub is_const: bool,
	pub is_volatile: bool,
	pub is_restrict: bool,
}

#[derive(Debug, Clone)]
pub struct FuncType {
	pub params: Vec<DataType>,
	pub ret: Box<DataType>,
	pub is_variadic: bool,
	pub is_inline: bool,
}

// TODO: add optional bitfields
#[derive(Debug, Clone)]
pub struct MemberType {
	pub name: Option<String>,
	pub dtype: Box<DataType>,
	pub bits: Option<u32>,
}

impl fmt::Display for MemberType {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		if let Some(name) = &self.name {
			write!(f, "{} {name};", self.dtype)
		} else {
			write!(f, "{};", self.dtype)
		}
	}
}

#[derive(Debug, Clone)]
pub struct StructType {
	pub members: Vec<MemberType>,
	pub is_incomplete: bool,
}

#[derive(Debug, Clone)]
pub struct UnionType {
	pub members: Vec<MemberType>,
	pub is_incomplete: bool,
}

#[derive(Debug, Clone)]
pub enum TypeKind {
	Poison,
	Void,
	Scalar(ScalarType),
	Struct(StructType),
	Union(UnionType),
	Enum(String),
	Function(FuncType),
	Pointer(PtrType),
	Array(ArrayType),
}

impl fmt::Display for TypeKind {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::Void => write!(f, "void"),
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
			Self::Struct(StructType {
				members,
				is_incomplete,
			}) => {
				if *is_incomplete {
					write!(f, "struct")
				} else {
					let mut s = String::from("struct {{\n");
					for mem in members {
						s.push_str(&format!("    {mem}"));
					}
					s.push_str("}}");
					write!(f, "{s}")
				}
			}
			_ => todo!("{:?}", self),
		}
	}
}

#[derive(Debug, Clone)]
pub struct DataType {
	pub kind: TypeKind,
	pub qual: TypeQual,
}

impl fmt::Display for DataType {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let mut qual_str = String::new();
		if self.qual.is_const {
			qual_str.push_str("const ");
		}
		if self.qual.is_volatile {
			qual_str.push_str("volatile ");
		}
		if self.qual.is_restrict {
			qual_str.push_str("restrict ");
		}
		write!(f, "{qual_str}{}", self.kind)
	}
}
