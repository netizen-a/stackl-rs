pub mod decl;
pub mod expr;
pub mod iter;
pub mod stmt;

pub use decl::*;
pub use expr::*;
pub use iter::*;
pub use stmt::*;
use super::tok;

use lalrpop_util::lalrpop_mod;
lalrpop_mod!(pub grammar, "/bin/stackl-cc/analysis/syn/grammar.rs");

/// (6.9) translation-unit
pub type TranslationUnit = Vec<ExternalDeclaration>;

/// (6.9) external-declaration
#[derive(Debug)]
pub enum ExternalDeclaration {
	FunctionDefinition(FunctionDefinition),
	Declaration(Declaration),
}

/// (6.9.1) function-definition
#[derive(Debug)]
pub struct FunctionDefinition {
	pub declaration_specifiers: Vec<DeclarationSpecifier>,
	pub declarator: Declarator,
	pub declaration_list: Vec<Declaration>,
	pub compound_stmt: CompoundStmt,
}

pub fn string_concat(v: Box<[tok::Token]>) -> tok::StrLit {
	let mut str_lit = tok::StrLit::default();
	for literal in v {
		let tmp = literal.kind.unwrap_str_lit();
		str_lit.seq.push_str(&tmp.seq);
		if !str_lit.is_wide {
			str_lit.is_wide = tmp.is_wide;
		}
	}
	str_lit
}
