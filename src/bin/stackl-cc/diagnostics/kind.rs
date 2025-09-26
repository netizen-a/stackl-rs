use crate::data_types::DataType;

#[derive(Debug)]
pub enum DiagKind {
	UnexpectedEof,
	UnexpectedEscape,
	UnrecognizedToken { expected: Vec<String> },
	InvalidToken,
	ExtraToken,
	HeaderNameError,
	MultStorageClasses,
	DuplicateSpecifier(String),
	BothSpecifiers(String, String),
	InvalidRestrict,
	TypeError { found: DataType, expected: DataType },
	MultipleTypes,
	TooLong,
	ImplicitInt(String),
	ArrayOfFunctions(String),
	FnRetFn(String),
	OmittedParamName,
	DeclIdentList,
	UnboundVLA,
	InvalidStar,
}
