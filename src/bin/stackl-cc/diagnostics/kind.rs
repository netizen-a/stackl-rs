#[derive(Debug)]
pub enum DiagKind {
	UnexpectedEof,
	UnexpectedEscape,
	InvalidToken,
	HeaderNameError,
	MultStorageClasses,
}

impl ToString for DiagKind {
	fn to_string(&self) -> String {
		let s = match self {
			DiagKind::UnexpectedEof => "unexpected end of file",
			DiagKind::UnexpectedEscape => "unexpected escape",
			DiagKind::InvalidToken => "invalid token",
			DiagKind::HeaderNameError => "header name error",
			DiagKind::MultStorageClasses => "multiple storage classes in declaration specifiers",
		};
		s.to_string()
	}
}
