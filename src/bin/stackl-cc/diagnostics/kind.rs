use crate::data_types::DataType;

#[derive(Debug)]
pub enum DiagKind {
	UnexpectedEof,
	UnexpectedEscape,
	InvalidToken,
	HeaderNameError,
	MultStorageClasses,
	InvalidRestrict,
	TypeError{
		found: DataType,
		expected: DataType,
	},
}
