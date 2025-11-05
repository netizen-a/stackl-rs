use crate::data_type::{
	ArrayLength,
	ArrayType,
	ScalarType,
	TypeKind,
};

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
	pub component: Box<DataLayout>,
	pub length: u32,
}

#[derive(Debug, Clone)]
pub struct RuntimeArrayLayout(Box<DataLayout>);

#[derive(Debug, Clone)]
pub struct FunctionLayout {
	pub params: Vec<DataLayout>,
	pub ret: Box<DataLayout>,
}

#[derive(Debug, Clone)]
pub struct StructLayout(Box<[DataLayout]>);

#[derive(Debug, Clone)]
pub enum DataLayout {
	Void,
	Bool,
	Pointer(Box<DataLayout>),
	Integer(IntegerLayout),
	Array(ArrayLayout),
	RuntimeArray(RuntimeArrayLayout),
	Function(FunctionLayout),
	Struct(StructLayout),
}

impl From<TypeKind> for DataLayout {
	fn from(value: TypeKind) -> Self {
		match value {
			TypeKind::Void => Self::Void,
			TypeKind::Scalar(ScalarType::Bool) => Self::Bool,
			TypeKind::Scalar(ScalarType::U8) => Self::Integer(IntegerLayout {
				width: 8,
				is_signed: false,
			}),
			TypeKind::Scalar(ScalarType::I8) => Self::Integer(IntegerLayout {
				width: 8,
				is_signed: true,
			}),
			TypeKind::Scalar(ScalarType::U16) => Self::Integer(IntegerLayout {
				width: 16,
				is_signed: false,
			}),
			TypeKind::Scalar(ScalarType::I16) => Self::Integer(IntegerLayout {
				width: 16,
				is_signed: true,
			}),
			TypeKind::Scalar(ScalarType::U32) => Self::Integer(IntegerLayout {
				width: 32,
				is_signed: false,
			}),
			TypeKind::Scalar(ScalarType::I32) => Self::Integer(IntegerLayout {
				width: 32,
				is_signed: true,
			}),
			TypeKind::Scalar(ScalarType::U64) => Self::Integer(IntegerLayout {
				width: 64,
				is_signed: false,
			}),
			TypeKind::Scalar(ScalarType::I64) => Self::Integer(IntegerLayout {
				width: 64,
				is_signed: true,
			}),
			TypeKind::Scalar(ScalarType::U128) => Self::Integer(IntegerLayout {
				width: 128,
				is_signed: false,
			}),
			TypeKind::Scalar(ScalarType::I128) => Self::Integer(IntegerLayout {
				width: 128,
				is_signed: true,
			}),
			TypeKind::Array(ArrayType {
				component,
				length: ArrayLength::Fixed(length),
				is_decayed: false,
				..
			}) => Self::Array(ArrayLayout {
				component: Box::new(Self::from(component.kind)),
				length,
			}),
			TypeKind::Array(ArrayType {
				component,
				length: ArrayLength::VLA(_),
				is_decayed: false,
				..
			}) => Self::RuntimeArray(RuntimeArrayLayout(Box::new(Self::from(component.kind)))),
			TypeKind::Array(ArrayType {
				component,
				is_decayed: true,
				..
			}) => Self::Pointer(Box::new(Self::from(component.kind))),
			TypeKind::Pointer(component) => Self::Pointer(Box::new(Self::from(component.kind))),
			_ => todo!(),
		}
	}
}
