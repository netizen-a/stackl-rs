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
	StorageClassSpecifier(StorageClassSpecifier),
	TypeSpecifier(TypeSpecifier),
	TypeQualifier(TypeQualifier),
	FunctionSpecifier(FunctionSpecifier),
}

/// (6.7.4) function-specifier
pub struct FunctionSpecifier(tok::Keyword);

/// (6.7) init-declarator
pub struct InitDeclarator {
	declarator: Declarator,
	initializer: Option<Initializer>,
}

/// (6.7.1) storage-class-specifier
pub enum StorageClassSpecifier {
	Typedef,
	Extern,
	Static,
	Auto,
	Register,
}

/// (6.7.2) type-specifier
pub enum TypeSpecifier {
	Void,
	Char,
	Short,
	Int,
	Long,
	Float,
	Double,
	Signed,
	Unsigned,
	Bool,
	StructOrUnionSpecifier(StructOrUnionSpecifier),
	EnumSpecifier(EnumSpecifier),
	TypedefName(TypedefName),
}

pub struct TypedefName(tok::Identifier);

pub struct EnumSpecifier {
	identifier: Option<tok::Identifier>,
	enumerator_list: Option<EnumeratorList>,
}

pub struct EnumeratorList(Vec<Enumerator>);

pub struct Enumerator {
	enumeration_constant: EnumerationConstant,
	constant_expression: expr::ConstantExpression,
}

pub struct EnumerationConstant(tok::Identifier);

pub struct StructOrUnionSpecifier {
	struct_or_union: StructOrUnion,
	identifier: Option<tok::Identifier>,
	struct_declaration_list: Option<StructDeclarationList>,
}

pub struct StructDeclarationList(Vec<StructDeclaration>);

pub struct StructDeclaration {
	specifier_qualifier_list: SpecifierQualifierList,
	struct_declaration_list: StructDeclaratorList,
}

pub struct StructDeclaratorList(Vec<StructDeclarator>);

pub struct StructDeclarator {
	declarator: Option<Declarator>,
	constant_expression: Option<expr::ConstantExpression>,
}

pub enum StructOrUnion {
	Struct,
	Union,
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
	Identifier(tok::Identifier),
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
	IdentifierList(Vec<tok::Identifier>),
}

/// (6.7.5) type-qualifier-list
pub struct TypeQualifierList(Vec<TypeQualifier>);

/// (6.7.5) parameter-type-list
pub struct ParameterTypeList {
	parameter_list: ParameterList,
	comma_ellipsis: bool,
}

pub struct ParameterList(Vec<ParameterDeclaration>);

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

pub struct SpecifierQualifierList(Vec<SpecifierQualifier>);

pub enum SpecifierQualifier {
	TypeSpecifier(TypeSpecifier),
	TypeQualifier(TypeQualifier),
}

pub struct InitializerList(Vec<(Option<Designation>, Initializer)>);

pub struct Designation(DesignatorList);

pub struct DesignatorList(Vec<Designator>);

pub enum Designator {
	ConstantExpression(expr::ConstantExpression),
	Dot(tok::Identifier),
}
