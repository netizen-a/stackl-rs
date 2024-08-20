// Copyright (c) 2024-2026 Jonathan A. Thomason

use crate::data_type::{
	ArrayLength,
	ArrayType,
	FuncType,
	ScalarType,
	TagKind,
	TypeKind,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct IntegerLayout {
	pub width: u32,
	pub is_signed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FloatLayout {
	pub width: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ArrayLayout {
	pub component: Box<DataLayout>,
	pub length: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RuntimeArrayLayout(Box<DataLayout>);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FunctionLayout {
	pub params: Vec<DataLayout>,
	pub ret: Box<DataLayout>,
	pub is_variadic: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct StructLayout(pub Box<[DataLayout]>);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PtrLayout(pub Box<DataLayout>);

/// This type is optimal for SSA code generation since it can be put in
/// hashmaps and have no redundant data layouts.
///
/// Note that qualifiers are not added here since they make the
/// types too redundant for SSA code generation
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DataLayout {
	Void,
	Bool,
	Pointer(PtrLayout),
	Integer(IntegerLayout),
	Float(FloatLayout),
	Array(ArrayLayout),
	RuntimeArray(RuntimeArrayLayout),
	Function(FunctionLayout),
	Struct(StructLayout),
}

impl TryFrom<TypeKind> for DataLayout {
	type Error = ();
	fn try_from(value: TypeKind) -> Result<Self, Self::Error> {
		let result = match value {
			TypeKind::Void => Self::Void,
			TypeKind::Scalar(ScalarType::Bool) => Self::Bool,
			TypeKind::Scalar(ScalarType::UChar) => Self::Integer(IntegerLayout {
				width: 8,
				is_signed: false,
			}),
			TypeKind::Scalar(ScalarType::SChar) => Self::Integer(IntegerLayout {
				width: 8,
				is_signed: true,
			}),
			TypeKind::Scalar(ScalarType::UShort | ScalarType::UInt) => {
				Self::Integer(IntegerLayout {
					width: 32,
					is_signed: false,
				})
			}
			TypeKind::Scalar(ScalarType::SShort | ScalarType::SInt) => {
				Self::Integer(IntegerLayout {
					width: 32,
					is_signed: true,
				})
			}
			TypeKind::Scalar(ScalarType::ULong) => Self::Integer(IntegerLayout {
				width: 64,
				is_signed: false,
			}),
			TypeKind::Scalar(ScalarType::SLong) => Self::Integer(IntegerLayout {
				width: 64,
				is_signed: true,
			}),
			TypeKind::Scalar(ScalarType::ULong2) => Self::Integer(IntegerLayout {
				width: 128,
				is_signed: false,
			}),
			TypeKind::Scalar(ScalarType::SLong2) => Self::Integer(IntegerLayout {
				width: 128,
				is_signed: true,
			}),
			TypeKind::Scalar(ScalarType::Float) => Self::Float(FloatLayout { width: 32 }),
			TypeKind::Scalar(ScalarType::Double | ScalarType::LongDouble) => {
				Self::Float(FloatLayout { width: 64 })
			}
			TypeKind::Array(ArrayType {
				component,
				length: ArrayLength::Fixed(length),
				is_decayed: false,
				..
			}) => Self::Array(ArrayLayout {
				component: Box::new(Self::try_from(component.kind)?),
				length,
			}),
			TypeKind::Array(ArrayType {
				component,
				length: ArrayLength::VLA(_),
				is_decayed: false,
				..
			}) => Self::RuntimeArray(RuntimeArrayLayout(Box::new(Self::try_from(
				component.kind,
			)?))),
			TypeKind::Array(ArrayType {
				component,
				is_decayed: true,
				..
			}) => Self::Pointer(PtrLayout(Box::new(Self::try_from(component.kind)?))),
			TypeKind::Pointer(component) => {
				Self::Pointer(PtrLayout(Box::new(Self::try_from(component.kind)?)))
			}
			TypeKind::Function(FuncType {
				params,
				ret,
				is_variadic,
				..
			}) => {
				let mut param_list: Vec<DataLayout> = vec![];
				for (i, param) in params.iter().enumerate() {
					let layout = Self::try_from(param.kind.clone())?;
					if layout == DataLayout::Void {
						debug_assert!(i == 0);
						continue;
					}
					param_list.push(layout)
				}

				let is_variadic: bool = if is_variadic { true } else { params.is_empty() };
				Self::Function(FunctionLayout {
					params: param_list,
					ret: Box::new(Self::try_from(ret.kind)?),
					is_variadic,
				})
			}
			TypeKind::Tag(TagKind::Struct(_, members)) => {
				let struct_result: Result<Vec<DataLayout>, ()> = members
					.into_iter()
					.map(|mem| Self::try_from(mem.dtype.kind))
					.collect();
				Self::Struct(StructLayout(struct_result?.into_boxed_slice()))
			}
			TypeKind::Poison => return Err(()),
			other => todo!("{other}"),
		};
		Ok(result)
	}
}
