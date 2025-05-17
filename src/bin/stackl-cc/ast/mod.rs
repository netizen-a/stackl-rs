mod decl;

pub use decl::*;

pub struct TranslationUnit(Vec<ExternalDeclaration>);

pub enum ExternalDeclaration {
	FunctionDefinition(FunctionDefinition),
	Declaration(Declaration),
}

pub struct FunctionDefinition {
	pub declaration_specifiers: Vec<DeclarationSpecifier>,
	pub declarator: Declarator,
}
