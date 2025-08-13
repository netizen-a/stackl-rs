use std::result;

#[derive(Debug)]
pub enum DiagKind {
	Unknown,
}

#[derive(Debug)]
pub struct Diagnostic {
	pub level: super::DiagLevel,
	pub kind: DiagKind,
	pub loc: (usize, usize),
}

pub type ResultTriple<Tok, Loc> = result::Result<(Loc, Tok, Loc), Diagnostic>;
