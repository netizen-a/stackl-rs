mod decl;
mod expr;
mod stmt;

pub use decl::*;
pub use expr::*;
pub use stmt::*;

/// (6.9) translation-unit
pub struct TranslationUnit {
	external_declaration: Vec<ExternalDeclaration>,
}

/// (6.9) external-declaration
pub enum ExternalDeclaration {
	FunctionDefinition(FunctionDefinition),
	Declaration(Declaration),
}

/// (6.9.1) function-definition
pub struct FunctionDefinition {
	pub declaration_specifiers: Vec<DeclarationSpecifier>,
	pub declarator: Declarator,
	pub declaration_list: Option<DeclarationList>,
	pub compound_statement: CompoundStatement,
}
