//! Declarations

pub struct Declaration {
	pub declaration_specifiers: Vec<DeclarationSpecifier>,
	pub init_declarator_list: Vec<InitDeclarator>,
}

pub enum DeclarationSpecifier {
	StorageClassSpecifier,
	TypeSpecifier,
	TypeQualifier,
	FunctionSpecifier,
}

pub struct InitDeclarator {
	declarator: Declarator,
	initializer: Option<Initializer>,
}

// TODO
pub struct Declarator {}

pub enum Initializer {
	AssignmentExpression,
	InitializerList,
}
