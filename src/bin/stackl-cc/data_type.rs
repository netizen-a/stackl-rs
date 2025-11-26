// Copyright (c) 2024-2025 Jonathan Thomason

use std::fmt;

use crate::analysis::syn;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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
	pub const fn is_floating(&self) -> bool {
		matches!(self, Self::Float | Self::Double | Self::LongDouble)
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
	pub fn is_signed(&self) -> Option<bool> {
		match self {
			Self::I8 | Self::I16 | Self::I32 | Self::I64 | Self::I128 => Some(true),
			Self::U8 | Self::U16 | Self::U32 | Self::U64 | Self::U128 => Some(false),
			_ => None,
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

#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
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

#[derive(Debug, Clone)]
pub struct MemberType {
	pub ident: Option<syn::Identifier>,
	pub dtype: Box<DataType>,
	pub bits: Option<u32>,
}

impl fmt::Display for MemberType {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		if let Some(ident) = &self.ident {
			write!(f, "{} {};", self.dtype, ident.name)
		} else {
			write!(f, "{};", self.dtype)
		}
	}
}

#[derive(Debug, Clone)]
pub struct EnumConst {
	pub tag_name: String,
	pub value: i32,
}

#[derive(Debug, Clone)]
pub enum TagKind {
	Struct(Option<String>, Vec<MemberType>),
	Union(Option<String>, Vec<MemberType>),
	Enum(Option<String>, Vec<(syn::Identifier, i32)>),
}

#[derive(Debug, Clone)]
pub enum TypeKind {
	Poison,
	Void,
	Scalar(ScalarType),
	Tag(TagKind),
	Function(FuncType),
	Pointer(Box<DataType>),
	Array(ArrayType),
	EnumConst(EnumConst),
}

impl TypeKind {
	pub fn is_incomplete(&self) -> bool {
		match self {
			Self::Tag(TagKind::Enum(Some(_), body)) if body.is_empty() => true,
			Self::Tag(TagKind::Struct(Some(_), body)) if body.is_empty() => true,
			Self::Tag(TagKind::Union(Some(_), body)) if body.is_empty() => true,
			_ => false,
		}
	}
	pub const fn is_integral(&self) -> bool {
		if let TypeKind::Scalar(scalar) = self {
			scalar.is_integral()
		} else {
			false
		}
	}
	pub const fn is_floating(&self) -> bool {
		if let TypeKind::Scalar(scalar) = self {
			scalar.is_floating()
		} else {
			false
		}
	}
	/// Recursively constructs a C type.
	/// The `context` parameter must be an empty `String`.
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
			Self::Poison => format!("{qual_str}{space}<poisoned>{context}"),
			Self::Void => format!("{qual_str}{space}void{context}"),
			Self::Scalar(ScalarType::Bool) => format!("{qual_str}{space}_Bool{context}"),
			Self::Scalar(ScalarType::U8) => format!("{qual_str}{space}unsigned char{context}"),
			Self::Scalar(ScalarType::I8) => format!("{qual_str}{space}char{context}"),
			Self::Scalar(ScalarType::U16) => format!("{qual_str}{space}unsigned short{context}"),
			Self::Scalar(ScalarType::I16) => format!("{qual_str}{space}short{context}"),
			Self::Scalar(ScalarType::U32) => format!("{qual_str}{space}unsigned int{context}"),
			Self::Scalar(ScalarType::I32) => format!("{qual_str}{space}int{context}"),
			Self::Scalar(ScalarType::U64) => format!("{qual_str}{space}unsigned long{context}"),
			Self::Scalar(ScalarType::I64) => format!("{qual_str}{space}long{context}"),
			Self::Scalar(ScalarType::U128) => {
				format!("{qual_str}{space}unsigned long long{context}")
			}
			Self::Scalar(ScalarType::I128) => format!("{qual_str}{space}long long{context}"),
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
			Self::Array(ArrayType {
				component, length, ..
			}) => {
				context.push('[');
				match length {
					ArrayLength::Fixed(fixed_length) => {
						context.push_str(&format!("{fixed_length}"))
					}
					ArrayLength::VLA(vla_length) => context.push('*'),
					ArrayLength::Incomplete => {}
				}

				context.push(']');
				component
					.kind
					.get_render(context, Some(component.qual.clone()))
			}
			Self::Function(FuncType {
				params,
				ret,
				is_variadic,
				..
			}) => {
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
				new_context.push_str(",...");
				new_context.push(')');
				new_context
			}
			Self::Tag(kind) => {
				let stub = match kind {
					TagKind::Struct(None, _) => String::from("struct <anonymous>"),
					TagKind::Union(None, _) => String::from("union <anonymous>"),
					TagKind::Enum(None, _) => String::from("enum <anonymous>"),
					TagKind::Struct(Some(tag_name), _) => format!("struct {tag_name}"),
					TagKind::Union(Some(tag_name), _) => format!("union {tag_name}"),
					TagKind::Enum(Some(tag_name), _) => format!("enum {tag_name}"),
				};
				format!("{qual_str}{space}{stub}")
			}
			Self::EnumConst(EnumConst { tag_name, .. }) => {
				format!("{qual_str}{space}enum {tag_name}")
			}
			other => todo!("{other:?}"),
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
	#[inline]
	pub fn is_incomplete(&self) -> bool {
		self.kind.is_incomplete()
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
