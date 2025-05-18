pub mod decl;
pub mod expr;
pub mod stmt;

/// (6.9) translation-unit
#[derive(Default)]
pub struct TranslationUnit {
	external_declaration: Vec<ExternalDeclaration>,
}

/// (6.9) external-declaration
pub enum ExternalDeclaration {
	FunctionDefinition(FunctionDefinition),
	Declaration(decl::Declaration),
}

/// (6.9.1) function-definition
pub struct FunctionDefinition {
	pub declaration_specifiers: Vec<decl::DeclarationSpecifier>,
	pub declarator: decl::Declarator,
	pub declaration_list: Option<decl::DeclarationList>,
	pub compound_statement: stmt::CompoundStatement,
}
