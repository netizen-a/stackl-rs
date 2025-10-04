pub mod decl;
pub mod expr;
pub mod iter;
pub mod stmt;

use super::tok;
use crate::diagnostics as diag;
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
	span: diag::Span,
}

impl Identifier {
	pub fn new(token: tok::Token) -> Self {
		match token.kind {
			tok::TokenKind::Ident(ident) => Self {
				name: ident.name,
				span: token.span,
			},
			_ => panic!("internal compiler error: failed to get identifier"),
		}
	}
}

impl ToSpan for Identifier {
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
	Asm(AsmStmt),
	Error,
}

/// (6.9.1) function-definition
#[derive(Debug)]
pub struct FunctionDefinition {
	pub specifiers: Specifiers,
	pub ident: Identifier,
	pub declarators: Vec<Declarator>,
	pub declaration_list: Vec<Declaration>,
	pub compound_stmt: CompoundStmt,
}

pub fn string_concat(v: Box<[tok::Token]>) -> tok::StrLit {
	let mut str_lit = tok::StrLit::default();
	let mut is_first = true;
	for literal in v {
		let span = literal.to_span();
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
	str_lit
}

pub use grammar::SyntaxParser;
