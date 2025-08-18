#[derive(Debug)]
pub enum DiagKind {
	UnexpectedEof,
	UnexpectedEscape,
	InvalidToken,
	HeaderNameError,
	MultStorageClasses,
}
