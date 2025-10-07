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
	pub name: Option<String>,
	pub members: Vec<MemberType>,
	pub is_incomplete: bool,
}

#[derive(Debug, Clone)]
pub struct UnionType {
	pub name: Option<String>,
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
	Enum(Option<String>),
	Function(FuncType),
	Pointer(Box<DataType>),
	Array(ArrayType),
}

impl TypeKind {
	fn get_render(&self, mut context: String, qual: Option<TypeQual>) -> String {
		let qual = qual.unwrap_or_default();
		let mut qual_str = String::new();
		if qual.is_const {
			qual_str.push_str("const");
		}
		if qual.is_volatile {
			if qual.is_const {
				qual_str.push(' ');
			}
			qual_str.push_str("volatile");
		}
		if qual.is_restrict {
			if qual.is_const || qual.is_volatile {
				qual_str.push(' ');
			}
			qual_str.push_str("restrict");
		}
		let space = if qual_str.is_empty() { "" } else { " " };
		match self {
			Self::Void => format!("void{context}"),
			Self::Scalar(ScalarType::Bool) => format!("{qual_str}{space}_Bool{context}"),
			Self::Scalar(ScalarType::U8) => format!("{qual_str}{space}unsigned char{context}"),
			Self::Scalar(ScalarType::I8) => format!("{qual_str}{space}char{context}"),
			Self::Scalar(ScalarType::U16) => format!("{qual_str}{space}unsigned short{context}"),
			Self::Scalar(ScalarType::I16) => format!("{qual_str}{space}short{context}"),
			Self::Scalar(ScalarType::U32) => format!("{qual_str}{space}unsigned int{context}"),
			Self::Scalar(ScalarType::I32) => format!("{qual_str}{space}int{context}"),
			Self::Scalar(ScalarType::U64) => format!("{qual_str}{space}unsigned long int{context}"),
			Self::Scalar(ScalarType::I64) => format!("{qual_str}{space}long int{context}"),
			Self::Scalar(ScalarType::U128) => {
				format!("{qual_str}{space}unsigned long long int{context}")
			}
			Self::Scalar(ScalarType::I128) => format!("{qual_str}{space}long long int{context}"),
			Self::Scalar(ScalarType::Float) => format!("{qual_str}{space}float{context}"),
			Self::Scalar(ScalarType::Double) => format!("{qual_str}{space}double{context}"),
			Self::Scalar(ScalarType::LongDouble) => {
				format!("{qual_str}{space}long double{context}")
			}
			Self::Pointer(inner) => {
				let mut new_context = format!("*{qual_str}");
				new_context.push_str(&context);
				inner.kind.get_render(new_context, Some(inner.qual.clone()))
			}
			Self::Array(ArrayType { component, .. }) => {
				context.push_str("[]");
				component
					.kind
					.get_render(context, Some(component.qual.clone()))
			}
			Self::Function(FuncType { params, ret, .. }) => {
				let mut new_context = String::new();
				new_context.push_str(&ret.kind.get_render(String::new(), Some(ret.qual.clone())));
				if !context.is_empty() {
					new_context.push('(');
					new_context.push_str(&context);
					new_context.push(')');
				}
				new_context.push('(');
				for (index, param) in params.iter().enumerate() {
					if index != 0 {
						new_context.push_str(", ");
					}
					new_context.push_str(
						&param
							.kind
							.get_render(String::new(), Some(param.qual.clone())),
					);
				}
				new_context.push(')');
				new_context
			}
			Self::Struct(StructType { name, .. }) => {
				format!(
					"{qual_str} struct {}",
					name.clone().unwrap_or("<anonymous>".to_string())
				)
			}
			Self::Union(UnionType { name, .. }) => {
				format!(
					"{qual_str} union {}",
					name.clone().unwrap_or("<anonymous>".to_string())
				)
			}
			Self::Enum(name) => {
				format!(
					"{qual_str} union {}",
					name.clone().unwrap_or("<anonymous>".to_string())
				)
			}
			_ => todo!(),
		}
	}
}

impl fmt::Display for TypeKind {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{}", self.get_render(String::new(), None))
	}
}

#[derive(Debug, Clone)]
pub struct DataType {
	pub kind: TypeKind,
	pub qual: TypeQual,
}

impl DataType {
	pub const POISON: DataType = DataType {
		kind: TypeKind::Poison,
		qual: TypeQual {
			is_const: false,
			is_volatile: false,
			is_restrict: false,
		},
	};
	#[inline]
	pub const fn is_poisoned(&self) -> bool {
		matches!(self.kind, TypeKind::Poison)
	}
}

impl fmt::Display for DataType {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(
			f,
			"{}",
			self.kind.get_render(String::new(), Some(self.qual.clone()))
		)
	}
}
