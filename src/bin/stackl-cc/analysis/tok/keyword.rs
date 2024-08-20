// Copyright (c) 2024-2026 Jonathan A. Thomason

use std::fmt;

use crate::diagnostics::lex;

#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum Keyword {
	Asm,
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
	Sizeof,
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

impl TryFrom<&str> for Keyword {
	type Error = lex::TryFromIdentifierError;
	fn try_from(value: &str) -> Result<Self, Self::Error> {
		let terminal = match value {
			"asm" => Keyword::Asm,
			"auto" => Keyword::Auto,
			"break" => Keyword::Break,
			"case" => Keyword::Case,
			"char" => Keyword::Char,
			"const" => Keyword::Const,
			"continue" => Keyword::Continue,
			"default" => Keyword::Default,
			"do" => Keyword::Do,
			"double" => Keyword::Double,
			"else" => Keyword::Else,
			"enum" => Keyword::Enum,
			"extern" => Keyword::Extern,
			"float" => Keyword::Float,
			"for" => Keyword::For,
			"goto" => Keyword::Goto,
			"if" => Keyword::If,
			"inline" => Keyword::Inline,
			"int" => Keyword::Int,
			"long" => Keyword::Long,
			"register" => Keyword::Register,
			"restrict" => Keyword::Restrict,
			"return" => Keyword::Return,
			"short" => Keyword::Short,
			"signed" => Keyword::Signed,
			"sizeof" => Keyword::Sizeof,
			"static" => Keyword::Static,
			"struct" => Keyword::Struct,
			"switch" => Keyword::Switch,
			"typedef" => Keyword::Typedef,
			"union" => Keyword::Union,
			"unsigned" => Keyword::Unsigned,
			"void" => Keyword::Void,
			"volatile" => Keyword::Volatile,
			"while" => Keyword::While,
			"_Bool" => Keyword::Bool,
			_ => return Err(lex::TryFromIdentifierError),
		};
		Ok(terminal)
	}
}

impl fmt::Display for Keyword {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let keyword = match self {
			Keyword::Asm => "asm",
			Keyword::Auto => "auto",
			Keyword::Break => "break",
			Keyword::Case => "case",
			Keyword::Char => "char",
			Keyword::Const => "const",
			Keyword::Continue => "continue",
			Keyword::Default => "default",
			Keyword::Do => "do",
			Keyword::Double => "double",
			Keyword::Else => "else",
			Keyword::Enum => "enum",
			Keyword::Extern => "extern",
			Keyword::Float => "float",
			Keyword::For => "for",
			Keyword::Goto => "goto",
			Keyword::If => "if",
			Keyword::Inline => "inline",
			Keyword::Int => "int",
			Keyword::Long => "long",
			Keyword::Register => "register",
			Keyword::Restrict => "restrict",
			Keyword::Return => "return",
			Keyword::Short => "short",
			Keyword::Signed => "signed",
			Keyword::Sizeof => "sizeof",
			Keyword::Static => "static",
			Keyword::Struct => "struct",
			Keyword::Switch => "switch",
			Keyword::Typedef => "typedef",
			Keyword::Union => "union",
			Keyword::Unsigned => "unsigned",
			Keyword::Void => "void",
			Keyword::Volatile => "volatile",
			Keyword::While => "while",
			Keyword::Bool => "_Bool",
		};
		write!(f, "{keyword}")
	}
}
