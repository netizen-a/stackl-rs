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
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct StructLayout(pub Box<[DataLayout]>);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PtrLayout(pub Box<DataLayout>);

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
				is_variadic: false,
				..
			}) => {
				let params_result: Result<Vec<DataLayout>, ()> = params
					.into_iter()
					.map(|param| Self::try_from(param.kind))
					.collect();
				Self::Function(FunctionLayout {
					params: params_result?,
					ret: Box::new(Self::try_from(ret.kind)?),
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
