// Copyright (c) 2024-2026 Jonathan A. Thomason

pub mod decl;
pub mod expr;
pub mod iter;
pub mod stmt;

use std::cell;

use super::tok;
use crate::{
	diagnostics as diag,
	synthesis::icg,
};
pub use decl::*;
use diag::ToSpan;
pub use expr::*;
pub use iter::*;
pub use stmt::*;

use lalrpop_util::lalrpop_mod;
lalrpop_mod!(pub grammar, "/bin/stackl-cc/analysis/syn/grammar.rs");

#[derive(Debug, Clone)]
pub struct Identifier {
	pub name: String,
	pub span: diag::Span,
}

impl TryFrom<tok::Token> for Identifier {
	type Error = diag::Diagnostic;
	fn try_from(token: tok::Token) -> Result<Self, Self::Error> {
		match token.kind {
			tok::TokenKind::Ident(ident) => Ok(Self {
				name: ident.name,
				span: token.span,
			}),
			_ => {
				let error = diag::Diagnostic::fatal(
					diag::DiagKind::Internal("failed to parse identifier"),
					Some(token.span),
				);
				Err(error)
			}
		}
	}
}

impl ToSpan for Identifier {
	fn to_span(&self) -> diag::Span {
		self.span.clone()
	}
}

#[derive(Debug, Clone)]
pub enum IntegerKind {
	U32(u32),
	I32(i32),
	U64(u64),
	I64(i64),
	U128(u128),
	I128(i128),
}

impl From<tok::IntegerConstant> for IntegerKind {
	fn from(value: tok::IntegerConstant) -> Self {
		match value {
			tok::IntegerConstant::U32(u32) => Self::U32(u32),
			tok::IntegerConstant::I32(i32) => Self::I32(i32),
			tok::IntegerConstant::U64(u64) => Self::U64(u64),
			tok::IntegerConstant::I64(i64) => Self::I64(i64),
			tok::IntegerConstant::U128(u128) => Self::U128(u128),
			tok::IntegerConstant::I128(i128) => Self::I128(i128),
		}
	}
}

#[derive(Debug, Clone)]
pub enum FloatingKind {
	Float(f32),
	Double(f64),
	LongDouble(f64),
}

impl From<tok::FloatingConstant> for FloatingKind {
	fn from(value: tok::FloatingConstant) -> Self {
		match value {
			tok::FloatingConstant::Float(inner) => Self::Float(inner),
			tok::FloatingConstant::Double(inner) => Self::Double(inner),
			tok::FloatingConstant::Long(inner) => Self::LongDouble(inner),
		}
	}
}

#[derive(Debug, Clone)]
pub struct CharKind {
	pub seq: String,
	pub is_wide: bool,
}

impl From<tok::CharConst> for CharKind {
	fn from(value: tok::CharConst) -> Self {
		Self {
			seq: value.seq,
			is_wide: value.is_wide,
		}
	}
}

#[derive(Debug, Clone)]
pub enum ConstantKind {
	Integer(IntegerKind),
	Floating(FloatingKind),
	CharConst(CharKind),
}

impl From<tok::Const> for ConstantKind {
	fn from(constant: tok::Const) -> Self {
		match constant {
			tok::Const::Integer(inner) => Self::Integer(IntegerKind::from(inner)),
			tok::Const::Floating(inner) => Self::Floating(FloatingKind::from(inner)),
			tok::Const::CharConst(inner) => Self::CharConst(CharKind::from(inner)),
		}
	}
}

#[derive(Debug, Clone)]
pub struct Constant {
	span: diag::Span,
	pub kind: ConstantKind,
}

impl TryFrom<tok::Token> for Constant {
	type Error = diag::Diagnostic;
	fn try_from(token: tok::Token) -> Result<Self, Self::Error> {
		match token.kind {
			tok::TokenKind::Const(constant) => Ok(Self {
				span: token.span,
				kind: ConstantKind::from(constant),
			}),
			_ => {
				let error = diag::Diagnostic::fatal(
					diag::DiagKind::Internal("failed to parse constant"),
					Some(token.span),
				);
				Err(error)
			}
		}
	}
}

impl ToSpan for Constant {
	fn to_span(&self) -> diag::Span {
		self.span.clone()
	}
}

#[derive(Debug, Clone, Default)]
pub struct StringLiteral {
	pub seq: String,
	pub is_wide: bool,
	span: diag::Span,
}

impl StringLiteral {
	fn new(str_lit: tok::StrLit, span: diag::Span) -> Self {
		Self {
			seq: str_lit.seq,
			is_wide: str_lit.is_wide,
			span,
		}
	}
}

impl ToSpan for StringLiteral {
	fn to_span(&self) -> diag::Span {
		self.span.clone()
	}
}

/// (6.9) translation-unit
pub type TranslationUnit = Vec<ExternalDeclaration>;

/// (6.9) external-declaration
#[derive(Debug)]
pub enum ExternalDeclaration {
	FunctionDefinition(FunctionDefinition),
	Declaration(Declaration),
	Pragma(tok::Pragma),
	Error,
}

/// (6.9.1) function-definition
#[derive(Debug)]
pub struct FunctionDefinition {
	pub specifiers: Specifiers,
	pub ident: Identifier,
	pub declarators: Vec<Declarator>,
	/// (6.9.1) declaration-list
	pub declaration_list: Box<[Declaration]>,
	pub compound_stmt: CompoundStmt,
}

pub fn string_concat(v: Box<[tok::Token]>) -> StringLiteral {
	let mut str_lit = tok::StrLit::default();
	let mut is_first = true;
	let once = cell::OnceCell::new();
	for literal in v {
		let span = once.get_or_init(|| literal.to_span());
		if is_first {
			str_lit.file_id = span.file_id;
			is_first = false;
		}
		let tmp = literal.kind.unwrap_str_lit();
		str_lit.seq.push_str(&tmp.seq);
		if !str_lit.is_wide {
			str_lit.is_wide = tmp.is_wide;
		}
	}
	StringLiteral::new(str_lit, once.get().unwrap().clone())
}

pub use grammar::SyntaxParser;
