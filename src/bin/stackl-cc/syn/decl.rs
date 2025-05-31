//! Declarations

use super::expr;
use crate::tok;

/// (6.9.1) declaration-list
pub struct DeclarationList(Vec<Declaration>);

/// (6.7) declaration
pub struct Declaration {
	/// (6.7) declaration-specifiers
	pub declaration_specifiers: Vec<DeclarationSpecifier>,
	/// (6.7) init-declarator-list
	pub init_declarator_list: Vec<InitDeclarator>,
}

/// (6.7) declaration-specifiers
pub enum DeclarationSpecifier {
	/// (6.7.1) storage-class-specifier
	StorageClassSpecifier(tok::Keyword),
	TypeSpecifier(TypeSpecifier),
	/// (6.7.3) type-qualifier
	TypeQualifier(tok::Keyword),
	/// (6.7.4) function-specifier
	FunctionSpecifier(tok::Keyword),
}

/// (6.7) init-declarator
pub struct InitDeclarator {
	declarator: Declarator,
	initializer: Option<Initializer>,
}

/// (6.7.2) type-specifier
pub enum TypeSpecifier {
	Keyword(tok::Keyword),
	StructOrUnionSpecifier(StructOrUnionSpecifier),
	EnumSpecifier(EnumSpecifier),
	/// (6.7.7) typedef-name
	TypedefName(tok::Ident),
}

/// (6.7.2.2) enum-specifier
pub struct EnumSpecifier {
	identifier: Option<tok::Ident>,
	enumerator_list: Option<EnumeratorList>,
}

/// (6.7.2.2) enumerator-list
pub struct EnumeratorList(Vec<Enumerator>);

/// (6.7.2.2) enumerator
pub struct Enumerator {
	enumeration_constant: EnumerationConstant,
	constant_expression: expr::ConstantExpression,
}

/// (6.4.4.3) enumeration-constant
pub struct EnumerationConstant(tok::Ident);

/// (6.7.2.1) struct-or-union-specifier
pub struct StructOrUnionSpecifier {
	/// (6.7.2.1) struct-or-union
	pub struct_or_union: tok::Keyword,
	pub identifier: Option<tok::Ident>,
	pub struct_declaration_list: Option<StructDeclarationList>,
}

/// (6.7.2.1) struct-declaration-list
pub struct StructDeclarationList(Vec<StructDeclaration>);

/// (6.7.2.1) struct-declaration
pub struct StructDeclaration {
	specifier_qualifier_list: SpecifierQualifierList,
	struct_declaration_list: StructDeclaratorList,
}

/// (6.7.2.1) struct-declarator-list
pub struct StructDeclaratorList(Vec<StructDeclarator>);

/// (6.7.2.1) struct-declarator
pub struct StructDeclarator {
	declarator: Option<Declarator>,
	constant_expression: Option<expr::ConstantExpression>,
}

/// (6.7.8) initializer
pub enum Initializer {
	AssignmentExpression(expr::AssignmentExpression),
	InitializerList(InitializerList),
}

/// (6.7.5) declarator
pub struct Declarator {
	pub pointer: Vec<Pointer>,
	pub direct_declarator: Vec<DirectDeclarator>,
}

/// (6.7.5) direct-declarator
pub enum DirectDeclarator {
	Identifier(tok::Ident),
	/// ( declarator )
	Declarator(Box<Declarator>),
	/// [ type-qualifier-list_opt assignment-expression_opt ]
	TypeQualifierList(
		Option<TypeQualifierList>,
		Option<expr::AssignmentExpression>,
	),
	/// [ static type-qualifier-list_opt assignment-expression ]
	StaticTypeQualifierList(Option<TypeQualifierList>, expr::AssignmentExpression),
	/// [ type-qualifier-list static assignment-expression ]
	TypeQualifierListStatic(TypeQualifierList, expr::AssignmentExpression),
	/// [ type-qualifier-list_opt * ]
	TypeQualifierListPointer(TypeQualifierList),
	/// ( parameter-type-list )
	ParameterTypeList(ParameterTypeList),
	/// ( identifier-list_opt )
	IdentifierList(Vec<tok::Ident>),
}

/// (6.7.5) type-qualifier-list
pub struct TypeQualifierList(Vec<TypeQualifier>);

/// (6.7.5) parameter-type-list
pub struct ParameterTypeList {
	/// (6.7.5) parameter-list
	pub parameter_list: Vec<ParameterDeclaration>,
	pub comma_ellipsis: bool,
}

/// (6.7.5) pointer
pub struct Pointer {
	/// (6.7.5) type-qualifier-list
	pub type_qualifier_list: Vec<TypeQualifier>,
}

/// (6.7.5) parameter-declaration
pub enum ParameterDeclaration {
	Declarator {
		/// (6.7) declaration-specifiers
		declaration_specifiers: Vec<DeclarationSpecifier>,
		/// (6.7.5) declarator
		declarator: Declarator,
	},
	AbstractDeclarator {
		/// (6.7) declaration-specifiers
		declaration_specifiers: Vec<DeclarationSpecifier>,
		abstract_declarator: Option<AbstractDeclarator>,
	},
}

/// (6.7.6) abstract-declarator
pub enum AbstractDeclarator {
	Pointer(Pointer),
	DirectAbstractDeclarator {
		pointer: Pointer,
		direct_abstract_declarator: Vec<DirectAbstractDeclarator>,
	},
}

/// (6.7.6) direct-abstract-declarator
pub enum DirectAbstractDeclarator {
	/// ( abstract-declarator )
	AbstractDeclarator(AbstractDeclarator),
	/// direct-abstract-declarator_opt [ type-qualifier-list_opt assignment-expression_opt ]
	TypeQualifierList(
		Option<TypeQualifierList>,
		Option<expr::AssignmentExpression>,
	),
	/// direct-abstract-declarator_opt [ static type-qualifier-list_opt assignment-expression ]
	StaticTypeQualifierList(Option<TypeQualifierList>, expr::AssignmentExpression),
	/// direct-abstract-declarator_opt [ type-qualifier-list static assignment-expression ]
	TypeQualifierListStatic(TypeQualifierList, expr::AssignmentExpression),
	/// direct-abstract-declarator_opt [ * }
	ArrayPointer,
	/// direct-abstract-declarator_opt ( parameter-type-list_opt )
	ParameterTypeList(Option<ParameterTypeList>),
}

/// (6.7.3) type-qualifier
pub enum TypeQualifier {
	Const,
	Restrict,
	Volatile,
}
/// (6.7.6) type-name
pub struct TypeName {
	/// specifier-qualifier-list
	pub specifier_qualifier_list: SpecifierQualifierList,
	/// abstract-declarator_opt
	pub abstract_declarator: Option<AbstractDeclarator>,
}

/// (6.7.2.1) specifier-qualifier-list
pub struct SpecifierQualifierList(Vec<SpecifierQualifier>);

pub enum SpecifierQualifier {
	TypeSpecifier(TypeSpecifier),
	TypeQualifier(TypeQualifier),
}

/// (6.7.8) initializer-list
pub struct InitializerList(Vec<(Option<Designation>, Initializer)>);

/// (6.7.8) designation
pub struct Designation(DesignatorList);

/// (6.7.8) designator-list
pub struct DesignatorList(Vec<Designator>);

/// (6.7.8) designator
pub enum Designator {
	ConstantExpression(expr::ConstantExpression),
	Dot(tok::Ident),
}
