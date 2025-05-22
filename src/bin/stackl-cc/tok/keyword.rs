use super::span;
use super::Identifier;
use crate::diag::*;
use std::fmt;

#[derive(Debug)]
#[non_exhaustive]
pub enum KeywordTerminal {
	Auto,
	Break,
	Case,
	Char,
	Const,
	Continue,
	Default,
	Do,
	Double,
	Else,
	Enum,
	Extern,
	Float,
	For,
	Goto,
	If,
	Inline,
	Int,
	Long,
	Register,
	Restrict,
	Return,
	Short,
	Signed,
	SizeOf,
	Static,
	Struct,
	Switch,
	Typedef,
	Union,
	Unsigned,
	Void,
	Volatile,
	While,
	Bool,
}

impl fmt::Display for KeywordTerminal {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let name = match self {
			Self::Auto => "auto",
			Self::Break => "break",
			Self::Case => "case",
			Self::Char => "char",
			Self::Const => "const",
			Self::Continue => "continue",
			Self::Default => "default",
			Self::Do => "do",
			Self::Double => "double",
			Self::Else => "else",
			Self::Enum => "enum",
			Self::Extern => "extern",
			Self::Float => "float",
			Self::For => "for",
			Self::Goto => "goto",
			Self::If => "if",
			Self::Inline => "inline",
			Self::Int => "int",
			Self::Long => "long",
			Self::Register => "register",
			Self::Restrict => "restrict",
			Self::Return => "return",
			Self::Short => "short",
			Self::Signed => "signed",
			Self::SizeOf => "sizeof",
			Self::Static => "static",
			Self::Struct => "struct",
			Self::Switch => "switch",
			Self::Typedef => "typedef",
			Self::Union => "union",
			Self::Unsigned => "unsigned",
			Self::Void => "void",
			Self::Volatile => "volatile",
			Self::While => "while",
			Self::Bool => "_Bool",
		};
		write!(f, "{name}")
	}
}

#[derive(Debug)]
pub struct Keyword {
	span: span::Span,
	pub term: KeywordTerminal,
}

impl span::Spanned for Keyword {
	fn span(&self) -> span::Span {
		self.span.clone()
	}
	fn set_span(&mut self, span: span::Span) {
		self.span = span;
	}
}

impl TryFrom<Identifier> for Keyword {
	type Error = lex::TryFromIdentifierError;
	fn try_from(value: Identifier) -> Result<Self, Self::Error> {
		use KeywordTerminal as Term;
		let terminal = match value.name.as_str() {
			"auto" => Term::Auto,
			"break" => Term::Break,
			"case" => Term::Case,
			"char" => Term::Char,
			"const" => Term::Const,
			"continue" => Term::Continue,
			"default" => Term::Default,
			"do" => Term::Do,
			"double" => Term::Double,
			"else" => Term::Else,
			"enum" => Term::Enum,
			"extern" => Term::Extern,
			"float" => Term::Float,
			"for" => Term::For,
			"goto" => Term::Goto,
			"if" => Term::If,
			"inline" => Term::Inline,
			"int" => Term::Int,
			"long" => Term::Long,
			"register" => Term::Register,
			"restrict" => Term::Restrict,
			"return" => Term::Return,
			"short" => Term::Short,
			"signed" => Term::Signed,
			"sizeof" => Term::SizeOf,
			"static" => Term::Static,
			"struct" => Term::Struct,
			"switch" => Term::Switch,
			"typedef" => Term::Typedef,
			"union" => Term::Union,
			"unsigned" => Term::Unsigned,
			"void" => Term::Void,
			"volatile" => Term::Volatile,
			"while" => Term::While,
			"_Bool" => Term::Bool,
			_ => return Err(lex::TryFromIdentifierError),
		};
		Ok(Keyword {
			span: value.span,
			term: terminal,
		})
	}
}

impl fmt::Display for Keyword {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{}{}", self.span, self.term)
	}
}
