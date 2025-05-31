pub mod decl;
pub mod expr;
pub mod stmt;

pub use decl::*;
pub use expr::*;
pub use stmt::*;

use lalrpop_util::lalrpop_mod;
lalrpop_mod!(pub grammar, "/bin/stackl-cc/syn/grammar.rs");

/// (6.9) translation-unit
#[derive(Debug, Default)]
pub struct TranslationUnit {
	external_declaration: Vec<ExternalDeclaration>,
}

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
	pub compound_statement: CompoundStatement,
}
